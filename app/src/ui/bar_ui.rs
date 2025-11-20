use ratatui::widgets::{Block, Borders, Gauge};
use ratatui::layout::Rect;
use ratatui::Frame;

/// Draw a determinate progress gauge inside `area` with `label` and `ratio`
/// where `ratio` is in [0.0, 1.0]. Uses the UI theme for styling.
pub fn draw_progress_bar(f: &mut Frame, area: Rect, label: &str, ratio: f64) {
	let theme = crate::ui::colors::current();
	let pct = (ratio.clamp(0.0, 1.0) * 100.0).round() as u16;
	let gauge = Gauge::default()
		.block(Block::default().borders(Borders::NONE).title(label).style(theme.preview_block_style))
		.ratio(ratio)
		.label(format!("{}%", pct));
	f.render_widget(gauge, area);
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