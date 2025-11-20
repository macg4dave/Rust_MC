//! Compatibility shim for path helpers.
//!
//! DEPRECATED: Prefer `fileZoom::fs_op::path` for new code. This module is a
//! backwards-compatibility shim that re-exports the primary implementation.
//!
//! Publicly re-exported items:
//! - `resolve_path` — resolve user-supplied paths for panel cwd changes.
//! - `PathError` — error type returned from `resolve_path`.
pub use crate::fs_op::path::{resolve_path, PathError};
