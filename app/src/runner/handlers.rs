//! Thin top-level key handlers that dispatch to smaller, focused submodules.
//!
//! This module keeps the top-level dispatch small and delegates mode-specific
//! handling into individual submodules (see the public submodules below).

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
use crate::app::settings::keybinds;
use crate::input::KeyCode;

/// Handle input when the app is displaying a general message dialog.
///
/// This extracts the `Mode::Message` branch from `handle_key` to keep the
/// top-level dispatcher concise and testable.
/// The `Mode::Message` logic is kept inline in the dispatcher below. Extracting
/// it into a helper that also took `&mut App` caused borrow conflicts with the
/// `&mut` borrow of `app.mode` performed by the `match`. The code below mirrors
/// the original behaviour but is clearer and documented.
/// Top-level key handler that dispatches into smaller submodules.
///
/// Returns `Ok(true)` when the caller should trigger a refresh/redraw.
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
                    *selected -= 1;
                } else {
                    *selected = buttons.len().saturating_sub(1);
                }
            } else if keybinds::is_right(&code) {
                *selected = (*selected + 1) % buttons.len();
            } else if keybinds::is_enter(&code) {
                // If an action mapping exists, execute the mapped action for
                // the selected button. Otherwise simply dismiss the dialog.
                if let Some(act) = crate::ui::dialogs::selection_to_action(*selected, actions.as_deref()) {
                    match crate::runner::commands::perform_action(app, act) {
                        Ok(_) => app.mode = Mode::Normal,
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


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::path::PathBuf;

    /// Helper to create an `App` initialised to a temporary directory.
    fn make_app_at_tmpdir() -> (crate::app::core::App, PathBuf) {
        let tmp = tempdir().expect("tempdir");
        let cwd = tmp.path().to_path_buf();
        let opts = crate::app::StartOptions { start_dir: Some(cwd.clone()), ..Default::default() };
        let app = crate::app::core::App::with_options(&opts).expect("with_options");
        (app, cwd)
    }

    #[test]
    fn message_mode_left_and_right_change_selection_wrapping() {
        let (mut app, _cwd) = make_app_at_tmpdir();

        app.mode = Mode::Message {
            title: "T".into(),
            content: "C".into(),
            buttons: vec!["One".into(), "Two".into(), "Three".into()],
            selected: 0,
            actions: None,
        };

        // Left from 0 wraps to last
        let _ = handle_key(&mut app, KeyCode::Left, 0).expect("handler");
        if let Mode::Message { selected, .. } = &app.mode {
            assert_eq!(*selected, 2);
        } else {
            panic!("expected Message mode");
        }

        // Right should advance (wraps around)
        let _ = handle_key(&mut app, KeyCode::Right, 0).expect("handler");
        if let Mode::Message { selected, .. } = &app.mode {
            assert_eq!(*selected, 0);
        } else {
            panic!("expected Message mode");
        }
    }

    #[test]
    fn message_mode_enter_without_action_dismisses() {
        let (mut app, _cwd) = make_app_at_tmpdir();

        app.mode = Mode::Message {
            title: "Hello".into(),
            content: "World".into(),
            buttons: vec!["OK".into()],
            selected: 0,
            actions: None,
        };

        let _ = handle_key(&mut app, KeyCode::Enter, 0).expect("handler");
        assert!(matches!(app.mode, Mode::Normal));
    }

    #[test]
    fn message_mode_enter_with_action_executes_action_and_returns_normal() {
        let (mut app, cwd) = make_app_at_tmpdir();

        let fname = "created_by_action.txt".to_string();

        app.mode = Mode::Message {
            title: "Create".into(),
            content: "Create file".into(),
            buttons: vec!["Create".into(), "Cancel".into()],
            selected: 0,
            actions: Some(vec![crate::app::Action::NewFile(fname.clone())]),
        };

        // Ensure file does not exist before
        let target = cwd.join(&fname);
        if target.exists() {
            let _ = std::fs::remove_file(&target);
        }

        let _ = handle_key(&mut app, KeyCode::Enter, 0).expect("handler");

        // The file should have been created and mode should be Normal
        assert!(target.exists(), "expected action to create file");
        assert!(matches!(app.mode, Mode::Normal));

        // cleanup
        let _ = std::fs::remove_file(&target);
    }
}
