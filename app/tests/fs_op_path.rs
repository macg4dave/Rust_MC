use fileZoom::fs_op::path::resolve_path;
use fileZoom::fs_op::path::PathError;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn empty_input_is_error() {
    let base = Path::new("/");
    let r = resolve_path("   ", base);
    assert!(r.is_err());
    assert_eq!(r.unwrap_err(), PathError::Empty);
}

#[test]
fn tilde_expands_to_home() {
    let td = TempDir::new().unwrap();
    std::env::set_var("HOME", td.path());
    let base = Path::new("/irrelevant");
    let got = resolve_path("~", base).unwrap();
    assert_eq!(got, td.path());
}

#[test]
fn relative_resolves_against_base() {
    let td = TempDir::new().unwrap();
    let sub = td.path().join("subdir");
    fs::create_dir_all(&sub).unwrap();
    let got = resolve_path("subdir", td.path()).unwrap();
    assert_eq!(got, sub);
}

#[test]
fn absolute_path_returns_as_is() {
    let td = TempDir::new().unwrap();
    let p = td.path().to_path_buf();
    let got = resolve_path(&p.to_string_lossy(), Path::new("/ignored")).unwrap();
    assert_eq!(got, p);
}

#[test]
fn file_is_not_directory() {
    let td = TempDir::new().unwrap();
    let f = td.path().join("file.txt");
    fs::write(&f, "hello").unwrap();
    let err = resolve_path(&f.to_string_lossy(), td.path()).unwrap_err();
    assert!(matches!(err, PathError::NotDirectory(p) if p == f));
}

#[test]
fn nonexistent_path_errors() {
    let td = TempDir::new().unwrap();
    let p = td.path().join("no-such-dir");
    let err = resolve_path(&p.to_string_lossy(), td.path()).unwrap_err();
    assert!(matches!(err, PathError::NotFound(q) if q == p));
}
