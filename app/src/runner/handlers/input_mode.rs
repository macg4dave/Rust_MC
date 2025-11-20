use crate::app::{App, InputKind, Mode};
use crate::errors;
use crate::input::KeyCode;
use std::path::PathBuf;

pub fn handle_input(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    if let Mode::Input { prompt: _, buffer, kind } = &mut app.mode {
        match code {
            KeyCode::Enter => {
                let input = buffer.clone();
                let kind_snapshot = *kind;
                app.mode = Mode::Normal;
                match kind_snapshot {
                    InputKind::Copy => {
                        let dst = PathBuf::from(input);
                        if let Err(err) = app.copy_selected_to(dst) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0 };
                        }
                    }
                    InputKind::Move => {
                        let dst = PathBuf::from(input);
                        if let Err(err) = app.move_selected_to(dst) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0 };
                        }
                    }
                    InputKind::Rename => {
                        if let Err(err) = app.rename_selected_to(input) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0 };
                        }
                    }
                    InputKind::NewFile => {
                        if let Err(err) = app.new_file(input) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0 };
                        }
                    }
                    InputKind::NewDir => {
                        if let Err(err) = app.new_dir(input) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0 };
                        }
                    }
                    InputKind::ChangePath => {
                        let p = PathBuf::from(input);
                        let panel = app.active_panel_mut();
                        panel.cwd = p;
                        if let Err(err) = app.refresh() {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0 };
                        }
                    }
                }
            }
            KeyCode::Backspace => { buffer.pop(); }
            KeyCode::Esc => { app.mode = Mode::Normal; }
            KeyCode::Char(c) => { buffer.push(c); }
            _ => {}
        }
    }
    Ok(false)
}
