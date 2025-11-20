use std::io::Read;
use std::path::Path;

/// A simple heuristic to decide whether a file is likely to be binary.
pub fn is_binary(buf: &[u8]) -> bool {
    // Immediately treat files containing a NUL as binary — text files rarely
    // contain embedded NULs and it's a quick fingerprint.
    if buf.contains(&0) {
        return true;
    }
    if buf.is_empty() {
        return false;
    }
    // If the byte sequence is not valid UTF-8, it's likely binary.
    if std::str::from_utf8(buf).is_err() {
        return true;
    }

    // For valid-UTF8 strings, use a character-level heuristic: count how many
    // Unicode scalar values are control characters (excluding whitespace).
    // If a significant proportion are control / unprintable, classify as
    // binary. Use a conservative threshold to avoid false positives.
    let s = unsafe { std::str::from_utf8_unchecked(buf) };
    let mut non_printable = 0usize;
    let mut total = 0usize;
    for ch in s.chars() {
        total += 1;
        // Allow common whitespace (space, tab, newline, carriage return)
        if ch == '\t' || ch == '\n' || ch == '\r' || ch == ' ' {
            continue;
        }
        if ch.is_control() {
            non_printable += 1;
        }
    }
    if total == 0 {
        return false;
    }
    (non_printable as f64) / (total as f64) > 0.30
}

// Directory preview: show at most `MAX_DIR_PREVIEW_ENTRIES` entries.
pub const MAX_DIR_PREVIEW_ENTRIES: usize = 50;

pub fn build_directory_preview(path: &Path) -> String {
    let mut s = format!("Directory: {}\n", path.display());
    if let Ok(list) = std::fs::read_dir(path) {
        for ent in list.flatten().take(MAX_DIR_PREVIEW_ENTRIES) {
            s.push_str(&format!("{}\n", ent.file_name().to_string_lossy()));
        }
    }
    s
}

// Return Ok(preview) or Err(reason) where reason is one of "binary" or
// "unreadable" to allow the caller to present the correct error message.
pub fn build_file_preview(path: &Path, max_bytes: usize) -> Result<String, String> {
    match std::fs::File::open(path) {
        Ok(mut f) => {
            let mut buf = Vec::new();
            match (&mut f).take(max_bytes as u64).read_to_end(&mut buf) {
                Ok(read_bytes) => {
                    if is_binary(&buf) {
                        return Err("binary".to_string());
                    }
                    let preview = String::from_utf8_lossy(&buf).into_owned();
                    let truncated = match f.metadata() {
                        Ok(md) => (md.len() as usize) > read_bytes,
                        Err(_) => false,
                    };
                    if truncated {
                        Ok(format!("{}\n... (truncated)", preview))
                    } else {
                        Ok(preview)
                    }
                }
                Err(_) => Err("unreadable".to_string()),
            }
        }
        Err(_) => Err("unreadable".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn is_binary_detects_null_byte() {
        let v = vec![b'a', 0, b'b'];
        assert!(is_binary(&v));
    }

    #[test]
    fn is_binary_returns_false_for_text() {
        let v = b"hello world\n".to_vec();
        assert!(!is_binary(&v));
    }

    #[test]
    fn is_binary_detects_invalid_utf8() {
        let invalid = vec![0xC3u8, 0x28u8];
        assert!(is_binary(&invalid));
    }

    #[test]
    fn build_file_preview_truncates() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("big.txt");
        let large = "x".repeat(1024 * 100 * 2);
        file.write_str(&large).unwrap();
        let res = build_file_preview(file.path(), 1024 * 100).unwrap();
        assert!(res.contains("(truncated)"));
    }

    #[test]
    fn build_file_preview_detects_binary() {
        let temp = assert_fs::TempDir::new().unwrap();
        let f = temp.child("b.bin");
        std::fs::write(f.path(), &[0u8, 1u8, 2u8]).unwrap();
        let r = build_file_preview(f.path(), 1024);
        assert!(r.is_err(), "expected binary preview to return Err");
    }

    #[test]
    fn build_file_preview_handles_bom() {
        let temp = assert_fs::TempDir::new().unwrap();
        let f = temp.child("bom.txt");
        // BOM + text
        let mut v = vec![0xEFu8, 0xBB, 0xBF];
        v.extend_from_slice(b"hello");
        std::fs::write(f.path(), &v).unwrap();
        let r = build_file_preview(f.path(), 1024).unwrap();
        assert!(r.contains("hello"));
    }

    #[test]
    fn build_directory_preview_limits() {
        let temp = assert_fs::TempDir::new().unwrap();
        let d = temp.child("dir");
        d.create_dir_all().unwrap();
        for i in 0..(MAX_DIR_PREVIEW_ENTRIES + 10) {
            d.child(format!("f{}.txt", i)).write_str("x").unwrap();
        }
        let s = build_directory_preview(d.path());
        let lines: Vec<&str> = s.lines().collect();
        assert!(lines.len() <= 1 + MAX_DIR_PREVIEW_ENTRIES);
    }
}
