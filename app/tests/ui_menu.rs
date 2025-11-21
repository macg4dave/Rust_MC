use fileZoom::ui::menu::menu_labels;

#[test]
fn menu_labels_expected() {
    let labels = menu_labels();
    assert_eq!(
        labels,
        vec!["File", "Copy", "Move", "New", "Sort", "Settings", "Help"]
    );
}
