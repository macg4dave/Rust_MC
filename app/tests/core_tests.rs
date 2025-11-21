use assert_fs::prelude::*;
use fileZoom::app::{App, Mode, Panel, Side, SortKey};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[test]
fn resolve_target_behaviour() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dir_path = temp.path().to_path_buf();
    // existing directory should join
    let t = fileZoom::fs_op::helpers::resolve_target(&dir_path, "file.txt");
    assert_eq!(t, dir_path.join("file.txt"));

    // trailing slash should join even if path doesn't exist
    let dst = PathBuf::from("some/where/");
    let t2 = fileZoom::fs_op::helpers::resolve_target(&dst, "x");
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
        menu_index: 0,
        menu_focused: false,
        preview_visible: false,
        command_line: None,
        settings: fileZoom::app::settings::write_settings::Settings::default(),
        op_progress_rx: None,
        op_cancel_flag: None,
        op_decision_tx: None,
        last_mouse_click_time: None,
        last_mouse_click_pos: None,
        drag_active: false,
        drag_start: None,
        drag_current: None,
        drag_button: None,
    };
    app.refresh().unwrap();

    // `entries` is domain-only after refactor; start at 0.
    let start = 0usize;
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
        menu_index: 0,
        menu_focused: false,
        preview_visible: false,
        command_line: None,
        settings: fileZoom::app::settings::write_settings::Settings::default(),
        op_progress_rx: None,
        op_cancel_flag: None,
        op_decision_tx: None,
        last_mouse_click_time: None,
        last_mouse_click_pos: None,
        drag_active: false,
        drag_start: None,
        drag_current: None,
        drag_button: None,
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
    // Update selection to the UI index (header + parent + entry index).
    let header_count = 1usize;
    let parent_count = if app.left.cwd.parent().is_some() {
        1usize
    } else {
        0usize
    };
    app.left.selected = header_count + parent_count + idx.unwrap();
    app.update_preview_for(Side::Left);
    assert!(app.left.preview.contains("(truncated)"));

    temp.close().unwrap();
}

#[test]
fn preview_shows_directory_entries_limited() {
    let temp = assert_fs::TempDir::new().unwrap();
    let d = temp.child("d");
    d.create_dir_all().unwrap();
    // Create more items than the preview limit
    let total = 60usize;
    for i in 0..total {
        d.child(format!("f{}.txt", i)).write_str("x").unwrap();
    }

    let cwd = temp.path().to_path_buf();
    let mut app = App {
        left: Panel::new(cwd.clone()),
        right: Panel::new(cwd.clone()),
        active: Side::Left,
        mode: Mode::Normal,
        sort: SortKey::Name,
        sort_desc: false,
        menu_index: 0,
        menu_focused: false,
        preview_visible: false,
        command_line: None,
        settings: fileZoom::app::settings::write_settings::Settings::default(),
        op_progress_rx: None,
        op_cancel_flag: None,
        op_decision_tx: None,
        last_mouse_click_time: None,
        last_mouse_click_pos: None,
        drag_active: false,
        drag_start: None,
        drag_current: None,
        drag_button: None,
    };
    app.refresh().unwrap();

    // find index of d in entries
    let mut idx = None;
    for (i, e) in app.left.entries.iter().enumerate() {
        if e.name == "d" {
            idx = Some(i);
            break;
        }
    }
    assert!(idx.is_some());
    // Update selection to the UI index (header + parent + entry index).
    let header_count = 1usize;
    let parent_count = if app.left.cwd.parent().is_some() {
        1usize
    } else {
        0usize
    };
    app.left.selected = header_count + parent_count + idx.unwrap();
    // precondition: set non-zero preview to ensure update overwrites it
    app.left.preview = "old".to_string();
    app.update_preview_for(Side::Left);

    // header + at most MAX_DIR_PREVIEW_ENTRIES lines
    let lines: Vec<&str> = app.left.preview.lines().collect();

    assert!(lines.len() <= 1 + fileZoom::app::core::preview_helpers::MAX_DIR_PREVIEW_ENTRIES);
    // ensure at least one file name is present
    assert!(app.left.preview.contains("f0.txt"));

    temp.close().unwrap();
}

