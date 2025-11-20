use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap, Clear};
use ratatui::Frame;

use crate::ui::colors::current as theme_current;

/// Draw the settings dialog. Shows a simple list of settings and footer
/// buttons Save/Cancel. `selected` indexes the currently focused row:
/// 0 = mouse_enabled, 1 = mouse_double_click_ms, 2 = Save, 3 = Cancel.
pub fn draw_settings(
    f: &mut Frame,
    area: Rect,
    app: &crate::app::App,
    selected: usize,
) {
    let rect = crate::ui::modal::centered_rect(area, 60, 10);
    let theme = theme_current();

    // Build body lines showing current settings; use styling to highlight the
    // currently focused row rather than a simple marker character.
    use ratatui::text::{Span, Line};
    let mouse_on = if app.settings.mouse_enabled { "On" } else { "Off" };
    let ms = app.settings.mouse_double_click_ms;

    let mut lines: Vec<Line> = Vec::new();
    let normal_style = theme.help_block_style;
    let highlight_style = theme.highlight_style;

    let line0 = Line::from(vec![
        Span::raw("Mouse support: "),
        Span::styled(mouse_on, if selected == 0 { highlight_style } else { normal_style }),
    ]);
    let line1 = Line::from(vec![
        Span::raw("Double-click timeout (ms): "),
        Span::styled(format!("{}", ms), if selected == 1 { highlight_style } else { normal_style }),
    ]);
    lines.push(line0);
    lines.push(line1);

    let body = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: true });

    let title_block = Block::default()
        .borders(Borders::ALL)
        .title("Settings")
        .style(theme.preview_block_style);

    f.render_widget(Clear, rect);
    f.render_widget(title_block, rect);

    let content_rect = Rect::new(
        rect.x + 1,
        rect.y + 1,
        rect.width.saturating_sub(2),
        rect.height.saturating_sub(3),
    );
    f.render_widget(body, content_rect);

    // Footer buttons Save / Cancel. Highlight according to selection index 2/3
    let buttons = ["Save", "Cancel"];
    // Map selection index into footer button index (2 -> 0, 3 -> 1). If selected < 2, no footer focus.
    let footer_selected = if selected >= 2 { selected - 2 } else { buttons.len() };
    render_buttons_with_options(
        f,
        rect,
        &buttons,
        footer_selected,
        theme.help_block_style,
        None,
        false,
    );
}

/// Move focus to the next element in a circular fashion.
pub fn move_focus_right(selected: usize, count: usize) -> usize {
    if count == 0 { 0 } else { (selected + 1) % count }
}

/// Move focus to the previous element in a circular fashion.
pub fn move_focus_left(selected: usize, count: usize) -> usize {
    if count == 0 { 0 } else { (selected + count - 1) % count }
}

/// Return the index that represents "accept" action (default: current selection).
pub fn accept_index(selected: usize) -> usize { selected }

/// Return the index that represents "cancel" action (default: last button).
pub fn cancel_index(buttons_len: usize) -> usize { buttons_len.saturating_sub(1) }

/// Render buttons with optional per-button styles and multiline support.
/// If `styles` is `None`, `style` is used for all.
pub fn render_buttons_with_options(
    f: &mut Frame,
    rect: Rect,
    buttons: &[&str],
    selected: usize,
    style: ratatui::style::Style,
    styles: Option<&[ratatui::style::Style]>,
    multiline: bool,
) {
    if !multiline {
        // simple single-line rendering
        render_buttons(f, rect, buttons, selected, style);
        return;
    }

    // multiline: render each button on its own centered line in the footer area
    let footer_height = (buttons.len() as u16).min(rect.height.saturating_sub(2));
    for (i, b) in buttons.iter().enumerate() {
        let idx = i as u16;
        let y = rect.y + rect.height.saturating_sub(2) - footer_height + idx;
        let mut btn_text = String::new();
        if i == selected { btn_text.push_str(&format!("[{}]", b)); } else { btn_text.push_str(&format!(" {} ", b)); }
        let btn_style = styles.and_then(|s| s.get(i)).cloned().unwrap_or(style);
        let para = Paragraph::new(btn_text).block(Block::default()).style(btn_style);
        let btn_rect = Rect::new(rect.x + 1, y, rect.width.saturating_sub(2), 1);
        f.render_widget(para, btn_rect);
    }
}

