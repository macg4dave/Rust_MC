//! Compatibility shim: path helpers live under `crate::fs_op::path`.
//!
//! Keep this module so callers importing `crate::app::core::path::resolve_path`
//! continue to compile. The canonical implementation is in
//! `crate::fs_op::path`.

pub use crate::fs_op::path::resolve_path;
