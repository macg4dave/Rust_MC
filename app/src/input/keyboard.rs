// Keyboard input helpers and crate-local key types.
/// Lightweight key code abstraction mirroring the common `crossterm::event::KeyCode`
/// variants used by the application. Keeping this small improves testability and
/// decouples application logic from `crossterm` internals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyCode {
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
    F(u8),
    Null,
    // Catch-all for less-used variants
    Other,
}

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
            CtKC::F(n) => KeyCode::F(n as u8),
            CtKC::Null => KeyCode::Null,
            _ => KeyCode::Other,
        }
    }
}

impl From<crossterm::event::KeyEvent> for KeyCode {
    fn from(ev: crossterm::event::KeyEvent) -> Self {
        // For now the application primarily cares about the logical key
        // code. Modifiers (Ctrl/Alt) may be encoded by callers if needed.
        KeyCode::from(ev.code)
    }
}

/// Convenience: check if a `KeyCode` is a printable character.
pub fn is_printable_key(k: &KeyCode) -> bool {
    matches!(k, KeyCode::Char(_))
}
