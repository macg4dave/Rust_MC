use crate::app::{App, Mode, Side};
use crate::input::mouse::{MouseButton, MouseEvent, MouseEventKind};
use anyhow::Result;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use std::time::Instant;

/// Handle a terminal mouse event for the application UI.
///
/// Returns `Ok(true)` if the event was handled and the caller should redraw
/// without further processing, `Ok(false)` if the event was either not
/// handled or further processing is desired. Errors propagate as `anyhow::Error`.
pub fn handle_mouse(app: &mut App, me: MouseEvent, term_rect: Rect) -> Result<bool> {
    use crate::ui::menu;

    // Build vertical layout once; reused by several handlers.
    let chunks = split_vertical(term_rect);

    // Fast path: scroll events (wheel) affect the active panel under cursor.
    if matches!(me.kind, MouseEventKind::ScrollUp | MouseEventKind::ScrollDown) {
        let main_chunks = split_main(chunks[2]);
        return handle_scroll(app, &me, &main_chunks);
    }

    // If settings modal is active, prefer handling clicks in the modal.
    if let Mode::Settings { .. } = &mut app.mode {
        if handle_settings_modal(app, &me, term_rect)? {
            return Ok(true);
        }
    }

    // If a submenu is open allow clicks on the status row (below the menu)
    // to activate submenu entries (header area only provides one extra
    // row in the compact layout so map that row to the first item).
    if app.menu_state.open && me.row >= chunks[1].y && me.row < chunks[1].y + chunks[1].height {
        let labels = menu::menu_labels();
        if !labels.is_empty() && me.column < term_rect.width {
            let idx = crate::ui::menu_model::MenuState::index_for_x(me.column, term_rect.width as u16, &labels);
            if idx == app.menu_index {
                // Map the click within the header area to a submenu row index
                // (row offset inside the header area). If the submenu vector
                // exists, clamp the selected index and activate it on left click.
                if let Some(top) = crate::ui::menu_model::MenuModel::default_model().0.get(app.menu_index) {
                    if let Some(sub) = &top.submenu {
                        let row_off = (me.row as i32 - chunks[1].y as i32) as usize;
                        let sel = std::cmp::min(row_off, sub.len().saturating_sub(1));
                        if matches!(me.kind, MouseEventKind::Down(MouseButton::Left)) {
                            app.menu_state.submenu_index = Some(sel);
                            app.menu_activate();
                            return Ok(true);
                        } else {
                            // Non-click interactions set selection but do not activate
                            app.menu_state.submenu_index = Some(sel);
                        }
                    }
                }
            }
        }
    }

    // Menu bar row
    if me.row >= chunks[0].y && me.row < chunks[0].y + chunks[0].height {
        let width = term_rect.width as u16;
        let labels = menu::menu_labels();
        if !labels.is_empty() {
            // Map the click X coordinate to a label index using the model helper.
            let idx = crate::ui::menu_model::MenuState::index_for_x(me.column, width, &labels);
            app.menu_index = std::cmp::min(idx, labels.len().saturating_sub(1));
            app.menu_focused = true;

            // If submenu is open and the user clicked the row below the top
            // row try to activate the submenu entry. (Header layout gives
            // only one extra row so we map that to the first submenu item.)
            if app.menu_state.open && me.row == chunks[0].y + 1 && matches!(me.kind, MouseEventKind::Down(MouseButton::Left)) {
                if let Some(top) = crate::ui::menu_model::MenuModel::default_model().0.get(app.menu_index) {
                    if top.submenu.is_some() {
                        app.menu_state.submenu_index = Some(0usize);
                        app.menu_activate();
                        return Ok(true);
                    }
                }
            }

            if matches!(me.kind, MouseEventKind::Down(MouseButton::Left)) {
                // If the clicked top label has a submenu, toggle it; otherwise
                // activate immediately for backwards compatibility.
                let model = crate::ui::menu_model::MenuModel::default_model();
                if let Some(top) = model.0.get(app.menu_index) {
                    if top.submenu.is_some() {
                        app.toggle_menu(app.menu_index);
                    } else {
                        app.menu_activate();
                    }
                } else {
                    app.menu_activate();
                }
                return Ok(true);
            }
        }
        return Ok(false);
    }

    // Panels area
    let main_chunks = split_main(chunks[2]);

    // Try to handle direct clicks on panels (select, context menu, start drag, double-click)
    if me.column >= main_chunks[0].x
        && me.column < main_chunks[0].x + main_chunks[0].width
        && handle_panel_click(main_chunks[0], Side::Left, app, &me)?
    {
        return Ok(false);
    }
    if me.column >= main_chunks[1].x
        && me.column < main_chunks[1].x + main_chunks[1].width
        && handle_panel_click(main_chunks[1], Side::Right, app, &me)?
    {
        return Ok(false);
    }

    // Update drag while dragging
    if matches!(me.kind, MouseEventKind::Drag(MouseButton::Left)) && handle_drag_update(&main_chunks, app, &me)? {
        return Ok(false);
    }

    // Finish drag on left-button release
    if matches!(me.kind, MouseEventKind::Up(MouseButton::Left)) && app.drag_active && app.drag_button == Some(MouseButton::Left) {
            app.drag_active = false;
            app.drag_current = Some((me.column, me.row));
            app.drag_start = None;
            app.drag_button = None;
            return Ok(false);
        }

    Ok(false)
}

