use crate::app::{App, InputKind, Mode};
use crate::errors;
use crate::input::KeyCode;
use crate::app::settings::keybinds;
use std::path::PathBuf;

pub fn handle_input(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    if let Mode::Input {
        prompt: _,
        buffer,
        kind,
    } = &mut app.mode
    {
        if keybinds::is_enter(&code) {
            // enter/submit
            let input = buffer.clone();
            let kind_snapshot = *kind;
            app.mode = Mode::Normal;
            match kind_snapshot {
                    InputKind::Copy => {
                        let dst = PathBuf::from(input);
                        if let Err(err) = app.copy_selected_to(dst) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: msg,
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                                actions: None,
                            };
                        }
                    }
                    InputKind::Move => {
                        let dst = PathBuf::from(input);
                        if let Err(err) = app.move_selected_to(dst) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: msg,
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                                actions: None,
                            };
                        }
                    }
                    InputKind::Rename => {
                        if let Err(err) = app.rename_selected_to(input) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: msg,
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                                actions: None,
                            };
                        }
                    }
                    InputKind::NewFile => {
                        if let Err(err) = app.new_file(input) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: msg,
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                                actions: None,
                            };
                        }
                    }
                    InputKind::NewDir => {
                        if let Err(err) = app.new_dir(input) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: msg,
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                                actions: None,
                            };
                        }
                    }
                    InputKind::ChangePath => {
                        let p = PathBuf::from(input);
                        let panel = app.active_panel_mut();
                        panel.cwd = p;
                        if let Err(err) = app.refresh() {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: msg,
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                                actions: None,
                            };
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
