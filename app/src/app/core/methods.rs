//! Core `App` methods: high-level operations modifying application state.
//!
//! This file exposes `impl App { ... }` helpers (refresh, navigation,
//! progress handling) that operate on the public `App` state defined in
//! `app::core::mod`.

use std::io;

use super::{init, App, Panel, Mode, Side, SortKey};

impl App {
    // Helper: refresh only the active panel
    pub fn refresh_active(&mut self) -> io::Result<()> {
        self.refresh_panel(self.active)
    }

    pub fn new() -> io::Result<Self> {
        let cwd = std::env::current_dir()?;
        let mut app = init::with_cwd(cwd);
        app.refresh()?;
        Ok(app)
    }

    /// Construct an App instance with default initial values but without
    /// performing filesystem I/O. This is useful for tests or callers that
    /// want to initialise state and control when `refresh` runs.
    // `with_cwd` moved to `app::core::init` to be reusable across core
    // submodules and tests. Use `super::init::with_cwd` when constructing
    // an App from a known working directory.
    /// Create an App with explicit startup options (for example a start
    /// directory or initial mouse setting). This mirrors `new` but uses
    /// `StartOptions` when provided so callers can control initial state
    /// without mutating global process state.
    pub fn with_options(opts: &crate::app::StartOptions) -> io::Result<Self> {
        let cwd = if let Some(d) = &opts.start_dir {
            d.clone()
        } else {
            std::env::current_dir()?
        };
        let mut app = App {
            left: Panel::new(cwd.clone()),
            right: Panel::new(cwd),
            active: Side::Left,
            mode: Mode::Normal,
            sort: SortKey::Name,
            sort_order: crate::app::types::SortOrder::Ascending,
            menu_index: 0,
            menu_focused: false,
            preview_visible: false,
            command_line: None,
            settings: crate::app::settings::write_settings::Settings::default(),
            op_progress_rx: None,
            op_cancel_flag: None,
            op_decision_tx: None,
            last_mouse_click_time: None,
            last_mouse_click_pos: None,
            drag_active: false,
            drag_start: None,
            drag_current: None,
            drag_button: None,
        };
        // Apply any immediate overrides requested by CLI options. Persisted
        // settings (loaded later) will be applied afterwards; callers that
        // want CLI to override persisted settings should reapply after
        // loading settings (event loop does this).
        if let Some(m) = opts.mouse_enabled {
            app.settings.mouse_enabled = m;
        }
        if let Some(s) = opts.show_hidden {
            app.settings.show_hidden = s;
        }
        if let Some(ref theme) = opts.theme {
            // Update persisted-in-memory setting and apply theme to UI
            app.settings.theme = theme.clone();
            crate::ui::colors::set_theme(theme.as_str());
        }
        app.refresh()?;
        Ok(app)
    }

    /// Toggle the preview pane visibility.
    pub fn toggle_preview(&mut self) {
        self.preview_visible = !self.preview_visible;
    }

