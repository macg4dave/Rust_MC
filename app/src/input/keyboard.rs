//! Keyboard input helpers and crate-local key types.
//!
//! This module provides a small, testable abstraction over terminal key
//! events. It intentionally avoids exposing `crossterm` types in the public
//! surface so application logic can be tested without a terminal.
use core::fmt;

/// Logical key code (application-level).
///
/// Mirrors the most commonly-used `crossterm` `KeyCode` variants while keeping
/// the enum compact and stable for the rest of the crate.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum KeyCode {
    /// Printable Unicode character.
    Char(char),
    Enter,
    Esc,
    Backspace,
    Tab,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Insert,
    /// Function key (F1..F12+). Value is the function index (1..).
    F(u8),
    /// No key (used by some platforms).
    Null,
    /// Any other key not represented above.
    Other,
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyCode::Char(c) => write!(f, "{}", c),
            KeyCode::F(n) => write!(f, "F{}", n),
            other => write!(f, "{:?}", other),
        }
    }
}

/// Modifier keys attached to a key event.
///
/// This is intentionally small and copyable to keep passing around keys cheap.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct KeyModifiers {
    /// Control key
    pub ctrl: bool,
    /// Alt/Meta key
    pub alt: bool,
    /// Shift key
    pub shift: bool,
    /// Logo / Windows / Command key
    pub logo: bool,
}

impl KeyModifiers {
    /// Returns true if any modifier is set.
    pub fn is_any(&self) -> bool {
        self.ctrl || self.alt || self.shift || self.logo
    }
}

/// Application-level key event: a `KeyCode` with optional modifiers.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Key {
    /// The logical key code.
    pub code: KeyCode,
    /// Associated modifiers.
    pub modifiers: KeyModifiers,
}

impl Key {
    /// Construct a key with no modifiers.
    pub fn simple(code: KeyCode) -> Self {
        Self { code, modifiers: KeyModifiers::default() }
    }

    /// Convenience: true if the event is a printable character (without
    /// control/alt modifiers). This is useful when deciding whether to insert
    /// text into an editor field.
    pub fn is_printable(&self) -> bool {
        matches!(self.code, KeyCode::Char(_)) && !self.modifiers.is_any()
    }
}

/// Backwards-compatible helper: return true if the `KeyCode` represents a
/// printable character. Prefer `Key::is_printable` when modifiers are
/// relevant.
pub fn is_printable_key(k: &KeyCode) -> bool {
    matches!(k, KeyCode::Char(_))
}

// Conversion from crossterm types is implemented here; crossterm is a direct
// dependency of this crate so these conversions are always available.
impl From<crossterm::event::KeyCode> for KeyCode {
    fn from(k: crossterm::event::KeyCode) -> Self {
        use crossterm::event::KeyCode as CtKC;
        match k {
            CtKC::Char(c) => KeyCode::Char(c),
            CtKC::Enter => KeyCode::Enter,
            CtKC::Esc => KeyCode::Esc,
            CtKC::Backspace => KeyCode::Backspace,
            CtKC::Tab => KeyCode::Tab,
            CtKC::Left => KeyCode::Left,
            CtKC::Right => KeyCode::Right,
            CtKC::Up => KeyCode::Up,
            CtKC::Down => KeyCode::Down,
            CtKC::Home => KeyCode::Home,
            CtKC::End => KeyCode::End,
            CtKC::PageUp => KeyCode::PageUp,
            CtKC::PageDown => KeyCode::PageDown,
            CtKC::Delete => KeyCode::Delete,
            CtKC::Insert => KeyCode::Insert,
            CtKC::F(n) => KeyCode::F(n),
            CtKC::Null => KeyCode::Null,
            _ => KeyCode::Other,
        }
    }
}

impl From<crossterm::event::KeyModifiers> for KeyModifiers {
    fn from(m: crossterm::event::KeyModifiers) -> Self {
        Self {
            ctrl: m.contains(crossterm::event::KeyModifiers::CONTROL),
            alt: m.contains(crossterm::event::KeyModifiers::ALT),
            shift: m.contains(crossterm::event::KeyModifiers::SHIFT),
            logo: m.contains(crossterm::event::KeyModifiers::SUPER),
        }
    }
}

impl From<crossterm::event::KeyEvent> for Key {
    fn from(ev: crossterm::event::KeyEvent) -> Self {
        Key { code: KeyCode::from(ev.code), modifiers: KeyModifiers::from(ev.modifiers) }
    }
}

/// Backwards-compatible conversion: some call-sites convert a `KeyEvent`
/// directly into the crate-local `KeyCode`. Preserve that behaviour so
/// existing code continues to work.
impl From<crossterm::event::KeyEvent> for KeyCode {
    fn from(ev: crossterm::event::KeyEvent) -> Self { KeyCode::from(ev.code) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn printable_char_without_modifiers_is_printable() {
        let k = Key::simple(KeyCode::Char('a'));
        assert!(k.is_printable());
    }

    #[test]
    fn printable_char_with_ctrl_is_not_printable() {
        let mut k = Key::simple(KeyCode::Char('a'));
        k.modifiers.ctrl = true;
        assert!(!k.is_printable());
    }

    #[test]
    fn modifier_is_any() {
        let m = KeyModifiers { ctrl: false, alt: true, shift: false, logo: false };
        assert!(m.is_any());
        let m2 = KeyModifiers::default();
        assert!(!m2.is_any());
    }
}
