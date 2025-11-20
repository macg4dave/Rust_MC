use assert_fs::prelude::*;
use fileZoom::app::{App, Panel, Side, SortKey};
use fileZoom::runner::progress::OperationDecision;
use fileZoom::input::KeyCode;
use std::time::Duration;

#[test]
fn conflict_overwrite() {
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
        sort_desc: false,
        menu_index: 0,
        menu_focused: false,
        op_progress_rx: None,
        op_cancel_flag: None,
        op_decision_tx: None,
    };
    app.refresh().unwrap();

    let mut idx = None;
    for (i, e) in app.left.entries.iter().enumerate() {
        if e.name == "a.txt" { idx = Some(i); break; }
    }
    assert!(idx.is_some());
    app.left.selections.insert(idx.unwrap());

    fileZoom::runner::handlers::handle_key(&mut app, KeyCode::F(5), 10).unwrap();

    let mut saw_conflict = false;
    if let Some(rx) = &app.op_progress_rx {
        loop {
            match rx.recv_timeout(Duration::from_secs(2)) {
                Ok(upd) => { if upd.conflict.is_some() { saw_conflict = true; break; } if upd.done { break; } }
                Err(_) => break,
            }
        }
    }
    assert!(saw_conflict, "expected worker to report a conflict");

    if let Some(tx) = &app.op_decision_tx { let _ = tx.send(OperationDecision::Overwrite); }

    if let Some(rx) = &app.op_progress_rx {
        loop {
            match rx.recv_timeout(Duration::from_secs(2)) { Ok(upd) => { if upd.done { break; } } Err(_) => break, }
        }
    }
    right.child("a.txt").assert("from-left");
    tmp.close().unwrap();
}

#[test]
fn conflict_skip() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let left = tmp.child("left");
    let right = tmp.child("right");
    left.create_dir_all().unwrap();
    right.create_dir_all().unwrap();

    left.child("a.txt").write_str("from-left-2").unwrap();
    right.child("a.txt").write_str("from-right-2").unwrap();

    let left_path = left.path().to_path_buf();
    let right_path = right.path().to_path_buf();

    let mut app = App {
        left: Panel::new(left_path.clone()),
        right: Panel::new(right_path.clone()),
        active: Side::Left,
        mode: fileZoom::app::Mode::Normal,
        sort: SortKey::Name,
        sort_desc: false,
        menu_index: 0,
        menu_focused: false,
        op_progress_rx: None,
        op_cancel_flag: None,
        op_decision_tx: None,
    };
    app.refresh().unwrap();

    let mut idx = None;
    for (i, e) in app.left.entries.iter().enumerate() {
        if e.name == "a.txt" { idx = Some(i); break; }
    }
    assert!(idx.is_some());
    app.left.selections.insert(idx.unwrap());

    fileZoom::runner::handlers::handle_key(&mut app, KeyCode::F(5), 10).unwrap();

    let mut saw_conflict = false;
    if let Some(rx) = &app.op_progress_rx {
        loop {
            match rx.recv_timeout(Duration::from_secs(2)) {
                Ok(upd) => { if upd.conflict.is_some() { saw_conflict = true; break; } if upd.done { break; } }
                Err(_) => break,
            }
        }
    }
    assert!(saw_conflict, "expected worker to report a conflict");

    if let Some(tx) = &app.op_decision_tx { let _ = tx.send(OperationDecision::Skip); }

    if let Some(rx) = &app.op_progress_rx {
        loop {
            match rx.recv_timeout(Duration::from_secs(2)) { Ok(upd) => { if upd.done { break; } } Err(_) => break, }
        }
    }

    right.child("a.txt").assert("from-right-2");
    tmp.close().unwrap();
}
