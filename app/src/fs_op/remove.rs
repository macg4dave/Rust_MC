use std::fmt;
use std::fs;
use std::path::Path;

/// Errors returned from `remove_path`.
#[derive(Debug)]
pub enum RemoveError {
    Io(std::io::Error),
}

impl fmt::Display for RemoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RemoveError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for RemoveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RemoveError::Io(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for RemoveError {
    fn from(e: std::io::Error) -> Self {
        RemoveError::Io(e)
    }
}

/// Remove a file or directory (recursively for directories).
pub fn remove_path<P: AsRef<Path>>(path: P) -> Result<(), RemoveError> {
    let p = path.as_ref();
    if p.is_dir() {
        fs::remove_dir_all(p).map_err(RemoveError::Io)?;
    } else if p.exists() {
        fs::remove_file(p).map_err(RemoveError::Io)?;
    }
    Ok(())
}
