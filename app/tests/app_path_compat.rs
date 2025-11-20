use fileZoom::app::path::{resolve_path, PathError};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn app_compat_empty_input_is_error() {
    let base = Path::new("/");
    let r = resolve_path("   ", base);
    assert!(r.is_err());
    assert_eq!(r.unwrap_err(), PathError::Empty);
}

#[test]
fn app_compat_relative_resolves_against_base() {
    let td = TempDir::new().unwrap();
    let sub = td.path().join("subdir");
    fs::create_dir_all(&sub).unwrap();
    let got = resolve_path("subdir", td.path()).unwrap();
    assert_eq!(got, sub);
}
