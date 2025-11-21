use std::path::Path;

/// Lightweight classification of a filesystem path's kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathType {
    /// The path does not exist.
    NotFound,
    /// The path exists and is a directory.
    Directory,
    /// The path exists and is a regular file.
    File,
    /// The path exists but is neither a regular file nor a directory
    /// (for example: socket, FIFO, block device, symlink without target, etc.).
    Other,
}

impl PathType {
    /// Classify `path` and return its `PathType`.
    ///
    /// This is a small, allocation-free helper intended for simple
    /// branching logic. It avoids repeated calls to `exists`/`is_dir`/`is_file`.
    pub fn of<P: AsRef<Path>>(path: P) -> Self {
        let p = path.as_ref();
        if !p.exists() {
            PathType::NotFound
        } else if p.is_dir() {
            PathType::Directory
        } else if p.is_file() {
            PathType::File
        } else {
            PathType::Other
        }
    }
}

/// Return `true` if the provided `path` exists.
pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    matches!(PathType::of(path), PathType::Directory | PathType::File | PathType::Other)
}

/// Return `true` if the provided `path` is a directory.
pub fn is_dir<P: AsRef<Path>>(path: P) -> bool {
    PathType::of(path) == PathType::Directory
}

/// Return `true` if the provided `path` is a regular file.
pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
    PathType::of(path) == PathType::File
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn path_type_nonexistent() {
        let tmp = tempdir().unwrap();
        let p = tmp.path().join("no_such_file_hopefully");
        assert_eq!(PathType::of(&p), PathType::NotFound);
        assert!(!exists(&p));
        assert!(!is_file(&p));
        assert!(!is_dir(&p));
    }

    #[test]
    fn path_type_file_and_dir() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("a.txt");
        fs::write(&file, b"hello").unwrap();
        assert_eq!(PathType::of(&file), PathType::File);
        assert!(exists(&file));
        assert!(is_file(&file));
        assert!(!is_dir(&file));

        let dir = tmp.path().join("subdir");
        fs::create_dir(&dir).unwrap();
        assert_eq!(PathType::of(&dir), PathType::Directory);
        assert!(exists(&dir));
        assert!(is_dir(&dir));
        assert!(!is_file(&dir));
    }
}
