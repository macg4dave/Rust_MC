use std::fs;
use std::path::Path;
use std::io;
use thiserror::Error;

/// Errors returned from filesystem remove operations in this module.
///
/// This type wraps underlying `std::io::Error` values. The API currently
/// intentionally treats "path does not exist" as success (no-op) so callers
/// can attempt removal without checking for existence first.
#[derive(Debug, Error)]
#[error("filesystem remove error: {0}")]
pub struct RemoveError(#[from] pub io::Error);

/// Remove a file or directory at `path`.
///
/// If `path` refers to a directory it will be removed recursively. If the
/// path does not exist this function returns `Ok(())` so callers do not need
/// to check for existence before attempting removal.
///
/// # Examples
///
/// ```no_run
/// use fileZoom::fs_op::remove::remove_path;
/// remove_path("/tmp/some_file.txt").expect("remove failed");
/// ```
pub fn remove_path(path: impl AsRef<Path>) -> Result<(), RemoveError> {
    let p = path.as_ref();

    if !p.exists() {
        // Nothing to do; removal of a non-existent path is a no-op.
        return Ok(());
    }

    if p.is_dir() {
        fs::remove_dir_all(p)?;
    } else {
        fs::remove_file(p)?;
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn remove_file_and_dir_ok() {
        let td = tempdir().expect("create temp dir");
        let dir = td.path().join("sub");
        std::fs::create_dir_all(&dir).expect("create subdir");
        let f = dir.join("f.txt");
        std::fs::write(&f, b"x").expect("write file");

        remove_path(&f).expect("remove file");
        assert!(!f.exists(), "file should be removed");

        remove_path(&dir).expect("remove dir");
        assert!(!dir.exists(), "dir should be removed");
    }

    #[test]
    fn remove_nonexistent_is_ok() {
        let td = tempdir().expect("tempdir");
        let p = td.path().join("does_not_exist");
        assert!(!p.exists());
        assert!(remove_path(&p).is_ok());
    }
}
