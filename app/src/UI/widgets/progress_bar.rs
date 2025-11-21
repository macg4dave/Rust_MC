use ratatui::{layout::Rect, widgets::{Block, Gauge, Borders}, Frame};
use crate::ui::UIState;

pub fn render(f: &mut Frame, area: Rect, state: &UIState) {
    let g = Gauge::default().block(Block::default().borders(Borders::ALL)).percent(state.progress as u16);
    f.render_widget(g, area);
}
