// Deprecated compatibility shim. See `app::path` deprecation attribute in
// `app/src/app.rs` — this file keeps the old import path working by
// re-exporting the canonical implementation under `fs_op::path`.

pub use crate::fs_op::path::*;
