use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::types::Entry;

use crate::ui::colors::current as theme_current;

/// Format the key file statistics for a lightweight text rendering.
pub fn format_file_stats(e: &Entry) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("Name: {}", e.name));
    lines.push(format!("Path: {}", e.path.display()));
    lines.push(if e.is_dir { "Type: directory".to_string() } else { "Type: file".to_string() });
    lines.push(format!("Size: {} bytes", e.size));
    let modified = e
        .modified
        .as_ref()
        .map(|dt| dt.to_string())
        .unwrap_or_else(|| "(unknown)".into());
    lines.push(format!("Modified: {}", modified));
    lines
}

/// Draw a simple file-stats view into `area` for an entry. This is intentionally
/// small and non-interactive — it provides a stable fallback rendering when a
/// textual preview is unavailable.
pub fn draw_file_stats(f: &mut Frame, area: Rect, entry: &Entry) {
    let theme = theme_current();
    let lines = format_file_stats(entry).join("\n");
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Details")
        .style(theme.preview_block_style);

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::Terminal;
    use chrono::Local;

    #[test]
    fn format_file_stats_contains_expected_fields() {
        let entry = Entry::file("foo.txt", std::path::PathBuf::from("/tmp/foo.txt"), 42, Some(Local::now()));
        let lines = format_file_stats(&entry);
        assert!(lines.iter().any(|l| l.starts_with("Name:")));
        assert!(lines.iter().any(|l| l.starts_with("Path:")));
        assert!(lines.iter().any(|l| l.starts_with("Size:")));
    }

    #[test]
    fn draw_file_stats_smoke_test() {
        let backend = TestBackend::new(80, 12);
        let mut terminal = Terminal::new(backend).expect("failed to create terminal");

        let entry = Entry::file("bar.bin", std::path::PathBuf::from("/var/bar.bin"), 1024, None);

        terminal
            .draw(|f| {
                let area = Rect::new(0, 0, 80, 12);
                draw_file_stats(f, area, &entry);
            })
            .expect("failed to draw");
    }
}
//place holder for file_stats ui module to show file information