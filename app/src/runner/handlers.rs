use crate::app::{Action, App, InputKind, Mode, Side};
use crate::errors;
use crate::input::KeyCode;
use std::path::PathBuf;
use crate::input::MouseEvent;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

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

/// Handle mouse events given the terminal drawable area `term_rect`.
/// Currently supports left-button clicks to focus panels and select rows,
/// and clicks on the top menu to focus/activate menu tabs.
pub fn handle_mouse(app: &mut App, me: MouseEvent, term_rect: Rect) -> anyhow::Result<bool> {
    use crate::ui::menu;
    // Only handle left-button down for now
    if !matches!(me.kind, crate::input::mouse::MouseEventKind::Down(crate::input::mouse::MouseButton::Left)) {
        return Ok(false);
    }

    // Recompute the same layout used by `ui::ui` so clicks map to areas.
    // Top menu (1), status (1), main panes (min), bottom help (1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ].as_ref())
        .split(term_rect);

    // If click in the menu row
    if me.row >= chunks[0].y && me.row < chunks[0].y + chunks[0].height {
        let width = term_rect.width as usize;
        let labels = menu::menu_labels();
        if !labels.is_empty() {
            let idx = (me.column as usize * labels.len()) / width;
            app.menu_index = std::cmp::min(idx, labels.len().saturating_sub(1));
            app.menu_focused = true;
        }
        return Ok(false);
    }

    // If click in main pane area, determine which side and set active/selection
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[2]);

    // Helper to handle click inside a given panel rect
    let handle_panel_click = |area: Rect, side: Side, app: &mut App, me: &MouseEvent| {
        if me.row >= area.y + 1 && me.row < area.y + area.height - 1 {
            // compute clicked ui row (0-based relative to panel.offset)
            let clicked = (me.row as i32 - (area.y as i32 + 1)) as usize;
            let panel = app.panel_mut(side);
            let new_sel = panel.offset.saturating_add(clicked);
            // clamp selection against maximum rows (header + parent + entries - 1)
            let max_rows = 1 + if panel.cwd.parent().is_some() { 1 } else { 0 } + panel.entries.len();
            panel.selected = std::cmp::min(new_sel, max_rows.saturating_sub(1));
            app.active = side;
            return true;
        }
        false
    };

    if me.column >= main_chunks[0].x && me.column < main_chunks[0].x + main_chunks[0].width {
        if handle_panel_click(main_chunks[0], Side::Left, app, &me) { return Ok(false); }
    }
    if me.column >= main_chunks[1].x && me.column < main_chunks[1].x + main_chunks[1].width {
        if handle_panel_click(main_chunks[1], Side::Right, app, &me) { return Ok(false); }
    }

    Ok(false)
}

fn handle_normal(app: &mut App, code: KeyCode, page_size: usize) -> anyhow::Result<bool> {
    match code {
        KeyCode::Char('q') => return Ok(true),
        KeyCode::Down => app.next(page_size),
        KeyCode::Up => app.previous(page_size),
        KeyCode::PageDown => app.page_down(page_size),
        KeyCode::PageUp => app.page_up(page_size),
        KeyCode::Enter if !app.menu_focused => {
            let panel = app.active_panel_mut();
            // Header row
            if panel.selected == 0 {
                let prompt = format!("Change path (current: {}):", panel.cwd.display());
                app.mode = Mode::Input {
                    prompt,
                    buffer: String::new(),
                    kind: InputKind::ChangePath,
                };
            } else {
                // Parent row (if exists)
                let parent_count = if panel.cwd.parent().is_some() {
                    1usize
                } else {
                    0usize
                };
                if panel.selected == 1 && parent_count == 1 {
                    if let Err(err) = app.go_up() {
                        let msg = errors::render_io_error(&err, None, None, None);
                        app.mode = Mode::Message {
                            title: "Error".to_string(),
                            content: msg,
                            buttons: vec!["OK".to_string()],
                            selected: 0,
                        };
                    }
                } else if let Some(e) = panel.selected_entry().cloned() {
                    if let Err(err) = app.enter() {
                        let path_s = e.path.display().to_string();
                        let msg = errors::render_io_error(&err, Some(&path_s), None, None);
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
            let e_opt = panel.selected_entry();
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
            let e_opt = panel.selected_entry();
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
            let e_opt = panel.selected_entry();
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
        KeyCode::F(1) => {
            // Toggle menu focus
            app.menu_focused = !app.menu_focused;
        }
        KeyCode::Left if app.menu_focused => {
            app.menu_prev();
        }
        KeyCode::Right if app.menu_focused => {
            app.menu_next();
        }
        KeyCode::Enter if app.menu_focused => {
            app.menu_activate();
            app.menu_focused = false;
        }
        KeyCode::Esc if app.menu_focused => {
            app.menu_focused = false;
        }
        KeyCode::Home => {
            app.active_panel_mut().selected = 0;
        }
        KeyCode::End => {
            let panel = app.active_panel_mut();
            if !panel.entries.is_empty() {
                let header_count = 1usize;
                let parent_count = if panel.cwd.parent().is_some() {
                    1usize
                } else {
                    0usize
                };
                panel.selected =
                    header_count + parent_count + panel.entries.len().saturating_sub(1);
            }
        }
        KeyCode::Char('p') => { /* toggle preview behavior */ }
        KeyCode::Char('t') => {
            crate::ui::colors::toggle();
        }
        KeyCode::Char('?') => {
            // Show interactive help overlay
            let content = "Keys:\n\nq: quit\nF1: toggle menu focus\nLeft/Right: menu navigation when focused\nEnter: open/activate\nBackspace: up\nd: delete\nc: copy\nm: move\nn/N: new file/dir\nR: rename\ns/S: sort (toggle desc)\nTab: switch panels\n?: show this help\n".to_string();
            app.mode = Mode::Message {
                title: "Help".to_string(),
                content,
                buttons: vec!["OK".to_string()],
                selected: 0,
            };
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
