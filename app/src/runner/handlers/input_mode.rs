//! Input-mode key handler.
//!
//! This module handles user keyboard input when the application is in
//! `Mode::Input`. It interprets printable characters, backspace, escape
//! and submit (enter) keys and dispatches actions based on the
//! `InputKind`.

use std::mem;
use std::path::PathBuf;

use crate::app::{App, InputKind, Mode};
use crate::app::settings::keybinds;
use crate::errors;
use crate::input::KeyCode;

/// Handle keyboard events while the app is in `Mode::Input`.
///
/// Returns `Ok(false)` by convention (no special redraw request).
pub fn handle_input(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    // Fast-path: only handle keys when we're in input mode.
    if let Mode::Input { prompt: _, buffer, kind } = &mut app.mode {
        if keybinds::is_enter(&code) {
            // Take ownership of the buffer without cloning.
            let input = mem::take(buffer);
            let kind_snapshot = *kind;

            // Leave input mode before performing potentially-failing IO so
            // the UI can reliably render error dialogs.
            app.mode = Mode::Normal;

            match kind_snapshot {
                InputKind::Copy => {
                    let dst = PathBuf::from(&input);
                    if let Err(e) = app.copy_selected_to(dst) {
                        set_error_message(app, errors::render_fsop_error(&e, None, None, None));
                    }
                }
                InputKind::Move => {
                    let dst = PathBuf::from(&input);
                    if let Err(e) = app.move_selected_to(dst) {
                        set_error_message(app, errors::render_fsop_error(&e, None, None, None));
                    }
                }
                InputKind::Rename => {
                    if let Err(e) = app.rename_selected_to(input) {
                        set_error_message(app, errors::render_fsop_error(&e, None, None, None));
                    }
                }
                InputKind::NewFile => {
                    if let Err(e) = app.new_file(input) {
                        set_error_message(app, errors::render_fsop_error(&e, None, None, None));
                    }
                }
                InputKind::NewDir => {
                    if let Err(e) = app.new_dir(input) {
                        set_error_message(app, errors::render_fsop_error(&e, None, None, None));
                    }
                }
                InputKind::ChangePath => {
                    let p = PathBuf::from(&input);
                    let panel = app.active_panel_mut();
                    panel.cwd = p;
                    if let Err(e) = app.refresh() {
                        set_error_message(app, errors::render_io_error(&e, None, None, None));
                    }
                }
            }
        } else if keybinds::is_backspace(&code) {
            buffer.pop();
        } else if keybinds::is_esc(&code) {
            app.mode = Mode::Normal;
        } else if let KeyCode::Char(c) = code {
            buffer.push(c);
        }
    }

    Ok(false)
}

/// Set a simple "Error" message dialog on the app.
fn set_error_message(app: &mut App, content: String) {
    app.mode = Mode::Message {
        title: "Error".to_string(),
        content,
        buttons: vec!["OK".to_string()],
        selected: 0,
        actions: None,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::core::App as CoreApp;

    #[test]
    fn char_inserts_into_buffer() {
        let mut app = CoreApp::new().unwrap();
        app.mode = Mode::Input { prompt: "".into(), buffer: String::new(), kind: InputKind::Rename };
        let _ = handle_input(&mut app, KeyCode::Char('x')).unwrap();
        if let Mode::Input { buffer, .. } = &app.mode {
            assert_eq!(buffer, "x");
        } else {
            panic!("expected Input mode")
        }
    }

    #[test]
    fn backspace_pops_character() {
        let mut app = CoreApp::new().unwrap();
        app.mode = Mode::Input { prompt: "".into(), buffer: "ab".into(), kind: InputKind::Rename };
        let _ = handle_input(&mut app, KeyCode::Backspace).unwrap();
        if let Mode::Input { buffer, .. } = &app.mode {
            assert_eq!(buffer, "a");
        } else {
            panic!("expected Input mode")
        }
    }

    #[test]
    fn esc_exits_input_mode() {
        let mut app = CoreApp::new().unwrap();
        app.mode = Mode::Input { prompt: "".into(), buffer: "".into(), kind: InputKind::Rename };
        let _ = handle_input(&mut app, KeyCode::Esc).unwrap();
        assert!(matches!(app.mode, Mode::Normal));
    }

    #[test]
    fn enter_with_copy_kind_runs_noop_when_nothing_selected() {
        let mut app = CoreApp::new().unwrap();
        app.mode = Mode::Input { prompt: "".into(), buffer: "dest".into(), kind: InputKind::Copy };
        let _ = handle_input(&mut app, KeyCode::Enter).unwrap();
        // No selection means operation is a no-op; app should be back to Normal.
        assert!(matches!(app.mode, Mode::Normal));
    }
}
