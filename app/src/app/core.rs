use std::cmp::min;
use std::io;

// chrono imported in Panel (file metadata reading)

use self::panel::Panel;
use super::types::{Entry, Mode, Side, SortKey};

pub struct App {
    pub left: Panel,
    pub right: Panel,
    pub active: Side,
    pub mode: Mode,
    pub sort: SortKey,
    pub sort_desc: bool,
}

// submodules live in `app/src/app/core/`

pub mod panel;
// `path` was previously a small shim module at `app/src/app/core/path.rs`.
// It has been removed in favor of re-exporting the canonical `app::path`
// compatibility shim that points to `crate::fs_op::path::resolve_path`.
// Re-export the `app::path` module under `app::core::path` so references like
// `crate::app::core::path::resolve_path` continue to work.
// Re-export the canonical path helpers into the `app::core` namespace so
// code referencing `crate::app::core::path` continues to work without using
// the deprecated `app::path` shim.
pub use crate::fs_op::path;
mod navigation;
mod preview;
pub mod preview_helpers;

impl App {
    // Helper: return mutable reference to the currently active panel
    // Made `pub(crate)` so other internal modules (for example `fs_op` helpers)
    // can operate on `App` without exposing this helper publicly.
    pub(crate) fn active_panel_mut(&mut self) -> &mut Panel {
        match self.active {
            Side::Left => &mut self.left,
            Side::Right => &mut self.right,
        }
    }

    /// Return a reference to the currently active panel (non-mutable).
    ///
    /// This mirrors `active_panel_mut` and is useful for read-only methods
    /// such as `selected_index`, allowing them to avoid direct field access
    /// and to keep the selection logic centralized.
    pub(crate) fn active_panel(&self) -> &Panel {
        match self.active {
            Side::Left => &self.left,
            Side::Right => &self.right,
        }
    }

    /// Return a mutable reference to the panel identified by `side`.
    ///
    /// This centralizes the pattern used across multiple methods and avoids
    /// repeating `match side` everywhere. It keeps borrow semantics simple and
    /// mirrors `active_panel_mut` used for the active side.
    pub(crate) fn panel_mut(&mut self, side: Side) -> &mut Panel {
        match side {
            Side::Left => &mut self.left,
            Side::Right => &mut self.right,
        }
    }

    // Read-only `panel(&self)` accessor removed - use `active_panel()` or
    // `panel_mut()` instead to access panels in a read-only or mutable way.

    // Helper: refresh only the active panel
    pub fn refresh_active(&mut self) -> io::Result<()> {
        self.refresh_panel(self.active)
    }

    // Resolve destination path for an operation: if `dst` looks like a directory
    // (exists or ends with a separator) then target becomes `dst.join(src_name)`.
    //
    // This is exposed as a public helper for tests.
    // `resolve_target` and `ensure_parent_exists` moved to `fs_op::helpers` to
    // keep filesystem helpers in the `fs_op` module where they belong.

    /// Maximum bytes to read for a file preview (100 KiB). Made public so
    /// integration tests can verify preview truncation.
    pub const MAX_PREVIEW_BYTES: usize = 100 * 1024;

    pub fn new() -> io::Result<Self> {
        let cwd = std::env::current_dir()?;
        let mut app = App {
            left: Panel::new(cwd.clone()),
            right: Panel::new(cwd),
            active: Side::Left,
            mode: Mode::Normal,
            sort: SortKey::Name,
            sort_desc: false,
        };
        app.refresh()?;
        Ok(app)
    }

    pub fn refresh(&mut self) -> io::Result<()> {
        self.refresh_panel(Side::Left)?;
        self.refresh_panel(Side::Right)?;
        Ok(())
    }

    /// Return the currently selected index for the active panel.
    ///
    /// This is a small helper to avoid repeating the `match self.active` logic
    /// across methods that need to consult the selected entry index. The
    /// selection is stored on the panel and is clamped by `refresh_panel`.
    pub(crate) fn selected_index(&self) -> usize {
        // Use the `active_panel()` helper so this method never needs to
        // duplicate matching on `self.active` and remains read-only.
        self.active_panel().selected
    }

    // `selected_entry` removed: not used and was producing a dead-code warning.

