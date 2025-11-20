use fileZoom::app::{App, Mode, Panel, Side, SortKey};
use fileZoom::Entry;
use std::path::PathBuf;

#[test]
fn app_navigation_next_prev_and_paging() {
    let cwd = PathBuf::from("/");
    let mut app = App {
        left: Panel::new(cwd.clone()),
        right: Panel::new(cwd.clone()),
        active: Side::Left,
        mode: Mode::Normal,
        sort: SortKey::Name,
        sort_desc: false,
    };
    // populate left entries with mock (directory) entries so preview doesn't try to read
    app.left.entries = (0..10)
        .map(|i| Entry::directory(format!("f{}", i), PathBuf::from(format!("/f{}", i)), None))
        .collect();

    // initial selected should be 0
    assert_eq!(app.left.selected, 0);
    app.next(3);
    assert_eq!(app.left.selected, 1);
    // page down by 3 -> 1 + 3 == 4
    app.page_down(3);
    assert_eq!(app.left.selected, 4);
    // page up by 2 -> 4 - 2 == 2
    app.page_up(2);
    assert_eq!(app.left.selected, 2);
    // previous -> 1
    app.previous(3);
    assert_eq!(app.left.selected, 1);

    // Switching active side should affect the correct panel
    app.active = Side::Right;
    app.right.entries = (0..3)
        .map(|i| Entry::directory(format!("r{}", i), PathBuf::from(format!("/r{}", i)), None))
        .collect();
    assert_eq!(app.right.selected, 0);
    app.next(3);
    assert_eq!(app.right.selected, 1);
}

#[test]
fn app_navigation_ensure_selection_visible() {
    let cwd = PathBuf::from("/");
    let mut app = App {
        left: Panel::new(cwd.clone()),
        right: Panel::new(cwd.clone()),
        active: Side::Left,
        mode: Mode::Normal,
        sort: SortKey::Name,
        sort_desc: false,
    };
    app.left.entries = (0..10)
        .map(|i| Entry::directory(format!("f{}", i), PathBuf::from(format!("/f{}", i)), None))
        .collect();

    // viewport of 3 rows
    let h = 3;
    app.left.offset = 0;
    app.left.selected = 0;
    app.ensure_selection_visible(h);
    assert_eq!(app.left.offset, 0);

    app.left.selected = 2;
    app.ensure_selection_visible(h);
    assert_eq!(app.left.offset, 0);

    app.left.selected = 3;
    app.ensure_selection_visible(h);
    assert_eq!(app.left.offset, 1);

    app.left.selected = 9;
    app.ensure_selection_visible(h);
    assert!(app.left.offset + h > app.left.selected);
}
