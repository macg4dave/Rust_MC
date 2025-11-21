use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Clear, Gauge, Paragraph};
use ratatui::Frame;

use crate::ui::colors::current as theme_current;

/// Draw a compact inline progress modal centered in `area`.
/// Layout: Title block -> one inline row containing a left label and a right gauge -> message/footer
pub fn draw_progress_modal(
    f: &mut Frame,
    area: Rect,
    title: &str,
    processed: usize,
    total: usize,
    message: &str,
    cancelled: bool,
) {
    let rect = crate::ui::modal::centered_percent(area, 50, 20);
    let theme = theme_current();

    // clear background and draw outer block
    f.render_widget(Clear, rect);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(theme.preview_block_style);
    f.render_widget(block, rect);

    // content area inside the block (leave 1 cell margin)
    let content_rect = Rect::new(
        rect.x + 1,
        rect.y + 1,
        rect.width.saturating_sub(2),
        rect.height.saturating_sub(3),
    );

    if content_rect.height == 0 {
        return; // nothing to draw
    }

    // Split into header (1), body (min), footer (1)
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(content_rect);

    // Header: show processed/total (left) and percent (right)
    let indeterminate = total == 0;
    let percent = if indeterminate {
        0.0
    } else {
        (processed as f64) / (total as f64)
    };
    let header_text = if indeterminate {
        format!("{}/?", processed)
    } else {
        format!("{}/{}", processed, total)
    };
    // Add an icon to indicate progress state
    let icon = if cancelled { "⏸" } else { "⏳" };
    let header_para = Paragraph::new(format!("{} {}", icon, header_text))
        .block(Block::default())
        .style(theme.header_style);
    f.render_widget(header_para, vchunks[0]);

    // Inline row: split horizontally into label and gauge
    let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(vchunks[1]);

    // Left: message or short status
    let msg = if message.is_empty() { "" } else { message };
    let left_para = Paragraph::new(msg.to_string()).block(Block::default());
    f.render_widget(left_para, hchunks[0]);

    // Right: gauge or animated spinner for indeterminate progress
    if indeterminate {
        // Compute a spinner frame from system time.
        let spinner = spinner_frame();
        // For indeterminate, show an animated pseudo-ratio to give motion.
        let anim_ratio = animated_ratio();
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .ratio(anim_ratio)
            .label(format!("{} {}", spinner, format_pct(anim_ratio)));
        f.render_widget(gauge, hchunks[1]);
    } else {
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .ratio(percent)
            .label(format_pct(percent));
        f.render_widget(gauge, hchunks[1]);
    }

    // Footer: show cancelling hint or usage
    let footer = if cancelled {
        "Cancelling..."
    } else {
        "Press Esc to cancel."
    };
    let footer_para = Paragraph::new(footer.to_string())
        .block(Block::default())
        .style(theme.help_block_style);
    f.render_widget(footer_para, vchunks[2]);
}

fn spinner_frame() -> &'static str {
    // Simple 10-frame braille spinner for smooth animation
    const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let idx = (current_millis() / 120) as usize % FRAMES.len();
    FRAMES[idx]
}

fn animated_ratio() -> f64 {
    // Return a smoothly changing ratio in [0.0, 1.0] based on system time.
    let ms = current_millis() as f64;
    let phase = (ms % 2000.0) / 2000.0; // 2s cycle
                                        // Use a triangular wave to move back and forth
    let tri = if phase < 0.5 {
        phase * 2.0
    } else {
        2.0 - phase * 2.0
    };
    tri.clamp(0.0, 1.0)
}

fn current_millis() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Simple helper to produce a percent string for tests and external use.
pub fn format_pct(ratio: f64) -> String {
    format!("{}%", (ratio.clamp(0.0, 1.0) * 100.0).round() as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_pct_bounds() {
        assert_eq!(format_pct(0.0), "0%");
        assert_eq!(format_pct(0.5), "50%");
        assert_eq!(format_pct(1.0), "100%");
        assert_eq!(format_pct(1.5), "100%");
        assert_eq!(format_pct(-0.1), "0%");
    }
}
