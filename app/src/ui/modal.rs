use tui::backend::Backend;
use tui::layout::Rect;
use tui::widgets::{Block, Borders, Paragraph};
use tui::Frame;

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
pub fn draw_modal<B: Backend>(f: &mut Frame<B>, area: Rect, prompt: &str, content: &str) {
    let rect = centered_rect(area, 80, 10);
    let theme = theme_current();
    let p = Paragraph::new(content.to_string()).block(
        Block::default()
            .borders(Borders::ALL)
            .title(prompt)
            .style(theme.preview_block_style),
    );
    f.render_widget(p, rect);
}