// --- Small helpers ---

fn split_vertical(term_rect: Rect) -> Vec<Rect> {
    let segs = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(term_rect);
    segs.iter().cloned().collect()
}

fn split_main(area: Rect) -> Vec<Rect> {
    let segs = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);
    segs.iter().cloned().collect()
}

fn list_height(area: Rect) -> usize {
    area.height.saturating_sub(2) as usize
}

fn handle_scroll(app: &mut App, me: &MouseEvent, main_chunks: &[Rect]) -> Result<bool> {
    if contained_in(me, main_chunks[0]) {
        app.active = Side::Left;
        let lh = list_height(main_chunks[0]);
        if matches!(me.kind, MouseEventKind::ScrollDown) {
            app.select_next(lh);
        } else {
            app.select_prev(lh);
        }
        return Ok(false);
    }

    if contained_in(me, main_chunks[1]) {
        app.active = Side::Right;
        let lh = list_height(main_chunks[1]);
        if matches!(me.kind, MouseEventKind::ScrollDown) {
            app.select_next(lh);
        } else {
            app.select_prev(lh);
        }
        return Ok(false);
    }

    Ok(false)
}

fn contained_in(me: &MouseEvent, area: Rect) -> bool {
    me.column >= area.x
        && me.column < area.x + area.width
        && me.row >= area.y
        && me.row < area.y + area.height
}

fn handle_settings_modal(app: &mut App, me: &MouseEvent, term_rect: Rect) -> Result<bool> {
    let rect = crate::ui::modal::centered_rect(term_rect, 60, 10);
    if !contained_in(me, rect) {
        return Ok(false);
    }

    // content rows start at rect.y + 1; footer buttons at rect.y + rect.height - 2
    let content_start = rect.y + 1;
    let footer_row = rect.y + rect.height.saturating_sub(2);

    if me.row >= content_start && me.row < footer_row {
        let clicked_line = (me.row - content_start) as usize;
        if matches!(me.kind, MouseEventKind::Down(MouseButton::Left)) {
            let sel = match clicked_line {
                0 => 0usize,
                1 => 1usize,
                _ => 0usize,
            };
            app.mode = Mode::Settings { selected: sel };
            if sel == 0 {
                app.settings.mouse_enabled = !app.settings.mouse_enabled;
            }
        } else {
            app.mode = Mode::Settings { selected: clicked_line };
        }
        return Ok(true);
    }

    if me.row == footer_row && matches!(me.kind, MouseEventKind::Down(MouseButton::Left)) {
        let mid = rect.x + rect.width / 2;
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
            app.mode = Mode::Normal;
        }
        return Ok(true);
    }

    Ok(false)
}

