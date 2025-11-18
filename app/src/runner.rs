use crate::ui;
use crate::app::{Action, App, InputKind, Mode, Side, SortKey};
use crate::app::path;
use crate::ui::colors;

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crate::input::{poll, read_event, InputEvent, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use tui::backend::CrosstermBackend;
use tui::Terminal;

pub fn run_app() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;

    loop {
        terminal.draw(|f| ui::ui(f, &app))?;

        if poll(Duration::from_millis(100))? {
            let iev = read_event()?;
            match iev {
                InputEvent::Key(key) => {
                    let code = key.code;
                    match &mut app.mode {
                        Mode::Normal => {
                            match code {
                                KeyCode::Char('q') => break,
                                KeyCode::Down => {
                                    app.next((terminal.size()?.height as usize).saturating_sub(4))
                                }
                                KeyCode::Up => app
                                    .previous((terminal.size()?.height as usize).saturating_sub(4)),
                                KeyCode::PageDown => app.page_down(
                                    (terminal.size()?.height as usize).saturating_sub(4),
                                ),
                                KeyCode::PageUp => app
                                    .page_up((terminal.size()?.height as usize).saturating_sub(4)),
                                KeyCode::Enter => {
                                    let e_opt = match app.active {
                                        Side::Left => app.left.entries.get(app.left.selected),
                                        Side::Right => app.right.entries.get(app.right.selected),
                                    };
                                    if let Some(e) = e_opt {
                                        let cwd_display = match app.active {
                                            Side::Left => app.left.cwd.display().to_string(),
                                            Side::Right => app.right.cwd.display().to_string(),
                                        };
                                        if e.name == cwd_display {
                                            let prompt = format!("Change path (current: {}):", e.path.display());
                                            app.mode = Mode::Input {
                                                prompt,
                                                buffer: String::new(),
                                                kind: InputKind::ChangePath,
                                            };
                                        } else {
                                            let _ = app.enter();
                                        }
                                    } else {
                                        let _ = app.enter();
                                    }
                                }
                                KeyCode::Backspace => {
                                    let _ = app.go_up();
                                }
                                KeyCode::Char('r') => {
                                    let _ = app.refresh();
                                }
                                KeyCode::Char('d') => {
                                    let e_opt = match app.active {
                                        Side::Left => app.left.entries.get(app.left.selected),
                                        Side::Right => app.right.entries.get(app.right.selected),
                                    };
                                    if let Some(e) = e_opt {
                                        let msg = format!("Delete {}? (y/n)", e.name);
                                        app.mode = Mode::Confirm {
                                            msg,
                                            on_yes: Action::DeleteSelected,
                                        };
                                    }
                                }
                                KeyCode::Char('c') => {
                                    let e_opt = match app.active {
                                        Side::Left => app.left.entries.get(app.left.selected),
                                        Side::Right => app.right.entries.get(app.right.selected),
                                    };
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
                                    let e_opt = match app.active {
                                        Side::Left => app.left.entries.get(app.left.selected),
                                        Side::Right => app.right.entries.get(app.right.selected),
                                    };
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
                                    let e_opt = match app.active {
                                        Side::Left => app.left.entries.get(app.left.selected),
                                        Side::Right => app.right.entries.get(app.right.selected),
                                    };
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
                                    app.sort = match app.sort {
                                        SortKey::Name => SortKey::Size,
                                        SortKey::Size => SortKey::Modified,
                                        SortKey::Modified => SortKey::Name,
                                    };
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
                                KeyCode::Home => match app.active {
                                    Side::Left => app.left.selected = 0,
                                    Side::Right => app.right.selected = 0,
                                },
                                KeyCode::End => match app.active {
                                    Side::Left => {
                                        if !app.left.entries.is_empty() {
                                            app.left.selected = app.left.entries.len() - 1
                                        }
                                    }
                                    Side::Right => {
                                        if !app.right.entries.is_empty() {
                                            app.right.selected = app.right.entries.len() - 1
                                        }
                                    }
                                },
                                KeyCode::Char('p') => { /* toggle preview behavior */ }
                                KeyCode::Char('t') => {
                                    // Toggle theme between default and dark
                                    colors::toggle();
                                }
                                KeyCode::Char('>') => match app.active {
                                    Side::Left => {
                                        app.left.preview_offset =
                                            app.left.preview_offset.saturating_add(5)
                                    }
                                    Side::Right => {
                                        app.right.preview_offset =
                                            app.right.preview_offset.saturating_add(5)
                                    }
                                },
                                KeyCode::Char('<') => match app.active {
                                    Side::Left => {
                                        app.left.preview_offset =
                                            app.left.preview_offset.saturating_sub(5)
                                    }
                                    Side::Right => {
                                        app.right.preview_offset =
                                            app.right.preview_offset.saturating_sub(5)
                                    }
                                },
                                _ => {}
                            }
                        }
                        Mode::Confirm { msg: _, on_yes } => match code {
                            KeyCode::Char('y') => {
                                match on_yes {
                                    Action::DeleteSelected => {
                                        let _ = app.delete_selected();
                                    }
                                    _ => {}
                                }
                                app.mode = Mode::Normal;
                            }
                            KeyCode::Char('n') | KeyCode::Esc => {
                                app.mode = Mode::Normal;
                            }
                            _ => {}
                        },
                        Mode::Input {
                            prompt: _,
                            buffer,
                            kind,
                        } => match code {
                            KeyCode::Enter => {
                                let input = buffer.trim().to_string();
                                match kind {
                                    InputKind::Copy => {
                                        let dst = if input.starts_with('/') {
                                            PathBuf::from(&input)
                                        } else {
                                            match app.active {
                                                Side::Left => app.left.cwd.join(&input),
                                                Side::Right => app.right.cwd.join(&input),
                                            }
                                        };
                                        let _ = app.copy_selected_to(dst);
                                    }
                                    InputKind::Move => {
                                        let dst = if input.starts_with('/') {
                                            PathBuf::from(&input)
                                        } else {
                                            match app.active {
                                                Side::Left => app.left.cwd.join(&input),
                                                Side::Right => app.right.cwd.join(&input),
                                            }
                                        };
                                        let _ = app.move_selected_to(dst);
                                    }
                                    InputKind::Rename => {
                                        let _ = app.rename_selected_to(input);
                                    }
                                    InputKind::NewFile => {
                                        let _ = app.new_file(input);
                                    }
                                    InputKind::NewDir => {
                                        let _ = app.new_dir(input);
                                    }
                                    InputKind::ChangePath => {
                                        let base = match app.active {
                                            Side::Left => app.left.cwd.clone(),
                                            Side::Right => app.right.cwd.clone(),
                                        };
                                        match path::resolve_path(&input, &base) {
                                            Ok(p) => {
                                                match app.active {
                                                    Side::Left => app.left.cwd = p.clone(),
                                                    Side::Right => app.right.cwd = p.clone(),
                                                }
                                                let _ = app.refresh();
                                            }
                                            Err(e_msg) => {
                                                let prompt = format!("Invalid path: {}. Change path (current: {}):", e_msg, base.display());
                                                app.mode = Mode::Input {
                                                    prompt,
                                                    buffer: input.clone(),
                                                    kind: InputKind::ChangePath,
                                                };
                                            }
                                            
                                        }
                                    }
                                }
                                app.mode = Mode::Normal;
                            }
                            KeyCode::Char(c) => {
                                buffer.push(c);
                            }
                            KeyCode::Backspace => {
                                buffer.pop();
                            }
                            KeyCode::Esc => {
                                app.mode = Mode::Normal;
                            }
                            _ => {}
                        },
                    }
                }
                _ => {}
            }
        }
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
