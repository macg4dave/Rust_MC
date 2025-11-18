use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tui::Frame;

use crate::app::{Entry, Panel};
use crate::ui::colors::current as theme_current;

pub fn draw_list<B: Backend>(f: &mut Frame<B>, area: Rect, panel: &Panel, active: bool) {
    let theme = theme_current();

    let list_height = (area.height as usize).saturating_sub(2); // account for borders/title
    let visible = if panel.entries.len() > panel.offset {
        &panel.entries[panel.offset..std::cmp::min(panel.offset + list_height, panel.entries.len())]
    } else {
        &[]
    };

    let items: Vec<ListItem> = visible
        .iter()
        .map(|e| {
            // Special header row: show full path across the line with distinct style
            if e.name == panel.cwd.display().to_string() {
                return crate::ui::header::render_header(&e.name);
            }

            let name = &e.name;
            let size = if e.is_dir { "<dir>".to_string() } else { format!("{}", e.size) };
            let mtime = e
                .modified
                .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "-".to_string());
            let text = format!("{:<40.40} {:>10} {:>16}", name, size, mtime);
            let style = if e.name == ".." {
                // Parent entry - show as directory-like
                theme.parent_style
            } else if e.is_dir {
                theme.dir_style
            } else {
                Style::default()
            };
            ListItem::new(Spans::from(vec![Span::styled(text, style)]))
        })
        .collect();

    let title = format!("{}", panel.cwd.display());
    let border_style = if active { theme.border_active } else { theme.border_inactive };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(border_style),
        )
        .highlight_style(theme.highlight_style);

    let mut state = tui::widgets::ListState::default();
    if panel.selected >= panel.offset && panel.selected < panel.offset + list_height {
        state.select(Some(panel.selected - panel.offset));
    } else {
        state.select(None);
    }
    f.render_stateful_widget(list, area, &mut state);
}

pub fn draw_preview<B: Backend>(f: &mut Frame<B>, area: Rect, panel: &Panel) {
    let max_lines = (area.height as usize).saturating_sub(2);
    let lines: Vec<&str> = panel.preview.lines().collect();
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
    let theme = theme_current();
    let preview = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Preview").style(theme.preview_block_style));
    f.render_widget(preview, area);
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn format_entry_line_for_file_and_dir() {
        let now = Local::now();
        let file = Entry {
            name: "file.txt".to_string(),
            path: std::path::PathBuf::from("/tmp/file.txt"),
            is_dir: false,
            size: 1234,
            modified: Some(now),
        };
        let dir = Entry {
            name: "somedir".to_string(),
            path: std::path::PathBuf::from("/tmp/somedir"),
            is_dir: true,
            size: 0,
            modified: None,
        };
        let fline = format_entry_line(&file);
        assert!(fline.contains("file.txt"));
        assert!(fline.contains("1234"));
        assert!(fline.contains(&now.format("%Y-%m-%d %H:%M").to_string()));

        let dline = format_entry_line(&dir);
        assert!(dline.contains("somedir"));
        assert!(dline.contains("<dir>"));
        assert!(dline.contains("-"));
    }
}
