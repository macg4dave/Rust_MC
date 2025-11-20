// Keyboard input helpers and type aliases.
pub use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Convenience: check if a `KeyEvent` is a printable character
pub fn is_printable_key(ev: &KeyEvent) -> bool {
    matches!(ev.code, KeyCode::Char(_))
}
