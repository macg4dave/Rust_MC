use chrono::Local;
use fileZoom::app::Entry;
use fileZoom::ui::panels::compute_scrollbar_thumb;
use fileZoom::ui::panels::format_entry_line;

#[test]
fn format_entry_line_for_file_and_dir() {
    let now = Local::now();
    let file = Entry::file(
        "file.txt",
        std::path::PathBuf::from("/tmp/file.txt"),
        1234,
        Some(now),
    );
    let dir = Entry::directory("somedir", std::path::PathBuf::from("/tmp/somedir"), None);
    let fline = format_entry_line(&file);
    assert!(fline.contains("file.txt"));
    assert!(fline.contains("1234"));
    assert!(fline.contains(&now.format("%Y-%m-%d %H:%M").to_string()));

    let dline = format_entry_line(&dir);
    assert!(dline.contains("somedir"));
    assert!(dline.contains("<dir>"));
    assert!(dline.contains("-"));
}

#[test]
fn compute_scrollbar_thumb_smoke() {
    // Simple cases
    assert_eq!(compute_scrollbar_thumb(10, 0, 0, 0), (0, 0));
    assert_eq!(compute_scrollbar_thumb(10, 5, 5, 0), (0, 0)); // visible >= total
                                                              // Typical case: height=10, total=100, visible=10 -> thumb size = 1, start = (offset*10)/100
    let (start, size) = compute_scrollbar_thumb(10, 100, 10, 0);
    assert_eq!(size, 1);
    assert_eq!(start, 0);
    let (start2, _) = compute_scrollbar_thumb(10, 100, 10, 50);
    assert_eq!(start2, 5);
    // Thumb saturates at top and bottom
    let (start3, size3) = compute_scrollbar_thumb(5, 10, 3, 9);
    assert!(start3 + size3 <= 5);
}
