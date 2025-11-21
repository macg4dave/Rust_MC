use ratatui::{layout::Rect, widgets::{Block, Paragraph, Borders}, Frame};
use crate::ui::{UIState, Theme};
use crate::ui::colors::current as current_colors;

pub fn render(f: &mut Frame, area: Rect, state: &UIState, _theme: &Theme) {
    let content = format!("Progress: {}% | {} items", state.progress, state.left_list.len());
    let colors = current_colors();
    let p = Paragraph::new(content).block(Block::default().borders(Borders::ALL).style(colors.footer_style));
    f.render_widget(p, area);
}
