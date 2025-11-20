use crate::app::{App, Action, Mode};
use crate::errors;
use crate::input::KeyCode;

pub fn handle_confirm(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    if let Mode::Confirm { on_yes, .. } = &mut app.mode {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                let action = on_yes.clone();
                app.mode = Mode::Normal;
                match action {
                    Action::DeleteSelected => {
                        if let Err(err) = app.delete_selected() {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0 };
                        }
                    }
                    Action::CopyTo(p) => {
                        if let Err(err) = app.copy_selected_to(p) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0 };
                        }
                    }
                    Action::MoveTo(p) => {
                        if let Err(err) = app.move_selected_to(p) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0 };
                        }
                    }
                    Action::RenameTo(name) => {
                        if let Err(err) = app.rename_selected_to(name) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0 };
                        }
                    }
                    Action::NewFile(name) => {
                        if let Err(err) = app.new_file(name) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0 };
                        }
                    }
                    Action::NewDir(name) => {
                        if let Err(err) = app.new_dir(name) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0 };
                        }
                    }
                }
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                app.mode = Mode::Normal;
            }
            _ => {}
        }
    }
    Ok(false)
}
