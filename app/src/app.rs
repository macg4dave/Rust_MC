// Lightweight compatibility shim which re-exports the canonical
// implementation of path helpers found under `fs_op::path`.
// Existing code that imports `crate::app::path` will continue to work,
// but use `fileZoom::fs_op::path` directly for new code.
pub mod core;
pub mod settings;
pub mod types;
pub mod text_editors;

use std::path::PathBuf;

/// Options that affect application startup and initial state.
/// These are intentionally minimal; add more fields as needed.
#[derive(Clone, Debug, Default)]
pub struct StartOptions {
	/// Optional directory to start in. When `None`, the current working
	/// directory is used.
	pub start_dir: Option<PathBuf>,
	/// Optional initial mouse enabled flag. When `None`, persisted
	/// settings (or defaults) are used.
	pub mouse_enabled: Option<bool>,

	/// Optional theme name to apply at startup (e.g. "dark"). When `None`
	/// the persisted setting (or default) is used.
	pub theme: Option<String>,

	/// Optional show-hidden override. When `Some(true)` the UI will show
	/// hidden files at startup; when `None` persisted/default settings are used.
	pub show_hidden: Option<bool>,

	/// Optional verbosity count (mapped from `-v`). When `None` no change
	/// is applied to logging beyond environment defaults.
	pub verbosity: Option<u8>,
}

pub use core::panel::Panel;
pub use core::App;
pub use types::{Action, Entry, InputKind, Mode, Side, SortKey};
// Deprecated compatibility shim: keep `crate::app::path` working for older code/tests.
pub use crate::fs_op::path;
