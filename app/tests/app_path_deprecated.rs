// This test ensures the deprecated compatibility shim `fileZoom::app::path`
// remains usable. The shim itself is marked `#[deprecated]` and references
// the canonical `fileZoom::fs_op::path` implementation.

#[allow(deprecated)]
use fileZoom::app::path::{resolve_path, PathError};
use std::path::Path;

#[test]
fn deprecated_app_path_still_resolves() {
    let base = Path::new("/");
    let r = resolve_path("   ", base);
    assert!(r.is_err());
    assert_eq!(r.unwrap_err(), PathError::Empty);
}
