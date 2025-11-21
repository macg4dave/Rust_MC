use crate::app::types::Entry;
use chrono::{DateTime, Local};
use std::io;
use std::path::PathBuf;

/// Lightweight panel state used by the application core.
///
/// This struct intentionally stores only UI-independent state so the core
/// can be unit-tested without rendering. It represents a single side of the
/// dual-pane file manager (left or right).
pub struct Panel {
    /// Current working directory shown by this panel.
    pub cwd: PathBuf,
    /// Listing of filesystem entries in the directory. This vector
    /// is domain-only (contains no UI synthetic rows like the header or `..`).
    pub entries: Vec<Entry>,
    /// UI selection index including any synthetic header/parent rows.
    /// This keeps the visual selection intuitive without coupling the
    /// panel data model to presentation details.
    pub selected: usize,
    /// UI scroll offset (index of the top-most visible UI row).
    pub offset: usize,
    /// File preview text for the selected entry (if any).
    pub preview: String,
    /// Scroll offset for the preview text.
    pub preview_offset: usize,
    /// Selected entry indices for multi-selection (domain indexes into `entries`).
    pub selections: std::collections::HashSet<usize>,
}

impl Panel {
    /// Create a new panel rooted at `cwd` with sensible defaults.
    pub fn new(cwd: PathBuf) -> Self {
        Panel {
            cwd,
            entries: Vec::new(),
            selected: 0,
            offset: 0,
            preview: String::new(),
            preview_offset: 0,
            selections: std::collections::HashSet::new(),
        }
    }

    /// Toggle selection of the currently selected entry (if any).
    pub fn toggle_selection(&mut self) {
        if let Some(idx) = {
            let header_count = 1usize;
            let parent_count = if self.cwd.parent().is_some() {
                1usize
            } else {
                0usize
            };
            if self.selected >= header_count + parent_count {
                Some(self.selected - header_count - parent_count)
            } else {
                None
            }
        } {
            if self.selections.contains(&idx) {
                self.selections.remove(&idx);
            } else {
                self.selections.insert(idx);
            }
        }
    }

    /// Clear all selections in this panel.
    pub fn clear_selections(&mut self) {
        self.selections.clear();
    }

    /// Return a reference to the currently selected entry, if present.
    /// Return a reference to the currently selected filesystem entry,
    /// if the UI selected index refers to an actual item (i.e. not the
    /// header or the parent row).
    pub fn selected_entry(&self) -> Option<&Entry> {
        let header_count = 1usize;
        let parent_count = if self.cwd.parent().is_some() {
            1usize
        } else {
            0usize
        };
        // Compute whether `selected` refers to an entry
        if self.selected >= header_count + parent_count {
            let idx = self.selected - header_count - parent_count;
            self.entries.get(idx)
        } else {
            None
        }
    }

    /// Move selection down by one, clamping at the last UI row.
    pub fn select_next(&mut self) {
        let max_rows = 1 + if self.cwd.parent().is_some() { 1 } else { 0 } + self.entries.len();
        if self.selected + 1 < max_rows {
            self.selected += 1;
        }
    }

    /// Move selection up by one, clamping at zero (makes header selectable).
    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Ensure `selected` is within bounds of the UI rows (header +
    /// maybe parent + entries).
    pub fn clamp_selected(&mut self) {
        let max_rows = 1 + if self.cwd.parent().is_some() { 1 } else { 0 } + self.entries.len();
        if max_rows == 0 {
            self.selected = 0;
        } else {
            self.selected = std::cmp::min(self.selected, max_rows.saturating_sub(1));
        }
    }

    /// Adjust `offset` so the selected row is visible within a viewport of
    /// `height` rows. Note that UI rows include synthetic header and parent rows.
    pub fn ensure_selected_visible(&mut self, height: usize) {
        if height == 0 {
            self.offset = 0;
            return;
        }
        let total_rows = 1 + if self.cwd.parent().is_some() { 1 } else { 0 } + self.entries.len();
        if total_rows == 0 {
            self.offset = 0;
            return;
        }
        // If selected is above the visible area, move offset up.
        if self.selected < self.offset {
            self.offset = self.selected;
            return;
        }
        // If selected is below the visible area, move offset down so it is visible.
        let max_offset = total_rows.saturating_sub(height);
        if self.selected >= self.offset + height {
            self.offset = std::cmp::min(self.selected + 1 - height, max_offset);
        } else if self.offset > max_offset {
            // Clamp offset if viewport is larger than remaining items.
            self.offset = max_offset;
        }
    }

    /// Set preview text and reset preview scroll.
    pub fn set_preview(&mut self, text: String) {
        self.preview = text;
        self.preview_offset = 0;
    }

    /// Read directory entries and return a Vec<Entry>.
    /// This centralises the filesystem access and metadata reading used by
    /// `App::refresh_panel` and keeps the Panel's path-related concerns in one place.
    pub(crate) fn read_entries(&self) -> io::Result<Vec<Entry>> {
        let mut ents = Vec::new();
        for entry in std::fs::read_dir(&self.cwd)? {
            let e = entry?;
            let meta = e.metadata()?;
            let modified = meta.modified().ok().map(DateTime::<Local>::from);
            let name = e.file_name().to_string_lossy().into_owned();
            let path = e.path();
            if meta.is_dir() {
                ents.push(Entry::directory(name, path, modified));
            } else {
                ents.push(Entry::file(name, path, meta.len(), modified));
            }
        }
        Ok(ents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn read_entries_returns_all_entries() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.child("a.txt").write_str("a").unwrap();
        temp.child("subdir").create_dir_all().unwrap();

        let p = Panel::new(temp.path().to_path_buf());
        let entries = p.read_entries().unwrap();
        // Expect at least the file and the directory
        let mut names: Vec<String> = entries.into_iter().map(|e| e.name).collect();
        names.sort();
        assert!(names.contains(&"a.txt".to_string()));
        assert!(names.contains(&"subdir".to_string()));
    }
}