/// Translate a selected button index into an application `Action` if a mapping
/// is provided. This is a small helper used by handlers to map UI button
/// selections to domain actions.
pub fn selection_to_action(selected: usize, actions: Option<&[crate::app::Action]>) -> Option<crate::app::Action> {
    actions.and_then(|a| a.get(selected)).cloned()
}

/// A small reusable dialog representation for popups.
///
/// This struct is a lightweight, purely-presentational helper used by the
/// UI rendering code. It intentionally does not carry callbacks or domain
/// /// actions — input handling and action execution live in the runner/handler
/// code where ownership and application logic are available.
///
/// Example:
///
/// let dlg = Dialog::new("Title", "Body text", &["OK", "Cancel"], 0);
/// dlg.draw(f, area, false);
///

/// Draw a centered dialog with a title, content and a small buttons/footer line.
pub fn draw_confirm(
    f: &mut Frame,
    area: Rect,
    prompt: &str,
    content: &str,
    buttons: &[&str],
    selected: usize,
) {
    let dlg = Dialog::new(prompt, content, buttons, selected).width_percent(60).height(8);
    dlg.draw(f, area, false);
}

/// Draw a simple informational dialog with an OK hint.
pub fn draw_info(
    f: &mut Frame,
    area: Rect,
    title: &str,
    content: &str,
    buttons: &[&str],
    selected: usize,
) {
    let dlg = Dialog::new(title, content, buttons, selected).width_percent(60).height(8);
    dlg.draw(f, area, false);
}

/// Draw an error dialog; styled like info but reserved for errors.
pub fn draw_error(
    f: &mut Frame,
    area: Rect,
    title: &str,
    content: &str,
    buttons: &[&str],
    selected: usize,
) {
    let dlg = Dialog::new(title, content, buttons, selected).width_percent(60).height(8);
    dlg.draw(f, area, true);
}

/// A small reusable dialog representation for popups.
pub struct Dialog<'a> {
    title: &'a str,
    content: &'a str,
    buttons: Vec<&'a str>,
    selected: usize,
    width_percent: u16,
    height: u16,
}

impl<'a> Dialog<'a> {
    /// Create a new dialog
    pub fn new(title: &'a str, content: &'a str, buttons: &'a [&'a str], selected: usize) -> Self {
        Self {
            title,
            content,
            buttons: buttons.to_vec(),
            selected,
            width_percent: 60,
            height: 8,
        }
    }

    /// Set width as percent of available area
    pub fn width_percent(mut self, p: u16) -> Self { self.width_percent = p; self }

    /// Set explicit height
    pub fn height(mut self, h: u16) -> Self { self.height = h; self }

    /// Draw the dialog. If `is_error` is true, style the title with red.
    pub fn draw(&self, f: &mut Frame, area: Rect, is_error: bool) {
        let rect = crate::ui::modal::centered_percent(area, self.width_percent, self.height as u16 * 100 / area.height.max(1));
        let theme = theme_current();

        let title_style = if is_error {
            theme.preview_block_style.fg(Color::Red)
        } else {
            theme.preview_block_style
        };

        let body = Paragraph::new(self.content.to_string())
            .block(Block::default().borders(Borders::NONE))
            .wrap(Wrap { trim: true });

        let title_block = Block::default()
            .borders(Borders::ALL)
            .title(self.title)
            .style(title_style);

        // clear background then render dialog block so popup stands out
        f.render_widget(Clear, rect);
        f.render_widget(title_block, rect);

        // content area inside the block (leave 1 cell margin)
        let content_rect = Rect::new(
            rect.x + 1,
            rect.y + 1,
            rect.width.saturating_sub(2),
            rect.height.saturating_sub(3),
        );
        f.render_widget(body, content_rect);

        render_buttons(f, rect, &self.buttons, self.selected, theme.help_block_style);
    }
}

/// Render a horizontal buttons line inside `rect`, using `style` for unselected text.
fn render_buttons(f: &mut Frame, rect: Rect, buttons: &[&str], selected: usize, style: ratatui::style::Style) {
    let mut btn_text = String::new();
    for (i, b) in buttons.iter().enumerate() {
        if i > 0 { btn_text.push_str("    "); }
        if i == selected { btn_text.push_str(&format!("[{}]", b)); } else { btn_text.push_str(&format!(" {} ", b)); }
    }
    let buttons_para = Paragraph::new(btn_text).block(Block::default()).style(style);
    let buttons_rect = Rect::new(rect.x + 1, rect.y + rect.height.saturating_sub(2), rect.width.saturating_sub(2), 1);
    f.render_widget(buttons_para, buttons_rect);
}
