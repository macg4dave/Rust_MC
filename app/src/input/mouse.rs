// Crate-local mouse types to decouple from `crossterm` internals.
/// Logical mouse buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other,
}

/// Logical mouse event kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseEventKind {
    Down(MouseButton),
    Up(MouseButton),
    /// Drag with a pressed mouse button (button-specific move).
    Drag(MouseButton),
    ScrollUp,
    ScrollDown,
    Move,
    Other,
}

/// Crate-level mouse event (position + kind).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseEvent {
    pub column: u16,
    pub row: u16,
    pub kind: MouseEventKind,
}

/// Convert from `crossterm::event::MouseEvent` into crate-local `MouseEvent`.
impl From<crossterm::event::MouseEvent> for MouseEvent {
    fn from(me: crossterm::event::MouseEvent) -> Self {
        use crossterm::event::{MouseButton as CtBtn, MouseEventKind as CtKind};
        let kind = match me.kind {
            CtKind::Down(bt) => MouseEventKind::Down(match bt {
                CtBtn::Left => MouseButton::Left,
                CtBtn::Right => MouseButton::Right,
                CtBtn::Middle => MouseButton::Middle,
            }),
            CtKind::Up(bt) => MouseEventKind::Up(match bt {
                CtBtn::Left => MouseButton::Left,
                CtBtn::Right => MouseButton::Right,
                CtBtn::Middle => MouseButton::Middle,
            }),
            CtKind::Drag(bt) => MouseEventKind::Drag(match bt {
                CtBtn::Left => MouseButton::Left,
                CtBtn::Right => MouseButton::Right,
                CtBtn::Middle => MouseButton::Middle,
            }),
            CtKind::ScrollUp => MouseEventKind::ScrollUp,
            CtKind::ScrollDown => MouseEventKind::ScrollDown,
            CtKind::Moved => MouseEventKind::Move,
            _ => MouseEventKind::Other,
        };
        MouseEvent {
            column: me.column,
            row: me.row,
            kind,
        }
    }
}

/// Helper to test if a mouse event is a left-click down.
pub fn is_left_down(ev: &MouseEvent) -> bool {
    matches!(ev.kind, MouseEventKind::Down(MouseButton::Left))
}

// Tests moved to `app/tests/input_mouse.rs` to centralize tests outside `src/`.
