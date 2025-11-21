//! Small utilities for working with symbolic links.
//!
//! These helpers centralize symlink-specific behavior so callers don't need to
//! duplicate platform distinctions. The functions are intentionally minimal —
//! they return `io::Result` and propagate underlying OS errors so callers can
//! decide whether to fall back to copying, elevation, or another strategy.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Create a symbolic link at `dst` that points to `src`.
///
/// Behaviour:
/// - On Unix this delegates to `std::os::unix::fs::symlink`.
/// - On Windows this chooses `symlink_dir` or `symlink_file` depending on
///   whether `src` looks like a directory (uses `metadata().is_dir()`).
///
/// This function does not attempt to elevate privileges on Windows; callers
/// must handle permission errors if needed.
#[allow(dead_code)]
pub(crate) fn create_symlink(src: &Path, dst: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink as unix_symlink;
        unix_symlink(src, dst)
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::{symlink_dir, symlink_file};

        // If `src` exists we can choose the right symlink kind. If it doesn't
        // exist, default to file symlink to avoid accidentally creating a
        // directory junction.
        let use_dir = src.metadata().map(|m| m.is_dir()).unwrap_or(false);
        if use_dir {
            symlink_dir(src, dst)
        } else {
            symlink_file(src, dst)
        }
    }
}

/// Returns `true` when `path` itself is a symbolic link.
///
/// This checks the *link* metadata (does not follow the link).
#[allow(dead_code)]
pub(crate) fn is_symlink(path: &Path) -> io::Result<bool> {
    Ok(path.symlink_metadata()?.file_type().is_symlink())
}

/// Read the target of a symbolic link.
///
/// Returns the path that the symbolic link points to. This is a thin wrapper
/// around `Path::read_link` for symmetry with other helpers.
#[allow(dead_code)]
pub(crate) fn read_symlink(path: &Path) -> io::Result<PathBuf> {
    path.read_link()
}

/// Remove a symbolic link at `path`.
///
/// On Unix `remove_file` is used. On Windows we prefer `remove_dir` when the
/// symlink target is a directory (determined via `metadata().is_dir()`) and
/// `remove_file` otherwise.
#[allow(dead_code)]
pub(crate) fn remove_symlink(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        fs::remove_file(path)
    }

    #[cfg(windows)]
    {
        // `metadata()` follows the symlink, so it tells us whether the target
        // is a directory or not; use the appropriate remove call.
        let meta = path.metadata()?;
        if meta.is_dir() {
            fs::remove_dir(path)
        } else {
            fs::remove_file(path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::Write;

    // These tests run on Unix CI; Windows symlink creation often requires
    // developer privileges and should be tested separately on Windows hosts.
    #[cfg(unix)]
    #[test]
    fn create_and_remove_file_symlink() -> io::Result<()> {
        let tmp = tempdir()?;
        let file = tmp.path().join("file.txt");
        let mut f = fs::File::create(&file)?;
        writeln!(f, "hello")?;

        let link = tmp.path().join("file.link");
        create_symlink(&file, &link)?;
        assert!(is_symlink(&link)?);
        let target = read_symlink(&link)?;
        assert_eq!(target.file_name(), Some(std::ffi::OsStr::new("file.txt")));

        remove_symlink(&link)?;
        assert!(!link.exists());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn create_and_remove_dir_symlink() -> io::Result<()> {
        let tmp = tempdir()?;
        let dir = tmp.path().join("somedir");
        fs::create_dir(&dir)?;

        let link = tmp.path().join("dir.link");
        create_symlink(&dir, &link)?;
        assert!(is_symlink(&link)?);
        remove_symlink(&link)?;
        assert!(!link.exists());
        Ok(())
    }
}
