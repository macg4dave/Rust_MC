use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Scrollbar};
use ratatui::widgets::{ScrollbarOrientation, ScrollbarState};
use ratatui::Frame;

use crate::app::Panel;

use crate::ui::colors::current as theme_current;
use crate::ui::util::columns_for_file_list;

use crate::ui::panels::UiEntry;

pub fn draw_list(f: &mut Frame, area: Rect, panel: &Panel, active: bool) {
    let theme = theme_current();

    // Top header row + main area inspired by ratatui example layout
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(area);
    let header_area = vchunks[0];
    let area = vchunks[1];

    // Small header showing path and counts (compact, single-line)
    let header_text = format!("{} — {} entries", panel.cwd.display(), panel.entries.len());
    crate::ui::header::draw_compact_header(f, header_area, &header_text);

    let list_height = (area.height as usize).saturating_sub(2); // account for borders/title
                                                                // Build UI rows: header, optional parent, then domain entries formatted with `UiEntry`.
    let mut ui_rows: Vec<UiEntry> = Vec::new();
    ui_rows.push(crate::ui::panels::UiEntry::header(panel.cwd.clone()));
    if let Some(parent) = panel.cwd.parent() {
        ui_rows.push(crate::ui::panels::UiEntry::parent(parent.to_path_buf()));
    }
    for e in panel.entries.iter().cloned() {
        ui_rows.push(crate::ui::panels::UiEntry::from_entry(e));
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
    header_spans.push(Span::styled(
        format!("{:<width$}", "Name", width = name_col),
        theme.header_style,
    ));
    header_spans.push(Span::raw(" │ "));
    header_spans.push(Span::styled(
        format!("{:<width$}", "Size", width = size_col as usize),
        theme.header_style,
    ));
    header_spans.push(Span::raw(" │ "));
    header_spans.push(Span::styled(
        format!("{:<width$}", "Modified", width = modified_col as usize),
        theme.header_style,
    ));
    header_spans.push(Span::raw(" │ "));
    header_spans.push(Span::styled(
        format!("{:<width$}", "rwx", width = perms_col as usize),
        theme.header_style,
    ));

    for (i, e) in visible.iter().enumerate() {
        if crate::ui::panels::is_entry_header(e) {
            items.push(crate::ui::header::render_header(&e.entry.name));
            items.push(ListItem::new(Text::from(Line::from(header_spans.clone()))));
            continue;
        }
        let style = if crate::ui::panels::is_entry_parent(e) {
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
        let parent_count = if panel.cwd.parent().is_some() {
            1usize
        } else {
            0usize
        };
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
        let mtime = e
            .entry
            .modified
            .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "-".to_string());
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
            let parent_count = if panel.cwd.parent().is_some() {
                1usize
            } else {
                0usize
            };
            let domain_idx = ui_index.saturating_sub(header_count + parent_count);
            if panel.selections.contains(&domain_idx) {
                "[x] "
            } else {
                "[ ] "
            }
        } else {
            "    "
        };
        spans.push(Span::styled(
            format!("{:<4}", marker),
            theme.help_block_style,
        ));
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
    let mut sb_state = ScrollbarState::new(total)
        .position(panel.offset)
        .viewport_content_length(list_height);
    let sb = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .thumb_style(theme.scrollbar_thumb_style)
        .track_style(theme.scrollbar_style);
    f.render_stateful_widget(sb, cols[1], &mut sb_state);
}
