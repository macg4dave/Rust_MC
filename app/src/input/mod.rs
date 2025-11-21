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

// Re-export a small, stable public surface for input types. Avoid a
// blanket `pub use *` so downstream modules only rely on the necessary
// symbols and refactors remain smaller.
pub use keyboard::{Key, KeyCode, KeyModifiers};
pub use mouse::{is_left_down, MouseButton, MouseEvent, MouseEventKind};

use std::time::Duration;

#[cfg(feature = "async-input")]
use crossterm::event::Event;
#[cfg(feature = "async-input")]
use std::sync::{mpsc::{self, Receiver as MpscReceiver}, OnceLock, Mutex};

use thiserror::Error;

// When async input is enabled we expose a small install point so the
// async EventStream producer (spawned from `main`) can forward events to
// the synchronous `read_event()` path without changing the rest of the
// runner. This uses a single global receiver which is set once at
// startup by the main thread.
#[cfg(feature = "async-input")]
static ASYNC_EVENT_RX: OnceLock<Mutex<MpscReceiver<Event>>> = OnceLock::new();

// Alias `AsyncReceiver` removed: prefer using the concrete `MpscReceiver<Event>` behind the `Mutex`.

/// Install a receiver that will be polled by `read_event()` before
/// falling back to `crossterm::event::read()`.
#[cfg(feature = "async-input")]
/// Install a cross-thread `mpsc::Receiver<Event>` that will be polled by
/// `read_event()` before falling back to `crossterm::event::read()`.
///
/// This is intended as a small integration point for programs that run an
/// async `EventStream` producer and want to forward events into the
/// existing synchronous runner without a larger refactor. The receiver is
/// stored in a `OnceLock` and can only be set once; subsequent calls are
/// ignored.
pub fn install_async_event_receiver(rx: MpscReceiver<Event>) {
    // We intentionally ignore the Result so callers may call this from
    // different startup paths without panicking if another initializer
    // already set the receiver. The first successful install wins.
    let _ = ASYNC_EVENT_RX.set(Mutex::new(rx));
}

/// Map a `crossterm::event::Event` into the crate-local `InputEvent`.
fn map_crossterm_event(ev: crossterm::event::Event) -> InputEvent {
    match ev {
        crossterm::event::Event::Key(k) => InputEvent::Key(k.into()),
        crossterm::event::Event::Mouse(m) => InputEvent::Mouse(m.into()),
        crossterm::event::Event::Resize(w, h) => InputEvent::Resize(w, h),
        _ => InputEvent::Other,
    }
}

/// Unified, cross-platform input event for the app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    /// Keyboard key event (crate-local `KeyCode`).
    Key(KeyCode),
    /// Mouse event (crate-local `MouseEvent`).
    Mouse(MouseEvent),
    /// Terminal resize: (width, height).
    Resize(u16, u16),
    /// Any other event (focus changes, unsupported kinds, ...).
    Other,
}

/// Typed input errors for the `input` module.
///
/// This enum aims to provide more granular error cases so callers can
/// match on specific conditions (for example to treat a disconnected
/// async receiver differently from an underlying IO error). The current
/// implementation preserves historical behaviour: an async receiver
/// disconnect still falls through to the synchronous `read()` path rather
/// than being returned as an error. The variant is provided for
/// consumers that choose to inspect or extend the behaviour later.
#[derive(Debug, Error)]
pub enum InputError {
    /// Error returned when `crossterm::event::read()` fails. The inner
    /// string contains a formatted representation of the original error.
    #[error("crossterm error: {0}")]
    Crossterm(String),

    /// Wrapper for low-level IO errors. Kept separate so callers that need
    /// to perform IO-specific handling can match on this variant.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// The async event receiver was disconnected. Note: current behaviour
    /// falls back to `crossterm::event::read()` rather than returning this
    /// error; the variant exists for callers who wish to change that
    /// behaviour in future.
    #[error("async event receiver disconnected")]
    AsyncReceiverDisconnected,
}

/// Read the next input event but return a typed `InputError` on failure.
///
/// This helper gives callers a concrete error type to match on while the
/// existing `read_event()` function keeps the legacy `anyhow::Result`
/// signature for compatibility.
pub fn read_event_typed() -> Result<InputEvent, InputError> {
    #[cfg(feature = "async-input")]
    {
        if let Some(rx_mutex) = ASYNC_EVENT_RX.get() {
            use mpsc::TryRecvError;
            match rx_mutex.lock().unwrap().try_recv() {
                Ok(ev) => return Ok(map_crossterm_event(ev)),
                Err(TryRecvError::Disconnected) => {
                    // Preserve historical behaviour: fall through to the
                    // synchronous `read()` path rather than returning an
                    // error so callers keep working when async producer dies.
                }
                Err(TryRecvError::Empty) => {}
            }
        }
    }

    crossterm::event::read()
        .map_err(|e| InputError::Crossterm(format!("{e:?}")))
        .map(map_crossterm_event)
}

/// Poll for an input event with a timeout. This delegates to `crossterm::event::poll`.
pub fn poll(timeout: Duration) -> anyhow::Result<bool> {
    Ok(crossterm::event::poll(timeout)?)
}

/// Read the next input event and map it to `InputEvent`. Mouse events are converted
/// into crate-local `MouseEvent` types via `From<crossterm::event::MouseEvent>`.
pub fn read_event() -> anyhow::Result<InputEvent> {
    // Delegate to the typed helper and convert the concrete error to
    // `anyhow::Error` for backwards compatibility with existing callers.
    read_event_typed().map_err(|e| anyhow::anyhow!(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_error_display() {
        let err = InputError::Crossterm("broken".into());
        let s = format!("{}", err);
        assert!(s.contains("crossterm error"));
    }

    #[cfg(feature = "async-input")]
    #[test]
    fn install_async_receiver_sets_once() {
        use std::sync::mpsc;
        use crossterm::event::Event;

        let (_tx, rx): (mpsc::Sender<Event>, mpsc::Receiver<Event>) = mpsc::channel();
        install_async_event_receiver(rx);
        assert!(ASYNC_EVENT_RX.get().is_some());
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
