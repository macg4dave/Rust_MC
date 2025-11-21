use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use rayon::prelude::*;

/// Errors returned by move/copy helpers in this module.
#[derive(Debug, thiserror::Error)]
pub enum MvError {
    /// Underlying IO error with optional source/destination context.
    #[error("IO error{context}: {source}")]
    Io {
        #[source]
        source: io::Error,
        /// Optional source path related to the error (if known).
        src: Option<PathBuf>,
        /// Optional destination path related to the error (if known).
        dest: Option<PathBuf>,
        #[allow(dead_code)]
        /// Internal helper computed for display; serde / tests don't rely on it.
        context: String,
    },

    /// Source path did not have a filename component where one was required.
    #[error("path has no filename")]
    MissingFilename,
}

impl From<io::Error> for MvError {
    fn from(source: io::Error) -> Self {
        MvError::Io { source, src: None, dest: None, context: String::new() }
    }
}

/// Rename a path within the same parent directory (keeps parent).
/// Rename a path within the same parent directory (keeps parent).
pub fn rename_path<P: AsRef<Path>>(path: P, new_name: &str) -> Result<(), MvError> {
    let p = path.as_ref();
    let parent = p.parent().ok_or(MvError::MissingFilename)?;
    let dest = parent.join(new_name);
    fs::rename(p, &dest)?;
    Ok(())
}

/// Copy path to `dest`. If `src` is a directory, copy recursively into `dest`.
/// Copy `src` to `dest`. If `src` is a directory it is copied recursively.
///
/// Symlinks that point to directories are resolved so the directory target
/// is copied (this matches historical behaviour expected by tests).
pub fn copy_path<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dest: Q) -> Result<(), MvError> {
    let s_orig = src.as_ref();
    let d = dest.as_ref();

    // Resolve symlink-to-dir to its canonical target when possible.
    let s_path = match fs::symlink_metadata(s_orig) {
        Ok(md) if md.file_type().is_symlink() => fs::canonicalize(s_orig).unwrap_or_else(|_| s_orig.to_path_buf()),
        _ => s_orig.to_path_buf(),
    };

    let s = s_path.as_path();

    if s.is_dir() {
        fs::create_dir_all(d)?;

        // Collect directory and file entries deterministically, then create
        // directories before copying files in parallel.
        let mut dirs_to_create: Vec<PathBuf> = Vec::new();
        let mut files_to_copy: Vec<(PathBuf, PathBuf)> = Vec::new();

        for entry in WalkDir::new(s).min_depth(1).follow_links(false) {
            let entry = entry.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let from = entry.path().to_path_buf();
            let rel = from.strip_prefix(s).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let dest_path = d.join(rel);

            if entry.file_type().is_dir() {
                dirs_to_create.push(dest_path);
            } else if entry.file_type().is_file() {
                files_to_copy.push((from, dest_path));
            }
        }

        dirs_to_create.sort();
        dirs_to_create.dedup();
        for dir in dirs_to_create {
            fs::create_dir_all(&dir)?;
        }

        let file_errors: Vec<MvError> = files_to_copy
            .into_par_iter()
            .filter_map(|(from, dest_path)| {
                if let Some(parent) = dest_path.parent() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        return Some(MvError::Io { source: e, src: Some(from.clone()), dest: Some(dest_path.clone()), context: format!("creating parent for {:?}", dest_path) });
                    }
                }
                match crate::fs_op::helpers::atomic_copy_file(&from, &dest_path) {
                    Ok(_) => None,
                    Err(e) => Some(MvError::Io { source: e, src: Some(from), dest: Some(dest_path), context: String::new() }),
                }
            })
            .collect();

        if let Some(e) = file_errors.into_iter().next() {
            return Err(e);
        }
    } else {
        // dest may be directory or file path. If dest is dir, copy into it.
        let final_dest = if d.exists() && d.is_dir() {
            d.join(s.file_name().ok_or(MvError::MissingFilename)?)
        } else {
            d.to_path_buf()
        };

        if let Some(parent) = final_dest.parent() {
            fs::create_dir_all(parent)?;
        }

        crate::fs_op::helpers::atomic_copy_file(s, &final_dest).map(|_| ())?;
    }

    Ok(())
}

/// Move (rename) path to `dest`. If `rename` fails (cross-device), fallback to copy+remove.
/// Move (rename) `src` to `dest`. Falls back to copy+remove on cross-device errors.
pub fn move_path<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dest: Q) -> Result<(), MvError> {
    let s = src.as_ref();
    let d = dest.as_ref();

    // If destination is an existing directory, move into it
    let final_dest: PathBuf = if d.exists() && d.is_dir() {
        d.join(s.file_name().ok_or(MvError::MissingFilename)?)
    } else {
        d.to_path_buf()
    };

    match fs::rename(s, &final_dest) {
        Ok(_) => Ok(()),
        Err(_) => {
            // try fallback: copy then remove
            copy_path(s, &final_dest)?;

            if s.is_dir() {
                fs::remove_dir_all(s)?;
            } else if s.exists() {
                fs::remove_file(s)?;
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn rename_missing_filename_returns_error() {
        // Root path has no filename; parent() is None.
        let root = Path::new("/");
        let res = rename_path(root, "newname");
        assert!(matches!(res, Err(MvError::MissingFilename)));
    }
}