fn handle_panel_click(area: Rect, side: Side, app: &mut App, me: &MouseEvent) -> Result<bool> {
    // clickable rows are between header and footer
    if !(me.row > area.y && me.row < area.y + area.height - 1) {
        return Ok(false);
    }

    let clicked = (me.row as i32 - (area.y as i32 + 1)) as usize;
    {
        let panel_mut = app.panel_mut(side);
        let new_sel = panel_mut.offset.saturating_add(clicked);
        let max_rows = 1 + if panel_mut.cwd.parent().is_some() { 1 } else { 0 } + panel_mut.entries.len();
        panel_mut.selected = std::cmp::min(new_sel, max_rows.saturating_sub(1));
    }
    app.active = side;

    // Start drag on left-button down
    if matches!(me.kind, MouseEventKind::Down(MouseButton::Left)) {
        app.drag_active = true;
        app.drag_start = Some((me.column, me.row));
        app.drag_current = Some((me.column, me.row));
        app.drag_button = Some(MouseButton::Left);
    }

    // Double-click detection
    if matches!(me.kind, MouseEventKind::Down(MouseButton::Left)) && app.settings.mouse_enabled {
        if let (Some(prev_t), Some((pc, pr))) = (app.last_mouse_click_time, app.last_mouse_click_pos) {
            let elapsed = Instant::now().saturating_duration_since(prev_t);
            if pc == me.column && pr == me.row && elapsed.as_millis() <= app.settings.mouse_double_click_ms as u128 {
                let _ = app.enter();
                app.last_mouse_click_time = None;
                app.last_mouse_click_pos = None;
                return Ok(true);
            }
        }
        app.last_mouse_click_time = Some(Instant::now());
        app.last_mouse_click_pos = Some((me.column, me.row));
    }

    // Right-click: open context menu for selected entry
    if matches!(me.kind, MouseEventKind::Down(MouseButton::Right)) {
        if let Some(e) = app.panel_mut(side).selected_entry().cloned() {
            let options = if app.settings.context_actions.is_empty() {
                vec!["View".into(), "Edit".into(), "Permissions".into(), "Cancel".into()]
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

    // For drag/up events, don't mark consumed here so outer handler can process them.
    if matches!(me.kind, MouseEventKind::Drag(_) | MouseEventKind::Up(_)) {
        return Ok(false);
    }

    Ok(true)
}

fn handle_drag_update(main_chunks: &[Rect], app: &mut App, me: &MouseEvent) -> Result<bool> {
    let try_update = |area: Rect, side: Side, app: &mut App, me: &MouseEvent| -> bool {
        if !(me.column >= area.x && me.column < area.x + area.width) {
            return false;
        }
        if app.drag_active && app.drag_button == Some(MouseButton::Left) {
            app.drag_current = Some((me.column, me.row));
            let drag_start_opt = app.drag_start;
            let panel_mut = app.panel_mut(side);
            panel_mut.clear_selections();
            let header_count = 1usize;
            let parent_count = if panel_mut.cwd.parent().is_some() { 1usize } else { 0usize };
            if let Some((sc, sr)) = drag_start_opt {
                // ensure the drag started inside this panel area (both column and row)
                if sc >= area.x && sc < area.x + area.width && sr > area.y && sr < area.y + area.height - 1 {
                    let start_clicked = (sr as i32 - (area.y as i32 + 1)) as usize;
                    let start_ui = panel_mut.offset.saturating_add(start_clicked);
                    if start_ui >= header_count + parent_count {
                        let start_domain = start_ui - header_count - parent_count;
                        let cur_row = me.row;
                        if cur_row > area.y && cur_row < area.y + area.height - 1 {
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
            return true;
        }
        false
    };

    if try_update(main_chunks[0], Side::Left, app, me) {
        return Ok(true);
    }
    if try_update(main_chunks[1], Side::Right, app, me) {
        return Ok(true);
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::mouse::{MouseButton, MouseEvent, MouseEventKind};
    use ratatui::layout::Rect;

    #[test]
    fn contained_in_detects_points() {
        let r = Rect::new(5, 3, 10, 6);
        let me = MouseEvent { column: 6, row: 4, kind: MouseEventKind::Down(MouseButton::Left) };
        assert!(contained_in(&me, r));
        let me2 = MouseEvent { column: 4, row: 4, kind: MouseEventKind::Down(MouseButton::Left) };
        assert!(!contained_in(&me2, r));
    }
}
