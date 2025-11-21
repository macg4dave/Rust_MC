#![allow(non_snake_case)]
pub mod app;
pub mod errors;
pub mod fs_op;
pub mod input;
pub mod parallel;
#[path = "runner/mod.rs"]
pub mod runner;
#[path = "ui/mod.rs"]
pub mod ui;
pub mod panic_hook;

// Building helpers (refactored from the make_fakefs binary)
#[path = "building/mod.rs"]
pub mod building;

// Test-only helpers. Enabled during `cargo test` or via the
// `test-helpers` feature in Cargo.toml so production builds don't include
// test scaffolding. Using `any(test, feature = "test-helpers")` makes the
// helpers available to the test build without requiring callers to pass the
// feature flag.
#[cfg(any(test, feature = "test-helpers"))]
pub mod test_helpers;

// Re-export the canonical path helpers from `fs_op::path` at the crate root.
// This keeps `fileZoom::path` available without re-exporting the deprecated
// compatibility shim `app::path`.
pub use crate::app::{Action, App, Entry, InputKind, Mode, Side, SortKey};
pub use crate::fs_op::path;

pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
