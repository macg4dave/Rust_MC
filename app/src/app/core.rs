use std::cmp::min;
use std::fs;
use std::io;

use chrono::{DateTime, Local};

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

mod core {
    // submodules live in `app/src/app/core/`
}

pub mod panel;
pub mod path;
mod navigation;
mod preview;

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

    // Helper: refresh only the active panel
    pub fn refresh_active(&mut self) -> io::Result<()> {
        self.refresh_panel(self.active)
    }

    /// Resolve destination path for an operation: if `dst` looks like a directory
    /// (exists or ends with a separator) then target becomes `dst.join(src_name)`.
    ///
    /// This is exposed as a public helper for tests.
    // `resolve_target` and `ensure_parent_exists` moved to `fs_op::helpers` to
    // keep filesystem helpers in the `fs_op` module where they belong.

    /// Maximum bytes to read for a file preview (100 KiB)
    ///
    /// Made public so integration tests can verify preview truncation.
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
        match self.active {
            Side::Left => self.left.selected,
            Side::Right => self.right.selected,
        }
    }

    // `selected_entry` removed: not used and was producing a dead-code warning.

    fn refresh_panel(&mut self, side: Side) -> io::Result<()> {
        let panel = match side {
            Side::Left => &mut self.left,
            Side::Right => &mut self.right,
        };
        let mut ents = Vec::new();
        for entry in fs::read_dir(&panel.cwd)? {
            let e = entry?;
            let meta = e.metadata()?;
            let modified = meta.modified().ok().map(|t| DateTime::<Local>::from(t));
            ents.push(Entry {
                name: e.file_name().to_string_lossy().into_owned(),
                path: e.path(),
                is_dir: meta.is_dir(),
                size: meta.len(),
                modified,
            });
        }
        // Single sort pass. For `Name` sort, keep directories first (so dirs
        // appear before files) then compare by name. For other sorts compare
        // by the selected key. Apply `sort_desc` by reversing once to avoid
        // multiple reversals.
        ents.sort_by(|a, b| {
            use std::cmp::Ordering;
            match self.sort {
                SortKey::Name => match (a.is_dir, b.is_dir) {
                    (true, false) => Ordering::Less,
                    (false, true) => Ordering::Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                },
                SortKey::Size => a.size.cmp(&b.size),
                SortKey::Modified => a.modified.cmp(&b.modified),
            }
        });
        if self.sort_desc {
            ents.reverse();
        }

        // Prepend a header row showing the full path, and a `..` entry to go up a level.
        // These are synthetic entries inserted at the front of the listing so the UI
        // can display the current path and provide an easy way to navigate up.
        let mut wrapped = Vec::new();
        // Header: non-directory entry with the full path as the name. Not enterable.
        wrapped.push(Entry {
            name: panel.cwd.display().to_string(),
            path: panel.cwd.clone(),
            is_dir: false,
            size: 0,
            modified: None,
        });
        // Parent: `..` entry if there is a parent directory.
        if let Some(parent) = panel.cwd.parent() {
            wrapped.push(Entry {
                name: "..".to_string(),
                path: parent.to_path_buf(),
                is_dir: true,
                size: 0,
                modified: None,
            });
        }
        wrapped.extend(ents);
        panel.entries = wrapped;
        panel.selected = min(panel.selected, panel.entries.len().saturating_sub(1));
        self.update_preview_for(side);
        Ok(())
    }
    // `update_preview_for` implemented in the `preview` submodule.
}
