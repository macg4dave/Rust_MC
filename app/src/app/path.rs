//! Path helpers for the `app` crate.
//!
//! Compatibility shim pointing to the canonical implementation under
//! `crate::fs_op::path`. Keeping this module allows previous imports like
//! `crate::app::path::resolve_path` to continue working.

pub use crate::fs_op::path::resolve_path;
