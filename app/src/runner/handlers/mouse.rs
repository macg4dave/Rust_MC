use crate::app::{App, Side, Mode};
use crate::input::MouseEvent;
use crate::input::mouse::MouseEventKind;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn handle_mouse(app: &mut App, me: MouseEvent, term_rect: Rect) -> anyhow::Result<bool> {
    use crate::ui::menu;
    // Handle scroll and left-button down
    match me.kind {
        MouseEventKind::Down(crate::input::mouse::MouseButton::Left) |
        MouseEventKind::Down(crate::input::mouse::MouseButton::Right) => {
            // proceed to click handling below
        }
        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ].as_ref())
                .split(term_rect);

            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(chunks[2]);

            let list_height = |area: Rect| -> usize { (area.height as usize).saturating_sub(2) };

            if me.column >= main_chunks[0].x && me.column < main_chunks[0].x + main_chunks[0].width &&
               me.row >= main_chunks[0].y && me.row < main_chunks[0].y + main_chunks[0].height {
                app.active = Side::Left;
                let lh = list_height(main_chunks[0]);
                if matches!(me.kind, MouseEventKind::ScrollDown) {
                    app.next(lh);
                } else {
                    app.previous(lh);
                }
                return Ok(false);
            }

            if me.column >= main_chunks[1].x && me.column < main_chunks[1].x + main_chunks[1].width &&
               me.row >= main_chunks[1].y && me.row < main_chunks[1].y + main_chunks[1].height {
                app.active = Side::Right;
                let lh = list_height(main_chunks[1]);
                if matches!(me.kind, MouseEventKind::ScrollDown) {
                    app.next(lh);
                } else {
                    app.previous(lh);
                }
                return Ok(false);
            }

            return Ok(false);
        }
        _ => return Ok(false),
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ].as_ref())
        .split(term_rect);

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

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[2]);

    let handle_panel_click = |area: Rect, side: Side, app: &mut App, me: &MouseEvent| {
        if me.row >= area.y + 1 && me.row < area.y + area.height - 1 {
            let clicked = (me.row as i32 - (area.y as i32 + 1)) as usize;
            {
                let panel_mut = app.panel_mut(side);
                let new_sel = panel_mut.offset.saturating_add(clicked);
                let max_rows = 1 + if panel_mut.cwd.parent().is_some() { 1 } else { 0 } + panel_mut.entries.len();
                panel_mut.selected = std::cmp::min(new_sel, max_rows.saturating_sub(1));
            }
            app.active = side;
            use crate::input::mouse::MouseButton;
            use crate::input::mouse::MouseEventKind;
            if matches!(me.kind, MouseEventKind::Down(MouseButton::Right)) {
                if let Some(e) = app.panel_mut(side).selected_entry().cloned() {
                    let options = if app.settings.context_actions.is_empty() {
                        vec!["View".to_string(), "Edit".to_string(), "Permissions".to_string(), "Cancel".to_string()]
                    } else {
                        app.settings.context_actions.clone()
                    };
                    app.mode = Mode::ContextMenu { title: format!("Actions: {}", e.name), options, selected: 0, path: e.path.clone() };
                }
            }
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
