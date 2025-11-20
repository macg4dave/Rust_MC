//! Split handlers: thin wrapper delegating to submodules to keep file sizes manageable.

pub mod confirm;
pub mod conflict;
pub mod context_menu;
pub mod input_mode;
pub mod mouse;
pub mod normal;
pub mod progress_mode;
pub mod settings;

pub use confirm::handle_confirm;
pub use conflict::handle_conflict;
pub use context_menu::handle_context_menu;
pub use input_mode::handle_input;
pub use mouse::handle_mouse;
pub use normal::handle_normal;
pub use progress_mode::handle_progress;
pub use settings::handle_settings;

use crate::app::{App, Mode};
use crate::input::KeyCode;
use crate::app::settings::keybinds;

/// Top-level key handler that dispatches into smaller submodules.
pub fn handle_key(app: &mut App, code: KeyCode, page_size: usize) -> anyhow::Result<bool> {
    match &mut app.mode {
        Mode::Normal => handle_normal(app, code, page_size),
        Mode::Progress { .. } => handle_progress(app, code),
        Mode::Conflict { .. } => handle_conflict(app, code),
        Mode::ContextMenu { .. } => handle_context_menu(app, code),
        Mode::Message {
            title: _,
            content: _,
            buttons,
            selected,
            actions,
        } => {
            if keybinds::is_left(&code) {
                if *selected > 0 {
                    *selected -= 1
                } else {
                    *selected = buttons.len().saturating_sub(1)
                }
            } else if keybinds::is_right(&code) {
                *selected = (*selected + 1) % buttons.len();
            } else if keybinds::is_enter(&code) {
                // If an action mapping exists, execute the mapped action for
                // the selected button. Otherwise simply dismiss the dialog.
                if let Some(act) =
                    crate::ui::dialogs::selection_to_action(*selected, actions.as_deref())
                {
                    match crate::runner::commands::perform_action(app, act) {
                        Ok(_) => {
                            app.mode = Mode::Normal;
                        }
                        Err(e) => {
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: format!("Action failed: {}", e),
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                                actions: None,
                            };
                        }
                    }
                } else {
                    app.mode = Mode::Normal;
                }
            } else if keybinds::is_esc(&code) || matches!(code, KeyCode::Char(_)) {
                app.mode = Mode::Normal;
            }
            Ok(false)
        }
        Mode::Confirm { .. } => handle_confirm(app, code),
        Mode::Input { .. } => handle_input(app, code),
        Mode::Settings { .. } => handle_settings(app, code),
    }
}
