use crate::app::types::Entry;
use chrono::{DateTime, Local};
use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Panel holds the minimal, UI-independent state for one side of the
/// dual-pane file manager. It intentionally keeps presentation details
/// (such as rendering rows) out of the model so the core can be tested
/// without a terminal.
#[derive(Debug)]
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
    pub selections: HashSet<usize>,
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
            selections: HashSet::new(),
        }
    }

    /// Toggle selection of the currently selected entry (if any).
    pub fn toggle_selection(&mut self) {
        if let Some(idx) = super::utils::ui_to_entry_index(self.selected, self) {
            // `HashSet::remove` returns whether the value was present.
            // If it wasn't present, insert it (toggle behaviour).
            if !self.selections.remove(&idx) {
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
        super::utils::ui_to_entry_index(self.selected, self)
            .and_then(|idx| self.entries.get(idx))
    }

    /// Move selection down by one, clamping at the last UI row.
    pub fn select_next(&mut self) {
        let max_rows = super::utils::ui_row_count(self);
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
        let max_rows = super::utils::ui_row_count(self);
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
        let total_rows = super::utils::ui_row_count(self);
        if total_rows == 0 {
            self.offset = 0;
            return;
        }
        // If `selected` is above the viewport, bring it to the top.
        if self.selected < self.offset {
            self.offset = self.selected;
            return;
        }

        // If `selected` is below the viewport, move the offset so it becomes
        // visible at the bottom of the viewport (or as low as possible).
        let max_offset = total_rows.saturating_sub(height);
        if self.selected >= self.offset + height {
            let desired = self.selected + 1 - height;
            self.offset = std::cmp::min(desired, max_offset);
        } else if self.offset > max_offset {
            // Clamp offset when viewport is larger than the remaining rows.
            self.offset = max_offset;
        }
    }

    /// Replace the preview text and reset the preview scroll offset.
    pub fn set_preview(&mut self, text: String) {
        self.preview = text;
        self.preview_offset = 0;
    }

    /// Read directory entries and return a Vec<Entry>.
    /// This centralises the filesystem access and metadata reading used by
    /// `App::refresh_panel` and keeps the Panel's path-related concerns in one place.
    /// Read the immediate children of the panel's `cwd` and return them as
    /// a `Vec<Entry>`. This is intentionally a thin wrapper around
    /// filesystem access so callers can handle errors appropriately.
    pub(crate) fn read_entries(&self) -> io::Result<Vec<Entry>> {
        let mut entries_vec = Vec::new();

        for dir_entry_result in WalkDir::new(&self.cwd)
            .min_depth(1)
            .max_depth(1)
            .follow_links(false)
        {
            let dir_entry = dir_entry_result
                .map_err(io::Error::other)?;

            let metadata = dir_entry.metadata()?;
            let modified_time = metadata.modified().ok().map(DateTime::<Local>::from);
            let name = dir_entry.file_name().to_string_lossy().into_owned();
            let path_buf = dir_entry.path().to_path_buf();

            let file_entry = if metadata.is_dir() {
                Entry::directory(name, path_buf, modified_time)
            } else {
                Entry::file(name, path_buf, metadata.len(), modified_time)
            };

            entries_vec.push(file_entry);
        }

        Ok(entries_vec)
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

    #[test]
    fn read_entries_empty_dir_returns_empty() {
        let temp = assert_fs::TempDir::new().unwrap();
        // no children created

        let p = Panel::new(temp.path().to_path_buf());
        let entries = p.read_entries().unwrap();
        assert!(entries.is_empty(), "expected no entries in empty temp dir");
    }
}
