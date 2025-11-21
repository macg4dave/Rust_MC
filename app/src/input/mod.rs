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

#[cfg(feature = "async-input")]
use crossterm::event::Event;
#[cfg(feature = "async-input")]
use std::sync::{mpsc::Receiver as MpscReceiver, OnceLock};

// When async input is enabled we expose a small install point so the
// async EventStream producer (spawned from `main`) can forward events to
// the synchronous `read_event()` path without changing the rest of the
// runner. This uses a single global receiver which is set once at
// startup by the main thread.
#[cfg(feature = "async-input")]
static ASYNC_EVENT_RX: OnceLock<MpscReceiver<Event>> = OnceLock::new();

/// Install a receiver that will be polled by `read_event()` before
/// falling back to `crossterm::event::read()`.
#[cfg(feature = "async-input")]
pub fn install_async_event_receiver(rx: MpscReceiver<Event>) {
    let _ = ASYNC_EVENT_RX.set(rx);
}

/// Unified, cross-platform input event for the app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    Key(crate::input::keyboard::KeyCode),
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
    // If async-input feature is enabled and an async event receiver was
    // installed, try it first (non-blocking). This allows an async
    // EventStream producer to deliver events into the existing sync
    // runner without a larger refactor.
    #[cfg(feature = "async-input")]
    {
        if let Some(rx) = ASYNC_EVENT_RX.get() {
            match rx.try_recv() {
                Ok(ev) => match ev {
                    crossterm::event::Event::Key(k) => return Ok(InputEvent::Key(k.into())),
                    crossterm::event::Event::Mouse(m) => return Ok(InputEvent::Mouse(m.into())),
                    crossterm::event::Event::Resize(w, h) => return Ok(InputEvent::Resize(w, h)),
                    _ => return Ok(InputEvent::Other),
                },
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    // Producer died; clear the receiver so we don't spin on it.
                    // Note: OnceLock cannot be cleared, so just ignore and fall
                    // through to the synchronous read path.
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }
    }
    match crossterm::event::read()? {
        crossterm::event::Event::Key(k) => Ok(InputEvent::Key(k.into())),
        crossterm::event::Event::Mouse(m) => Ok(InputEvent::Mouse(m.into())),
        crossterm::event::Event::Resize(w, h) => Ok(InputEvent::Resize(w, h)),
        _ => Ok(InputEvent::Other),
    }
}

// Async input helpers (feature-gated). When the `async-input` feature is
// enabled the crate exposes an `async_input` module which provides an
// `EventStream`-based helper compatible with async runtimes.
#[cfg(feature = "async-input")]
pub mod async_input;

// Notes for async vs sync usage and diagnostics:
// - For synchronous applications we prefer `crossterm::event::poll(timeout)`
//   (used throughout the runner) to avoid blocking indefinitely and to
//   allow batching/aggregation of bursts of events.
// - For async applications consider `crossterm`'s `EventStream` feature which
//   integrates with `tokio`/`async-std` and provides a Stream of events.
// - `read_event` will return an error if the underlying `crossterm::event::read`
//   fails; callers may log and continue on transient IO errors to keep the
//   application resilient (the runner already coalesces and logs input errors).
