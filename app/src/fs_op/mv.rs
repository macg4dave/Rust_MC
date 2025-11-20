use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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
    if s.is_dir() {
        // ensure dest exists
        fs::create_dir_all(d).map_err(MvError::Io)?;

        // Walk the source directory recursively and mirror into `d`.
        for entry in WalkDir::new(s).min_depth(1).follow_links(false) {
            let entry = entry.map_err(|e| MvError::Io(io::Error::new(io::ErrorKind::Other, e)))?;
            let from = entry.path();
            let rel = from.strip_prefix(s).map_err(|e| MvError::Io(io::Error::new(io::ErrorKind::Other, e)))?;
            let dest_path = d.join(rel);
            let ft = entry.file_type();

            if ft.is_dir() {
                fs::create_dir_all(&dest_path).map_err(MvError::Io)?;
            } else {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent).map_err(MvError::Io)?;
                }
                fs::copy(&from, &dest_path).map_err(MvError::Io)?;
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
        fs::copy(s, &final_dest).map_err(MvError::Io)?;
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
