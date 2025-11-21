use ratatui::style::{Color, Style};
use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct Theme { pub bg: Color, pub fg: Color, pub accent: Color }

#[derive(Deserialize)]
struct Pal { bg: String, fg: String, accent: String }

impl Theme {
    pub fn dark() -> Self { Self{ bg: Color::Rgb(11,12,13), fg: Color::Gray, accent: Color::Cyan } }
    pub fn light() -> Self { Self{ bg: Color::White, fg: Color::Black, accent: Color::Blue } }
    pub fn style_fg(&self) -> Style { Style::default().fg(self.fg).bg(self.bg) }
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        let v: toml::Value = toml::from_str(s)?;
        if let Some(p) = v.get("palette") {
            let p: Pal = p.clone().try_into()?; let bg = parse_hex(&p.bg); let fg = parse_hex(&p.fg); let ac = parse_hex(&p.accent);
            return Ok(Self{ bg, fg, accent: ac });
        }
        Ok(Self::dark())
    }
}

fn parse_hex(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    if s.len() == 6 { if let (Ok(r),Ok(g),Ok(b)) = (u8::from_str_radix(&s[0..2],16), u8::from_str_radix(&s[2..4],16), u8::from_str_radix(&s[4..6],16)) { return Color::Rgb(r,g,b); } }
    Color::Reset
}
