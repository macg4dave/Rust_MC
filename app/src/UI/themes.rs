use ratatui::style::{Color, Style};
use serde::Deserialize;
#[derive(Clone, Debug)]
pub struct Theme { pub bg: Color, pub fg: Color, pub accent: Color, pub panels: Option<PanelTokens> }

#[derive(Clone, Debug, Default)]
pub struct PanelTokens {
    pub panel_bg: Option<Color>,
    pub panel_fg: Option<Color>,
    pub panel_border: Option<Color>,
    pub panel_title_fg: Option<Color>,
    pub menu_bg: Option<Color>,
    pub menu_fg: Option<Color>,
    pub selected_bg: Option<Color>,
    pub selected_fg: Option<Color>,
    pub inactive_selected_bg: Option<Color>,
    pub preview_bg: Option<Color>,
    pub preview_fg: Option<Color>,
    pub header_bg: Option<Color>,
    pub header_fg: Option<Color>,
    pub footer_bg: Option<Color>,
    pub footer_fg: Option<Color>,
    pub scrollbar_thumb_bg: Option<Color>,
    pub scrollbar_track_bg: Option<Color>,
}

#[derive(Deserialize)]
struct Pal { bg: String, fg: String, accent: String }

#[derive(Deserialize)]
struct PanelsToml {
    panel_bg: Option<String>,
    panel_fg: Option<String>,
    panel_border: Option<String>,
    panel_title_fg: Option<String>,
    selected_bg: Option<String>,
    selected_fg: Option<String>,
    inactive_selected_bg: Option<String>,
    preview_bg: Option<String>,
    preview_fg: Option<String>,
    header_bg: Option<String>,
    header_fg: Option<String>,
    footer_bg: Option<String>,
    footer_fg: Option<String>,
    scrollbar_thumb_bg: Option<String>,
    scrollbar_track_bg: Option<String>,
    menu_bg: Option<String>,
    menu_fg: Option<String>,
}

impl Theme {
    pub fn dark() -> Self { Self{ bg: Color::Rgb(11,12,13), fg: Color::Gray, accent: Color::Cyan, panels: None } }
    pub fn light() -> Self { Self{ bg: Color::White, fg: Color::Black, accent: Color::Blue, panels: None } }
    pub fn style_fg(&self) -> Style { Style::default().fg(self.fg).bg(self.bg) }
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        let v: toml::Value = toml::from_str(s)?;
        let mut panels = None;
        if let Some(tab) = v.get("panels") {
            if let Ok(pt) = tab.clone().try_into::<PanelsToml>() {
                let make = |s: &Option<String>| s.as_ref().map(|x| parse_hex(x));
                panels = Some(PanelTokens {
                    panel_bg: make(&pt.panel_bg),
                    panel_fg: make(&pt.panel_fg),
                    panel_border: make(&pt.panel_border),
                    panel_title_fg: make(&pt.panel_title_fg),
                    menu_bg: make(&pt.menu_bg),
                    menu_fg: make(&pt.menu_fg),
                    selected_bg: make(&pt.selected_bg),
                    selected_fg: make(&pt.selected_fg),
                    inactive_selected_bg: make(&pt.inactive_selected_bg),
                    preview_bg: make(&pt.preview_bg),
                    preview_fg: make(&pt.preview_fg),
                    header_bg: make(&pt.header_bg),
                    header_fg: make(&pt.header_fg),
                    footer_bg: make(&pt.footer_bg),
                    footer_fg: make(&pt.footer_fg),
                    scrollbar_thumb_bg: make(&pt.scrollbar_thumb_bg),
                    scrollbar_track_bg: make(&pt.scrollbar_track_bg),
                });
            }
        }

        if let Some(p) = v.get("palette") {
            let p: Pal = p.clone().try_into()?; let bg = parse_hex(&p.bg); let fg = parse_hex(&p.fg); let ac = parse_hex(&p.accent);
            return Ok(Self{ bg, fg, accent: ac, panels });
        }
        Ok(Self::dark())
    }
}

fn parse_hex(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    if s.len() == 6 { if let (Ok(r),Ok(g),Ok(b)) = (u8::from_str_radix(&s[0..2],16), u8::from_str_radix(&s[2..4],16), u8::from_str_radix(&s[4..6],16)) { return Color::Rgb(r,g,b); } }
    Color::Reset
}
