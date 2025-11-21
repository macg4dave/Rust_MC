use ratatui::{layout::Rect, widgets::{Paragraph, Block, Borders}, text::Span};
use ratatui::Frame;
use crate::ui::{UIState, Theme};
use crate::ui::colors::current as current_colors;

pub fn render(f: &mut Frame, area: Rect, state: &UIState, _theme: &Theme) {
    let colors = current_colors();
    let p = Paragraph::new(Span::raw(" fileZoom — left/right panels | adaptive UI "))
        .block(Block::default().borders(Borders::ALL).title(" header ").style(colors.header_style))
        .style(colors.header_style);
    f.render_widget(p, area);

    // If the menu is open render the submenu inside the header area so the
    // submenu items appear just below the top menu bar. UIState currently
    // provides the selected top index and the submenu selected index.
    if state.menu_open {
        if let Some(model_top) = crate::ui::menu_model::MenuModel::default_model().0.get(state.menu_selected) {
            if let Some(sub) = &model_top.submenu {
                // Convert submenu items to owned strings for the widget.
                let labels: Vec<String> = sub.iter().map(|it| it.label.clone()).collect();
                // Render submenu using the submenu widget.
                crate::ui::widgets::submenu::render(f, area, &labels, state.menu_sub_selected);
            }
        }
    }
}
