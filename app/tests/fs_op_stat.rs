use fileZoom::fs_op::stat::{exists, is_dir, is_file};
use tempfile::tempdir;

#[test]
fn stat_checks() {
    let td = tempdir().unwrap();
    let f = td.path().join("file.txt");
    std::fs::write(&f, b"ok").unwrap();
    assert!(exists(&f));
    assert!(is_file(&f));
    assert!(!is_dir(&f));
}
