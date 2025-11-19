pub mod app;
pub mod errors;
pub mod fs_op;
pub mod input;
#[path = "runner/mod.rs"]
pub mod runner;
#[path = "ui/mod.rs"]
pub mod ui;

// Building helpers (refactored from the make_fakefs binary)
#[path = "building/mod.rs"]
pub mod building;

// Test-only helpers. Enabled via the `test-helpers` feature in Cargo.toml so
// production builds don't include test scaffolding.
#[cfg(feature = "test-helpers")]
pub mod test_helpers;

pub use crate::app::path;
pub use crate::app::{Action, App, Entry, InputKind, Mode, Side, SortKey};

pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