    /// Poll an active progress receiver and update the `Mode::Progress` state
    /// accordingly. This should be called periodically from the event loop so
    /// the UI can reflect progress updates and completion.
    pub fn poll_progress(&mut self) {
        // Poll and consume available progress updates, keeping only the
        // most-recent one. If the channel closes we clear the receiver.
        if let Some(rx) = self.op_progress_rx.as_ref() {
            let mut last: Option<crate::runner::progress::ProgressUpdate> = None;
            while let Ok(update) = rx.try_recv() {
                last = Some(update);
            }

            // If channel is closed, ensure receiver is cleared and return.
            if let Err(std::sync::mpsc::TryRecvError::Disconnected) = rx.try_recv() {
                self.op_progress_rx = None;
                return;
            }

            if let Some(update) = last {
                if let Some(conflict_path) = update.conflict {
                    self.mode = Mode::Conflict {
                        path: conflict_path,
                        selected: 0,
                        apply_all: false,
                    };
                    return;
                }

                if update.done {
                    self.op_progress_rx = None;
                    self.op_cancel_flag = None;
                    self.op_decision_tx = None;

                    if let Some(err_msg) = update.error {
                        self.mode = Mode::Message {
                            title: "Error".to_string(),
                            content: err_msg,
                            buttons: vec!["OK".to_string()],
                            selected: 0,
                            actions: None,
                        };
                    } else {
                        let content = format!("{} items processed", update.processed);
                        self.mode = Mode::Message {
                            title: "Done".to_string(),
                            content,
                            buttons: vec!["OK".to_string()],
                            selected: 0,
                            actions: None,
                        };
                    }

                    self.left.clear_selections();
                    self.right.clear_selections();
                    let _ = self.refresh();
                } else {
                    let message = update.message.unwrap_or_default();
                    self.mode = Mode::Progress {
                        title: if message.is_empty() { "Progress".to_string() } else { message.clone() },
                        processed: update.processed,
                        total: update.total,
                        message,
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

    /// Refresh only the specified panel side. This allows callers (for
    /// example filesystem watchers) to update just the affected panel
    /// instead of forcing a full two-panel refresh.
    pub fn refresh_side(&mut self, side: Side) -> io::Result<()> {
        self.refresh_panel(side)
    }

    /// Switches the menu selection to the next tab (wraps around).
    pub fn menu_next(&mut self) {
        let n = crate::ui::menu::menu_labels().len();
        if n == 0 {
            return;
        }
        self.menu_index = (self.menu_index + 1) % n;
    }

    /// Switches the menu selection to the previous tab (wraps around).
    pub fn menu_prev(&mut self) {
        let n = crate::ui::menu::menu_labels().len();
        if n == 0 {
            return;
        }
        self.menu_index = (self.menu_index + n - 1) % n;
    }

    /// Activate currently selected menu item (for now show a message).
    pub fn menu_activate(&mut self) {
        let labels = crate::ui::menu::menu_labels();
        if let Some(lbl) = labels.get(self.menu_index) {
            if *lbl == "Settings" {
                self.mode = Mode::Settings { selected: 0 };
            } else {
                let content = format!("Menu '{}' selected", lbl);
                self.mode = Mode::Message {
                    title: lbl.to_string(),
                    content,
                    buttons: vec!["OK".to_string()],
                    selected: 0,
                    actions: None,
                };
            }
        }
    }

    fn refresh_panel(&mut self, side: Side) -> io::Result<()> {
        let panel = match side {
            Side::Left => &mut self.left,
            Side::Right => &mut self.right,
        };
        // Read directory entries once via a helper so the iteration and
        // filesystem interaction can be easily unit-tested or refactored.
        let mut entries = panel.read_entries()?;

        // Single sort pass. For `Name` sort, keep directories first (so dirs
        // appear before files) then compare by name. For other sorts compare
        // by the selected key. Apply `sort_desc` by reversing once to avoid
        // multiple reversals.
        match self.sort {
            SortKey::Name => entries.sort_by_key(|entry| (!entry.is_dir, entry.name.to_lowercase())),
            SortKey::Size => entries.sort_by_key(|entry| entry.size),
            SortKey::Modified => entries.sort_by_key(|entry| entry.modified),
        }

        if self.sort_order == crate::app::types::SortOrder::Descending {
            entries.reverse();
        }

        // Keep `panel.entries` as a pure domain list: only filesystem
        // entries (no synthetic header/parent). Store the read entries
        // directly and clamp UI selection/offset against the UI row
        // count (header + parent + entries).
        panel.entries = entries;
        let visible_rows = super::utils::ui_row_count(panel);
        let last_index = visible_rows.saturating_sub(1);
        if panel.selected > last_index {
            panel.selected = last_index;
        }
        if panel.offset > last_index {
            panel.offset = last_index;
        }
        self.update_preview_for(side);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn with_cwd_initialises_panels() {
        let tmp = tempdir().expect("tempdir");
        let cwd = tmp.path().to_path_buf();
        let app = super::init::with_cwd(cwd.clone());
        assert_eq!(app.left.cwd, cwd);
        assert_eq!(app.right.cwd, cwd);
        assert!(!app.preview_visible);
        assert!(matches!(app.mode, Mode::Normal));
    }

    #[test]
    fn toggle_preview_changes_flag() {
        let tmp = tempdir().expect("tempdir");
        let cwd = tmp.path().to_path_buf();
        let mut app = super::init::with_cwd(cwd);
        assert!(!app.preview_visible);
        app.toggle_preview();
        assert!(app.preview_visible);
        app.toggle_preview();
        assert!(!app.preview_visible);
    }

    #[test]
    fn menu_wraps_around() {
        let tmp = tempdir().expect("tempdir");
        let cwd = tmp.path().to_path_buf();
        let mut app = super::init::with_cwd(cwd);
        let n = crate::ui::menu::menu_labels().len();
        if n == 0 {
            return;
        }
        app.menu_index = n - 1;
        app.menu_next();
        assert_eq!(app.menu_index, 0);
        app.menu_prev();
        assert_eq!(app.menu_index, n - 1);
    }
}
