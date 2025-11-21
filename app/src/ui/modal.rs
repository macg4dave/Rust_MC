use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::ui::colors::current as theme_current;

/// Compute a centered rectangle inside `area` with width `w` and height `h`.
pub fn centered_rect(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    let x = (area.width - w) / 2 + area.x;
    let y = (area.height - h) / 2 + area.y;
    Rect::new(x, y, w, h)
}

/// Draw a centered modal dialog with a prompt title and content.
pub fn draw_modal(f: &mut Frame, area: Rect, prompt: &str, content: &str) {
    let rect = centered_rect(area, 80, 10);
    let theme = theme_current();
    let p = Paragraph::new(content.to_string()).block(
        Block::default()
            .borders(Borders::ALL)
            .title(prompt)
            .style(theme.preview_block_style),
    );
    // Clear underlying area before drawing popup so it stands out (like ratatui popup example)
    f.render_widget(Clear, rect);
    f.render_widget(p, rect);
}

/// Create a centered rect using percentage of the available area.
pub fn centered_percent(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    // Compute width/height as percentages of available area and center the rect.
    let w = ((area.width as u32 * percent_x as u32) / 100) as u16;
    let h = ((area.height as u32 * percent_y as u32) / 100) as u16;
    let w = w.max(1).min(area.width);
    let h = h.max(1).min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}

/// Draw a generic popup using percentage-based sizing and clearing the background.
pub fn draw_popup(
    f: &mut Frame,
    area: Rect,
    percent_x: u16,
    percent_y: u16,
    title: &str,
    content: &str,
) {
    let rect = centered_percent(area, percent_x, percent_y);
    let theme = theme_current();
    f.render_widget(Clear, rect);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(theme.preview_block_style);
    let p = Paragraph::new(content.to_string()).block(block);
    f.render_widget(p, rect);
}
