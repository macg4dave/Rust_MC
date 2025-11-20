use chrono::Local;
use fileZoom::app::Entry;
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
