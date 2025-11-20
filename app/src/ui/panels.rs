use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Color};
use ratatui::text::{Span, Line, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, ListState, Scrollbar};
use ratatui::widgets::{ScrollbarState, ScrollbarOrientation};
use ratatui::widgets::canvas::{Canvas, Points as CanvasPoints, Line as CanvasLine};
// marker symbols no longer needed here
use ratatui::Frame;

use crate::app::{Entry, Panel};
use std::path::PathBuf;

/// UI-only wrapper around a domain `Entry` that carries presentation
/// metadata such as the preformatted display line and whether the row
/// is synthetic (header or `..`). This keeps UI concerns out of the
/// core `Entry` model.
#[derive(Clone, Debug)]
pub struct UiEntry {
    pub entry: Entry,
    pub display: String,
    pub synthetic: bool,
}

impl UiEntry {
    /// Create a UiEntry from a domain `Entry`, computing the display
    /// line via the UI formatter.
    pub fn from_entry(e: Entry) -> Self {
        UiEntry {
            display: format_entry_line(&e),
            entry: e,
            synthetic: false,
        }
    }

    /// Create a header UiEntry that displays the full path.
    pub fn header(path: PathBuf) -> Self {
        let display = path.display().to_string();
        UiEntry {
            display: display.clone(),
            entry: Entry::file(display, path, 0, None),
            synthetic: true,
        }
    }

    /// Create a parent (`..`) UiEntry pointing to `parent`.
    pub fn parent(parent: PathBuf) -> Self {
        UiEntry {
            display: "..".to_string(),
            entry: Entry::directory("..", parent, None),
            synthetic: true,
        }
    }

    pub fn is_header(&self) -> bool {
        self.synthetic && !self.entry.is_dir
    }

    pub fn is_parent(&self) -> bool {
        self.synthetic && self.entry.is_dir && self.entry.name == ".."
    }
}
use crate::ui::colors::current as theme_current;
use crate::ui::util::columns_for_file_list;
// PathBuf is intentionally not used here — keep imports minimal

