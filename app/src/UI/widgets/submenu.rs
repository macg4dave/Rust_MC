use ratatui::{Frame, layout::Rect, widgets::{Block, Borders, List, ListItem}};
use crate::ui::colors::current as current_colors;

/// Render a small vertical submenu inside the header area when a top label
/// is open. The submenu will display items stacked top->down and highlight
/// the selected submenu index when present.
pub fn render(f: &mut Frame, area: Rect, labels: &Vec<String>, selected: Option<usize>) {
    if labels.is_empty() { return; }

    // Render the submenu as a simple list inside the supplied area. If the
    // area is too small this will naturally truncate.
    let colors = current_colors();
    let items: Vec<ListItem> = labels
        .iter()
        .enumerate()
        .map(|(i, s)| {
            if Some(i) == selected {
                ListItem::new(format!("> {}", s))
            } else {
                ListItem::new(s.clone())
            }
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("submenu")).style(colors.menu_inactive_style);
    f.render_widget(list, area);
}
