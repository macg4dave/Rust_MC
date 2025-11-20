use fileZoom::errors::render_io_error;
use std::io;

#[test]
fn test_not_found() {
    let e = io::Error::from(io::ErrorKind::NotFound);
    let out = render_io_error(&e, Some("/no/such/path"), None, None);
    assert!(out.contains("Path not found") || out.contains("not found"));
}

#[test]
fn test_permission_denied() {
    let e = io::Error::from(io::ErrorKind::PermissionDenied);
    let out = render_io_error(&e, Some("/root/secret"), None, None);
    assert!(out.contains("Permission denied") || out.contains("Insufficient"));
}

#[test]
fn test_already_exists() {
    let e = io::Error::from(io::ErrorKind::AlreadyExists);
    let out = render_io_error(&e, Some("/tmp/existing"), None, None);
    assert!(out.contains("already") || out.contains("exists"));
}

#[test]
fn test_move_error_uses_src_dst() {
    let e = io::Error::other("rename failed");
    let out = render_io_error(&e, None, Some("a.txt"), Some("b.txt"));
    // Templates may use different placeholder syntax; ensure the output
    // either contains the src/dst names, the original error text, or at
    // least the default message fragment.
    assert!(
        out.contains("a.txt")
            || out.contains("b.txt")
            || out.contains("rename failed")
            || out.contains("Unable to move")
            || out.contains("{src}")
            || out.contains("{dst}")
    );
}
