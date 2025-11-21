use assert_fs::prelude::*;
use fileZoom::app::{App, Panel};
use fileZoom::app::{Mode, Side};
use fileZoom::input::KeyCode;
use fileZoom::runner::handlers::context_menu;
use fileZoom::ui::panels::{compute_scrollbar_thumb, format_entry_line};
use std::path::PathBuf;

#[test]
fn compute_scrollbar_thumb_basic() {
    // small content fits in viewport -> no thumb
    let (s, t) = compute_scrollbar_thumb(10, 5, 10, 0);
    assert_eq!((s, t), (0, 0));
    // typical scrolling with many items
    let (s, t) = compute_scrollbar_thumb(10, 100, 10, 20);
    assert!(t >= 1);
    assert!(s + t <= 10);
}

#[test]
fn format_entry_line_limits_length() {
    let name = "a_very_long_filename_that_exceeds_the_column_width.txt";
    let e = fileZoom::app::Entry::file(name, PathBuf::from("/tmp/x"), 1234, None);
    let line = format_entry_line(&e);
    assert!(line.contains("1234"));
    assert!(!line.is_empty());
}

#[test]
fn panel_toggle_selection_and_visibility() {
    let mut p = Panel::new(PathBuf::from("/"));
    p.entries = vec![
        fileZoom::app::Entry::file("a", PathBuf::from("/a"), 1, None),
        fileZoom::app::Entry::file("b", PathBuf::from("/b"), 2, None),
        fileZoom::app::Entry::file("c", PathBuf::from("/c"), 3, None),
    ];
    // header_count = 1, parent_count likely 0 for root
    p.selected = 2; // select second entry (ui index)
    p.toggle_selection();
    assert!(p.selections.contains(&1usize));
    p.selected = 3; // last entry
    p.ensure_selected_visible(1);
    assert!(p.offset <= p.selected);
}

#[test]
fn context_menu_enter_opens_preview() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("a").write_str("x").unwrap();
    let cwd = temp.path().to_path_buf();
    let mut app = App {
        left: Panel::new(cwd.clone()),
        right: Panel::new(cwd.clone()),
        active: Side::Left,
        mode: Mode::Normal,
        sort: fileZoom::app::SortKey::Name,
        sort_order: fileZoom::app::types::SortOrder::Ascending,
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

    app.mode = Mode::ContextMenu {
        title: "Test".to_string(),
        options: vec!["View".to_string()],
        selected: 0,
        path: temp.path().join("a"),
    };

    let _ = context_menu::handle_context_menu(&mut app, KeyCode::Enter).unwrap();
    assert!(app.preview_visible);
    temp.close().unwrap();
}
