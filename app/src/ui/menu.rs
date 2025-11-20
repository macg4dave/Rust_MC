use ratatui::layout::Rect;
use ratatui::text::{Span, Line};
use ratatui::widgets::{Block, Tabs, Borders};
use ratatui::Frame;

use crate::app::App;
use crate::ui::colors::current as theme_current;

/// Return the ordered labels used for the top menu.
pub fn menu_labels() -> Vec<&'static str> {
    vec!["File", "Copy", "Move", "New", "Sort", "Help"]
}

/// Draw the top menu bar. The menu is currently static and non-interactive.
pub fn draw_menu(f: &mut Frame, area: Rect, app: &App) {
    let labels = menu_labels();
    let theme = theme_current();

    // Use Tabs to render a top menu; selection is driven by app.menu_index
    // Build display labels with small icons so tests can still assert
    // on the canonical `menu_labels()` values while the UI renders
    // a slightly richer label for the user.
    let display_labels: Vec<String> = labels
        .iter()
        .map(|t| match *t {
            "File" => "📁 File".to_string(),
            "Copy" => "📄 Copy".to_string(),
            "Move" => "✂️ Move".to_string(),
            "New" => "➕ New".to_string(),
            "Sort" => "↕ Sort".to_string(),
            "Help" => "❓ Help".to_string(),
            other => other.to_string(),
        })
        .collect();

    // Style each title so the currently selected tab is visibly highlighted
    let titles: Vec<Line> = display_labels
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let s = if i == app.menu_index {
                theme.highlight_style
            } else {
                theme.help_block_style
            };
            Line::from(Span::styled(t.as_str(), s))
        })
        .collect();
    let block = Block::default().borders(Borders::BOTTOM).style(theme.help_block_style);
    let mut tabs = Tabs::new(titles)
        .select(app.menu_index)
        .block(block);
    // Apply highlight style when the menu is focused so selection is obvious
    if app.menu_focused {
        tabs = tabs.highlight_style(theme.highlight_style);
    } else {
        tabs = tabs.highlight_style(theme.help_block_style);
    }
    f.render_widget(tabs, area);
}
