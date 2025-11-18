//! Input helpers and unified input types.
//!
//! This module provides a small abstraction layer over terminal input events
//! (keyboard, mouse, and resize) to make handling input easier and platform
//! independent inside the application. It currently uses `crossterm` for
//! low-level event polling and reading, but exposes crate-local types for
//! mouse events so the rest of the codebase does not depend directly on
//! `crossterm` internals.
//!
//! Examples
//!
//! ```ignore
//! use std::time::Duration;
//! // poll for any input for 100ms
//! if crate::input::poll(Duration::from_millis(100))? {
//!     match crate::input::read_event()? {
//!         crate::input::InputEvent::Key(k) => { /* handle keyboard */ }
//!         crate::input::InputEvent::Mouse(m) => { /* handle mouse */ }
//!         crate::input::InputEvent::Resize(w,h) => { /* handle resize */ }
//!         _ => {}
//!     }
//! }
//! ```
//
pub mod keyboard;
pub mod mouse;

pub use keyboard::*;
pub use mouse::*;

use std::time::Duration;

/// Unified, cross-platform input event for the app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
	Key(KeyEvent),
	Mouse(MouseEvent),
	Resize(u16, u16),
	Other,
}

/// Poll for an input event with a timeout. This delegates to `crossterm::event::poll`.
pub fn poll(timeout: Duration) -> anyhow::Result<bool> {
	Ok(crossterm::event::poll(timeout)?)
}

/// Read the next input event and map it to `InputEvent`. Mouse events are converted
/// into crate-local `MouseEvent` types via `From<crossterm::event::MouseEvent>`.
pub fn read_event() -> anyhow::Result<InputEvent> {
	match crossterm::event::read()? {
		crossterm::event::Event::Key(k) => Ok(InputEvent::Key(k)),
		crossterm::event::Event::Mouse(m) => Ok(InputEvent::Mouse(m.into())),
		crossterm::event::Event::Resize(w, h) => Ok(InputEvent::Resize(w, h)),
		_ => Ok(InputEvent::Other),
	}
}
