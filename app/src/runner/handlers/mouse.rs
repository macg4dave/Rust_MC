use crate::app::{App, Mode, Side};
use crate::input::mouse::MouseEventKind;
use crate::input::MouseEvent;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn handle_mouse(app: &mut App, me: MouseEvent, term_rect: Rect) -> anyhow::Result<bool> {
    use crate::ui::menu;
    // Handle scroll and left-button down
    match me.kind {
        MouseEventKind::Down(_) | MouseEventKind::Up(_) | MouseEventKind::Drag(_) => {
            // proceed to click/drag handling below
        }
        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Min(0),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(term_rect);

            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(chunks[2]);

            let list_height = |area: Rect| -> usize { (area.height as usize).saturating_sub(2) };

            if me.column >= main_chunks[0].x
                && me.column < main_chunks[0].x + main_chunks[0].width
                && me.row >= main_chunks[0].y
                && me.row < main_chunks[0].y + main_chunks[0].height
            {
                app.active = Side::Left;
                let lh = list_height(main_chunks[0]);
                if matches!(me.kind, MouseEventKind::ScrollDown) {
                    app.next(lh);
                } else {
                    app.previous(lh);
                }
                return Ok(false);
            }

            if me.column >= main_chunks[1].x
                && me.column < main_chunks[1].x + main_chunks[1].width
                && me.row >= main_chunks[1].y
                && me.row < main_chunks[1].y + main_chunks[1].height
            {
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
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(term_rect);

    // If the settings modal is active, handle clicks inside it first.
    if let crate::app::Mode::Settings { selected: _ } = &mut app.mode {
        let rect = crate::ui::modal::centered_rect(term_rect, 60, 10);
        // clicked inside dialog area?
        if me.column >= rect.x
            && me.column < rect.x + rect.width
            && me.row >= rect.y
            && me.row < rect.y + rect.height
        {
            // content rows start at rect.y + 1; footer buttons at rect.y + rect.height - 2
            let content_start = rect.y + 1;
            let footer_row = rect.y + rect.height.saturating_sub(2);
            if me.row >= content_start && me.row < footer_row {
                let clicked_line = (me.row - content_start) as usize;
                // map click to selected index: 0 -> mouse_enabled, 1 -> timeout
                if matches!(
                    me.kind,
                    MouseEventKind::Down(crate::input::mouse::MouseButton::Left)
                ) {
                    // update selection and possibly toggle
                    let sel = match clicked_line {
                        0 => 0usize,
                        1 => 1usize,
                        _ => 0usize,
                    };
                    app.mode = crate::app::Mode::Settings { selected: sel };
                    if sel == 0 {
                        app.settings.mouse_enabled = !app.settings.mouse_enabled;
                    }
                } else {
                    // just move focus
                    app.mode = crate::app::Mode::Settings {
                        selected: clicked_line,
                    };
                }
                return Ok(true);
            }
            if me.row == footer_row {
                // determine which footer button was clicked; assume two buttons roughly left/right halves
                let mid = rect.x + rect.width / 2;
                if matches!(
                    me.kind,
                    MouseEventKind::Down(crate::input::mouse::MouseButton::Left)
                ) {
                    if me.column < mid {
                        // Save
                        match crate::app::settings::save_settings(&app.settings) {
                            Ok(_) => {
                                app.mode = Mode::Message {
                                    title: "Settings Saved".to_string(),
                                    content: "Settings persisted".to_string(),
                                    buttons: vec!["OK".to_string()],
                                    selected: 0,
                                    actions: None,
                                };
                            }
                            Err(e) => {
                                app.mode = Mode::Message {
                                    title: "Error".to_string(),
                                    content: format!("Failed to save settings: {}", e),
                                    buttons: vec!["OK".to_string()],
                                    selected: 0,
                                    actions: None,
                                };
                            }
                        }
                    } else {
                        // Cancel
                        app.mode = Mode::Normal;
                    }
                    return Ok(true);
                }
            }
            return Ok(false);
        }
    }

    if me.row >= chunks[0].y && me.row < chunks[0].y + chunks[0].height {
        let width = term_rect.width as usize;
        let labels = menu::menu_labels();
        if !labels.is_empty() {
            let idx = (me.column as usize * labels.len()) / width;
            app.menu_index = std::cmp::min(idx, labels.len().saturating_sub(1));
            app.menu_focused = true;
            use crate::input::mouse::MouseButton;
            use crate::input::mouse::MouseEventKind;
            if matches!(me.kind, MouseEventKind::Down(MouseButton::Left)) {
                // Activate the selected menu item on left-click
                app.menu_activate();
                return Ok(true);
            }
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
                let max_rows =
                    1 + if panel_mut.cwd.parent().is_some() {
                        1
                    } else {
                        0
                    } + panel_mut.entries.len();
                panel_mut.selected = std::cmp::min(new_sel, max_rows.saturating_sub(1));
            }
            app.active = side;
            // If left button down inside panel, start a drag selection
            {
                use crate::input::mouse::MouseButton as Btn;
                if matches!(me.kind, MouseEventKind::Down(Btn::Left)) {
                    app.drag_active = true;
                    app.drag_start = Some((me.column, me.row));
                    app.drag_current = Some((me.column, me.row));
                    app.drag_button = Some(Btn::Left);
                }
            }
            // Double-click detection: if mouse support enabled and two left-clicks
            // happened at same position within the configured timeout, treat as
            // enter (open directory) action.
            use std::time::Instant;
            if matches!(me.kind, MouseEventKind::Down(MouseButton::Left)) {
                if app.settings.mouse_enabled {
                    let now = Instant::now();
                    if let (Some(prev_t), Some((pc, pr))) =
                        (app.last_mouse_click_time, app.last_mouse_click_pos)
                    {
                        let elapsed = now.saturating_duration_since(prev_t);
                        if pc == me.column
                            && pr == me.row
                            && elapsed.as_millis() <= app.settings.mouse_double_click_ms as u128
                        {
                            // Double-click detected: attempt to enter the selection
                            let _ = app.enter();
                            // Clear last click so a subsequent click won't re-trigger
                            app.last_mouse_click_time = None;
                            app.last_mouse_click_pos = None;
                            return true;
                        }
                    }
                    // Not a double-click, record this click for future detection.
                    app.last_mouse_click_time = Some(Instant::now());
                    app.last_mouse_click_pos = Some((me.column, me.row));
                }
            }
            use crate::input::mouse::MouseButton;
            use crate::input::mouse::MouseEventKind;
            if matches!(me.kind, MouseEventKind::Down(MouseButton::Right)) {
                if let Some(e) = app.panel_mut(side).selected_entry().cloned() {
                    let options = if app.settings.context_actions.is_empty() {
                        vec![
                            "View".to_string(),
                            "Edit".to_string(),
                            "Permissions".to_string(),
                            "Cancel".to_string(),
                        ]
                    } else {
                        app.settings.context_actions.clone()
                    };
                    app.mode = Mode::ContextMenu {
                        title: format!("Actions: {}", e.name),
                        options,
                        selected: 0,
                        path: e.path.clone(),
                    };
                }
            }
            // For drag/up events we want outer handler to continue to allow
            // specialized drag handling. For other events (down/right) we
            // consider the click consumed.
            if matches!(me.kind, MouseEventKind::Drag(_) | MouseEventKind::Up(_)) {
                return false;
            }
            return true;
        }
        false
    };

    if me.column >= main_chunks[0].x && me.column < main_chunks[0].x + main_chunks[0].width {
        if handle_panel_click(main_chunks[0], Side::Left, app, &me) {
            return Ok(false);
        }
    }
    if me.column >= main_chunks[1].x && me.column < main_chunks[1].x + main_chunks[1].width {
        if handle_panel_click(main_chunks[1], Side::Right, app, &me) {
            return Ok(false);
        }
    }

    // Update drag selection while dragging, and finish drag on button release.
    use crate::input::mouse::MouseButton;
    if matches!(me.kind, MouseEventKind::Drag(MouseButton::Left)) {
        // determine which panel the cursor is over
        if me.column >= main_chunks[0].x && me.column < main_chunks[0].x + main_chunks[0].width {
            let area = main_chunks[0];
            let side = Side::Left;
            if app.drag_active && app.drag_button == Some(MouseButton::Left) {
                app.drag_current = Some((me.column, me.row));
                let drag_start_opt = app.drag_start;
                let panel_mut = app.panel_mut(side);
                panel_mut.clear_selections();
                let header_count = 1usize;
                let parent_count = if panel_mut.cwd.parent().is_some() {
                    1usize
                } else {
                    0usize
                };
                if let Some((sc, sr)) = drag_start_opt {
                    // ensure the drag started inside this panel area (both column and row)
                    if sc >= area.x
                        && sc < area.x + area.width
                        && sr >= area.y + 1
                        && sr < area.y + area.height - 1
                    {
                        let start_clicked = (sr as i32 - (area.y as i32 + 1)) as usize;
                        let start_ui = panel_mut.offset.saturating_add(start_clicked);
                        if start_ui >= header_count + parent_count {
                            let start_domain = start_ui - header_count - parent_count;
                            let cur_row = me.row;
                            if cur_row >= area.y + 1 && cur_row < area.y + area.height - 1 {
                                let cur_clicked = (cur_row as i32 - (area.y as i32 + 1)) as usize;
                                let cur_ui = panel_mut.offset.saturating_add(cur_clicked);
                                if cur_ui >= header_count + parent_count {
                                    let cur_domain = cur_ui - header_count - parent_count;
                                    let (lo, hi) = if start_domain <= cur_domain {
                                        (start_domain, cur_domain)
                                    } else {
                                        (cur_domain, start_domain)
                                    };
                                    for i in lo..=hi {
                                        if i < panel_mut.entries.len() {
                                            panel_mut.selections.insert(i);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                return Ok(false);
            }
        }
        if me.column >= main_chunks[1].x && me.column < main_chunks[1].x + main_chunks[1].width {
            let area = main_chunks[1];
            let side = Side::Right;
            if app.drag_active && app.drag_button == Some(MouseButton::Left) {
                app.drag_current = Some((me.column, me.row));
                let drag_start_opt = app.drag_start;
                let panel_mut = app.panel_mut(side);
                panel_mut.clear_selections();
                let header_count = 1usize;
                let parent_count = if panel_mut.cwd.parent().is_some() {
                    1usize
                } else {
                    0usize
                };
                if let Some((sc, sr)) = drag_start_opt {
                    // ensure the drag started inside this panel area (both column and row)
                    if sc >= area.x
                        && sc < area.x + area.width
                        && sr >= area.y + 1
                        && sr < area.y + area.height - 1
                    {
                        let start_clicked = (sr as i32 - (area.y as i32 + 1)) as usize;
                        let start_ui = panel_mut.offset.saturating_add(start_clicked);
                        if start_ui >= header_count + parent_count {
                            let start_domain = start_ui - header_count - parent_count;
                            let cur_row = me.row;
                            if cur_row >= area.y + 1 && cur_row < area.y + area.height - 1 {
                                let cur_clicked = (cur_row as i32 - (area.y as i32 + 1)) as usize;
                                let cur_ui = panel_mut.offset.saturating_add(cur_clicked);
                                if cur_ui >= header_count + parent_count {
                                    let cur_domain = cur_ui - header_count - parent_count;
                                    let (lo, hi) = if start_domain <= cur_domain {
                                        (start_domain, cur_domain)
                                    } else {
                                        (cur_domain, start_domain)
                                    };
                                    for i in lo..=hi {
                                        if i < panel_mut.entries.len() {
                                            panel_mut.selections.insert(i);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                return Ok(false);
            }
        }
    }

    if matches!(me.kind, MouseEventKind::Up(MouseButton::Left)) {
        if app.drag_active && app.drag_button == Some(MouseButton::Left) {
            app.drag_active = false;
            app.drag_current = Some((me.column, me.row));
            app.drag_start = None;
            app.drag_button = None;
            return Ok(false);
        }
    }

    Ok(false)
}
