use fileZoom::app::{App, Mode};
use fileZoom::runner::handlers;
use fileZoom::input::KeyCode;
use fileZoom::input::mouse::{MouseEvent, MouseEventKind, MouseButton};
use ratatui::layout::Rect;

#[test]
fn keyboard_open_submenu_and_activate() {
    let mut app = App::new().unwrap();
    // target the "New" top label (index 3 in default model)
    app.menu_index = 3;
    app.menu_focused = true;

    // Enter should open the submenu
    handlers::handle_key(&mut app, KeyCode::Enter, 10).unwrap();
    assert!(app.menu_state.open);

    // navigate down to second submenu item
    handlers::handle_key(&mut app, KeyCode::Down, 10).unwrap();
    assert_eq!(app.menu_state.submenu_index, Some(1));

    // activate (Enter) should create an input prompt for new dir
    handlers::handle_key(&mut app, KeyCode::Enter, 10).unwrap();
    match app.mode {
        Mode::Input { kind, .. } => {
            assert_eq!(kind, fileZoom::app::InputKind::NewDir);
        }
        other => panic!("expected Input mode, got: {:?}", other),
    }
}

#[test]
fn mouse_open_submenu_then_click_first_item_activates() {
    let mut app = App::new().unwrap();
    // approximate column that maps to label index 3 when width 80
    let term = Rect::new(0, 0, 80, 24);
    let click_top = MouseEvent { column: 35, row: 0, kind: MouseEventKind::Down(MouseButton::Left) };
    let res = handlers::handle_mouse(&mut app, click_top, term).unwrap();
    assert!(res);
    assert!(app.menu_state.open);

    // clicking the row beneath the top (row 1) activates the first submenu item
    let click_sub = MouseEvent { column: 35, row: 1, kind: MouseEventKind::Down(MouseButton::Left) };
    let res2 = handlers::handle_mouse(&mut app, click_sub, term).unwrap();
    assert!(res2);
    match app.mode {
        Mode::Input { kind, .. } => assert_eq!(kind, fileZoom::app::InputKind::NewFile),
        other => panic!("expected NewFile input mode, got: {:?}", other),
    }
}

#[test]
fn mouse_open_submenu_then_click_second_item_activates() {
    let mut app = App::new().unwrap();
    let term = Rect::new(0, 0, 80, 24);
    // open the top submenu by clicking the top label (approximate x)
    let click_top = MouseEvent { column: 35, row: 0, kind: MouseEventKind::Down(MouseButton::Left) };
    let _ = handlers::handle_mouse(&mut app, click_top, term).unwrap();
    assert!(app.menu_state.open);

    // clicking the second row inside the header should activate the second submenu item
    let click_sub = MouseEvent { column: 35, row: 2, kind: MouseEventKind::Down(MouseButton::Left) };
    let res2 = handlers::handle_mouse(&mut app, click_sub, term).unwrap();
    assert!(res2);
    match app.mode {
        Mode::Input { kind, .. } => assert_eq!(kind, fileZoom::app::InputKind::NewDir),
        other => panic!("expected NewDir input mode, got: {:?}", other),
    }
}

#[test]
fn menu_click_copy_starts_progress() {
    let mut app = App::new().unwrap();
    // click near the area that maps to the Copy label (index 1)
    let term = ratatui::layout::Rect::new(0, 0, 80, 24);
    let me = fileZoom::input::mouse::MouseEvent { column: 12, row: 0, kind: MouseEventKind::Down(MouseButton::Left) };
    // select a source path so copy has something to operate on
    app.left.entries = (0..1).map(|i| fileZoom::Entry::directory(format!("f{}", i), std::path::PathBuf::from(format!("/f{}", i)), None)).collect();
    app.left.selections.insert(0);
    let res = handlers::handle_mouse(&mut app, me, term).unwrap();
    assert!(res);
    // Copy is a direct action that should start a background progress
    assert!(matches!(app.mode, Mode::Progress { .. }));
}

#[test]
fn menu_enter_move_starts_progress_when_focused() {
    let mut app = App::new().unwrap();
    // move focus to the top menu and set index to Move (2)
    app.menu_index = 2;
    app.menu_focused = true;
    // ensure a source entry is selected so move has something to act on
    app.left.entries = (0..1).map(|i| fileZoom::Entry::directory(format!("d{}", i), std::path::PathBuf::from(format!("/d{}", i)), None)).collect();
    app.left.selections.insert(0);
    // pressing Enter should activate move action
    handlers::handle_key(&mut app, fileZoom::input::KeyCode::Enter, 10).unwrap();
    assert!(matches!(app.mode, Mode::Progress { .. }));
}

#[test]
fn menu_enter_sort_cycles_sort_key() {
    let mut app = App::new().unwrap();
    app.menu_index = 4; // Sort
    app.menu_focused = true;
    let prev = app.sort;
    handlers::handle_key(&mut app, fileZoom::input::KeyCode::Enter, 10).unwrap();
    // Sort should have advanced
    assert_eq!(app.sort, prev.next());
}