    fn refresh_panel(&mut self, side: Side) -> io::Result<()> {
        let panel = match side {
            Side::Left => &mut self.left,
            Side::Right => &mut self.right,
        };
        // Read directory entries once via a helper so the iteration and
        // filesystem interaction can be easily unit-tested or refactored.
        let mut ents = panel.read_entries()?;
        // Single sort pass. For `Name` sort, keep directories first (so dirs
        // appear before files) then compare by name. For other sorts compare
        // by the selected key. Apply `sort_desc` by reversing once to avoid
        // multiple reversals.
        match self.sort {
            SortKey::Name => {
                // Use `sort_by_key` so the lowercase key is computed once per
                // element instead of on every comparison which avoids repeated
                // allocations performed by `to_lowercase()` per comparison.
                ents.sort_by_key(|e| (!e.is_dir, e.name.to_lowercase()));
            }
            SortKey::Size => ents.sort_by_key(|e| e.size),
            SortKey::Modified => ents.sort_by_key(|e| e.modified),
        }
        if self.sort_desc {
            ents.reverse();
        }

        // Prepend a header row showing the full path, and a `..` entry to go up a level.
        // These are synthetic entries inserted at the front of the listing so the UI
        // can display the current path and provide an easy way to navigate up.
        let mut wrapped = Vec::new();
        // Header: non-directory entry with the full path as the name. Not enterable.
        wrapped.push(Entry::header(panel.cwd.clone()));
        // Parent: `..` entry if there is a parent directory.
        if let Some(parent) = panel.cwd.parent() {
            wrapped.push(Entry::parent(parent.to_path_buf()));
        }
        wrapped.extend(ents);
        panel.entries = wrapped;
        panel.selected = min(panel.selected, panel.entries.len().saturating_sub(1));
        self.update_preview_for(side);
        Ok(())
    }
    // `update_preview_for` implemented in the `preview` submodule.
}

/// Read directory entries and return a vector of `Entry`s populated with
/// the common metadata fields. This is a small helper extracted from
/// `refresh_panel` to make the filesystem-detection and tests easier to
/// reason about and to keep IO-related code in one place.
// read_entries moved to `Panel::read_entries` in `panel.rs`.

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn selected_index_reflects_active_panel_unit() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.child("a.txt").write_str("1").unwrap();
        temp.child("b.txt").write_str("2").unwrap();
        temp.child("c.txt").write_str("3").unwrap();

        let cwd = temp.path().to_path_buf();
        let mut app = App {
            left: Panel::new(cwd.clone()),
            right: Panel::new(cwd.clone()),
            active: Side::Left,
            mode: Mode::Normal,
            sort: SortKey::Name,
            sort_desc: false,
        };
        app.refresh().unwrap();

        // find index of a.txt
        let mut left_idx = None;
        for (i, e) in app.left.entries.iter().enumerate() {
            if e.name == "a.txt" {
                left_idx = Some(i);
                break;
            }
        }
        assert!(left_idx.is_some());
        app.left.selected = left_idx.unwrap();
        app.active = Side::Left;
        assert_eq!(app.selected_index(), app.left.selected);

        // for right panel
        let mut right_idx = None;
        for (i, e) in app.right.entries.iter().enumerate() {
            if e.name == "b.txt" {
                right_idx = Some(i);
                break;
            }
        }
        assert!(right_idx.is_some());
        app.right.selected = right_idx.unwrap();
        app.active = Side::Right;
        assert_eq!(app.selected_index(), app.right.selected);

        temp.close().unwrap();
    }

    #[test]
    fn panel_mut_match() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.child("a.txt").write_str("1").unwrap();
        let cwd = temp.path().to_path_buf();
        let mut app = App {
            left: Panel::new(cwd.clone()),
            right: Panel::new(cwd.clone()),
            active: Side::Left,
            mode: Mode::Normal,
            sort: SortKey::Name,
            sort_desc: false,
        };
        app.refresh().unwrap();
        // modify left via panel_mut and check read through panel
        let left_name_before = app.left.cwd.clone();
        let panel_mut = app.panel_mut(Side::Left);
        panel_mut.cwd = std::path::PathBuf::from(".");
        let left_name_after = app.left.cwd.clone();
        assert_eq!(left_name_after, std::path::PathBuf::from("."));
        assert_ne!(left_name_before, left_name_after);
    }
}
