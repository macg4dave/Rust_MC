use std::fmt;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process;
use std::time::SystemTime;

use std::result::Result as StdResult;

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
    fn new(path: PathBuf) -> Self {
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
#[derive(Debug)]
pub enum PermissionError {
    Io(std::io::Error),
    Unsupported,
}

impl fmt::Display for PermissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionError::Io(e) => write!(f, "IO error: {}", e),
            PermissionError::Unsupported => write!(f, "operation not supported on this platform"),
        }
    }
}

impl std::error::Error for PermissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PermissionError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for PermissionError {
    fn from(e: std::io::Error) -> Self {
        PermissionError::Io(e)
    }
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
pub fn inspect_permissions<P: AsRef<Path>>(
    path: P,
    test_write: bool,
) -> StdResult<PermissionInfo, PermissionError> {
    let path = path.as_ref().to_path_buf();
    let mut info = PermissionInfo::new(path.clone());

    let meta = fs::metadata(&path).map_err(PermissionError::Io)?;
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
        fs::read_dir(&path).is_ok()
    } else {
        OpenOptions::new().read(true).open(&path).is_ok()
    };

    // Best-effort write check. If test_write is false, prefer metadata only
    // (non destructive). If true, attempt to open/create a probe to verify.
    if !test_write {
        // conservative check: if metadata says readonly then false, otherwise
        // optimistic true for files and directories (will still fail at action time)
        info.can_write = !info.readonly;
    } else if info.is_dir {
        // attempt to create a probe file inside the directory
        let probe_name = format!(
            ".perm_probe_{}_{}",
            process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or_default()
        );
        let probe_path = path.join(probe_name);
        // Use atomic write for the probe so that the creation is safe and
        // the implementation leaves no partial files. Remove the probe
        // afterwards and treat the sequence as success only if both
        // operations succeed.
        let created = match crate::fs_op::helpers::atomic_write(&probe_path, b".") {
            Ok(()) => fs::remove_file(&probe_path).is_ok(),
            Err(_) => false,
        };
        info.can_write = created;
    } else {
        info.can_write = OpenOptions::new().write(true).open(&path).is_ok();
    }

    Ok(info)
}

/// Attempt to change permissions (Unix only).
///
/// On non-Unix platforms this returns an error indicating unsupported.
#[cfg(unix)]
pub fn change_permissions<P: AsRef<Path>>(path: P, mode: u32) -> StdResult<(), PermissionError> {
    use std::os::unix::fs::PermissionsExt;

    let path = path.as_ref();
    let perm = fs::Permissions::from_mode(mode);
    fs::set_permissions(path, perm).map_err(PermissionError::Io)?;
    Ok(())
}

#[cfg(not(unix))]
pub fn change_permissions<P: AsRef<Path>>(_path: P, _mode: u32) -> StdResult<(), PermissionError> {
    Err(PermissionError::Unsupported)
}

/// Helper to render a human-friendly octal mode when available.
pub fn format_unix_mode(mode: Option<u32>) -> String {
    match mode {
        Some(m) => format!("{:#o}", m),
        None => "n/a".to_string(),
    }
}
