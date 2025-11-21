use ratatui::{layout::Rect, widgets::{Block, Paragraph, Borders}, Frame};

/// Render a simple horizontal main menu above the header.
/// Active item is bracketed for emphasis. Uses header_style when available.
pub fn render(f: &mut Frame, area: Rect, active_index: usize, focused: bool) {
    let labels = crate::ui::menu::menu_labels();
    let mut parts: Vec<String> = Vec::new();
    for (i, l) in labels.iter().enumerate() {
        if i == active_index { parts.push(format!("[{}]", l)); } else { parts.push(l.to_string()); }
    }
    let content = parts.join(" | ");
    let colors = crate::ui::colors::current();
    let style = if focused { colors.menu_style } else { colors.menu_inactive_style };
    // If the allocated vertical height is too small to show the bordered
    // block (needs 3 rows for top border, content, bottom border), render
    // the content without borders so the active label remains visible.
    let p = if area.height >= 3 {
        Paragraph::new(content).block(Block::default().borders(Borders::ALL)).style(style)
    } else {
        Paragraph::new(content).style(style)
    };
    f.render_widget(p, area);
}
