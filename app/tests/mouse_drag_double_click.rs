use fileZoom::app::App;
use fileZoom::input::mouse::{MouseEvent, MouseEventKind};
use fileZoom::runner::handlers;
use fileZoom::Entry;
use ratatui::layout::Rect;
use std::path::PathBuf;

/// Starting a drag outside a panel then dragging into it must not create selections.
#[test]
fn drag_start_outside_panel_creates_no_selection() {
    let cwd = PathBuf::from("/");
    let mut app = App {
        left: fileZoom::app::core::panel::Panel::new(cwd.clone()),
        right: fileZoom::app::core::panel::Panel::new(cwd.clone()),
        active: fileZoom::app::types::Side::Left,
        mode: fileZoom::app::types::Mode::Normal,
        sort: fileZoom::app::types::SortKey::Name,
        sort_order: fileZoom::app::types::SortOrder::Ascending,
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
    app.left.entries = (0..6)
        .map(|i| Entry::directory(format!("f{}", i), PathBuf::from(format!("/f{}", i)), None))
        .collect();
    app.left.selected = 0;
    app.left.clear_selections();

    let term = Rect::new(0, 0, 80, 24);
    // Start drag outside panel (row 1 is menu/header area), drag into row 5
    let down = MouseEvent {
        column: 2,
        row: 1,
        kind: MouseEventKind::Down(fileZoom::input::mouse::MouseButton::Left),
    };
    handlers::handle_mouse(&mut app, down, term).unwrap();

    let drag = MouseEvent {
        column: 2,
        row: 5,
        kind: MouseEventKind::Drag(fileZoom::input::mouse::MouseButton::Left),
    };
    handlers::handle_mouse(&mut app, drag, term).unwrap();

    let up = MouseEvent {
        column: 2,
        row: 5,
        kind: MouseEventKind::Up(fileZoom::input::mouse::MouseButton::Left),
    };
    handlers::handle_mouse(&mut app, up, term).unwrap();

    // No selections should be created because the drag didn't start inside the panel.
    assert!(app.left.selections.is_empty());
}

/// When double-click timeout is zero, two quick clicks should NOT trigger an enter.
#[test]
fn double_click_respected_by_timeout_zero() {
    let mut app = App::new().unwrap();
    app.left.entries = (0..1)
        .map(|i| Entry::directory(format!("d{}", i), PathBuf::from(format!("/d{}", i)), None))
        .collect();
    app.left.selected = 0;

    // disable double-click by setting timeout to zero
    app.settings.mouse_double_click_ms = 0;

    let term = Rect::new(0, 0, 80, 24);
    let me = MouseEvent {
        column: 2,
        row: 5,
        kind: MouseEventKind::Down(fileZoom::input::mouse::MouseButton::Left),
    };
    handlers::handle_mouse(&mut app, me, term).unwrap();
    // wait slightly longer than zero so the second click is outside the timeout
    std::thread::sleep(std::time::Duration::from_millis(2));
    handlers::handle_mouse(&mut app, me, term).unwrap();

    // With timeout 0 the second click should not be considered a double-click
    // so CWD should remain unchanged (not entered into the directory).
    assert!(!app.left.cwd.ends_with("/d0"));
}

/// Two clicks at different positions must not be treated as a double-click.
#[test]
fn double_click_different_positions_does_not_enter() {
    let mut app = App::new().unwrap();
    app.left.entries = (0..2)
        .map(|i| Entry::directory(format!("d{}", i), PathBuf::from(format!("/d{}", i)), None))
        .collect();
    app.left.selected = 0;
    app.settings.mouse_double_click_ms = 1000;

    let term = Rect::new(0, 0, 80, 24);
    let first = MouseEvent {
        column: 2,
        row: 5,
        kind: MouseEventKind::Down(fileZoom::input::mouse::MouseButton::Left),
    };
    let second = MouseEvent {
        column: 10,
        row: 6,
        kind: MouseEventKind::Down(fileZoom::input::mouse::MouseButton::Left),
    };
    handlers::handle_mouse(&mut app, first, term).unwrap();
    handlers::handle_mouse(&mut app, second, term).unwrap();

    // Ensure we did not enter either directory as a double-click
    assert!(!app.left.cwd.ends_with("/d0"));
    assert!(!app.left.cwd.ends_with("/d1"));
}
