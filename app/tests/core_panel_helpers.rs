use assert_fs::prelude::*;
use fileZoom::app::core::panel::Panel;
use fileZoom::app::core::App;
use fileZoom::app::settings::write_settings::Settings;
use fileZoom::app::types::{Mode, Side, SortKey};

#[test]
fn selected_index_reflects_active_panel_unit() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("a.txt").write_str("1").unwrap();
    temp.child("b.txt").write_str("2").unwrap();
    temp.child("c.txt").write_str("3").unwrap();

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
        settings: Settings::default(),
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

    // find index of a.txt
    let mut left_idx = None;
    for (i, e) in app.left.entries.iter().enumerate() {
        if e.name == "a.txt" {
            left_idx = Some(i);
            break;
        }
    }
    assert!(left_idx.is_some());
    let header_count = 1usize;
    let parent_count = if app.left.cwd.parent().is_some() {
        1usize
    } else {
        0usize
    };
    let ui_left_idx = header_count + parent_count + left_idx.unwrap();
    app.left.selected = ui_left_idx;
    app.active = Side::Left;
    assert_eq!(app.selected_index(), Some(left_idx.unwrap()));

    // for right panel
    let mut right_idx = None;
    for (i, e) in app.right.entries.iter().enumerate() {
        if e.name == "b.txt" {
            right_idx = Some(i);
            break;
        }
    }
    assert!(right_idx.is_some());
    let parent_count_r = if app.right.cwd.parent().is_some() {
        1usize
    } else {
        0usize
    };
    let ui_right_idx = header_count + parent_count_r + right_idx.unwrap();
    app.right.selected = ui_right_idx;
    app.active = Side::Right;
    assert_eq!(app.selected_index(), Some(right_idx.unwrap()));

    temp.close().unwrap();
}

#[test]
fn panel_mut_match() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("a.txt").write_str("1").unwrap();
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
        settings: Settings::default(),
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
    // modify left via panel_mut and check read through panel
    let left_name_before = app.left.cwd.clone();
    let panel_mut = app.panel_mut(Side::Left);
    panel_mut.cwd = std::path::PathBuf::from(".");
    let left_name_after = app.left.cwd.clone();
    assert_eq!(left_name_after, std::path::PathBuf::from("."));
    assert_ne!(left_name_before, left_name_after);
}
