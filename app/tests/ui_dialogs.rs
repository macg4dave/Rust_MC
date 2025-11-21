use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;

use fileZoom::ui::dialogs::Dialog;

#[test]
fn render_dialog_with_test_backend() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("failed to create terminal");
    terminal
        .draw(|f| {
            let area = Rect::new(0, 0, 80, 24);
            let dlg = Dialog::new("Title", "This is a test body.", &["Ok", "Cancel"], 0);
            dlg.draw(f, area, false);
        })
        .expect("failed to draw");

    // Basic smoke-test: rendering succeeds without panic. Detailed visual
    // assertions are intentionally omitted to keep the test resilient to
    // layout changes in CI.
}
