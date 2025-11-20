use crate::app::{App, Mode};
use crate::input::KeyCode;

pub fn handle_context_menu(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    match &mut app.mode {
        Mode::ContextMenu { title: _, options, selected, path: _ } => {
            match code {
                KeyCode::Left | KeyCode::Up => {
                    if *selected > 0 { *selected -= 1; }
                }
                KeyCode::Right | KeyCode::Down => {
                    if *selected + 1 < options.len() { *selected += 1; }
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    app.mode = Mode::Normal;
                }
                KeyCode::Enter => {
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
                                    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
                                    let cmd = format!("{} \"{}\"", editor, e.path.display());
                                    match std::process::Command::new("sh").arg("-c").arg(cmd).spawn() {
                                        Ok(_) => {
                                            app.mode = Mode::Message {
                                                title: "Edit".to_string(),
                                                content: format!("Launched editor: {}", editor),
                                                buttons: vec!["OK".to_string()],
                                                selected: 0,
                                            };
                                        }
                                        Err(_) => {
                                            app.mode = Mode::Message {
                                                title: "Edit".to_string(),
                                                content: "Failed to launch editor".to_string(),
                                                buttons: vec!["OK".to_string()],
                                                selected: 0,
                                            };
                                        }
                                    }
                                } else {
                                    app.mode = Mode::Message {
                                        title: "Edit".to_string(),
                                        content: "No entry selected".to_string(),
                                        buttons: vec!["OK".to_string()],
                                        selected: 0,
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
                                                };
                                            }
                                            #[cfg(not(unix))]
                                            {
                                                app.mode = Mode::Message {
                                                    title: "Permissions".to_string(),
                                                    content: format!("{}: (platform-specific metadata)", e.name),
                                                    buttons: vec!["OK".to_string()],
                                                    selected: 0,
                                                };
                                            }
                                        }
                                        Err(_) => {
                                            app.mode = Mode::Message {
                                                title: "Permissions".to_string(),
                                                content: "Cannot read metadata".to_string(),
                                                buttons: vec!["OK".to_string()],
                                                selected: 0,
                                            };
                                        }
                                    }
                                } else {
                                    app.mode = Mode::Message {
                                        title: "Permissions".to_string(),
                                        content: "No entry selected".to_string(),
                                        buttons: vec!["OK".to_string()],
                                        selected: 0,
                                    };
                                }
                            }
                            _ => {
                                app.mode = Mode::Message {
                                    title: "Action".to_string(),
                                    content: format!("Action '{}' not implemented", ch),
                                    buttons: vec!["OK".to_string()],
                                    selected: 0,
                                };
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(false)
}
