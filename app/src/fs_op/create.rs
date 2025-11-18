use std::path::{Path, PathBuf};
use std::fmt;

/// Errors returned when creating files or directories.
#[derive(Debug)]
pub enum CreateError {
    Io(std::io::Error),
    AlreadyExists(PathBuf),
}

impl fmt::Display for CreateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CreateError::Io(e) => write!(f, "IO error: {}", e),
            CreateError::AlreadyExists(p) => write!(f, "already exists: {:?}", p),
        }
    }
}

impl std::error::Error for CreateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CreateError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CreateError {
    fn from(e: std::io::Error) -> Self {
        CreateError::Io(e)
    }
}

/// Create an empty file at `path`. Fails if the file already exists.
pub fn create_file<P: AsRef<Path>>(path: P) -> Result<(), CreateError> {
    let p = path.as_ref();
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(CreateError::Io)?;
    }
    // Create an empty file atomically by writing zero bytes via the
    // shared helper. This avoids races and leaves no partial file.
    match crate::fs_op::helpers::atomic_write(&p.to_path_buf(), &[]) {
        Ok(()) => Ok(()),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                Err(CreateError::AlreadyExists(p.to_path_buf()))
            } else {
                Err(CreateError::Io(e))
            }
        }
    }
}

/// Create directory and parents.
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<(), CreateError> {
    std::fs::create_dir_all(path.as_ref()).map_err(CreateError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn create_file_and_dir() {
        let td = tempdir().unwrap();
        let dir = td.path().join("a/b");
        let file = dir.join("f.txt");
        create_dir_all(&dir).unwrap();
        create_file(&file).unwrap();
        assert!(file.exists());
        // Ensure no leftover atomic temp files are present after success.
        let mut tmp_leftovers = 0;
        for e in std::fs::read_dir(dir).unwrap() {
            if let Ok(e) = e {
                if let Some(name) = e.file_name().to_str() {
                    if name.starts_with(".tmp_atomic_write.") {
                        tmp_leftovers += 1;
                    }
                }
            }
        }
        assert_eq!(tmp_leftovers, 0, "found leftover atomic temp files");
    }
}
