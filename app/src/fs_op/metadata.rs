use std::fs;
use std::io;
use std::path::Path;

// Minimal metadata preservation helpers.
//
// These helpers provide small utilities to copy common metadata (permissions)
// from `src` to `dst`. They are intentionally conservative and focus on
// permissions because ownership/UID/GID aren't portable across systems.

/// Copy file permission bits from `src` to `dst` when possible.
#[allow(dead_code)]
pub(crate) fn copy_permissions(src: &Path, dst: &Path) -> io::Result<()> {
    let perms = fs::metadata(src)?.permissions();
    fs::set_permissions(dst, perms)
}

/// Placeholder for future metadata copying (timestamps, ownership).
/// Implement platform-specific behavior here when needed.
#[allow(dead_code)]
pub(crate) fn preserve_all_metadata(_src: &Path, _dst: &Path) -> io::Result<()> {
    // No-op for now; explicit helpers can be added when the project
    // decides to preserve timestamps/ownership.
    Ok(())
}
