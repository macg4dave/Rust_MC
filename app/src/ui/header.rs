use tui::text::{Span, Spans};
use tui::widgets::ListItem;

use crate::ui::colors::current as theme_current;

/// Render a header row displaying the full path for a panel.
pub fn render_header(path_display: &str) -> ListItem<'_> {
    let theme = theme_current();
    let text = path_display.to_string();
    let style = theme.header_style;
    ListItem::new(Spans::from(vec![Span::styled(text, style)]))
}
