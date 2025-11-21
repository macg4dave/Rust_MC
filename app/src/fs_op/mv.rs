use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use rayon::prelude::*;

/// Errors returned by move/copy helpers.
#[derive(Debug)]
pub enum MvError {
    Io(std::io::Error),
    MissingFilename,
}

impl fmt::Display for MvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MvError::Io(e) => write!(f, "IO error: {}", e),
            MvError::MissingFilename => write!(f, "path has no filename"),
        }
    }
}

impl std::error::Error for MvError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MvError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for MvError {
    fn from(e: std::io::Error) -> Self {
        MvError::Io(e)
    }
}

/// Rename a path within the same parent directory (keeps parent).
pub fn rename_path<P: AsRef<Path>>(path: P, new_name: &str) -> Result<(), MvError> {
    let p = path.as_ref();
    let parent = p.parent().ok_or(MvError::MissingFilename)?;
    let dest = parent.join(new_name);
    fs::rename(p, &dest).map_err(MvError::Io)?;
    Ok(())
}

/// Copy path to `dest`. If `src` is a directory, copy recursively into `dest`.
pub fn copy_path<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dest: Q) -> Result<(), MvError> {
    let s = src.as_ref();
    let d = dest.as_ref();
    // Resolve symlinks so copying a symlink-to-dir behaves like copying
    // the directory target (the test-suite expects this behaviour).
    let mut src_path_buf = s.to_path_buf();
    if let Ok(md) = fs::symlink_metadata(&src_path_buf) {
        if md.file_type().is_symlink() {
            if let Ok(resolved) = fs::canonicalize(&src_path_buf) {
                src_path_buf = resolved;
            }
        }
    }

    let s = src_path_buf.as_path();

    if s.is_dir() {
        // Use fs_extra to copy the directory contents into `d`.
        fs::create_dir_all(d).map_err(MvError::Io)?;
        // Walk the source directory recursively and mirror into `d`.
        // Collect entries first so we can create directories deterministically
        // then copy files in parallel to improve throughput on multi-core systems.
        let mut dirs_to_create: Vec<PathBuf> = Vec::new();
        let mut files_to_copy: Vec<(PathBuf, PathBuf)> = Vec::new();

        for entry in WalkDir::new(s).min_depth(1).follow_links(false) {
            let entry = entry.map_err(|e| MvError::Io(io::Error::new(io::ErrorKind::Other, e)))?;
            let from = entry.path().to_path_buf();
            let rel = from
                .strip_prefix(s)
                .map_err(|e| MvError::Io(io::Error::new(io::ErrorKind::Other, e)))?;
            let dest_path = d.join(rel);
            let ft = entry.file_type();

            if ft.is_dir() {
                dirs_to_create.push(dest_path);
            } else if ft.is_file() {
                files_to_copy.push((from, dest_path));
            } else {
                // Skip other file types (symlinks, device nodes)
            }
        }

        // Create directories sequentially to avoid races when creating parents.
        dirs_to_create.sort();
        dirs_to_create.dedup();
        for dir in dirs_to_create {
            fs::create_dir_all(&dir).map_err(MvError::Io)?;
        }

        // Copy files in parallel. Collect results and return the first error if any.
        let file_results: Vec<Result<(), io::Error>> = files_to_copy
            .into_par_iter()
            .map(|(from, dest_path)| {
                if let Some(parent) = dest_path.parent() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        return Err(e);
                    }
                }
                crate::fs_op::helpers::atomic_copy_file(&from, &dest_path).map(|_| ())
            })
            .collect();

        for r in file_results {
            if let Err(e) = r {
                return Err(MvError::Io(e));
            }
        }
    } else {
        // dest may be directory or file path. If dest is dir, copy into it.
        let final_dest = if d.exists() && d.is_dir() {
            d.join(s.file_name().ok_or(MvError::MissingFilename)?)
        } else {
            d.to_path_buf()
        };
        if let Some(parent) = final_dest.parent() {
            fs::create_dir_all(parent).map_err(MvError::Io)?;
        }
        // Use atomic copy helper to preserve atomic semantics used elsewhere
        crate::fs_op::helpers::atomic_copy_file(s, &final_dest)
            .map(|_| ())
            .map_err(MvError::Io)?;
    }
    Ok(())
}

/// Move (rename) path to `dest`. If `rename` fails (cross-device), fallback to copy+remove.
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
        Err(_e) => {
            // try fallback
            copy_path(s, &final_dest).map_err(|ce| match ce {
                MvError::Io(ioe) => MvError::Io(io::Error::other(format!(
                    "fallback copying {:?} -> {:?}: {:?}",
                    s, final_dest, ioe
                ))),
                other => other,
            })?;
            // remove original (file or dir)
            if s.is_dir() {
                fs::remove_dir_all(s).map_err(MvError::Io)?;
            } else if s.exists() {
                fs::remove_file(s).map_err(MvError::Io)?;
            }
            Ok(())
        }
    }
}
