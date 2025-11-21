use fileZoom::ui::themes::Theme;

#[test]
fn load_dark_theme_from_file() {
    let p = format!("{}/resources/themes/dark.toml", env!("CARGO_MANIFEST_DIR"));
    let s = std::fs::read_to_string(p).expect("read theme");
    let t = Theme::from_toml(&s).expect("parse");
    // expect accent not reset
    assert_ne!(format!("{:?}", t.accent), format!("{:?}", fileZoom::ui::themes::Theme::dark().accent));
}
