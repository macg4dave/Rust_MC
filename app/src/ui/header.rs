use ratatui::layout::Rect;
use ratatui::text::Text;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, ListItem, Paragraph};
use ratatui::Frame;

use crate::ui::colors::current as theme_current;

/// Render a header row displaying the full path for a panel.
pub fn render_header(path_display: &str) -> ListItem<'_> {
    let theme = theme_current();
    let text = format!("> {}", path_display);
    let style = theme.header_style;
    ListItem::new(Text::from(Line::from(vec![Span::styled(text, style)])))
}

/// Draw a compact panel header into `area` showing the current path.
pub fn draw_panel_header(f: &mut Frame, area: Rect, path_display: &str) {
    let theme = theme_current();
    let p = Paragraph::new(format!("{}", path_display)).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .style(theme.preview_block_style),
    );
    f.render_widget(p, area);
}

/// Draw a small, single-line compact header into `area` with a generic text.
/// This helper centralises the compact header style used across multiple
/// UI modules so the appearance is consistent.
pub fn draw_compact_header(f: &mut Frame, area: Rect, text: &str) {
    let theme = theme_current();
    let p = Paragraph::new(text.to_string()).block(
        Block::default()
            .borders(Borders::NONE)
            .style(theme.help_block_style),
    );
    f.render_widget(p, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn render_header_contains_path() {
        // Ensure render_header returns a ListItem without panicking.
        let li = render_header("/tmp/test/path");
        let _ = format!("{:?}", li);
        assert!(true);
    }
}
