// Tests for make_fakefs.rs
// These tests use assert_cmd and predicates crates for CLI testing
// Add to Cargo.toml [dev-dependencies]:
// assert_cmd = "2"
// predicates = "3"

use std::fs;
use std::path::Path;

// Use the test-only helpers provided by the library. These require running
// tests with `--features test-helpers` (the feature exists in Cargo.toml).
use fileZoom::test_helpers::make_fakefs as helpers;

#[test]
fn test_generate_fixtures_creates_files() {
    let dir = helpers::generate_fixtures().expect("create fixtures");
    assert!(dir.exists());
    assert!(dir.join("file1.txt").exists());
    assert!(dir.join("dirA").exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
#[cfg(unix)]
fn test_apply_permissions_unix() {
    let dir = helpers::generate_fixtures().expect("create fixtures");
    let file = dir.join("file1.txt");
    helpers::apply_permissions(&dir).expect("set perms");
    use std::os::unix::fs::PermissionsExt;
    let metadata = fs::metadata(&file).unwrap();
    assert_eq!(metadata.permissions().mode() & 0o777, 0o644);
    let _ = fs::remove_dir_all(&dir);
}
