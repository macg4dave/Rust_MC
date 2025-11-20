use crate::app::{Action, App, InputKind, Mode, Side};
use crate::errors;
use crate::input::KeyCode;
use std::path::PathBuf;

pub fn handle_key(app: &mut App, code: KeyCode, page_size: usize) -> anyhow::Result<bool> {
    match &mut app.mode {
        Mode::Normal => handle_normal(app, code, page_size),
        Mode::Message { .. } => {
            // Dismiss message on Enter, Esc, or any key
            match code {
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char(_) => app.mode = Mode::Normal,
                _ => {}
            }
            Ok(false)
        }
        Mode::Confirm { .. } => handle_confirm(app, code),
        Mode::Input { .. } => handle_input(app, code),
    }
}

fn handle_normal(app: &mut App, code: KeyCode, page_size: usize) -> anyhow::Result<bool> {
    match code {
        KeyCode::Char('q') => return Ok(true),
        KeyCode::Down => app.next(page_size),
        KeyCode::Up => app.previous(page_size),
        KeyCode::PageDown => app.page_down(page_size),
        KeyCode::PageUp => app.page_up(page_size),
        KeyCode::Enter => {
            let panel = app.active_panel_mut();
            let e_opt = panel.entries.get(panel.selected).cloned();

            if let Some(e) = e_opt {
                if crate::ui::panels::is_entry_header(&e) {
                    let prompt = format!("Change path (current: {}):", e.path.display());
                    app.mode = Mode::Input {
                        prompt,
                        buffer: String::new(),
                        kind: InputKind::ChangePath,
                    };
                } else if let Err(err) = app.enter() {
                    let path_s = e.path.display().to_string();
                    let msg = errors::render_io_error(&err, Some(&path_s), None, None);
                    app.mode = Mode::Message {
                        title: "Error".to_string(),
                        content: msg,
                        buttons: vec!["OK".to_string()],
                        selected: 0,
                    };
                }
            } else if let Err(err) = app.enter() {
                let msg = errors::render_io_error(&err, None, None, None);
                app.mode = Mode::Message {
                    title: "Error".to_string(),
                    content: msg,
                    buttons: vec!["OK".to_string()],
                    selected: 0,
                };
            }
        }
        KeyCode::Backspace => {
            if let Err(err) = app.go_up() {
                let msg = errors::render_io_error(&err, None, None, None);
                app.mode = Mode::Message {
                    title: "Error".to_string(),
                    content: msg,
                    buttons: vec!["OK".to_string()],
                    selected: 0,
                };
            }
        }
        KeyCode::Char('r') => {
            if let Err(err) = app.refresh() {
                let msg = errors::render_io_error(&err, None, None, None);
                app.mode = Mode::Message {
                    title: "Error".to_string(),
                    content: msg,
                    buttons: vec!["OK".to_string()],
                    selected: 0,
                };
            }
        }
        KeyCode::Char('d') => {
            let panel = app.active_panel_mut();
            let e_opt = panel.entries.get(panel.selected);
            if let Some(e) = e_opt {
                let msg = format!("Delete {}? (y/n)", e.name);
                app.mode = Mode::Confirm {
                    msg,
                    on_yes: Action::DeleteSelected,
                    selected: 0,
                };
            }
        }
        KeyCode::Char('c') => {
            let panel = app.active_panel_mut();
            let e_opt = panel.entries.get(panel.selected);
            if let Some(e) = e_opt {
                let prompt = format!("Copy {} to:", e.name);
                app.mode = Mode::Input {
                    prompt,
                    buffer: String::new(),
                    kind: InputKind::Copy,
                };
            }
        }
        KeyCode::Char('m') => {
            let panel = app.active_panel_mut();
            let e_opt = panel.entries.get(panel.selected);
            if let Some(e) = e_opt {
                let prompt = format!("Move {} to:", e.name);
                app.mode = Mode::Input {
                    prompt,
                    buffer: String::new(),
                    kind: InputKind::Move,
                };
            }
        }
        KeyCode::Char('n') => {
            app.mode = Mode::Input {
                prompt: "New file name:".to_string(),
                buffer: String::new(),
                kind: InputKind::NewFile,
            };
        }
        KeyCode::Char('N') => {
            app.mode = Mode::Input {
                prompt: "New dir name:".to_string(),
                buffer: String::new(),
                kind: InputKind::NewDir,
            };
        }
        KeyCode::Char('R') => {
            let panel = app.active_panel_mut();
            let e_opt = panel.entries.get(panel.selected);
            if let Some(e) = e_opt {
                let prompt = format!("Rename {} to:", e.name);
                app.mode = Mode::Input {
                    prompt,
                    buffer: String::new(),
                    kind: InputKind::Rename,
                };
            }
        }
        KeyCode::Char('s') => {
            app.sort = app.sort.next();
            app.refresh()?;
        }
        KeyCode::Char('S') => {
            app.sort_desc = !app.sort_desc;
            app.refresh()?;
        }
        KeyCode::Char(' ') => { /* reserved */ }
        KeyCode::Tab => {
            app.active = match app.active {
                Side::Left => Side::Right,
                Side::Right => Side::Left,
            };
        }
        KeyCode::Home => {
            app.active_panel_mut().selected = 0;
        }
        KeyCode::End => {
            let panel = app.active_panel_mut();
            if !panel.entries.is_empty() {
                panel.selected = panel.entries.len() - 1;
            }
        }
        KeyCode::Char('p') => { /* toggle preview behavior */ }
        KeyCode::Char('t') => {
            crate::ui::colors::toggle();
        }
        KeyCode::Char('>') => {
            let panel = app.active_panel_mut();
            panel.preview_offset = panel.preview_offset.saturating_add(5);
        }
        KeyCode::Char('<') => {
            let panel = app.active_panel_mut();
            panel.preview_offset = panel.preview_offset.saturating_sub(5);
        }
        _ => {}
    }
    Ok(false)
}

