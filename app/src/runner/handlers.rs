//! Split handlers: thin wrapper delegating to submodules to keep file sizes manageable.

pub mod normal;
pub mod mouse;
pub mod context_menu;
pub mod conflict;
pub mod progress_mode;
pub mod confirm;
pub mod input_mode;

pub use normal::handle_normal;
pub use mouse::handle_mouse;
pub use context_menu::handle_context_menu;
pub use conflict::handle_conflict;
pub use progress_mode::handle_progress;
pub use confirm::handle_confirm;
pub use input_mode::handle_input;

use crate::app::{App, Mode};
use crate::input::KeyCode;

/// Top-level key handler that dispatches into smaller submodules.
pub fn handle_key(app: &mut App, code: KeyCode, page_size: usize) -> anyhow::Result<bool> {
    match &mut app.mode {
        Mode::Normal => handle_normal(app, code, page_size),
        Mode::Progress { .. } => handle_progress(app, code),
        Mode::Conflict { .. } => handle_conflict(app, code),
        Mode::ContextMenu { .. } => handle_context_menu(app, code),
        Mode::Message { .. } => {
            match code {
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char(_) => app.mode = Mode::Normal,
                _ => {}
            }
            Ok(false)
        }
        Mode::Confirm { .. } => handle_confirm(app, code),
        Mode::Input { .. } => handle_input(app, code),
    }
}
