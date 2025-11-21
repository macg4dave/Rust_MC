use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use thiserror::Error;
use walkdir::WalkDir;

/// Basic permission information for a filesystem path.
#[derive(Debug, Clone)]
pub struct PermissionInfo {
    /// The inspected path (absolute or relative as provided).
    pub path: PathBuf,
    /// If available on the platform, the raw Unix mode bits (e.g. 0o644).
    pub unix_mode: Option<u32>,
    /// Whether the underlying metadata says this path is readonly.
    pub readonly: bool,
    /// Whether a read attempt succeeded (best-effort).
    pub can_read: bool,
    /// Whether a write attempt succeeded (best-effort). This may be false when
    /// `test_write` was false and only metadata was used.
    pub can_write: bool,
    /// Whether the execute bit is set (Unix best-effort).
    pub can_execute: bool,
    /// Whether the path is a directory.
    pub is_dir: bool,
}

impl PermissionInfo {
    /// Create a new `PermissionInfo` for `path` with default (false/None)
    /// values. Callers should populate fields after metadata inspection.
    pub fn new(path: PathBuf) -> Self {
        PermissionInfo {
            path,
            unix_mode: None,
            readonly: false,
            can_read: false,
            can_write: false,
            can_execute: false,
            is_dir: false,
        }
    }
}

/// Errors that can occur while inspecting or changing permissions.
#[derive(Debug, Error)]
pub enum PermissionError {
    /// Underlying I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Operation is not supported on this platform (e.g. setting Unix-only
    /// mode on Windows).
    #[error("operation not supported on this platform")]
    Unsupported,
}

/// Inspect permissions for `path`.
///
/// - `test_write`: when `true`, the function will perform a non-destructive
///   write test for directories by creating and removing a small probe file.
///   For files it will attempt to open for writing. When `false` the function
///   uses metadata and best-effort open checks without creating files.
///
/// This function is best-effort and is intended to give the TUI enough
/// information to decide how to present actions to the user. It avoids making
/// destructive changes by default.
/// Inspect permissions for `path`.
///
/// - `test_write`: when `true`, the function will perform a non-destructive
///   write test for directories by creating and removing a small probe file.
///   For files it will attempt to open for writing. When `false` the function
///   uses metadata and best-effort open checks without creating files.
pub fn inspect_permissions<P: AsRef<Path>>(
    path: P,
    test_write: bool,
) -> Result<PermissionInfo, PermissionError> {
    let path = path.as_ref().to_path_buf();
    let mut info = PermissionInfo::new(path.clone());

    let meta = fs::metadata(&path)?;
    info.is_dir = meta.is_dir();
    info.readonly = meta.permissions().readonly();

    // unix-specific mode if available
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        info.unix_mode = Some(meta.mode());
        // treat execute as any of the exec bits set
        info.can_execute = (meta.mode() & 0o111) != 0;
    }

    // Best-effort read check
    info.can_read = if info.is_dir {
        // Use WalkDir to probe directory readability without listing deeply.
        WalkDir::new(&path)
            .max_depth(0)
            .into_iter()
            .next()
            .is_some_and(|r| r.is_ok())
    } else {
        OpenOptions::new().read(true).open(&path).is_ok()
    };

    // Best-effort write check. If test_write is false, prefer metadata only
    // (non destructive). If true, attempt to open/create a probe to verify.
    info.can_write = if !test_write {
        // conservative check: if metadata says readonly then false, otherwise
        // optimistic true for files and directories (will still fail at action time)
        !info.readonly
    } else if info.is_dir {
        // attempt to create a probe file inside the directory
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        let probe_name = format!(".perm_probe_{}_{}", process::id(), nanos);
        let probe_path = path.join(probe_name);

        // Use atomic write for the probe so that the creation is safe and
        // the implementation leaves no partial files. Remove the probe
        // afterwards and treat the sequence as success only if both
        // operations succeed.
        match crate::fs_op::helpers::atomic_write(&probe_path, b".") {
            Ok(()) => fs::remove_file(&probe_path).is_ok(),
            Err(_) => false,
        }
    } else {
        OpenOptions::new().write(true).open(&path).is_ok()
    };

    Ok(info)
}

/// Attempt to change permissions (Unix only).
///
/// On non-Unix platforms this returns an error indicating unsupported.
#[cfg(unix)]
pub fn change_permissions<P: AsRef<Path>>(path: P, mode: u32) -> Result<(), PermissionError> {
    use std::os::unix::fs::PermissionsExt;

    let path = path.as_ref();
    let perm = fs::Permissions::from_mode(mode);
    fs::set_permissions(path, perm)?;
    Ok(())
}

#[cfg(not(unix))]
pub fn change_permissions<P: AsRef<Path>>(_path: P, _mode: u32) -> Result<(), PermissionError> {
    Err(PermissionError::Unsupported)
}

/// Helper to render a human-friendly octal mode when available.
pub fn format_unix_mode(mode: Option<u32>) -> String {
    mode.map(|m| format!("{:#o}", m)).unwrap_or_else(|| "n/a".to_string())
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, NamedTempFile};
    use std::io::Write;

    #[test]
    fn format_unix_mode_some_and_none() {
        assert_eq!(format_unix_mode(Some(0o644)), "0o644");
        assert_eq!(format_unix_mode(None), "n/a");
    }

    #[test]
    fn inspect_permissions_file_read_write() {
        let mut f = NamedTempFile::new().expect("create temp file");
        writeln!(f, "hello").expect("write");
        let p = f.path().to_path_buf();

        let info = inspect_permissions(&p, false).expect("inspect");
        assert!(!info.is_dir);
        assert!(info.can_read, "file should be readable");
        assert!(info.can_write, "file should be writable when not testing write");

        let info2 = inspect_permissions(&p, true).expect("inspect test_write");
        assert!(info2.can_read);
        // write test should succeed for a regular temp file
        assert!(info2.can_write, "file should be writable with test_write=true");
    }

    #[test]
    fn inspect_permissions_dir_probe() {
        let d = tempdir().expect("tempdir");
        let info = inspect_permissions(d.path(), true).expect("inspect dir");
        assert!(info.is_dir);
        assert!(info.can_read, "dir should be readable");
        assert!(info.can_write, "dir should be writable (probe create/remove)");
    }
}
