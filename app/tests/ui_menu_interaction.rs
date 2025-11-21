use fileZoom::app::core::App;
use fileZoom::input::KeyCode;
use fileZoom::runner::handlers::mouse::handle_mouse;
use fileZoom::ui::command_line::CommandLineState;
use ratatui::layout::Rect;

#[test]
fn menu_click_activates_item() {
    let mut app = App::new().unwrap();
    // ensure labels exist
    let labels = fileZoom::ui::menu::menu_labels();
    if labels.is_empty() {
        return;
    }
    // click roughly in the first third of the width to select index 0
    let width = 80u16;
    let col = 2u16;
    let me = fileZoom::input::mouse::MouseEvent {
        column: col,
        row: 0,
        kind: fileZoom::input::mouse::MouseEventKind::Down(
            fileZoom::input::mouse::MouseButton::Left,
        ),
    };
    let rect = Rect::new(0, 0, width, 24);
    let _ = handle_mouse(&mut app, me, rect).unwrap();
    // after activation the mode should be a Message with the menu title
    match app.mode {
        fileZoom::app::Mode::Message { ref title, .. } => {
            assert_eq!(title, &labels[app.menu_index]);
        }
        _ => panic!("Expected Message mode after menu activation"),
    }
}

#[test]
fn command_line_toggle_preview() {
    let mut app = App::new().unwrap();
    assert!(!app.preview_visible);
    app.command_line = Some(CommandLineState {
        visible: true,
        buffer: String::new(),
        cursor: 0,
    });
    // type `toggle-preview` character by character
    for c in "toggle-preview".chars() {
        let _ = fileZoom::ui::command_line::handle_input(&mut app, KeyCode::Char(c)).unwrap();
    }
    // press Enter
    let _ = fileZoom::ui::command_line::handle_input(&mut app, KeyCode::Enter).unwrap();
    assert!(app.preview_visible);
}
