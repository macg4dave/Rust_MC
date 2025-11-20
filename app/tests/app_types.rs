use fileZoom::app::Action;
use fileZoom::app::Entry;
use std::path::PathBuf;

#[test]
fn action_debug_and_clone() {
    let p = PathBuf::from("/tmp/x");
    let a = Action::CopyTo(p.clone());
    let b = a.clone();
    assert_eq!(format!("{:?}", a), format!("{:?}", b));
}

#[test]
fn entry_equality() {
    let e1 = Entry::file("a", PathBuf::from("a"), 0, None);
    let e2 = e1.clone();
    assert_eq!(e1, e2);
}

#[test]
fn synthetic_header_and_parent_detection() {
    let cwd = PathBuf::from("/tmp");
    let h = fileZoom::app::Entry::header(cwd.clone());
    assert!(fileZoom::ui::panels::is_entry_header(&h));
    assert!(!fileZoom::ui::panels::is_entry_parent(&h));

    let parent = fileZoom::app::Entry::parent(PathBuf::from("/"));
    assert!(fileZoom::ui::panels::is_entry_parent(&parent));
    assert!(!fileZoom::ui::panels::is_entry_header(&parent));
}

#[test]
fn sortkey_next_cycles() {
    use fileZoom::app::SortKey;
    assert_eq!(SortKey::Name.next(), SortKey::Size);
    assert_eq!(SortKey::Size.next(), SortKey::Modified);
    assert_eq!(SortKey::Modified.next(), SortKey::Name);
}

#[test]
fn mode_default_is_normal() {
    use fileZoom::app::Mode;
    let d: Mode = Default::default();
    match d {
        Mode::Normal => (),
        _ => panic!("Default Mode should be Normal"),
    }
}

#[test]
fn action_and_enum_display() {
    use fileZoom::app::{Action, Side, SortKey};
    use std::path::PathBuf;
    let p = PathBuf::from("/tmp/x");
    let a = Action::CopyTo(p.clone());
    assert!(format!("{}", a).contains("/tmp/x"));
    assert_eq!(format!("{}", SortKey::Name), "Name");
    assert_eq!(format!("{}", SortKey::Size), "Size");
    assert_eq!(format!("{}", SortKey::Modified), "Modified");
    assert_eq!(format!("{}", Side::Left), "Left");
    assert_eq!(format!("{}", Side::Right), "Right");
}
