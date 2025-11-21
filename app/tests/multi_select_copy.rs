use assert_fs::prelude::*;
use fileZoom::app::{App, Panel, Side, SortKey};
// `PathBuf` not required by name here; remove explicit import to avoid warning
use fileZoom::input::KeyCode;
use fileZoom::runner::handlers;
use predicates::prelude::*;
use std::time::Duration;

#[test]
fn multi_select_copy_background() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let left_dir = tmp.child("left");
    let right_dir = tmp.child("right");
    left_dir.create_dir_all().unwrap();
    right_dir.create_dir_all().unwrap();

    // create files in left
    left_dir.child("a.txt").write_str("a").unwrap();
    left_dir.child("b.txt").write_str("b").unwrap();

    let left_path = left_dir.path().to_path_buf();
    let right_path = right_dir.path().to_path_buf();

    let mut app = App {
        left: Panel::new(left_path.clone()),
        right: Panel::new(right_path.clone()),
        active: Side::Left,
        mode: fileZoom::app::Mode::Normal,
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

    // select both entries by index
    // entries are domain-only; find indexes
    let mut a_idx = None;
    let mut b_idx = None;
    for (i, e) in app.left.entries.iter().enumerate() {
        if e.name == "a.txt" {
            a_idx = Some(i);
        }
        if e.name == "b.txt" {
            b_idx = Some(i);
        }
    }
    assert!(a_idx.is_some() && b_idx.is_some());
    app.left.selections.insert(a_idx.unwrap());
    app.left.selections.insert(b_idx.unwrap());

    // Trigger F5 (background copy)
    handlers::handle_key(&mut app, KeyCode::F(5), 10).unwrap();

    // Wait for background operation to finish by polling receiver
    if let Some(rx) = &app.op_progress_rx {
        loop {
            match rx.recv_timeout(Duration::from_secs(2)) {
                Ok(upd) => {
                    if upd.done {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    }

    // Verify files exist in right dir
    right_dir.child("a.txt").assert(predicate::path::exists());
    right_dir.child("b.txt").assert(predicate::path::exists());

    tmp.close().unwrap();
}
