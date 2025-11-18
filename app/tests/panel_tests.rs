use app::app::Panel;
use app::Entry;
use std::path::PathBuf;

#[test]
fn panel_defaults_and_selected_entry() {
    let cwd = PathBuf::from("/tmp");
    let p = Panel::new(cwd.clone());
    assert_eq!(p.cwd, cwd);
    assert_eq!(p.entries.len(), 0);
    assert_eq!(p.selected, 0);
    assert_eq!(p.offset, 0);
    assert_eq!(p.preview, "");
    assert_eq!(p.preview_offset, 0);
}

#[test]
fn select_next_prev_and_clamp() {
    let mut p = Panel::new(PathBuf::from("/"));
    // populate entries with mock entries
    p.entries = (0..5)
        .map(|i| Entry {
            name: format!("f{}", i),
            path: PathBuf::from(format!("/f{}", i)),
            is_dir: false,
            size: 0,
            modified: None,
        })
        .collect();
    assert_eq!(p.selected, 0);
    p.select_next();
    assert_eq!(p.selected, 1);
    p.select_prev();
    assert_eq!(p.selected, 0);
    // move to last
    for _ in 0..10 {
        p.select_next();
    }
    assert_eq!(p.selected, 4);
    // clamp down when entries shrink
    p.entries.truncate(2);
    p.clamp_selected();
    assert_eq!(p.selected, 1);
}

#[test]
fn ensure_selected_visible_basic() {
    let mut p = Panel::new(PathBuf::from("/"));
    p.entries = (0..10)
        .map(|i| Entry {
            name: format!("f{}", i),
            path: PathBuf::from(format!("/f{}", i)),
            is_dir: false,
            size: 0,
            modified: None,
        })
        .collect();
    // viewport of 3 rows
    let h = 3;
    p.selected = 0;
    p.offset = 0;
    p.ensure_selected_visible(h);
    assert_eq!(p.offset, 0);

    p.selected = 2;
    p.ensure_selected_visible(h);
    assert_eq!(p.offset, 0);

    p.selected = 3;
    p.ensure_selected_visible(h);
    assert_eq!(p.offset, 1);

    p.selected = 9;
    p.ensure_selected_visible(h);
    // offset should be such that selected is visible within viewport
    assert!(p.offset + h > p.selected);
}

#[test]
fn ensure_selected_visible_zero_height_and_single_item() {
    // zero height: should reset offset to 0 regardless of entries
    let mut p = Panel::new(PathBuf::from("/"));
    p.entries = (0..3)
        .map(|i| Entry {
            name: format!("f{}", i),
            path: PathBuf::from(format!("/f{}", i)),
            is_dir: false,
            size: 0,
            modified: None,
        })
        .collect();
    p.offset = 2;
    p.selected = 2;
    p.ensure_selected_visible(0);
    assert_eq!(p.offset, 0);

    // single item viewport: ensure offset keeps selected visible
    let mut q = Panel::new(PathBuf::from("/"));
    q.entries = (0..1)
        .map(|i| Entry {
            name: format!("g{}", i),
            path: PathBuf::from(format!("/g{}", i)),
            is_dir: false,
            size: 0,
            modified: None,
        })
        .collect();
    q.selected = 0;
    q.offset = 5; // intentionally out of range
    q.ensure_selected_visible(1);
    assert_eq!(q.offset, 0);
}
