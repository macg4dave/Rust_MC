use fileZoom::app::core::App;
use fileZoom::input::KeyCode;
use fileZoom::runner::handlers;
use fileZoom::runner::handlers::mouse::handle_mouse;
use ratatui::layout::Rect;

#[test]
fn settings_keyboard_interaction() {
    let mut app = App::new().unwrap();
    let labels = fileZoom::ui::menu::menu_labels();
    let idx = labels
        .iter()
        .position(|&s| s == "Settings")
        .expect("Settings label present");
    app.menu_index = idx;
    app.menu_activate();
    // should be in Settings mode
    match &app.mode {
        fileZoom::app::Mode::Settings { selected } => {
            assert_eq!(*selected, 0);
        }
        _ => panic!("Expected Settings mode"),
    }

    // Toggle mouse_enabled (default true -> false)
    handlers::handle_key(&mut app, KeyCode::Enter, 10).unwrap();
    assert_eq!(app.settings.mouse_enabled, false);

    // Move focus to timeout and increase it
    handlers::handle_key(&mut app, KeyCode::Down, 10).unwrap();
    // increase by 50ms via Right
    let before = app.settings.mouse_double_click_ms;
    handlers::handle_key(&mut app, KeyCode::Right, 10).unwrap();
    assert_eq!(app.settings.mouse_double_click_ms, (before + 50).min(5000));

    // Move to Save and press Enter
    handlers::handle_key(&mut app, KeyCode::Down, 10).unwrap();
    handlers::handle_key(&mut app, KeyCode::Enter, 10).unwrap();
    match &app.mode {
        fileZoom::app::Mode::Message { title, .. } => {
            assert_eq!(title, "Settings Saved");
        }
        _ => panic!("Expected Message after saving settings"),
    }
}

#[test]
fn settings_mouse_click_toggle_and_save() {
    let mut app = App::new().unwrap();
    let labels = fileZoom::ui::menu::menu_labels();
    let idx = labels
        .iter()
        .position(|&s| s == "Settings")
        .expect("Settings label present");
    app.menu_index = idx;
    app.menu_activate();

    let area = Rect::new(0, 0, 80, 24);
    let rect = fileZoom::ui::modal::centered_rect(area, 60, 10);

    // Click the first content line (mouse_enabled)
    let me = fileZoom::input::mouse::MouseEvent {
        column: rect.x + 2,
        row: rect.y + 1,
        kind: fileZoom::input::mouse::MouseEventKind::Down(
            fileZoom::input::mouse::MouseButton::Left,
        ),
    };
    let _ = handle_mouse(&mut app, me, area).unwrap();
    assert_eq!(app.settings.mouse_enabled, false);

    // Click Save (footer left half)
    let footer_row = rect.y + rect.height.saturating_sub(2);
    let me2 = fileZoom::input::mouse::MouseEvent {
        column: rect.x + 1,
        row: footer_row,
        kind: fileZoom::input::mouse::MouseEventKind::Down(
            fileZoom::input::mouse::MouseButton::Left,
        ),
    };
    let _ = handle_mouse(&mut app, me2, area).unwrap();
    match &app.mode {
        fileZoom::app::Mode::Message { title, .. } => {
            assert_eq!(title, "Settings Saved");
        }
        _ => panic!("Expected Message after saving settings via mouse"),
    }
}
