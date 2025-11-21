//! App initialization helpers (pure constructors used by tests and callers).
//!
//! Keep functions here side-effect free: they should construct `App`
//! instances without performing filesystem I/O.

use std::path::PathBuf;

use super::{App, Panel, Side, Mode, SortKey};

/// App initialization helpers.
///
/// This module is intended to hold constructors and small helpers that
/// create or initialise `App` state without performing filesystem I/O.
/// Keep only pure initialisation helpers here so other core modules and
/// tests can reuse consistent defaults.
pub(crate) fn with_cwd(cwd: PathBuf) -> App {
    App {
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
    }
}
