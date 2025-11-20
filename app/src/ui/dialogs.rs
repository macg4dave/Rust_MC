use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::Color;
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::Frame;

use crate::ui::colors::current as theme_current;

/// Draw a centered dialog with a title, content and a small buttons/footer line.
pub fn draw_confirm<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    prompt: &str,
    content: &str,
    buttons: &[&str],
    selected: usize,
) {
    let rect = crate::ui::modal::centered_rect(area, 60, 8);
    let theme = theme_current();

    let body = Paragraph::new(content.to_string())
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: true });

    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(prompt)
        .style(theme.preview_block_style);

    f.render_widget(title_block, rect);

    // content area inside the block (leave 1 cell margin)
    let content_rect = Rect::new(
        rect.x + 1,
        rect.y + 1,
        rect.width.saturating_sub(2),
        rect.height.saturating_sub(3),
    );
    f.render_widget(body, content_rect);

    // Render buttons with highlight for selected
    let mut btn_text = String::new();
    for (i, b) in buttons.iter().enumerate() {
        if i > 0 {
            btn_text.push_str("    ");
        }
        if i == selected {
            // will render full line then overlay highlight by rendering styled paragraph
            btn_text.push_str(&format!("[{}]", b));
        } else {
            btn_text.push_str(&format!(" {} ", b));
        }
    }

    let buttons_para = Paragraph::new(btn_text)
        .block(Block::default())
        .style(theme.help_block_style);
    let buttons_rect = Rect::new(
        rect.x + 1,
        rect.y + rect.height.saturating_sub(2),
        rect.width.saturating_sub(2),
        1,
    );
    f.render_widget(buttons_para, buttons_rect);
}

/// Draw a simple informational dialog with an OK hint.
pub fn draw_info<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    title: &str,
    content: &str,
    buttons: &[&str],
    selected: usize,
) {
    let rect = crate::ui::modal::centered_rect(area, 60, 8);
    let theme = theme_current();

    let body = Paragraph::new(content.to_string())
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: true });

    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(theme.preview_block_style);
    f.render_widget(title_block, rect);

    let content_rect = Rect::new(
        rect.x + 1,
        rect.y + 1,
        rect.width.saturating_sub(2),
        rect.height.saturating_sub(3),
    );
    f.render_widget(body, content_rect);

    // Render buttons similarly to confirm
    let mut btn_text = String::new();
    for (i, b) in buttons.iter().enumerate() {
        if i > 0 {
            btn_text.push_str("    ");
        }
        if i == selected {
            btn_text.push_str(&format!("[{}]", b));
        } else {
            btn_text.push_str(&format!(" {} ", b));
        }
    }

    let buttons_para = Paragraph::new(btn_text)
        .block(Block::default())
        .style(theme.help_block_style);
    let buttons_rect = Rect::new(
        rect.x + 1,
        rect.y + rect.height.saturating_sub(2),
        rect.width.saturating_sub(2),
        1,
    );
    f.render_widget(buttons_para, buttons_rect);
}

/// Draw an error dialog; styled like info but reserved for errors.
pub fn draw_error<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    title: &str,
    content: &str,
    buttons: &[&str],
    selected: usize,
) {
    // Style title with red foreground for errors
    let rect = crate::ui::modal::centered_rect(area, 60, 8);
    let theme = theme_current();
    let title_style = theme.preview_block_style.fg(Color::Red);

    let body = Paragraph::new(content.to_string())
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: true });

    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(title_style);
    f.render_widget(title_block, rect);

    let content_rect = Rect::new(
        rect.x + 1,
        rect.y + 1,
        rect.width.saturating_sub(2),
        rect.height.saturating_sub(3),
    );
    f.render_widget(body, content_rect);

    // Render buttons
    let mut btn_text = String::new();
    for (i, b) in buttons.iter().enumerate() {
        if i > 0 {
            btn_text.push_str("    ");
        }
        if i == selected {
            btn_text.push_str(&format!("[{}]", b));
        } else {
            btn_text.push_str(&format!(" {} ", b));
        }
    }
    let buttons_para = Paragraph::new(btn_text)
        .block(Block::default())
        .style(theme.help_block_style);
    let buttons_rect = Rect::new(
        rect.x + 1,
        rect.y + rect.height.saturating_sub(2),
        rect.width.saturating_sub(2),
        1,
    );
    f.render_widget(buttons_para, buttons_rect);
}
