use ratatui::{layout::Rect, widgets::{List, ListItem, Block, Borders, ListState}};
use ratatui::Frame;
use crate::ui::Theme;

/// Render a file list for the given items and selected index. This is a
/// small widget intended to be reused for both left and right panels.
pub fn render(f: &mut Frame, area: Rect, items_src: &[String], selected: usize, _theme: &Theme) {
    let colors = crate::ui::colors::current();
    let items: Vec<ListItem> = items_src.iter().map(|s| ListItem::new(s.clone())).collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Files").style(colors.panel_block_style))
        .highlight_style(colors.panel_selected_style);
    // `selected` may be out of bounds for simple samples; `List` will ignore then.
    let mut state = ListState::default();
    state.select(if selected < items_src.len() { Some(selected) } else { None });
    f.render_stateful_widget(list, area, &mut state);
}
