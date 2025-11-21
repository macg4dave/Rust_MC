use fileZoom::ui::Theme;
use ratatui::style::Color;

#[test]
fn parse_theme_with_panel_tokens() {
    let s = r###"
    palette = { bg = "#010203", fg = "#0A0B0C", accent = "#AABBCC" }
    [panels]
    selected_bg = "#FF00FF"
    panel_bg = "#101112"
    preview_fg = "#123456"
    "###;

    let t = Theme::from_toml(s).expect("parse toml");
    assert!(t.panels.is_some());
    let p = t.panels.unwrap();
    assert_eq!(p.selected_bg, Some(Color::Rgb(255,0,255)));
    assert_eq!(p.panel_bg, Some(Color::Rgb(16,17,18)));
    assert_eq!(p.preview_fg, Some(Color::Rgb(18,52,86)));
}
