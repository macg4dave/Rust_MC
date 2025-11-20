use std::cmp::min;
use std::io;

// chrono imported in Panel (file metadata reading)

use self::panel::Panel;
use super::types::{Mode, Side, SortKey};

pub struct App {
    pub left: Panel,
    pub right: Panel,
    pub active: Side,
    pub mode: Mode,
    pub sort: SortKey,
    pub sort_desc: bool,
    pub menu_index: usize,
    pub menu_focused: bool,
    /// Receiver for progress updates from background file operations.
    pub op_progress_rx: Option<std::sync::mpsc::Receiver<crate::runner::progress::ProgressUpdate>>,
    /// Cancel flag shared with background operation thread (if any).
    pub op_cancel_flag: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    /// Sender for communicating user's decision back to the background worker
    /// when a file-exists conflict is presented.
    pub op_decision_tx: Option<std::sync::mpsc::Sender<crate::runner::progress::OperationDecision>>,
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
            menu_index: 0,
            menu_focused: false,
            op_progress_rx: None,
            op_cancel_flag: None,
            op_decision_tx: None,
        };
        app.refresh()?;
        Ok(app)
    }

    /// Poll an active progress receiver and update the `Mode::Progress` state
    /// accordingly. This should be called periodically from the event loop so
    /// the UI can reflect progress updates and completion.
    pub fn poll_progress(&mut self) {
        if let Some(rx) = &self.op_progress_rx {
            // Drain available updates - keep the last one
            let mut last: Option<crate::runner::progress::ProgressUpdate> = None;
            loop {
                match rx.try_recv() {
                    Ok(u) => last = Some(u),
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        // Channel closed unexpectedly - clear receiver and exit
                        self.op_progress_rx = None;
                        break;
                    }
                }
            }
            if let Some(upd) = last {
                    if upd.conflict.is_some() {
                        // Present conflict modal to user and leave decision channel
                        if let Some(p) = &upd.conflict {
                            self.mode = Mode::Conflict { path: p.clone(), selected: 0, apply_all: false };
                            return;
                        }
                    }
                if upd.done {
                    // Operation finished: clear receiver and show a message or error
                    self.op_progress_rx = None;
                    // clear cancel flag as operation is complete
                    self.op_cancel_flag = None;
                    self.op_decision_tx = None;
                    if let Some(err) = upd.error {
                        self.mode = Mode::Message {
                            title: "Error".to_string(),
                            content: err,
                            buttons: vec!["OK".to_string()],
                            selected: 0,
                        };
                    } else {
                        let content = format!("{} items processed", upd.processed);
                        self.mode = Mode::Message {
                            title: "Done".to_string(),
                            content,
                            buttons: vec!["OK".to_string()],
                            selected: 0,
                        };
                    }
                    // Clear any multi-selections after successful/failed operation
                    self.left.clear_selections();
                    self.right.clear_selections();
                    // Refresh panels after operation
                    let _ = self.refresh();
                } else {
                    // Update progress mode
                    self.mode = Mode::Progress {
                        title: upd.message.clone().unwrap_or_else(|| "Progress".to_string()),
                        processed: upd.processed,
                        total: upd.total,
                        message: upd.message.unwrap_or_default(),
                        cancelled: false,
                    };
                }
            }
        }
    }

    pub fn refresh(&mut self) -> io::Result<()> {
        self.refresh_panel(Side::Left)?;
        self.refresh_panel(Side::Right)?;
        Ok(())
    }

    /// Switches the menu selection to the next tab (wraps around).
    pub fn menu_next(&mut self) {
        let n = crate::ui::menu::menu_labels().len();
        if n == 0 { return; }
        self.menu_index = (self.menu_index + 1) % n;
    }

    /// Switches the menu selection to the previous tab (wraps around).
    pub fn menu_prev(&mut self) {
        let n = crate::ui::menu::menu_labels().len();
        if n == 0 { return; }
        self.menu_index = (self.menu_index + n - 1) % n;
    }

    /// Activate currently selected menu item (for now show a message).
    pub fn menu_activate(&mut self) {
        let labels = crate::ui::menu::menu_labels();
        if let Some(lbl) = labels.get(self.menu_index) {
            let content = format!("Menu '{}' selected", lbl);
            self.mode = Mode::Message {
                title: lbl.to_string(),
                content,
                buttons: vec!["OK".to_string()],
                selected: 0,
            };
        }
    }

    /// Return the currently selected index for the active panel.
    ///
    /// This is a small helper to avoid repeating the `match self.active` logic
    /// across methods that need to consult the selected entry index. The
    /// selection is stored on the panel and is clamped by `refresh_panel`.
    /// Return the currently selected index within `panel.entries`, if the UI
    /// selected index maps to a filesystem entry (i.e. not the header or parent row).
    pub(crate) fn selected_index(&self) -> Option<usize> {
        let panel = self.active_panel();
        let header_count = 1usize;
        let parent_count = if panel.cwd.parent().is_some() {
            1usize
        } else {
            0usize
        };
        let sel = panel.selected;
        if sel >= header_count + parent_count {
            Some(sel - header_count - parent_count)
        } else {
            None
        }
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

        // Keep `panel.entries` as a pure domain list: only filesystem
        // entries (no synthetic header/parent). Store the read entries
        // directly and clamp UI selection/offset against the UI row
        // count (header + parent + entries).
        panel.entries = ents;
        let max_rows = 1 + if panel.cwd.parent().is_some() { 1 } else { 0 } + panel.entries.len();
        panel.selected = min(panel.selected, max_rows.saturating_sub(1));
        panel.offset = min(panel.offset, max_rows.saturating_sub(1));
        self.update_preview_for(side);
        Ok(())
    }
    // `update_preview_for` implemented in the `preview` submodule.
}

// Read directory entries and return a vector of `Entry`s populated with
// the common metadata fields. This is a small helper extracted from
// `refresh_panel` to make the filesystem-detection and tests easier to
// reason about and to keep IO-related code in one place.
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
            menu_index: 0,
            menu_focused: false,
            op_progress_rx: None,
            op_cancel_flag: None,
            op_decision_tx: None,
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
        let header_count = 1usize;
        let parent_count = if app.left.cwd.parent().is_some() {
            1usize
        } else {
            0usize
        };
        let ui_left_idx = header_count + parent_count + left_idx.unwrap();
        app.left.selected = ui_left_idx;
        app.active = Side::Left;
        assert_eq!(app.selected_index(), Some(left_idx.unwrap()));

        // for right panel
        let mut right_idx = None;
        for (i, e) in app.right.entries.iter().enumerate() {
            if e.name == "b.txt" {
                right_idx = Some(i);
                break;
            }
        }
        assert!(right_idx.is_some());
        let parent_count_r = if app.right.cwd.parent().is_some() {
            1usize
        } else {
            0usize
        };
        let ui_right_idx = header_count + parent_count_r + right_idx.unwrap();
        app.right.selected = ui_right_idx;
        app.active = Side::Right;
        assert_eq!(app.selected_index(), Some(right_idx.unwrap()));

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
            menu_index: 0,
            menu_focused: false,
            op_progress_rx: None,
            op_cancel_flag: None,
            op_decision_tx: None,
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
