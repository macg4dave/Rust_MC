use ratatui::style::{Color, Modifier, Style};
use std::sync::{OnceLock, RwLock};

#[derive(Clone, Debug)]
pub struct Theme {
    pub border_active: Style,
    pub border_inactive: Style,

    pub dir_style: Style,
    pub parent_style: Style,

    pub highlight_style: Style,

    pub preview_block_style: Style,

    pub help_block_style: Style,

    pub header_style: Style,
    pub scrollbar_style: Style,
    pub scrollbar_thumb_style: Style,
}

impl Theme {
    pub fn dark() -> Self {
        Theme {
            border_active: Style::default().fg(Color::Yellow),
            border_inactive: Style::default().fg(Color::Gray),

            dir_style: Style::default().fg(Color::LightBlue),
            parent_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::ITALIC),

            highlight_style: Style::default()
                .bg(Color::Blue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),

            preview_block_style: Style::default().fg(Color::LightBlue),

            help_block_style: Style::default().fg(Color::Gray).bg(Color::Black),

            header_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            scrollbar_style: Style::default().fg(Color::Gray),
            scrollbar_thumb_style: Style::default().fg(Color::White).bg(Color::Blue),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            border_active: Style::default().fg(Color::Yellow),
            border_inactive: Style::default(),

            dir_style: Style::default().fg(Color::Blue),
            parent_style: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::ITALIC),

            highlight_style: Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Gray),

            preview_block_style: Style::default(),

            help_block_style: Style::default(),

            header_style: Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
            scrollbar_style: Style::default().fg(Color::Gray),
            scrollbar_thumb_style: Style::default().fg(Color::Black).bg(Color::Gray),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThemeName {
    Default,
    Dark,
}

#[derive(Clone, Debug)]
struct ThemeState {
    name: ThemeName,
    theme: Theme,
}

static THEME_STATE: OnceLock<RwLock<ThemeState>> = OnceLock::new();

fn init_state() -> RwLock<ThemeState> {
    RwLock::new(ThemeState {
        name: ThemeName::Default,
        theme: Theme::default(),
    })
}

pub fn current() -> Theme {
    THEME_STATE
        .get_or_init(init_state)
        .read()
        .unwrap()
        .theme
        .clone()
}

pub fn set_theme(name: &str) {
    let mut guard = THEME_STATE.get_or_init(init_state).write().unwrap();
    match name {
        "dark" => {
            guard.name = ThemeName::Dark;
            guard.theme = Theme::dark();
        }
        _ => {
            guard.name = ThemeName::Default;
            guard.theme = Theme::default();
        }
    }
}

pub fn toggle() {
    let mut guard = THEME_STATE.get_or_init(init_state).write().unwrap();
    match guard.name {
        ThemeName::Default => {
            guard.name = ThemeName::Dark;
            guard.theme = Theme::dark();
        }
        ThemeName::Dark => {
            guard.name = ThemeName::Default;
            guard.theme = Theme::default();
        }
    }
}

pub fn color_samples() -> Vec<(String, Style)> {
    let named: &[(&str, Color)] = &[
        ("Black", Color::Black),
        ("DarkGray", Color::DarkGray),
        ("Red", Color::Red),
        ("LightRed", Color::LightRed),
        ("Green", Color::Green),
        ("LightGreen", Color::LightGreen),
        ("Yellow", Color::Yellow),
        ("LightYellow", Color::LightYellow),
        ("Blue", Color::Blue),
        ("LightBlue", Color::LightBlue),
        ("Magenta", Color::Magenta),
        ("LightMagenta", Color::LightMagenta),
        ("Cyan", Color::Cyan),
        ("LightCyan", Color::LightCyan),
        ("Gray", Color::Gray),
        ("White", Color::White),
    ];

    named
        .iter()
        .map(|(name, color)| (name.to_string(), sample_style_for_color(*color)))
        .collect()
}

pub fn modifier_samples() -> Vec<(String, Style)> {
    let base = Style::default().fg(Color::White).bg(Color::Black);

    let samples = vec![
        ("Normal".to_string(), base),
        ("Bold".to_string(), base.add_modifier(Modifier::BOLD)),
        ("Italic".to_string(), base.add_modifier(Modifier::ITALIC)),
        (
            "Underlined".to_string(),
            base.add_modifier(Modifier::UNDERLINED),
        ),
        (
            "Reversed".to_string(),
            base.add_modifier(Modifier::REVERSED),
        ),
        (
            "CrossedOut".to_string(),
            base.add_modifier(Modifier::CROSSED_OUT),
        ),
        ("Dim".to_string(), base.add_modifier(Modifier::DIM)),
    ];

    samples
}

pub fn apply_modifiers(mut style: Style, mods: &[&str]) -> Style {
    for m in mods {
        match m.to_ascii_lowercase().as_str() {
            "bold" => style = style.add_modifier(Modifier::BOLD),
            "italic" => style = style.add_modifier(Modifier::ITALIC),
            "underlined" | "underline" => style = style.add_modifier(Modifier::UNDERLINED),
            "slow_blink" | "slowblink" | "slow-blink" => {
                style = style.add_modifier(Modifier::SLOW_BLINK)
            }
            "rapid_blink" | "rapidblink" | "rapid-blink" => {
                style = style.add_modifier(Modifier::RAPID_BLINK)
            }
            "reversed" | "reverse" => style = style.add_modifier(Modifier::REVERSED),
            "crossed_out" | "crossedout" | "crossed-out" => {
                style = style.add_modifier(Modifier::CROSSED_OUT)
            }
            "dim" => style = style.add_modifier(Modifier::DIM),
            "hidden" => style = style.add_modifier(Modifier::HIDDEN),
            _ => { /* ignore unknown */ }
        }
    }

    style
}

fn sample_style_for_color(bg: Color) -> Style {
    use Color::*;

    let fg = match bg {
        Black | DarkGray | Red | Blue | Magenta => Color::White,
        LightRed | LightGreen | LightYellow | LightBlue | LightMagenta | LightCyan | White
        | Gray | Green | Yellow | Cyan => Color::Black,
        _ => Color::White,
    };

    Style::default().bg(bg).fg(fg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifier_samples_non_empty() {
        let s = modifier_samples();
        assert!(!s.is_empty());
    }

    #[test]
    fn apply_modifiers_sets_expected() {
        let base = Style::default();
        let got = apply_modifiers(base, &["bold", "italic"]);
        let expected = base
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::ITALIC);
        assert_eq!(got, expected);
    }
}
