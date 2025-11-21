use assert_fs::prelude::*;
use fileZoom::app::{App, Panel, Side, SortKey};
use fileZoom::input::KeyCode;
use fileZoom::runner::progress::OperationDecision;
use std::time::Duration;
use predicates::prelude::*;

/// When a conflict occurs and the UI sends OperationDecision::Cancel the
/// worker should terminate and send a final ProgressUpdate with done==true
/// and an error message describing cancellation. The destination file
/// should remain unchanged.
#[test]
fn conflict_cancel_by_user() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let left = tmp.child("left");
    let right = tmp.child("right");
    left.create_dir_all().unwrap();
    right.create_dir_all().unwrap();

    left.child("a.txt").write_str("from-left").unwrap();
    right.child("a.txt").write_str("from-right").unwrap();

    let left_path = left.path().to_path_buf();
    let right_path = right.path().to_path_buf();

    let mut app = App {
        left: Panel::new(left_path.clone()),
        right: Panel::new(right_path.clone()),
        active: Side::Left,
        mode: fileZoom::app::Mode::Normal,
        sort: SortKey::Name,
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

    // select the file
    let mut idx = None;
    for (i, e) in app.left.entries.iter().enumerate() {
        if e.name == "a.txt" { idx = Some(i); break; }
    }
    assert!(idx.is_some());
    app.left.selections.insert(idx.unwrap());

    // start copy operation
    fileZoom::runner::handlers::handle_key(&mut app, KeyCode::F(5), 10).unwrap();

    // wait for a conflict report
    let mut saw_conflict = false;
    if let Some(rx) = &app.op_progress_rx {
        while let Ok(upd) = rx.recv_timeout(Duration::from_secs(2)) {
            if upd.conflict.is_some() { saw_conflict = true; break; }
            if upd.done { break; }
        }
    }
    assert!(saw_conflict, "expected worker to report a conflict");

    // send Cancel decision from the UI side
    if let Some(tx) = &app.op_decision_tx { let _ = tx.send(OperationDecision::Cancel); }

    // ensure the worker finishes and reported a cancellation
    let mut saw_cancelled = false;
    if let Some(rx) = &app.op_progress_rx {
        while let Ok(upd) = rx.recv_timeout(Duration::from_secs(2)) {
            if upd.done {
                if let Some(err) = &upd.error {
                    if err.contains("Cancelled") { saw_cancelled = true; }
                }
                break;
            }
        }
    }

    assert!(saw_cancelled, "expected final progress update to indicate cancellation");
    // destination should still have the original content
    right.child("a.txt").assert("from-right");
    tmp.close().unwrap();
}

/// Test cancellation of a running worker by setting the shared cancel flag
/// after one item completed. We ensure the worker observes the flag and
/// terminates early with a Cancelled ProgressUpdate; the second item should
/// not be copied.
#[test]
fn cancel_mid_operation_via_flag() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let left = tmp.child("left");
    let right = tmp.child("right");
    left.create_dir_all().unwrap();
    right.create_dir_all().unwrap();

    // two files on the left; create one conflict so per-item path is used
    left.child("a.txt").write_str("a-src").unwrap();
    // make b.txt large so copying takes measurable time and our test can
    // reliably set the cancel flag before the worker completes the second item
    // make b.txt large so copying takes measurable time and our test can
    // reliably set the cancel flag before the worker completes the second item
    // (~4MB)
    let large = "b".repeat(4_000_000);
    left.child("b.txt").write_str(&large).unwrap();
    right.child("a.txt").write_str("a-dst").unwrap();

    let left_path = left.path().to_path_buf();
    let right_path = right.path().to_path_buf();

    let mut app = App {
        left: Panel::new(left_path.clone()),
        right: Panel::new(right_path.clone()),
        active: Side::Left,
        mode: fileZoom::app::Mode::Normal,
        sort: SortKey::Name,
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

    // select both entries for copy
    for (i, e) in app.left.entries.iter().enumerate() {
        if e.name == "a.txt" || e.name == "b.txt" { app.left.selections.insert(i); }
    }

    // start copying
    fileZoom::runner::handlers::handle_key(&mut app, KeyCode::F(5), 10).unwrap();

    // Immediately request cancellation through the shared cancel flag. This
    // tests the worker observes the flag and stops quickly.
    if let Some(flag) = app.op_cancel_flag.take() { flag.store(true, std::sync::atomic::Ordering::SeqCst); }

    // Now wait for final cancelled update
    let mut saw_cancel = false;
    if let Some(rx) = &app.op_progress_rx {
        while let Ok(upd) = rx.recv_timeout(Duration::from_secs(5)) {
            if upd.done {
                if let Some(err) = &upd.error { if err.contains("Cancelled") { saw_cancel = true; } }
                break;
            }
        }
    }

    assert!(saw_cancel, "expected worker to finish by observing cancel flag");

    // b.txt should NOT be present in destination because we cancelled before it finished.
    right.child("b.txt").assert(predicate::path::missing());

    tmp.close().unwrap();
}
