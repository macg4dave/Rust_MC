//! Lightweight compatibility layer re-exporting select filesystem operations.
//!
//! Historically these helpers were exposed from `fs_op::files`. The
//! implementations live in more focused submodules (e.g. `create`, `mv`,
//! `permissions`, `remove`, `stat`). Re-exporting them here preserves the
//! previous public API so callers can continue to import from
//! `fileZoom::fs_op::files::{...}` without churn.
//!
//! This module intentionally only re-exports a small, stable surface:
//! - construction helpers (`create_file`, `create_dir_all`)
//! - move/copy/rename helpers
//! - permission inspection / formatting
//! - basic path predicates and removal
//!
//! Prefer using the more specific submodules for new code (for example
//! `fileZoom::fs_op::create::create_file`) but keep this module for
//! backwards compatibility and tests.

/// Create helpers (files & directories).
pub use crate::fs_op::create::{create_dir_all, create_file};

/// Move/copy/rename helpers.
pub use crate::fs_op::mv::{copy_path, move_path, rename_path};

/// Permission helpers and related types.
pub use crate::fs_op::permissions::{
    change_permissions,
    format_unix_mode,
    inspect_permissions,
    PermissionInfo,
};

/// Remove a path (file or directory) recursively when appropriate.
pub use crate::fs_op::remove::remove_path;

/// Stat-based helpers for simple predicates.
pub use crate::fs_op::stat::{exists, is_dir, is_file};
