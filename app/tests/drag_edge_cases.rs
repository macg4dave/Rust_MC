use fileZoom::app::App;
use fileZoom::runner::handlers;
use fileZoom::Entry;
use fileZoom::input::mouse::{MouseEvent, MouseEventKind};
use ratatui::layout::{Rect, Layout, Constraint, Direction};
use std::path::PathBuf;

fn main_chunks(term_rect: Rect) -> [ratatui::layout::Rect; 2] {
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

    [main_chunks[0], main_chunks[1]]
}

#[test]
fn drag_does_not_select_across_panels() {
    let cwd = PathBuf::from("/");
    let mut app = App {
        left: fileZoom::app::core::panel::Panel::new(cwd.clone()),
        right: fileZoom::app::core::panel::Panel::new(cwd.clone()),
        active: fileZoom::app::types::Side::Left,
        mode: fileZoom::app::types::Mode::Normal,
        sort: fileZoom::app::types::SortKey::Name,
        sort_desc: false,
        menu_index: 0,
        menu_focused: false,
        preview_visible: false,
        command_line: None,
        settings: fileZoom::app::settings::write_settings::Settings::default(),
        op_progress_rx: None,
        op_cancel_flag: None,
        op_decision_tx: None,
        last_mouse_click_time: None,
        last_mouse_click_pos: None,
        drag_active: false,
        drag_start: None,
        drag_current: None,
        drag_button: None,
    };

    // populate entries for both panels
    app.left.entries = (0..6)
        .map(|i| Entry::directory(format!("l{}", i), PathBuf::from(format!("/l{}", i)), None))
        .collect();
    app.left.selected = 0;
    app.left.clear_selections();

    app.right.entries = (0..6)
        .map(|i| Entry::directory(format!("r{}", i), PathBuf::from(format!("/r{}", i)), None))
        .collect();
    app.right.selected = 0;
    app.right.clear_selections();

    let term = Rect::new(0, 0, 80, 24);
    let areas = main_chunks(term);
    let left_area = areas[0];
    let right_area = areas[1];

    // compute a row that corresponds to a domain entry in left panel
    let header_count = 1usize;
    let parent_count = if app.left.cwd.parent().is_some() { 1usize } else { 0usize };
    let first_domain_row = left_area.y + 1 + (header_count + parent_count) as u16;

    let down = MouseEvent { column: left_area.x + 2, row: first_domain_row, kind: MouseEventKind::Down(fileZoom::input::mouse::MouseButton::Left) };
    handlers::handle_mouse(&mut app, down, term).unwrap();

    // Drag into the right panel (should NOT select items in right panel)
    let drag = MouseEvent { column: right_area.x + 2, row: first_domain_row + 2, kind: MouseEventKind::Drag(fileZoom::input::mouse::MouseButton::Left) };
    handlers::handle_mouse(&mut app, drag, term).unwrap();
    let up = MouseEvent { column: right_area.x + 2, row: first_domain_row + 2, kind: MouseEventKind::Up(fileZoom::input::mouse::MouseButton::Left) };
    handlers::handle_mouse(&mut app, up, term).unwrap();

    // right panel should have no selections
    assert!(app.right.selections.is_empty(), "right panel should not be selected by cross-panel drag");
}