pub fn draw_list(f: &mut Frame, area: Rect, panel: &Panel, active: bool) {
    let theme = theme_current();

    let list_height = (area.height as usize).saturating_sub(2); // account for borders/title
                                                                // Build UI rows: header, optional parent, then domain entries formatted with `UiEntry`.
    let mut ui_rows: Vec<UiEntry> = Vec::new();
    ui_rows.push(UiEntry::header(panel.cwd.clone()));
    if let Some(parent) = panel.cwd.parent() {
        ui_rows.push(UiEntry::parent(parent.to_path_buf()));
    }
    for e in panel.entries.iter().cloned() {
        ui_rows.push(UiEntry::from_entry(e));
    }
    let visible = if ui_rows.len() > panel.offset {
        &ui_rows[panel.offset..std::cmp::min(panel.offset + list_height, ui_rows.len())]
    } else {
        &[]
    };

    // Split area into the list and a 1-cell vertical scrollbar area; we need the
    // panel width to compute column sizes for the list rows. We delegate
    // column rect generation to our helper which uses Ratatui Layout constraints.
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
        .split(area);

    let size_col = 10u16;
    let modified_col = 16u16;
    let perms_col = 4u16;
    // Compute Rects for the columns using flexible ratio for the name column.
    let inner_cols = columns_for_file_list(cols[0], (1, 1), size_col, modified_col, perms_col);
    let name_col = inner_cols[0].width as usize;

    let mut items: Vec<ListItem> = Vec::new();
    // Build a column header item
    let mut header_spans: Vec<Span> = Vec::new();
    header_spans.push(Span::styled(format!("{:<width$}", "Name", width = name_col), theme.header_style));
    header_spans.push(Span::raw(" │ "));
    header_spans.push(Span::styled(format!("{:<width$}", "Size", width = size_col as usize), theme.header_style));
    header_spans.push(Span::raw(" │ "));
    header_spans.push(Span::styled(format!("{:<width$}", "Modified", width = modified_col as usize), theme.header_style));
    header_spans.push(Span::raw(" │ "));
    header_spans.push(Span::styled(format!("{:<width$}", "rwx", width = perms_col as usize), theme.header_style));
    // We'll construct the columns header from `header_spans` when needed.

    for (i, e) in visible.iter().enumerate() {
        if is_entry_header(e) {
            items.push(crate::ui::header::render_header(&e.entry.name));
            items.push(ListItem::new(Text::from(Line::from(header_spans.clone()))));
            continue;
        }
        let style = if is_entry_parent(e) {
            theme.parent_style
        } else if e.entry.is_dir {
            theme.dir_style
        } else {
            Style::default()
        };
        // Build spans for columns: name | size | modified | perms
        let icon = if e.entry.is_dir { "📁 " } else { "📄 " };
        // Determine if this visible ui row corresponds to a domain entry and
        // whether it is selected in the panel's multi-selection set.
        let ui_index = panel.offset + i;
        let header_count = 1usize;
        let parent_count = if panel.cwd.parent().is_some() { 1usize } else { 0usize };
        let mut name_text = format!("{}{}", icon, e.display);
        if !e.synthetic {
            let domain_idx = ui_index.saturating_sub(header_count + parent_count);
            if panel.selections.contains(&domain_idx) {
                name_text = format!("* {}", name_text);
            } else {
                name_text = format!("  {}", name_text);
            }
        }
        let name_field = if name_text.len() > name_col {
            name_text[..name_col].to_string()
        } else {
            format!("{:<width$}", name_text, width = name_col)
        };
        let size_field = if e.entry.is_dir {
            format!("{:<width$}", "<DIR>", width = size_col as usize)
        } else {
            format!("{:>width$}", e.entry.size, width = size_col as usize)
        };
        let mtime = e.entry.modified.map(|d| d.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_else(|| "-".to_string());
            let mtime_field = if mtime.len() > modified_col as usize {
            mtime[..modified_col as usize].to_string()
        } else {
            format!("{:>width$}", mtime, width = modified_col as usize)
        };
        let perms_field = "rwx".to_string();

        let mut spans: Vec<Span> = Vec::new();
        // Render a compact selection marker separate from the filename so it can be styled.
        let marker = if !e.synthetic {
            let ui_index = panel.offset + i;
            let header_count = 1usize;
            let parent_count = if panel.cwd.parent().is_some() { 1usize } else { 0usize };
            let domain_idx = ui_index.saturating_sub(header_count + parent_count);
            if panel.selections.contains(&domain_idx) { "[x] " } else { "[ ] " }
        } else {
            "    "
        };
        spans.push(Span::styled(format!("{:<4}", marker), theme.help_block_style));
        spans.push(Span::styled(name_field, style));
        spans.push(Span::raw(" │ "));
        spans.push(Span::styled(size_field, theme.help_block_style));
            spans.push(Span::raw(" │ "));
        spans.push(Span::styled(mtime_field, theme.help_block_style));
            spans.push(Span::raw(" │ "));
        spans.push(Span::styled(perms_field, theme.help_block_style));
        items.push(ListItem::new(Text::from(Line::from(spans))));
    }

    let title = format!("{}", panel.cwd.display());
    let border_style = if active {
        theme.border_active
    } else {
        theme.border_inactive
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(border_style),
        )
        .highlight_style(theme.highlight_style);

    let mut state = ListState::default();
    if panel.selected >= panel.offset && panel.selected < panel.offset + list_height {
        state.select(Some(panel.selected - panel.offset));
    } else {
        state.select(None);
    }
    f.render_stateful_widget(list, cols[0], &mut state);

    // Render vertical scrollbar at right-side column using ratatui::widgets::Scrollbar
    let total = ui_rows.len();
    let mut sb_state = ScrollbarState::new(total).position(panel.offset).viewport_content_length(list_height);
    let sb = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .thumb_style(theme.scrollbar_thumb_style)
        .track_style(theme.scrollbar_style);
    f.render_stateful_widget(sb, cols[1], &mut sb_state);
}

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
    let preview = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Preview")
            .style(theme.preview_block_style),
    );
    f.render_widget(preview, cols[0]);
    let max_lines = (cols[0].height as usize).saturating_sub(2);
    // Render scrollbar for preview using ratatui::widgets::Scrollbar
    let mut sb_state = ScrollbarState::new(lines.len()).position(panel.preview_offset).viewport_content_length(max_lines);
    let sb = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .thumb_style(theme.scrollbar_thumb_style)
        .track_style(theme.scrollbar_style);
    f.render_stateful_widget(sb, cols[1], &mut sb_state);
}

/// Draw a small chart visualizing file sizes in the panel.
/// This adapts concepts from the ratatui chart example and is intentionally
/// lightweight: it produces a simple line/dataset chart of file sizes by index.
pub fn draw_chart(f: &mut Frame, area: Rect, panel: &Panel) {
    // Collect numeric data from panel entries (skip synthetic rows).
    let mut points: Vec<(f64, f64)> = Vec::new();
    for (i, e) in panel.entries.iter().enumerate() {
        // Use index as X and size (or 0 for directories) as Y
        let y = if e.is_dir { 0.0 } else { e.size as f64 };
        points.push((i as f64, y));
    }

    let theme = theme_current();

    if points.is_empty() {
        let p = Paragraph::new("No numeric data for chart").block(
            Block::default().borders(Borders::ALL).title("Chart").style(theme.preview_block_style),
        );
        f.render_widget(p, area);
        return;
    }

    // Determine Y bounds (min/max) with some padding so the chart looks nicer.
    let ys: Vec<f64> = points.iter().map(|(_, y)| *y).collect();
    let y_min = ys.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let y_pad = ((y_max - y_min) * 0.1).max(1.0);

    // Use a Canvas widget to render points and a connecting line.
    let x_min = 0.0f64;
    let x_max = (points.len().saturating_sub(1)) as f64;
    let y_min_plot = y_min - y_pad;
    let y_max_plot = y_max + y_pad;

    let canvas = Canvas::default()
        .block(Block::default().borders(Borders::ALL).title("Sizes"))
        .x_bounds([x_min, x_max])
        .y_bounds([y_min_plot, y_max_plot]);

    f.render_widget(canvas.paint(|ctx| {
        // Draw points
        ctx.draw(&CanvasPoints { coords: &points, color: Color::Cyan });
        // Draw connecting line segments between consecutive points
        for pair in points.windows(2) {
            if let [(x1, y1), (x2, y2)] = pair {
                ctx.draw(&CanvasLine { x1: *x1, y1: *y1, x2: *x2, y2: *y2, color: Color::Yellow });
            }
        }
    }), area);
}

