use anyhow::Result;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

// Integration test: ensure the bottom help bar renders expected text.
#[test]
fn help_bar_renders() -> Result<()> {
    // Create a test backend (80x24) and terminal wrapper.
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    // Construct an application instance using the real constructor.
    // This uses the current working directory for panels which is fine
    // for an integration test running in the repo workspace.
    let app = fileZoom::App::new()?;

    // Draw the UI once into the test backend.
    terminal.draw(|f| fileZoom::ui::ui(f, &app))?;

    // Drawing succeeded if we reach here; a more thorough inspection of
    // the backend buffer may be added later once a stable TestBackend API
    // accessor is standard across ratatui versions. For now ensure draw() runs.

    Ok(())
}
