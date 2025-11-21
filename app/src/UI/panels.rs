use ratatui::{layout::Rect, widgets::{Block, Paragraph, Borders}, Frame};
use crate::app::Panel;

#[derive(Clone, Debug)]
pub enum UiEntry {
    Header(std::path::PathBuf),
    Parent(std::path::PathBuf),
}

impl UiEntry {
    pub fn header(path: std::path::PathBuf) -> Self { UiEntry::Header(path) }
    pub fn parent(path: std::path::PathBuf) -> Self { UiEntry::Parent(path) }
}

pub fn is_entry_header(e: &UiEntry) -> bool { matches!(e, UiEntry::Header(_)) }
pub fn is_entry_parent(e: &UiEntry) -> bool { matches!(e, UiEntry::Parent(_)) }

pub fn draw_preview(f: &mut Frame, area: Rect, panel: &Panel) {
    let txt = if panel.preview.is_empty() { "(no preview)".to_string() } else { panel.preview.clone() };
    let p = Paragraph::new(txt).block(Block::default().borders(Borders::ALL).title("Preview"));
    f.render_widget(p, area);
}

pub fn compute_scrollbar_thumb(height: u16, total: usize, visible: usize, offset: usize) -> (u16, u16) {
    if total == 0 || visible == 0 || visible >= total { return (0, 0); }
    let h = height as u32; let tot = total as u32; let vis = visible as u32; let off = offset as u32;
    let mut size = (vis * h / tot) as u16; if size == 0 { size = 1; }
    let denom = tot.saturating_sub(vis);
    let mut start = if denom == 0 { 0 } else { ((off * (h - size as u32)) / denom) as u16 };
    if start as u32 + size as u32 > h { start = h.saturating_sub(size as u32) as u16; }
    (start, size)
}

use crate::app::Entry;
pub fn format_entry_line(e: &Entry) -> String {
    let time = e.modified.as_ref().map(|d| d.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_else(|| "-".into());
    let size = if e.is_dir { "<dir>".into() } else { format!("{}", e.size) };
    format!("{}  {}  {}", e.name, size, time)
}
