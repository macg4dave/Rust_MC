use crate::app::{App, Mode};
use crate::input::KeyCode;
use crate::app::settings::keybinds;

pub fn handle_context_menu(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    match &mut app.mode {
        Mode::ContextMenu {
            title: _,
            options,
            selected,
            path: _,
        } => {
            if keybinds::is_left(&code) || keybinds::is_up(&code) {
                if *selected > 0 {
                    *selected -= 1;
                }
            } else if keybinds::is_right(&code) || keybinds::is_down(&code) {
                if *selected + 1 < options.len() {
                    *selected += 1;
                }
            } else if keybinds::is_char(&code, 'q') || keybinds::is_esc(&code) {
                app.mode = Mode::Normal;
            } else if keybinds::is_enter(&code) {
                // Snapshot the chosen option and dismiss the menu
                let choice = options.get(*selected).cloned();
                app.mode = Mode::Normal;
                if let Some(ch) = choice {
                    match ch.as_str() {
                        "View" | "Open" => {
                            app.preview_visible = true;
                            let _ = app.update_preview_for(app.active);
                        }
                        "Edit" => {
                            if let Some(e) = app.active_panel().selected_entry() {
                                let editor = std::env::var("EDITOR")
                                    .unwrap_or_else(|_| "vi".to_string());
                                let use_integrated = app.settings.prefer_integrated_vim
                                    || editor == "vi"
                                    || editor == "vim";

                                if use_integrated {
                                    match crate::app::text_editors::vim_support::spawn_vim(&e.path) {
                                        Ok(_) => {
                                            app.mode = Mode::Message {
                                                title: "Edit".to_string(),
                                                content: format!("Launched vim for: {}", e.name),
                                                buttons: vec!["OK".to_string()],
                                                selected: 0,
                                                actions: None,
                                            };
                                        }
                                        Err(_) => {
                                            app.mode = Mode::Message {
                                                title: "Edit".to_string(),
                                                content: "Failed to launch vim".to_string(),
                                                buttons: vec!["OK".to_string()],
                                                selected: 0,
                                                actions: None,
                                            };
                                        }
                                    }
                                } else {
                                    let cmd = format!("{} \"{}\"", editor, e.path.display());
                                    match std::process::Command::new("sh")
                                        .arg("-c")
                                        .arg(cmd)
                                        .spawn()
                                    {
                                        Ok(_) => {
                                            app.mode = Mode::Message {
                                                title: "Edit".to_string(),
                                                content: format!("Launched editor: {}", editor),
                                                buttons: vec!["OK".to_string()],
                                                selected: 0,
                                                actions: None,
                                            };
                                        }
                                        Err(_) => {
                                            app.mode = Mode::Message {
                                                title: "Edit".to_string(),
                                                content: "Failed to launch editor".to_string(),
                                                buttons: vec!["OK".to_string()],
                                                selected: 0,
                                                actions: None,
                                            };
                                        }
                                    }
                                }
                            } else {
                                app.mode = Mode::Message {
                                    title: "Edit".to_string(),
                                    content: "No entry selected".to_string(),
                                    buttons: vec!["OK".to_string()],
                                    selected: 0,
                                    actions: None,
                                };
                            }
                        }
                        "Permissions" | "Inspect Permissions" => {
                            if let Some(e) = app.active_panel().selected_entry() {
                                match std::fs::metadata(&e.path) {
                                    Ok(md) => {
                                        #[cfg(unix)]
                                        {
                                            use std::os::unix::fs::PermissionsExt;
                                            let mode = md.permissions().mode();
                                            app.mode = Mode::Message {
                                                title: "Permissions".to_string(),
                                                content: format!("{}: {:o}", e.name, mode),
                                                buttons: vec!["OK".to_string()],
                                                selected: 0,
                                                actions: None,
                                            };
                                        }
                                        #[cfg(not(unix))]
                                        {
                                            app.mode = Mode::Message {
                                                title: "Permissions".to_string(),
                                                content: format!(
                                                    "{}: (platform-specific metadata)",
                                                    e.name
                                                ),
                                                buttons: vec!["OK".to_string()],
                                                selected: 0,
                                                actions: None,
                                            };
                                        }
                                    }
                                    Err(_) => {
                                        app.mode = Mode::Message {
                                            title: "Permissions".to_string(),
                                            content: "Cannot read metadata".to_string(),
                                            buttons: vec!["OK".to_string()],
                                            selected: 0,
                                            actions: None,
                                        };
                                    }
                                }
                            } else {
                                app.mode = Mode::Message {
                                    title: "Permissions".to_string(),
                                    content: "No entry selected".to_string(),
                                    buttons: vec!["OK".to_string()],
                                    selected: 0,
                                    actions: None,
                                };
                            }
                        }
                        _ => {
                            app.mode = Mode::Message {
                                title: "Action".to_string(),
                                content: format!("Action '{}' not implemented", ch),
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                                actions: None,
                            };
                        }
                    }
                }
            }
        }
        _ => {}
    }
    Ok(false)
}