#[test]
fn preview_resets_preview_offset() {
    let temp = assert_fs::TempDir::new().unwrap();
    let f = temp.child("small.txt");
    f.write_str("hello\nthere\n").unwrap();

    let cwd = temp.path().to_path_buf();
    let mut app = App {
        left: Panel::new(cwd.clone()),
        right: Panel::new(cwd.clone()),
        active: Side::Left,
        mode: Mode::Normal,
        sort: SortKey::Name,
        sort_desc: false,
        menu_index: 0,
        menu_focused: false,
        preview_visible: false,
        command_line: None,
        settings: fileZoom::app::settings::write_settings::Settings::default(),
        op_progress_rx: None,
        op_cancel_flag: None,
        op_decision_tx: None,
        last_mouse_click_time: None,
        last_mouse_click_pos: None,
        drag_active: false,
        drag_start: None,
        drag_current: None,
        drag_button: None,
    };
    app.refresh().unwrap();

    // find index of small.txt
    let mut idx = None;
    for (i, e) in app.left.entries.iter().enumerate() {
        if e.name == "small.txt" {
            idx = Some(i);
            break;
        }
    }
    assert!(idx.is_some());
    // Update selection to the UI index (header + parent + entry index).
    let header_count = 1usize;
    let parent_count = if app.left.cwd.parent().is_some() {
        1usize
    } else {
        0usize
    };
    app.left.selected = header_count + parent_count + idx.unwrap();
    app.left.preview_offset = 10; // set non-zero offset
    app.update_preview_for(Side::Left);
    assert_eq!(app.left.preview_offset, 0);

    temp.close().unwrap();
}

#[test]
fn preview_handles_very_long_filename() {
    let temp = assert_fs::TempDir::new().unwrap();
    let mut name = String::from("verylong");
    // create a long file name (200 chars)
    while name.len() < 200 {
        name.push('x');
    }
    let f = temp.child(&name);
    f.write_str("hello").unwrap();

    let cwd = temp.path().to_path_buf();
    let mut app = App {
        left: Panel::new(cwd.clone()),
        right: Panel::new(cwd.clone()),
        active: Side::Left,
        mode: Mode::Normal,
        sort: SortKey::Name,
        sort_desc: false,
        menu_index: 0,
        menu_focused: false,
        preview_visible: false,
        command_line: None,
        settings: fileZoom::app::settings::write_settings::Settings::default(),
        op_progress_rx: None,
        op_cancel_flag: None,
        op_decision_tx: None,
        last_mouse_click_time: None,
        last_mouse_click_pos: None,
        drag_active: false,
        drag_start: None,
        drag_current: None,
        drag_button: None,
    };
    app.refresh().unwrap();

    // find index of long filename
    let mut idx = None;
    for (i, e) in app.left.entries.iter().enumerate() {
        if e.name == name {
            idx = Some(i);
            break;
        }
    }
    assert!(idx.is_some());
    // Update selection to the UI index (header + parent + entry index).
    let header_count = 1usize;
    let parent_count = if app.left.cwd.parent().is_some() {
        1usize
    } else {
        0usize
    };
    app.left.selected = header_count + parent_count + idx.unwrap();
    app.update_preview_for(Side::Left);
    assert!(app.left.preview.contains("hello"));

    temp.close().unwrap();
}

// See `core.rs` unit tests for tests of private helpers like `selected_index`.

#[test]
#[cfg(unix)]
fn preview_unreadable_file_shows_message() {
    use std::fs;
    let temp = assert_fs::TempDir::new().unwrap();
    let f = temp.child("cannot_read.txt");
    f.write_str("hello").unwrap();

    let p = f.path();
    let mut perms = fs::metadata(p).unwrap().permissions();
    // remove all permissions so file cannot be opened
    perms.set_mode(0o000);
    fs::set_permissions(p, perms.clone()).unwrap();

    let cwd = temp.path().to_path_buf();
    let mut app = App {
        left: Panel::new(cwd.clone()),
        right: Panel::new(cwd.clone()),
        active: Side::Left,
        mode: Mode::Normal,
        sort: SortKey::Name,
        sort_desc: false,
        menu_index: 0,
        menu_focused: false,
        preview_visible: false,
        command_line: None,
        settings: fileZoom::app::settings::write_settings::Settings::default(),
        op_progress_rx: None,
        op_cancel_flag: None,
        op_decision_tx: None,
        last_mouse_click_time: None,
        last_mouse_click_pos: None,
        drag_active: false,
        drag_start: None,
        drag_current: None,
        drag_button: None,
    };
    app.refresh().unwrap();

    // find index of cannot_read.txt
    let mut idx = None;
    for (i, e) in app.left.entries.iter().enumerate() {
        if e.name == "cannot_read.txt" {
            idx = Some(i);
            break;
        }
    }
    assert!(idx.is_some());
    // Update selection to the UI index (header + parent + entry index).
    let header_count = 1usize;
    let parent_count = if app.left.cwd.parent().is_some() {
        1usize
    } else {
        0usize
    };
    app.left.selected = header_count + parent_count + idx.unwrap();
    app.update_preview_for(Side::Left);
    // (no debug) ensure unreadable file preview is handled
    assert!(app.left.preview.contains("Cannot preview file"));

    // restore perms so cleanup can occur
    perms.set_mode(0o600);
    fs::set_permissions(p, perms).unwrap();

    temp.close().unwrap();
}
