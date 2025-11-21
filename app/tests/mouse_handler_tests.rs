use fileZoom::app::{App, Side};
use fileZoom::input::mouse::{MouseEvent, MouseEventKind};
use fileZoom::runner::handlers;
use fileZoom::Entry;
use ratatui::layout::Rect;
use std::path::PathBuf;

#[test]
fn scroll_down_over_left_panel_moves_selection_down() {
    let mut app = App::new().unwrap();
    // populate left entries
    app.left.entries = (0..10)
        .map(|i| Entry::directory(format!("f{}", i), PathBuf::from(format!("/f{}", i)), None))
        .collect();
    app.left.selected = 0;

    // terminal area 80x24 -> compute main panel rows dynamically
    let term = Rect::new(0, 0, 80, 24);
    let header_count = 1usize;
    let parent_count = if app.left.cwd.parent().is_some() { 1usize } else { 0usize };
    let first_domain_row = 4 + 1 + (header_count + parent_count) as u16;
    // click/scroll within left panel area (column 2, first domain row)
    let me = MouseEvent {
        column: 2,
        row: first_domain_row,
        kind: MouseEventKind::ScrollDown,
    };
    handlers::handle_mouse(&mut app, me, term).unwrap();
    assert_eq!(app.active, Side::Left);
    assert_eq!(app.left.selected, 1);
}

#[test]
fn scroll_up_over_left_panel_moves_selection_up() {
    let mut app = App::new().unwrap();
    app.left.entries = (0..10)
        .map(|i| Entry::directory(format!("f{}", i), PathBuf::from(format!("/f{}", i)), None))
        .collect();
    app.left.selected = 2;

    let term = Rect::new(0, 0, 80, 24);
    let header_count = 1usize;
    let parent_count = if app.left.cwd.parent().is_some() { 1usize } else { 0usize };
    let first_domain_row = 4 + 1 + (header_count + parent_count) as u16;
    let me = MouseEvent {
        column: 2,
        row: first_domain_row,
        kind: MouseEventKind::ScrollUp,
    };
    handlers::handle_mouse(&mut app, me, term).unwrap();
    assert_eq!(app.active, Side::Left);
    assert_eq!(app.left.selected, 1);
}

#[test]
fn scroll_down_over_right_panel_moves_selection_down() {
    let mut app = App::new().unwrap();
    app.right.entries = (0..10)
        .map(|i| Entry::directory(format!("r{}", i), PathBuf::from(format!("/r{}", i)), None))
        .collect();
    app.right.selected = 0;

    let term = Rect::new(0, 0, 80, 24);
    // choose a column in the right half (e.g., 60) and the first domain row
    let header_count = 1usize;
    let parent_count = if app.right.cwd.parent().is_some() { 1usize } else { 0usize };
    let first_domain_row = 4 + 1 + (header_count + parent_count) as u16;
    let me = MouseEvent {
        column: 60,
        row: first_domain_row,
        kind: MouseEventKind::ScrollDown,
    };
    handlers::handle_mouse(&mut app, me, term).unwrap();
    assert_eq!(app.active, Side::Right);
    assert_eq!(app.right.selected, 1);
}
