use crate::app::{App, Action, Mode};
use crate::errors;
use crate::input::KeyCode;

pub fn handle_confirm(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    if let Mode::Confirm { on_yes, selected, msg: _ } = &mut app.mode {
        match code {
            KeyCode::Left => {
                // toggle between options (commonly 0 = Yes, 1 = No)
                if *selected > 0 { *selected -= 1 } else { *selected = 1 }
            }
            KeyCode::Right => { *selected = (*selected + 1) % 2; }
            KeyCode::Enter => {
                // act based on selection: 0 => yes, otherwise cancel
                if *selected == 0 {
                    let action = on_yes.clone();
                    app.mode = Mode::Normal;
                    match action {
                        Action::DeleteSelected => {
                            if let Err(err) = app.delete_selected() {
                                let msg = errors::render_io_error(&err, None, None, None);
                                app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0, actions: None };
                            }
                        }
                        Action::CopyTo(p) => {
                            if let Err(err) = app.copy_selected_to(p) {
                                let msg = errors::render_io_error(&err, None, None, None);
                                app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0, actions: None };
                            }
                        }
                        Action::MoveTo(p) => {
                            if let Err(err) = app.move_selected_to(p) {
                                let msg = errors::render_io_error(&err, None, None, None);
                                app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0, actions: None };
                            }
                        }
                        Action::RenameTo(name) => {
                            if let Err(err) = app.rename_selected_to(name) {
                                let msg = errors::render_io_error(&err, None, None, None);
                                app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0, actions: None };
                            }
                        }
                        Action::NewFile(name) => {
                            if let Err(err) = app.new_file(name) {
                                let msg = errors::render_io_error(&err, None, None, None);
                                app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0, actions: None };
                            }
                        }
                        Action::NewDir(name) => {
                            if let Err(err) = app.new_dir(name) {
                                let msg = errors::render_io_error(&err, None, None, None);
                                app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0, actions: None };
                            }
                        }
                    }
                } else {
                    // cancel
                    app.mode = Mode::Normal;
                }
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                // treat as immediate yes
                let action = on_yes.clone();
                app.mode = Mode::Normal;
                match action {
                    Action::DeleteSelected => {
                        if let Err(err) = app.delete_selected() {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0, actions: None };
                        }
                    }
                    Action::CopyTo(p) => {
                        if let Err(err) = app.copy_selected_to(p) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0, actions: None };
                        }
                    }
                    Action::MoveTo(p) => {
                        if let Err(err) = app.move_selected_to(p) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0, actions: None };
                        }
                    }
                    Action::RenameTo(name) => {
                        if let Err(err) = app.rename_selected_to(name) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0, actions: None };
                        }
                    }
                    Action::NewFile(name) => {
                        if let Err(err) = app.new_file(name) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0, actions: None };
                        }
                    }
                    Action::NewDir(name) => {
                        if let Err(err) = app.new_dir(name) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message { title: "Error".to_string(), content: msg, buttons: vec!["OK".to_string()], selected: 0, actions: None };
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
