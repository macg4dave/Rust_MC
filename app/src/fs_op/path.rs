use directories_next::UserDirs;
use std::path::{Path, PathBuf};

/// Errors that can occur when resolving a user-supplied path.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum PathError {
    /// The provided input string was empty or contained only whitespace.
    #[error("empty path")]
    Empty,

    /// The user's home directory could not be determined when expanding `~`.
    #[error("could not determine home directory")]
    HomeNotFound,

    /// The resolved path does not exist on the filesystem.
    #[error("path does not exist: {0}")]
    NotFound(PathBuf),

    /// The resolved path exists but is not a directory.
    #[error("not a directory: {0}")]
    NotDirectory(PathBuf),
}

/// Resolve and validate a user-supplied path for changing panel cwd.
///
/// Behaviour:
/// - Empty `input` is an error.
/// - A leading `~` is expanded to the user's home directory (uses `HOME` or
///   `USERPROFILE` environment variables).
/// - Absolute paths are returned as-is.
/// - Relative paths are resolved relative to `base`.
/// - The returned path must exist and be a directory; otherwise a `PathError`
///   describing the problem is returned.
pub fn resolve_path(input: &str, base: &Path) -> Result<PathBuf, PathError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(PathError::Empty);
    }

    let candidate = if input.starts_with('~') {
        expand_tilde(input).ok_or(PathError::HomeNotFound)?
    } else {
        let p = Path::new(input);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            base.join(p)
        }
    };

    if !candidate.exists() {
        return Err(PathError::NotFound(candidate));
    }
    if !candidate.is_dir() {
        return Err(PathError::NotDirectory(candidate));
    }
    Ok(candidate)
}

// Expand a path beginning with `~` into a `PathBuf` pointing at the user's
// home directory. Returns `None` when the home directory cannot be determined.
fn expand_tilde(input: &str) -> Option<PathBuf> {
    // `input` begins with `~`.
    let rest = input.trim_start_matches('~');

    // Prefer `directories_next` for a reliable, cross-platform home dir.
    if let Some(ud) = UserDirs::new() {
        let mut p = ud.home_dir().to_path_buf();
        if !rest.is_empty() {
                let trimmed = rest.trim_start_matches(['/', '\\']);
            p.push(trimmed);
        }
        return Some(p);
    }

    // Fallback to environment variables for compatibility.
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    let mut p = PathBuf::from(home);
    if !rest.is_empty() {
            let trimmed = rest.trim_start_matches(['/', '\\']);
        p.push(trimmed);
    }
    Some(p)
}

/* Unit tests moved to integration tests under `app/tests/` to centralize
   fs-op path behaviour checks. */