fn handle_confirm(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    if let Mode::Confirm { on_yes, .. } = &mut app.mode {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                // Clone the action out so we drop the borrow on app.mode
                let action = on_yes.clone();
                // leave normal mode before performing file operations
                app.mode = Mode::Normal;
                match action {
                    Action::DeleteSelected => {
                        if let Err(err) = app.delete_selected() {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: msg,
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                            };
                        }
                    }
                    Action::CopyTo(p) => {
                        if let Err(err) = app.copy_selected_to(p) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: msg,
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                            };
                        }
                    }
                    Action::MoveTo(p) => {
                        if let Err(err) = app.move_selected_to(p) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: msg,
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                            };
                        }
                    }
                    Action::RenameTo(name) => {
                        if let Err(err) = app.rename_selected_to(name) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: msg,
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                            };
                        }
                    }
                    Action::NewFile(name) => {
                        if let Err(err) = app.new_file(name) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: msg,
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                            };
                        }
                    }
                    Action::NewDir(name) => {
                        if let Err(err) = app.new_dir(name) {
                            let msg = errors::render_io_error(&err, None, None, None);
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: msg,
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                            };
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

fn handle_input(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    if let Mode::Input {
        prompt: _,
        buffer,
        kind,
    } = &mut app.mode
    {
        match code {
            KeyCode::Enter => {
                // Snapshot buffer and kind, then leave Input mode so we
                // can perform mutable operations on `app`.
                let input = buffer.clone();
                // `InputKind` is `Copy`, `Clone`, `Copy` so dereference directly
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
                            };
                        }
                    }
                }
            }
            KeyCode::Backspace => {
                buffer.pop();
            }
            KeyCode::Esc => {
                app.mode = Mode::Normal;
            }
            KeyCode::Char(c) => {
                buffer.push(c);
            }
            _ => {}
        }
    }
    Ok(false)
}
