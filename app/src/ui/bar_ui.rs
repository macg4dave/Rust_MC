use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Clear};
use ratatui::layout::{Rect, Layout, Constraint, Direction};
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
	let block = Block::default().borders(Borders::ALL).title(title).style(theme.preview_block_style);
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
		.constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)].as_ref())
		.split(content_rect);

	// Header: show processed/total (left) and percent (right)
	let percent = if total == 0 { 0.0 } else { (processed as f64) / (total as f64) };
	let header_text = format!("{}/{}", processed, total);
	let header_para = Paragraph::new(header_text).block(Block::default()).style(theme.header_style);
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

	// Right: gauge
	let gauge = Gauge::default()
		.block(Block::default().borders(Borders::NONE))
		.ratio(percent)
		.label(format_pct(percent));
	f.render_widget(gauge, hchunks[1]);

	// Footer: show cancelling hint or usage
	let footer = if cancelled { "Cancelling..." } else { "Press Esc to cancel." };
	let footer_para = Paragraph::new(footer.to_string()).block(Block::default()).style(theme.help_block_style);
	f.render_widget(footer_para, vchunks[2]);
}

/// Simple helper to produce a percent string for tests and external use.
pub fn format_pct(ratio: f64) -> String { format!("{}%", (ratio.clamp(0.0,1.0)*100.0).round() as u16) }

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