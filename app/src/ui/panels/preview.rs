use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::Frame;

use crate::app::Panel;

use crate::ui::colors::current as theme_current;

pub fn draw_preview(f: &mut Frame, area: Rect, panel: &Panel) {
    let max_lines = (area.height as usize).saturating_sub(2);
    let lines: Vec<&str> = panel.preview.lines().collect();
    // Resort to rendering the full preview into a temporary buffer and splicing a viewport
    // out of it as a convenience — this mirrors the pattern used in Ratatui examples.
    let _text = lines.iter().fold(String::new(), |mut acc, l| {
        acc.push_str(l);
        acc.push('\n');
        acc
    });
    let theme = theme_current();

    // Add a compact header above the preview and split remaining area
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(area);
    let header_area = vchunks[0];
    let area = vchunks[1];

    let header_text = format!("Preview — {} lines", lines.len());
    crate::ui::header::draw_compact_header(f, header_area, &header_text);

    // split area into main preview and a vertical scrollbar
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
        .split(area);
    // Render preview into a buffer and splice the visible region based on preview_offset
    let visible = if panel.preview_offset < lines.len() {
        &lines[panel.preview_offset..std::cmp::min(panel.preview_offset + max_lines, lines.len())]
    } else {
        &[]
    };
    let text = visible.iter().fold(String::new(), |mut acc, l| {
        acc.push_str(l);
        acc.push('\n');
        acc
    });
    // If we have textual preview content render it; otherwise fall back to a
    // concise file-details view for the selected entry (if present) or a
    // simple "no preview" placeholder.
    if !text.trim().is_empty() {
        let preview = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Preview")
                .style(theme.preview_block_style),
        );
        f.render_widget(preview, cols[0]);
    } else if let Some(entry) = panel.selected_entry() {
        // Draw a compact file details block into the preview region.
        crate::ui::file_stats_ui::draw_file_stats(f, cols[0], entry);
    } else {
        let preview = Paragraph::new("No preview available").block(
            Block::default()
                .borders(Borders::ALL)
                .title("Preview")
                .style(theme.preview_block_style),
        );
        f.render_widget(preview, cols[0]);
    }
    let max_lines = (cols[0].height as usize).saturating_sub(2);
    // Render scrollbar for preview using ratatui::widgets::Scrollbar
    let mut sb_state = ScrollbarState::new(lines.len())
        .position(panel.preview_offset)
        .viewport_content_length(max_lines);
    let sb = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .thumb_style(theme.scrollbar_thumb_style)
        .track_style(theme.scrollbar_style);
    f.render_stateful_widget(sb, cols[1], &mut sb_state);
}
