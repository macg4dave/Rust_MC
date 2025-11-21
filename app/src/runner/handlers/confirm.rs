use crate::app::{Action, App, Mode};
use crate::errors;
use crate::input::KeyCode;
use crate::app::settings::keybinds;

pub fn handle_confirm(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    if let Mode::Confirm {
        on_yes,
        selected,
        msg: _,
    } = &mut app.mode
    {
        if keybinds::is_left(&code) {
            // toggle between options (commonly 0 = Yes, 1 = No)
            if *selected > 0 {
                *selected -= 1
            } else {
                *selected = 1
            }
        } else if keybinds::is_right(&code) {
            *selected = (*selected + 1) % 2;
        } else if keybinds::is_enter(&code) {
            // act based on selection: 0 => yes, otherwise cancel
            if *selected == 0 {
                let action = on_yes.clone();
                app.mode = Mode::Normal;
                match action {
                    Action::DeleteSelected => {
                        if let Err(err) = app.delete_selected() {
                            let msg = errors::render_fsop_error(&err, None, None, None);
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: msg,
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                                actions: None,
                            };
                        }
                    }
                    Action::CopyTo(p) => {
                        if let Err(err) = app.copy_selected_to(p) {
                            let msg = errors::render_fsop_error(&err, None, None, None);
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: msg,
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                                actions: None,
                            };
                        }
                    }
                    Action::MoveTo(p) => {
                        if let Err(err) = app.move_selected_to(p) {
                            let msg = errors::render_fsop_error(&err, None, None, None);
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: msg,
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                                actions: None,
                            };
                        }
                    }
                    Action::RenameTo(name) => {
                        if let Err(err) = app.rename_selected_to(name) {
                            let msg = errors::render_fsop_error(&err, None, None, None);
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: msg,
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                                actions: None,
                            };
                        }
                    }
                    Action::NewFile(name) => {
                        if let Err(err) = app.new_file(name) {
                            let msg = errors::render_fsop_error(&err, None, None, None);
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: msg,
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                                actions: None,
                            };
                        }
                    }
                    Action::NewDir(name) => {
                        if let Err(err) = app.new_dir(name) {
                            let msg = errors::render_fsop_error(&err, None, None, None);
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
            } else {
                // cancel
                app.mode = Mode::Normal;
            }
        } else if keybinds::is_char(&code, 'y') || keybinds::is_char(&code, 'Y') {
            // treat as immediate yes
            let action = on_yes.clone();
            app.mode = Mode::Normal;
            match action {
                Action::DeleteSelected => {
                    if let Err(err) = app.delete_selected() {
                        let msg = errors::render_fsop_error(&err, None, None, None);
                        app.mode = Mode::Message {
                            title: "Error".to_string(),
                            content: msg,
                            buttons: vec!["OK".to_string()],
                            selected: 0,
                            actions: None,
                        };
                    }
                }
                Action::CopyTo(p) => {
                    if let Err(err) = app.copy_selected_to(p) {
                        let msg = errors::render_fsop_error(&err, None, None, None);
                        app.mode = Mode::Message {
                            title: "Error".to_string(),
                            content: msg,
                            buttons: vec!["OK".to_string()],
                            selected: 0,
                            actions: None,
                        };
                    }
                }
                Action::MoveTo(p) => {
                    if let Err(err) = app.move_selected_to(p) {
                        let msg = errors::render_fsop_error(&err, None, None, None);
                        app.mode = Mode::Message {
                            title: "Error".to_string(),
                            content: msg,
                            buttons: vec!["OK".to_string()],
                            selected: 0,
                            actions: None,
                        };
                    }
                }
                Action::RenameTo(name) => {
                    if let Err(err) = app.rename_selected_to(name) {
                        let msg = errors::render_fsop_error(&err, None, None, None);
                        app.mode = Mode::Message {
                            title: "Error".to_string(),
                            content: msg,
                            buttons: vec!["OK".to_string()],
                            selected: 0,
                            actions: None,
                        };
                    }
                }
                Action::NewFile(name) => {
                    if let Err(err) = app.new_file(name) {
                        let msg = errors::render_fsop_error(&err, None, None, None);
                        app.mode = Mode::Message {
                            title: "Error".to_string(),
                            content: msg,
                            buttons: vec!["OK".to_string()],
                            selected: 0,
                            actions: None,
                        };
                    }
                }
                Action::NewDir(name) => {
                    if let Err(err) = app.new_dir(name) {
                        let msg = errors::render_fsop_error(&err, None, None, None);
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
        } else if keybinds::is_char(&code, 'n') || keybinds::is_esc(&code) {
            app.mode = Mode::Normal;
        }
    }
    Ok(false)
}
