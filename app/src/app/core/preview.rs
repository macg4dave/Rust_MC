use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::App;
use crate::app::types::Side;

/// Maximum number of directory entries to include in a directory preview.
/// Maximum number of directory entries to include in a directory preview.
pub const MAX_DIR_PREVIEW_ENTRIES: usize = 50;

/// Errors that may occur when attempting to build a preview for a path.
#[derive(Debug, PartialEq, Eq)]
pub enum PreviewError {
    /// The file appears to be binary (contains NUL or invalid UTF-8 or
    /// a high proportion of non-printable characters).
    Binary,
    /// The file could not be opened or read.
    Unreadable,
}

impl std::fmt::Display for PreviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreviewError::Binary => write!(f, "file appears to be binary"),
            PreviewError::Unreadable => write!(f, "file could not be read"),
        }
    }
}

impl std::error::Error for PreviewError {}

/// Heuristic that returns `true` when the provided byte slice looks like a
/// binary blob rather than text. The function is conservative: it aims to
/// avoid false-positives on text files while still catching obvious binary
/// content (NUL bytes, invalid UTF-8, or many control characters).
pub fn is_binary(buffer: &[u8]) -> bool {
    // Presence of a NUL byte is an excellent indicator of binary data.
    if buffer.contains(&0) {
        return true;
    }

    if buffer.is_empty() {
        return false;
    }

    // If the bytes are not valid UTF-8 then treat as binary.
    let text = match std::str::from_utf8(buffer) {
        Ok(s) => s,
        Err(_) => return true,
    };

    // Count characters that are control characters (excluding common
    // whitespace). If a significant proportion of characters are
    // non-printable, we consider the buffer binary.
    const NON_PRINTABLE_RATIO_THRESHOLD: f64 = 0.30;

    let total_chars = text.chars().count();
    if total_chars == 0 {
        return false;
    }

    let non_printable = text
        .chars()
        .filter(|&ch| {
            matches!(ch, '\u{0000}'..='\u{001F}' | '\u{007F}'..='\u{009F}')
                && !(ch == '\t' || ch == '\n' || ch == '\r' || ch == ' ')
        })
        .count();

    (non_printable as f64) / (total_chars as f64) > NON_PRINTABLE_RATIO_THRESHOLD
}

/// Build a small textual preview of a directory. The string begins with a
/// header line containing the directory path, followed by up to
/// `MAX_DIR_PREVIEW_ENTRIES` file/directory names (sorted, one per line).
pub fn build_directory_preview(path: &Path) -> String {
    let mut out = format!("Directory: {}\n", path.display());
    if !path.is_dir() {
        return out;
    }

    let mut names: Vec<String> = match fs::read_dir(path) {
        Ok(rd) => rd
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect(),
        Err(_) => Vec::new(),
    };

    names.sort();

    for name in names.into_iter().take(MAX_DIR_PREVIEW_ENTRIES) {
        out.push_str(&name);
        out.push('\n');
    }

    out
}

/// Read up to `max_bytes` from `path` and produce a string preview. Returns
/// `Err(PreviewError::Binary)` if the sample looks like a binary file, and
/// `Err(PreviewError::Unreadable)` if the file could not be read.
pub fn build_file_preview(path: &Path, max_bytes: usize) -> Result<String, PreviewError> {
    let file = File::open(path).map_err(|_| PreviewError::Unreadable)?;

    let mut reader = file.take(max_bytes as u64);
    let mut buf = Vec::with_capacity(max_bytes.min(8 * 1024));
    reader
        .read_to_end(&mut buf)
        .map_err(|_| PreviewError::Unreadable)?;

    if is_binary(&buf) {
        return Err(PreviewError::Binary);
    }

    // Convert to UTF-8 string without panicking; strip BOM if present.
    let mut preview = String::from_utf8_lossy(&buf).into_owned();
    if preview.starts_with('\u{FEFF}') {
        preview = preview.trim_start_matches('\u{FEFF}').to_string();
    }

    // If the file is longer than the bytes we sampled, indicate truncation.
    let truncated = match fs::metadata(path) {
        Ok(md) => (md.len() as usize) > buf.len(),
        Err(_) => false,
    };

    if truncated {
        Ok(format!("{}\n... (truncated)", preview))
    } else {
        Ok(preview)
    }
}

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
                // Read up to the module-level `MAX_PREVIEW_BYTES` for previews.
                match build_file_preview(&e.path, super::MAX_PREVIEW_BYTES) {
                    Ok(s) => panel.set_preview(s),
                    Err(PreviewError::Binary) => panel.set_preview(format!(
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

// Unit tests for the preview helpers.

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn is_binary_detects_nul_and_non_utf8() {
        let text = b"hello world";
        assert!(!is_binary(text));

        let with_nul = b"hello\0world";
        assert!(is_binary(with_nul));

        // Invalid UTF-8 (continuation byte without leader)
        let invalid = &[0x80u8, 0x80u8, 0x80u8];
        assert!(is_binary(invalid));
    }

    #[test]
    fn build_file_preview_text_and_truncation() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("t.txt");
        let mut f = File::create(&file_path).unwrap();
        write!(f, "{}", "a".repeat(1024)).unwrap();

        let s = build_file_preview(&file_path, 512).unwrap();
        assert!(s.contains("... (truncated)"));

        let s2 = build_file_preview(&file_path, 2048).unwrap();
        assert!(!s2.contains("... (truncated)"));
    }

    #[test]
    fn build_directory_preview_lists_entries() {
        let dir = tempdir().unwrap();
        let p = dir.path();
        File::create(p.join("a.txt")).unwrap();
        File::create(p.join("b.txt")).unwrap();

        let preview = build_directory_preview(p);
        assert!(preview.contains("Directory:"));
        assert!(preview.contains("a.txt"));
        assert!(preview.contains("b.txt"));
    }
}
