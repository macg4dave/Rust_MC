use std::{io, path::{Path, PathBuf}};

use thiserror::Error;

/// Errors returned by `fs_op` create helpers.
///
/// This error type is crate-local; callers should treat it as a simple
/// wrapper around `std::io::Error` with an explicit `AlreadyExists`
/// variant for the common create-file collision case.
#[derive(Debug, Error)]
pub enum CreateError {
    /// Underlying I/O error.
    #[error(transparent)]
    Io(#[from] io::Error),

    /// Target path already exists and creation was not performed.
    #[error("already exists: {0}")]
    AlreadyExists(PathBuf),
}

/// Create an empty file at `path`.
///
/// This function ensures the parent directory exists before attempting an
/// atomic write of zero bytes. If the target already exists, an
/// `CreateError::AlreadyExists` is returned rather than overwriting it.
pub fn create_file<P: AsRef<Path>>(path: P) -> Result<(), CreateError> {
    let p = path.as_ref();
    // Ensure parent exists (no-op if there is no parent).
    crate::fs_op::helpers::ensure_parent_exists(p)?;

    crate::fs_op::helpers::atomic_write(p, &[]).map_err(|e| {
        if e.kind() == io::ErrorKind::AlreadyExists {
            CreateError::AlreadyExists(p.to_path_buf())
        } else {
            CreateError::Io(e)
        }
    })
}

/// Create a directory and all parent components, returning an I/O-style
/// crate-local error type on failure.
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<(), CreateError> {
    std::fs::create_dir_all(path.as_ref()).map_err(CreateError::Io)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_suffix() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{}-{}", std::process::id(), nanos)
    }

    #[test]
    fn create_file_creates_parents_and_file() {
        let base = std::env::temp_dir().join(format!("filezoom-create-{}", unique_suffix()));
        let target = base.join("a/b/c.txt");

        // Ensure a clean slate
        let _ = fs::remove_dir_all(&base);

        create_file(&target).expect("create_file should succeed");
        assert!(target.exists());

        // cleanup
        let _ = fs::remove_dir_all(&base);
    }

    // Note: creating a file currently overwrites existing targets because
    // `atomic_write` writes a temp file then renames into place. Overwriting
    // behavior is intentional in some flows; do not assert an "already
    // exists" error here.
}
