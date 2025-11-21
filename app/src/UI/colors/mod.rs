use ratatui::style::{Style, Color};
use crate::ui::Theme;
use once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(Clone, Debug)]
pub struct Colors {
	pub panel_block_style: Style,
	pub panel_selected_style: Style,
	pub panel_inactive_selected_style: Style,
	pub header_style: Style,
	pub menu_style: Style,
	pub menu_inactive_style: Style,
	pub footer_style: Style,
	pub preview_block_style: Style,
	pub scrollbar_thumb_style: Style,
	pub scrollbar_track_style: Style,
}

static CURRENT: Lazy<Mutex<Colors>> = Lazy::new(|| Mutex::new(Colors {
	panel_block_style: Style::default(),
	panel_selected_style: Style::default(),
	panel_inactive_selected_style: Style::default(),
	header_style: Style::default(),
	footer_style: Style::default(),
	preview_block_style: Style::default(),
	menu_style: Style::default(),
	menu_inactive_style: Style::default(),
	scrollbar_thumb_style: Style::default(),
	scrollbar_track_style: Style::default(),
}));

pub fn set_theme(name: &str) {
	match name {
		"dark" => set_from_theme(&crate::ui::Theme::dark()),
		"light" => set_from_theme(&crate::ui::Theme::light()),
		_ => {}
	}
}

/// Derive concrete runtime Styles from the provided Theme and store them.
pub fn set_from_theme(theme: &Theme) {
	let mut g = CURRENT.lock().unwrap();
	let panels = theme.panels.clone().unwrap_or_default();

	let panel_bg = panels.panel_bg.unwrap_or(theme.bg);
	let panel_fg = panels.panel_fg.unwrap_or(theme.fg);
	// panel_border and panel_title_fg are available for future use

	let selected_bg = panels.selected_bg.unwrap_or(theme.accent);
	let selected_fg = panels.selected_fg.unwrap_or(theme.fg);
	let inactive_selected_bg = panels.inactive_selected_bg.unwrap_or(theme.bg);

	let header_bg = panels.header_bg.unwrap_or(theme.bg);
	let header_fg = panels.header_fg.unwrap_or(theme.fg);
	let menu_bg = panels.menu_bg.unwrap_or(theme.bg);
	let menu_fg = panels.menu_fg.unwrap_or(theme.fg);
	let footer_bg = panels.footer_bg.unwrap_or(theme.bg);
	let footer_fg = panels.footer_fg.unwrap_or(theme.fg);

	let preview_bg = panels.preview_bg.unwrap_or(theme.bg);
	let preview_fg = panels.preview_fg.unwrap_or(theme.fg);

	let scrollbar_thumb = panels.scrollbar_thumb_bg.unwrap_or(theme.accent);
	let scrollbar_track = panels.scrollbar_track_bg.unwrap_or(theme.bg);

	*g = Colors {
		panel_block_style: Style::default().fg(panel_fg).bg(panel_bg),
		panel_selected_style: Style::default().fg(selected_fg).bg(selected_bg),
		panel_inactive_selected_style: Style::default().fg(selected_fg).bg(inactive_selected_bg),
		header_style: Style::default().bg(header_bg).fg(header_fg),
		menu_style: Style::default().bg(menu_bg).fg(menu_fg),
		menu_inactive_style: Style::default().bg(menu_bg).fg(theme.fg),
		footer_style: Style::default().bg(footer_bg).fg(footer_fg),
		preview_block_style: Style::default().fg(preview_fg).bg(preview_bg),
		scrollbar_thumb_style: Style::default().bg(scrollbar_thumb),
		scrollbar_track_style: Style::default().bg(scrollbar_track),
	};
}

pub fn current() -> Colors { CURRENT.lock().unwrap().clone() }

pub fn toggle() {
	let cur = current();
	if cur.preview_block_style.bg == Some(Color::Black) {
		set_from_theme(&crate::ui::Theme::light())
	} else {
		set_from_theme(&crate::ui::Theme::dark())
	}
}
