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
			CtKind::Drag(_) => MouseEventKind::Move,
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

#[cfg(test)]
mod tests {
	use super::*;
	use crossterm::event::{MouseEvent as CtME, MouseEventKind as CtKind, MouseButton as CtBtn, KeyModifiers};

	#[test]
	fn convert_left_down() {
		let ct = CtME {
			kind: CtKind::Down(CtBtn::Left),
			column: 10,
			row: 4,
			modifiers: KeyModifiers::NONE,
		};
		let me: MouseEvent = ct.into();
		assert_eq!(me.column, 10);
		assert_eq!(me.row, 4);
		assert!(matches!(me.kind, MouseEventKind::Down(MouseButton::Left)));
	}

	#[test]
	fn convert_scrolls() {
		let up = CtME {
			kind: CtKind::ScrollUp,
			column: 0,
			row: 0,
			modifiers: KeyModifiers::NONE,
		};
		let down = CtME {
			kind: CtKind::ScrollDown,
			column: 1,
			row: 2,
			modifiers: KeyModifiers::NONE,
		};
		let up_conv: MouseEvent = up.into();
		let down_conv: MouseEvent = down.into();
		assert!(matches!(up_conv.kind, MouseEventKind::ScrollUp));
		assert!(matches!(down_conv.kind, MouseEventKind::ScrollDown));
	}
}
