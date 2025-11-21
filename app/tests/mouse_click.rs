use fileZoom::app::{App, Mode, Side};
use fileZoom::input::mouse::{MouseEvent, MouseEventKind};
use fileZoom::runner::handlers;
use fileZoom::Entry;
use ratatui::layout::Rect;
use std::path::PathBuf;

#[test]
fn left_click_selects_entry_in_left_panel() {
    let mut app = App::new().unwrap();
    app.left.entries = (0..5)
        .map(|i| Entry::directory(format!("f{}", i), PathBuf::from(format!("/f{}", i)), None))
        .collect();
    app.left.selected = 0;

    let term = Rect::new(0, 0, 80, 24);
    // row 4 -> clicked index = row - (chunks[2].y + 1) == 4 - 3 == 1
        let header_count = 1usize;
        let parent_count = if app.left.cwd.parent().is_some() { 1usize } else { 0usize };
        let first_domain_row = 4 + 1 + (header_count + parent_count) as u16;
        let me = MouseEvent {
            column: 2,
            row: first_domain_row,
        kind: MouseEventKind::Down(fileZoom::input::mouse::MouseButton::Left),
    };
    handlers::handle_mouse(&mut app, me, term).unwrap();
    assert_eq!(app.active, Side::Left);
    // expected selected index is header + parent count
    let header_count = 1usize;
    let parent_count = if app.left.cwd.parent().is_some() { 1usize } else { 0usize };
    assert_eq!(app.left.selected, header_count + parent_count);
}

#[test]
fn right_click_opens_context_menu_for_selected_entry() {
    let mut app = App::new().unwrap();
    app.left.entries = (0..3)
        .map(|i| Entry::directory(format!("f{}", i), PathBuf::from(format!("/f{}", i)), None))
        .collect();
    app.left.selected = 0;

    let term = Rect::new(0, 0, 80, 24);
    // right-click the second item (account for parent row; click row 5)
        let header_count = 1usize;
        let parent_count = if app.left.cwd.parent().is_some() { 1usize } else { 0usize };
        let second_item_row = 4 + 1 + (header_count + parent_count) as u16 + 1;
        let me = MouseEvent {
            column: 2,
            row: second_item_row,
        kind: MouseEventKind::Down(fileZoom::input::mouse::MouseButton::Right),
    };
    handlers::handle_mouse(&mut app, me, term).unwrap();
    match &app.mode {
        Mode::ContextMenu {
            title: _,
            options,
            selected: sel,
            path: _,
        } => {
            assert_eq!(*sel, 0);
            assert!(!options.is_empty());
        }
        other => panic!("expected ContextMenu mode, got: {:?}", other),
    }
}

#[test]
fn clicking_top_menu_activates_menu_item() {
    let mut app = App::new().unwrap();
    // click near left side of top menu to select first label
    let term = Rect::new(0, 0, 80, 24);
    let me = MouseEvent {
        column: 2,
        row: 0,
        kind: MouseEventKind::Down(fileZoom::input::mouse::MouseButton::Left),
    };
    // handler returns Ok(true) when menu activation occurs; File has a submenu
    let res = handlers::handle_mouse(&mut app, me, term).unwrap();
    assert!(res);
    // File has a submenu in the new model so we expect the top menu to be open
    assert!(app.menu_focused);
    assert!(app.menu_state.open);
    assert_eq!(app.menu_index, 0);
}

#[test]
fn double_click_enters_directory_in_left_panel() {
    let mut app = App::new().unwrap();
    app.left.entries = (0..1)
        .map(|i| Entry::directory(format!("d{}", i), PathBuf::from(format!("/d{}", i)), None))
        .collect();
    app.left.selected = 0;
    // Make double-click timeout generous so test timing isn't flaky
    app.settings.mouse_double_click_ms = 1000;

    let term = Rect::new(0, 0, 80, 24);
    // click the first item: account for header+parent synthetic rows (row 5)
        let header_count = 1usize;
        let parent_count = if app.left.cwd.parent().is_some() { 1usize } else { 0usize };
        let first_domain_row = 4 + 1 + (header_count + parent_count) as u16;
        let me = MouseEvent {
            column: 2,
            row: first_domain_row,
        kind: MouseEventKind::Down(fileZoom::input::mouse::MouseButton::Left),
    };
    // first click selects
    handlers::handle_mouse(&mut app, me, term).unwrap();
    // second click within timeout should trigger enter()
    handlers::handle_mouse(&mut app, me, term).unwrap();

    // After double-click the left panel cwd should have changed to the entry path
    assert!(app.left.cwd.ends_with("/d0"));
}
