use std::fmt;
use std::path::{Path, PathBuf};

/// Errors that can occur when resolving a user-supplied path.
#[derive(Debug, PartialEq, Eq)]
pub enum PathError {
    Empty,
    HomeNotFound,
    NotFound(PathBuf),
    NotDirectory(PathBuf),
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathError::Empty => write!(f, "empty path"),
            PathError::HomeNotFound => write!(f, "could not determine home directory"),
            PathError::NotFound(p) => write!(f, "path does not exist: {}", p.display()),
            PathError::NotDirectory(p) => write!(f, "not a directory: {}", p.display()),
        }
    }
}

impl std::error::Error for PathError {}

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
        match expand_tilde(input) {
            Some(p) => p,
            None => return Err(PathError::HomeNotFound),
        }
    } else {
        let p = PathBuf::from(input);
        if p.is_absolute() {
            p
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
    // Accept both `HOME` (Unix) and `USERPROFILE` (Windows) for portability.
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    let rest = input.trim_start_matches('~');
    let mut p = PathBuf::from(home);
    if !rest.is_empty() {
        // Trim leading separators so `~/foo` and `~foo` behave sensibly.
        let trimmed = rest.trim_start_matches(|c| ['/', '\\'].contains(&c));
        p.push(trimmed);
    }
    Some(p)
}