#[test]
fn drag_with_parent_row_present_selects_correct_domain_indices() {
    let cwd = PathBuf::from("/tmp/somewhere"); // has a parent
    let mut app = App {
        left: fileZoom::app::core::panel::Panel::new(cwd.clone()),
        right: fileZoom::app::core::panel::Panel::new(PathBuf::from("/")),
        active: fileZoom::app::types::Side::Left,
        mode: fileZoom::app::types::Mode::Normal,
        sort: fileZoom::app::types::SortKey::Name,
        sort_desc: false,
        menu_index: 0,
        menu_focused: false,
        preview_visible: false,
        command_line: None,
        settings: fileZoom::app::settings::write_settings::Settings::default(),
        op_progress_rx: None,
        op_cancel_flag: None,
        op_decision_tx: None,
        last_mouse_click_time: None,
        last_mouse_click_pos: None,
        drag_active: false,
        drag_start: None,
        drag_current: None,
        drag_button: None,
    };

    // populate left entries
    app.left.entries = (0..8)
        .map(|i| Entry::directory(format!("f{}", i), PathBuf::from(format!("/f{}", i)), None))
        .collect();
    app.left.selected = 0;
    app.left.clear_selections();

    let term = Rect::new(0, 0, 80, 24);
    let left_area = main_chunks(term)[0];

    let header_count = 1usize;
    let parent_count = if app.left.cwd.parent().is_some() { 1usize } else { 0usize };
    // first domain row (after header + parent)
    let first_domain_row = left_area.y + 1 + (header_count + parent_count) as u16;

    // click and drag down two domain rows
    let down = MouseEvent { column: left_area.x + 2, row: first_domain_row, kind: MouseEventKind::Down(fileZoom::input::mouse::MouseButton::Left) };
    handlers::handle_mouse(&mut app, down, term).unwrap();
    let drag = MouseEvent { column: left_area.x + 2, row: first_domain_row + 2, kind: MouseEventKind::Drag(fileZoom::input::mouse::MouseButton::Left) };
    handlers::handle_mouse(&mut app, drag, term).unwrap();
    let up = MouseEvent { column: left_area.x + 2, row: first_domain_row + 2, kind: MouseEventKind::Up(fileZoom::input::mouse::MouseButton::Left) };
    handlers::handle_mouse(&mut app, up, term).unwrap();

    // Expect selections to cover domain indices 0..=2
    for i in 0..=2usize {
        assert!(app.left.selections.contains(&i), "expected selection to contain {}", i);
    }
}

#[test]
fn drag_with_panel_offset_respects_offset() {
    let cwd = PathBuf::from("/");
    let mut app = App {
        left: fileZoom::app::core::panel::Panel::new(cwd.clone()),
        right: fileZoom::app::core::panel::Panel::new(cwd.clone()),
        active: fileZoom::app::types::Side::Left,
        mode: fileZoom::app::types::Mode::Normal,
        sort: fileZoom::app::types::SortKey::Name,
        sort_desc: false,
        menu_index: 0,
        menu_focused: false,
        preview_visible: false,
        command_line: None,
        settings: fileZoom::app::settings::write_settings::Settings::default(),
        op_progress_rx: None,
        op_cancel_flag: None,
        op_decision_tx: None,
        last_mouse_click_time: None,
        last_mouse_click_pos: None,
        drag_active: false,
        drag_start: None,
        drag_current: None,
        drag_button: None,
    };

    // many entries so offset matters
    app.left.entries = (0..20)
        .map(|i| Entry::directory(format!("f{}", i), PathBuf::from(format!("/f{}", i)), None))
        .collect();
    app.left.selected = 0;
    app.left.clear_selections();
    app.left.offset = 3; // scrolled down

    let term = Rect::new(0, 0, 80, 24);
    let left_area = main_chunks(term)[0];

    let header_count = 1usize;
    let parent_count = if app.left.cwd.parent().is_some() { 1usize } else { 0usize };
    // choose the first visible displayed domain row (clicked = 0 visible domain)
    let clicked = 0usize;
    let click_row = left_area.y + 1 + (header_count + parent_count) as u16 + (clicked as u16);

    let down = MouseEvent { column: left_area.x + 2, row: click_row, kind: MouseEventKind::Down(fileZoom::input::mouse::MouseButton::Left) };
    handlers::handle_mouse(&mut app, down, term).unwrap();
    let drag = MouseEvent { column: left_area.x + 2, row: click_row + 2, kind: MouseEventKind::Drag(fileZoom::input::mouse::MouseButton::Left) };
    handlers::handle_mouse(&mut app, drag, term).unwrap();
    let up = MouseEvent { column: left_area.x + 2, row: click_row + 2, kind: MouseEventKind::Up(fileZoom::input::mouse::MouseButton::Left) };
    handlers::handle_mouse(&mut app, up, term).unwrap();

    // Calculate expected domain indices selected given offset
    // According to the handler logic the selected domain index equals offset + clicked
    let expected_first = app.left.offset + clicked;
    assert!(app.left.selections.contains(&expected_first), "expected selection to contain offset-adjusted domain index {}", expected_first);
}
