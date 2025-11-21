use fileZoom::ui::{Theme};
use fileZoom::ui::colors::{set_from_theme, current};
use ratatui::style::Color;

#[test]
fn colors_derive_panel_selected_from_theme_tokens() {
    let s = r###"
    palette = { bg = "#000000", fg = "#FFFFFF", accent = "#00FF00" }
    [panels]
    selected_bg = "#FF00FF"
    panel_bg = "#101010"
    "###;

    let theme = Theme::from_toml(s).expect("parsed");
    set_from_theme(&theme);

    let cols = current();
    assert_eq!(cols.panel_selected_style.bg, Some(Color::Rgb(255,0,255)));
    assert_eq!(cols.panel_block_style.bg, Some(Color::Rgb(16,16,16)));
    // menu tokens fallback to palette values when not provided.
    assert_eq!(cols.menu_style.bg, Some(Color::Rgb(0,0,0)));
    assert_eq!(cols.menu_style.fg, Some(Color::Rgb(255,255,255)));
}
