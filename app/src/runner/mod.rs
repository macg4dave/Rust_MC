//! Runner module - orchestrates the TUI application run loop.
//!
//! This module is intentionally thin; implementation lives in submodules to
//! keep code organized: `terminal` for terminal setup, `event_loop` for the
//! main loop, and `commands` for pure helpers that mutate `App` state.

pub mod commands;
pub mod event_loop_main;
pub mod handlers;
pub mod progress;
pub mod terminal;
#[cfg(feature = "fs-watch")]
pub mod watch_helpers;

pub use event_loop_main::run_app;
