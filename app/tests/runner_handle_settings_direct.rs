use fileZoom::app::core::App;
use fileZoom::app::Mode;
use fileZoom::input::KeyCode;
use fileZoom::runner::handlers::handle_settings;

/// Integration test that exercises `handle_settings` directly (keyboard
/// flow) by constructing an `App`, entering Settings mode and simulating
/// a few key presses. This complements the existing tests which dispatch
/// through `handle_key` and mouse handlers.
#[test]
fn settings_handle_settings_direct() {
    let mut app = App::new().unwrap();

    // Activate the Settings menu (same approach used by other tests).
    let labels = fileZoom::ui::menu::menu_labels();
    let idx = labels
        .iter()
        .position(|&s| s == "Settings")
        .expect("Settings label present");
    app.menu_index = idx;
    app.menu_activate();

    // Ensure we are in Settings mode initially.
    match &app.mode {
        Mode::Settings { selected } => assert_eq!(*selected, 0),
        _ => panic!("Expected Settings mode"),
    }

    // Press Enter to toggle mouse_enabled (default true -> false)
    handle_settings(&mut app, KeyCode::Enter).unwrap();
    assert!(!app.settings.mouse_enabled);

    // Move focus to timeout and increase it by 50ms
    handle_settings(&mut app, KeyCode::Down).unwrap();
    let before = app.settings.mouse_double_click_ms;
    handle_settings(&mut app, KeyCode::Right).unwrap();
    assert_eq!(app.settings.mouse_double_click_ms, (before + 50).min(5000));

    // Move to Save and press Enter; expect a Message modal announcing save
    handle_settings(&mut app, KeyCode::Down).unwrap();
    handle_settings(&mut app, KeyCode::Enter).unwrap();
    match &app.mode {
        Mode::Message { title, .. } => assert_eq!(title, "Settings Saved"),
        _ => panic!("Expected Message after saving settings"),
    }
}
