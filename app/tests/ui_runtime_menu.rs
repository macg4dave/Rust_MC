use anyhow::Result;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

// Ensure the runner UI entrypoint uses real App state and that a draw
// succeeds when the menu is focused. This exercises the `ui::ui` path
// used by the event loop.
#[test]
fn runtime_ui_respects_menu_state() -> Result<()> {
    let backend = TestBackend::new(80, 3);
    let mut terminal = Terminal::new(backend)?;

    // Create an application instance and set menu focused to true to
    // simulate a user opening the top-level menu at runtime.
    let mut app = fileZoom::App::new()?;

    app.menu_focused = true;
    app.menu_index = 1;

    // Draw using the runtime UI path. This used to render a static sample
    // state — verify that the draw runs using the real app and that the
    // top menu line in the backend buffer includes the expected active
    // (bracketed) menu label for index 1 ("Copy").
    terminal.draw(|f| fileZoom::ui::ui(f, &app))?;

    let buf = terminal.backend_mut().buffer();
    let width = buf.area().width as u16;
    // The menu is rendered as a bordered block 3 rows tall, so the
    // content (including the bracketed active item) appears on the
    // second row (index 1) rather than the top border row.
    // Layout can vary on very small terminals: instead of assuming a
    // fixed row index, check that the backend buffer contains the
    // expected active label anywhere in the visible area.
    let mut full = String::new();
    for y in 0..buf.area().height as u16 {
        for x in 0..width {
            if let Some(c) = buf.cell((x, y)) { full.push_str(c.symbol()); }
        }
        full.push('\n');
    }
    assert!(full.contains("[Copy]"), "runtime UI did not show active menu: {}", full);

    Ok(())
}
