use crate::app::Entry;
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

// Keep this small module focused on the type and simple helpers. The
// formatter and other helpers live in this module so other panel
// submodules can use them via `crate::ui::panels::...`.

/// UI helpers that detect header and parent ("..") synthetic rows.
pub fn is_entry_header(e: &UiEntry) -> bool {
    e.is_header()
}

pub fn is_entry_parent(e: &UiEntry) -> bool {
    e.is_parent()
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
