use tui::backend::Backend;
use tui::layout::Rect;

use tui::text::{Span, Spans};
use tui::widgets::{Block, Paragraph};
use tui::Frame;

use crate::app::App;
use crate::ui::colors::current as theme_current;

/// Return the ordered labels used for the top menu.
pub fn menu_labels() -> Vec<&'static str> {
    vec!["File", "Copy", "Move", "New", "Sort", "Help"]
}

/// Draw the top menu bar. The menu is currently static and non-interactive.
pub fn draw_menu<B: Backend>(f: &mut Frame<B>, area: Rect, _app: &App) {
    let menu_items = menu_labels();
    let mut parts: Vec<Span> = Vec::new();
    let theme = theme_current();
    for (i, it) in menu_items.iter().enumerate() {
        if i > 0 {
            parts.push(Span::raw("  "));
        }
        parts.push(Span::styled(*it, theme.help_block_style));
    }
    let spans = vec![Spans::from(parts)];
    let menu = Paragraph::new(spans).block(Block::default());
    f.render_widget(menu, area);
}
