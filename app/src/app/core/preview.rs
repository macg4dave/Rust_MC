use super::*;
use crate::app::core::preview_helpers::{build_directory_preview, build_file_preview};

impl App {
    pub fn update_preview_for(&mut self, side: Side) {
        let panel = self.panel_mut(side);
        // Update the panel's `preview` text for the currently selected entry.
        //
        // For directories this is a small list of contained entries. For files
        // this reads up to `App::MAX_PREVIEW_BYTES` bytes to avoid large
        // memory usage. Preview updates must also reset `preview_offset` so
        // the preview scroll position is consistent.
        // Use the Panel API so preview/preview_offset semantics are centralized
        // - `selected_entry` encapsulates bounds-safe access
        // - `set_preview` resets `preview_offset` to zero
        if let Some(e) = panel.selected_entry() {
            if e.is_dir {
                let s = build_directory_preview(&e.path);
                panel.set_preview(s);
            } else {
                // Read up to MAX_PREVIEW_BYTES from the file for preview.
                match build_file_preview(&e.path, App::MAX_PREVIEW_BYTES) {
                    Ok(s) => panel.set_preview(s),
                    Err(ref reason) if reason == "binary" => panel.set_preview(format!(
                        "Binary file: {} (preview not available)",
                        e.path.display()
                    )),
                    Err(_) => panel.set_preview(format!(
                        "Cannot preview file: {} (unreadable)",
                        e.path.display()
                    )),
                }
            }
        } else {
            panel.set_preview(String::new());
        }
    }
}

// Helper tests moved to the `preview_helpers` module.