/// Draw a compact sparkline-like visualization of file sizes.
///
/// This is a lightweight, text-based mini-canvas inspired by the
/// `ratatui` canvas example — it provides a quick visual summary of
/// file sizes using a single-line sparkline made from block characters.
pub fn draw_sparkline(f: &mut Frame, area: Rect, panel: &Panel) {
    // Unicode sparkline blocks from low -> high
    const BLOCKS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    let theme = theme_current();

    // Collect numeric data from panel entries (skip synthetic rows).
    let values: Vec<f64> = panel
        .entries
        .iter()
        .map(|e| if e.is_dir { 0.0 } else { e.size as f64 })
        .collect();

    if values.is_empty() {
        let p = Paragraph::new("No data").block(
            Block::default().borders(Borders::ALL).title("Sparkline").style(theme.preview_block_style),
        );
        f.render_widget(p, area);
        return;
    }

    // Determine drawing width (leave 2 chars for borders/padding)
    let width = area.width.saturating_sub(2) as usize;
    let width = std::cmp::max(1, width);

    // If we have more values than width, downsample by taking the max in each bucket
    let mut buckets: Vec<f64> = Vec::with_capacity(width);
    if values.len() <= width {
        buckets.extend(values.iter().cloned());
    } else {
        let chunk = (values.len() as f64) / (width as f64);
        for i in 0..width {
            let start = (i as f64 * chunk).floor() as usize;
            let end = ((i as f64 + 1.0) * chunk).ceil() as usize;
            let end = std::cmp::min(end, values.len());
            if start >= end {
                buckets.push(0.0);
            } else {
                let max = values[start..end].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                buckets.push(if max.is_finite() { max } else { 0.0 });
            }
        }
    }

    // Normalize buckets to 0..(BLOCKS.len()-1)
    let max_v = buckets.iter().cloned().fold(0./0., f64::max);
    let min_v = buckets.iter().cloned().fold(0./0., f64::min);
    let max_v = if max_v.is_finite() { max_v } else { 0.0 };
    let min_v = if min_v.is_finite() { min_v } else { 0.0 };
    let range = (max_v - min_v).max(1.0);

    let mut spark = String::with_capacity(buckets.len());
    for v in buckets.iter() {
        let norm = ((*v - min_v) / range).clamp(0.0, 1.0);
        let idx = (norm * ((BLOCKS.len() - 1) as f64)).round() as usize;
        spark.push(BLOCKS[idx]);
    }

    // Also show min/max numbers in a compact footer line
    let footer = format!("min:{:.0} max:{:.0}", min_v, max_v);

    // Build paragraph with the sparkline and footer stacked vertically
    let text = format!("{}\n{}", spark, footer);
    let p = Paragraph::new(text).block(
        Block::default().borders(Borders::ALL).title("Sparkline").style(theme.preview_block_style),
    );
    f.render_widget(p, area);
}

/// Compute the scrollbar thumb start position and size for a given viewport height
/// and scrollable content characteristics.
pub fn compute_scrollbar_thumb(height: usize, total: usize, visible: usize, offset: usize) -> (usize, usize) {
    if total == 0 || visible == 0 || total <= visible || height == 0 {
        return (0, 0);
    }
    let thumb_size = std::cmp::max(1, (visible * height) / total);
    let mut start = if total > 0 { (offset * height) / total } else { 0 };
    if start + thumb_size > height {
        if thumb_size >= height {
            start = 0;
        } else {
            start = height - thumb_size;
        }
    }
    (start, thumb_size)
}

/// Format a directory entry into the fixed-width textual line used by the list.
///
/// This mirrors the formatting used by `draw_list`.
pub fn format_entry_line(e: &Entry) -> String {
    let name = &e.name;
    let size = if e.is_dir {
        "<dir>".to_string()
    } else {
        format!("{}", e.size)
    };
    let mtime = e
        .modified
        .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "-".to_string());
    format!("{:<40.40} {:>10} {:>16}", name, size, mtime)
}

/// UI helpers that detect header and parent ("..") synthetic rows.
///
/// These are UI-only helpers placed in the `ui` module so the core/`Entry`
/// data type does not need to carry presentation methods. Callers that are
/// concerned with presentation should use these helpers instead of adding
/// methods to `Entry` itself.
pub fn is_entry_header(e: &UiEntry) -> bool {
    e.is_header()
}

pub fn is_entry_parent(e: &UiEntry) -> bool {
    e.is_parent()
}
