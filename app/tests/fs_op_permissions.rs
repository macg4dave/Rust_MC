use fileZoom::fs_op::permissions::inspect_permissions;
use tempfile::tempdir;

#[test]
fn probe_writes_do_not_leave_tmp_files() {
    let td = tempdir().unwrap();
    let p = td.path().to_path_buf();
    // perform a write probe for the directory
    let info = inspect_permissions(&p, true).unwrap();
    // Ensure call completed and read the result to avoid unused variable lint.
    let _ = info.can_write;

    // ensure no leftover atomic temp files
    let mut tmp_leftovers = 0;
    for e in std::fs::read_dir(&p).unwrap().flatten() {
        if let Some(name) = e.file_name().to_str() {
            if name.starts_with(".tmp_atomic_write.") {
                tmp_leftovers += 1;
            }
        }
    }
    assert_eq!(
        tmp_leftovers, 0,
        "found leftover atomic temp files after probe"
    );
}
