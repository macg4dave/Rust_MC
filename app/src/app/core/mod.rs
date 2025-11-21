// `min` and `io` are used in `methods` submodule; import there instead.

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
    /// Whether the preview pane is visible in the UI.
    pub preview_visible: bool,
    /// Optional command-line state when user opens the command input.
    pub command_line: Option<crate::ui::command_line::CommandLineState>,
    /// User settings loaded from disk.
    pub settings: crate::app::settings::write_settings::Settings,
    /// Receiver for progress updates from background file operations.
    pub op_progress_rx: Option<std::sync::mpsc::Receiver<crate::runner::progress::ProgressUpdate>>,
    /// Cancel flag shared with background operation thread (if any).
    pub op_cancel_flag: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    /// Sender for communicating user's decision back to the background worker
    /// when a file-exists conflict is presented.
    pub op_decision_tx: Option<std::sync::mpsc::Sender<crate::runner::progress::OperationDecision>>,
    /// Last mouse click timestamp (used for double-click detection).
    pub last_mouse_click_time: Option<std::time::Instant>,
    /// Last mouse click position (column, row).
    pub last_mouse_click_pos: Option<(u16, u16)>,
    /// Whether a drag operation is currently active.
    pub drag_active: bool,
    /// Drag start position (column, row).
    pub drag_start: Option<(u16, u16)>,
    /// Current drag position (column, row).
    pub drag_current: Option<(u16, u16)>,
    /// Which mouse button started the drag.
    pub drag_button: Option<crate::input::mouse::MouseButton>,
}

// submodules live in `app/src/app/core/`
pub mod panel;
// Re-export the canonical path helpers into the `app::core` namespace so
// code referencing `crate::app::core::path` continues to work without using
// the deprecated `app::path` shim.
pub use crate::fs_op::path;
mod navigation;
mod preview;
pub mod preview_helpers;

mod methods;

/// Maximum bytes to read for a file preview (100 KiB). Made public so
/// integration tests can verify preview truncation.
pub const MAX_PREVIEW_BYTES: usize = 100 * 1024;

impl App {
    /// Maximum bytes to read for a file preview (100 KiB).
    pub const MAX_PREVIEW_BYTES: usize = 100 * 1024;
    // Helper: return mutable reference to the currently active panel
    // Made `pub(crate)` so other internal modules (for example `fs_op` helpers)
    // can operate on `App` without exposing this helper publicly.
    pub fn active_panel_mut(&mut self) -> &mut Panel {
        match self.active {
            Side::Left => &mut self.left,
            Side::Right => &mut self.right,
        }
    }

    /// Return a reference to the currently active panel (non-mutable).
    pub fn active_panel(&self) -> &Panel {
        match self.active {
            Side::Left => &self.left,
            Side::Right => &self.right,
        }
    }

    /// Return a mutable reference to the panel identified by `side`.
    pub fn panel_mut(&mut self, side: Side) -> &mut Panel {
        match side {
            Side::Left => &mut self.left,
            Side::Right => &mut self.right,
        }
    }

    /// Return the currently selected index for the active panel.
    pub fn selected_index(&self) -> Option<usize> {
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
}

// Unit tests were moved to integration tests under `app/tests/`.
// `#[cfg(test)] mod tests;` removed to avoid including an external file.
