// Keyboard input helpers and type aliases.
pub use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Convenience: check if a `KeyEvent` is a printable character
pub fn is_printable_key(ev: &KeyEvent) -> bool {
	match ev.code {
		KeyCode::Char(_) => true,
		_ => false,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crossterm::event::{KeyEvent as CtKeyEvent, KeyCode as CtKeyCode, KeyModifiers};

	#[test]
	fn printable_char_detected() {
		let ev = CtKeyEvent::new(CtKeyCode::Char('a'), KeyModifiers::NONE);
		assert!(is_printable_key(&ev));
	}

	#[test]
	fn non_printable_keys_not_detected() {
		let ev = CtKeyEvent::new(CtKeyCode::Enter, KeyModifiers::NONE);
		assert!(!is_printable_key(&ev));
		let ev2 = CtKeyEvent::new(CtKeyCode::Up, KeyModifiers::NONE);
		assert!(!is_printable_key(&ev2));
	}

	#[test]
	fn modifier_ctrl_char_is_printable() {
		let ev = CtKeyEvent::new(CtKeyCode::Char('c'), KeyModifiers::CONTROL);
		// Current helper only inspects the KeyCode variant, so this remains printable
		assert!(is_printable_key(&ev));
	}

	#[test]
	fn modifier_alt_char_is_printable() {
		let ev = CtKeyEvent::new(CtKeyCode::Char('x'), KeyModifiers::ALT);
		assert!(is_printable_key(&ev));
	}

	#[test]
	fn function_key_not_printable() {
		let ev = CtKeyEvent::new(CtKeyCode::F(1), KeyModifiers::NONE);
		assert!(!is_printable_key(&ev));
	}
}
