use fileZoom::ui::themes::Theme;

#[test]
fn default_themes_differ() {
    let d = Theme::dark();
    let l = Theme::light();
    assert_ne!(format!("{:?}", d.fg), format!("{:?}", l.fg));
}
