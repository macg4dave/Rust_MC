use super::*;
use std::cmp::min;
use std::io;

impl App {
    // Helper: refresh only the active panel
    pub fn refresh_active(&mut self) -> io::Result<()> {
        self.refresh_panel(self.active)
    }

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
                            actions: None,
                        };
                    } else {
                        let content = format!("{} items processed", upd.processed);
                        self.mode = Mode::Message {
                            title: "Done".to_string(),
                            content,
                            buttons: vec!["OK".to_string()],
                            selected: 0,
                            actions: None,
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
        let mut ents = panel.read_entries()?;
        // Single sort pass. For `Name` sort, keep directories first (so dirs
        // appear before files) then compare by name. For other sorts compare
        // by the selected key. Apply `sort_desc` by reversing once to avoid
        // multiple reversals.
        match self.sort {
            SortKey::Name => {
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
}
