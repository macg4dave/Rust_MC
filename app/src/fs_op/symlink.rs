use std::io;
use std::path::Path;

// Minimal symlink helper utilities.
//
// These are intentionally small helpers used by higher-level operations when
// symlink-specific behavior is required. The project currently doesn't use
// symlinks extensively; these functions centralize the handling so behavior
// can be adjusted in one place.

/// Create a symbolic link pointing at `src` named `dst`.
///
/// On platforms that don't support symlinks this will return an error from
/// the underlying stdlib. Callers should decide whether to fall back to copy.
#[allow(dead_code)]
pub(crate) fn create_symlink(src: &Path, dst: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(src, dst)
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::symlink_file;
        // Note: creating directory symlinks on Windows requires a different
        // function and elevated privileges; callers should ensure correct
        // usage.
        symlink_file(src, dst)
    }
}

/// Detect whether `path` is a symlink. Returns true for symbolic links.
#[allow(dead_code)]
pub(crate) fn is_symlink(path: &Path) -> io::Result<bool> {
    Ok(path.symlink_metadata()?.file_type().is_symlink())
}
