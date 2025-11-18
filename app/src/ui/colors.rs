use std::sync::{OnceLock, RwLock};
use tui::style::{Color, Modifier, Style};

/// Theme holds the styles used across the UI. Cloneable so callers can
/// take a snapshot for rendering without holding locks.
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
}

impl Theme {
    /// Default (light-friendly) theme.
    pub fn default() -> Self {
        Theme {
            border_active: Style::default().fg(Color::Yellow),
            border_inactive: Style::default(),

            dir_style: Style::default().fg(Color::Blue),
            parent_style: Style::default().fg(Color::Blue).add_modifier(Modifier::ITALIC),

            highlight_style: Style::default().add_modifier(Modifier::BOLD).bg(Color::Gray),

            preview_block_style: Style::default(),

            help_block_style: Style::default(),

            header_style: Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        }
    }

    /// Dark theme variant.
    pub fn dark() -> Self {
        Theme {
            border_active: Style::default().fg(Color::Yellow),
            border_inactive: Style::default().fg(Color::Gray),

            dir_style: Style::default().fg(Color::LightBlue),
            parent_style: Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC),

            highlight_style: Style::default().bg(Color::Blue).fg(Color::Black).add_modifier(Modifier::BOLD),

            preview_block_style: Style::default().fg(Color::LightBlue),

            help_block_style: Style::default().fg(Color::Gray).bg(Color::Black),

            header_style: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
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
    RwLock::new(ThemeState { name: ThemeName::Default, theme: Theme::default() })
}

/// Return a clone of the current theme.
pub fn current() -> Theme {
    THEME_STATE
        .get_or_init(init_state)
        .read()
        .unwrap()
        .theme
        .clone()
}

/// Set the active theme by name. Supported names: "default", "dark".
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

/// Toggle between default and dark themes.
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
