use assert_fs::prelude::*;
use fileZoom::app::{App, Panel};
use fileZoom::input::KeyCode;

#[test]
fn f3_opens_context_menu_and_view_shows_preview() {
    let temp = assert_fs::TempDir::new().unwrap();
    let f = temp.child("file.txt");
    f.write_str("hello world").unwrap();

    let cwd = temp.path().to_path_buf();
    let mut app = App::new().unwrap();
    // point left panel to temp
    app.left = Panel::new(cwd.clone());
    app.right = Panel::new(cwd.clone());
    app.refresh().unwrap();

    // find index of file.txt
    let mut idx = None;
    for (i, e) in app.left.entries.iter().enumerate() {
        if e.name == "file.txt" {
            idx = Some(i);
            break;
        }
    }
    assert!(idx.is_some());
    let header_count = 1usize;
    let parent_count = if app.left.cwd.parent().is_some() {
        1usize
    } else {
        0usize
    };
    app.left.selected = header_count + parent_count + idx.unwrap();

    // Press F3 to open context menu
    fileZoom::runner::handlers::handle_key(&mut app, KeyCode::F(3), 10).unwrap();
    match app.mode {
        fileZoom::app::Mode::ContextMenu { .. } => {}
        _ => panic!("expected ContextMenu mode after F3"),
    }

    // Press Enter (default selected option 0 -> 'View')
    fileZoom::runner::handlers::handle_key(&mut app, KeyCode::Enter, 10).unwrap();

    // Preview should be visible and contain our file contents
    assert!(app.preview_visible);
    assert!(app.left.preview.contains("hello world"));

    temp.close().unwrap();
}

#[test]
fn right_click_opens_context_menu() {
    use fileZoom::input::mouse::{MouseButton, MouseEvent, MouseEventKind};
    use ratatui::layout::Rect;

    let temp = assert_fs::TempDir::new().unwrap();
    let f = temp.child("rfile.txt");
    f.write_str("hi").unwrap();

    let cwd = temp.path().to_path_buf();
    let mut app = App::new().unwrap();
    app.left = Panel::new(cwd.clone());
    app.refresh().unwrap();

    // right-click near left panel first entry (column 2, row 5 to hit the actual file row)
    let term = Rect::new(0, 0, 80, 24);
    let me = MouseEvent {
        column: 2,
        row: 5,
        kind: MouseEventKind::Down(MouseButton::Right),
    };
    fileZoom::runner::handlers::handle_mouse(&mut app, me, term).unwrap();
    match app.mode {
        fileZoom::app::Mode::ContextMenu { .. } => {}
        _ => panic!("expected ContextMenu mode after right-click"),
    }

    temp.close().unwrap();
}
