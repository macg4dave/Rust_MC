use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;

use fileZoom::ui::panels::draw_preview;
use fileZoom::app::Panel;
use fileZoom::app::types::Entry;

#[test]
fn preview_fallback_to_file_stats() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("failed to create terminal");

    // Create a panel rooted at / (no parent row) with a single entry and
    // an empty textual preview so draw_preview should fall back to the
    // file-stats UI path.
    let mut panel = Panel::new(std::path::PathBuf::from("/"));
    panel.entries = vec![Entry::file("foo.txt", std::path::PathBuf::from("/foo.txt"), 1234, None)];
    panel.selected = 1; // header is 0 -> entry at 1 when no parent
    panel.preview = String::new();

    terminal
        .draw(|f| {
            let area = Rect::new(0, 0, 80, 24);
            draw_preview(f, area, &panel);
        })
        .expect("failed to draw");
}
