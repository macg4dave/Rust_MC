use app::app::{App, Mode, Panel, Side, SortKey};
use assert_fs::prelude::*;
use std::path::PathBuf;

#[test]
fn resolve_target_behaviour() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dir_path = temp.path().to_path_buf();
    // existing directory should join
    let t = app::fs_op::helpers::resolve_target(&dir_path, "file.txt");
    assert_eq!(t, dir_path.join("file.txt"));

    // trailing slash should join even if path doesn't exist
    let dst = PathBuf::from("some/where/");
    let t2 = app::fs_op::helpers::resolve_target(&dst, "x");
    assert_eq!(t2, dst.join("x"));

    temp.close().unwrap();
}

#[test]
fn sort_name_puts_dirs_first() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("b_dir").create_dir_all().unwrap();
    temp.child("a.txt").write_str("hello").unwrap();

    let cwd = temp.path().to_path_buf();
    let mut app = App {
        left: Panel::new(cwd.clone()),
        right: Panel::new(cwd.clone()),
        active: Side::Left,
        mode: Mode::Normal,
        sort: SortKey::Name,
        sort_desc: false,
    };
    app.refresh().unwrap();

    // entries: [header, parent(if exists), ...actual entries]
    let start = 1 + if app
        .left
        .entries
        .get(1)
        .map(|e| e.name == "..")
        .unwrap_or(false)
    {
        1
    } else {
        0
    };
    // expected dirs first
    assert_eq!(app.left.entries[start].name, "b_dir");
    assert_eq!(app.left.entries[start + 1].name, "a.txt");

    temp.close().unwrap();
}

#[test]
fn preview_truncates_large_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let big = temp.child("big.txt");
    let large = "x".repeat(App::MAX_PREVIEW_BYTES * 2);
    big.write_str(&large).unwrap();

    let cwd = temp.path().to_path_buf();
    let mut app = App {
        left: Panel::new(cwd.clone()),
        right: Panel::new(cwd.clone()),
        active: Side::Left,
        mode: Mode::Normal,
        sort: SortKey::Name,
        sort_desc: false,
    };
    app.refresh().unwrap();

    // find index of big.txt in entries
    let mut idx = None;
    for (i, e) in app.left.entries.iter().enumerate() {
        if e.name == "big.txt" {
            idx = Some(i);
            break;
        }
    }
    assert!(idx.is_some());
    app.left.selected = idx.unwrap();
    app.update_preview_for(Side::Left);
    assert!(app.left.preview.contains("(truncated)"));

    temp.close().unwrap();
}
