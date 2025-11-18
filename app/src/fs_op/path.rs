use std::path::{Path, PathBuf};
use std::fmt;

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
        let trimmed = rest.trim_start_matches(|c| c == '/' || c == '\\');
        p.push(trimmed);
    }
    Some(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn empty_input_is_error() {
        let base = Path::new("/");
        let r = resolve_path("   ", base);
        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), PathError::Empty);
    }

    #[test]
    fn tilde_expands_to_home() {
        let td = TempDir::new().unwrap();
        // Temporarily set HOME to our temp dir
        std::env::set_var("HOME", td.path());
        let base = Path::new("/irrelevant");
        let got = resolve_path("~", base).unwrap();
        assert_eq!(got, td.path());
    }

    #[test]
    fn relative_resolves_against_base() {
        let td = TempDir::new().unwrap();
        let sub = td.path().join("subdir");
        fs::create_dir_all(&sub).unwrap();
        let got = resolve_path("subdir", td.path()).unwrap();
        assert_eq!(got, sub);
    }

    #[test]
    fn absolute_path_returns_as_is() {
        let td = TempDir::new().unwrap();
        let p = td.path().to_path_buf();
        let got = resolve_path(&p.to_string_lossy(), Path::new("/ignored")).unwrap();
        assert_eq!(got, p);
    }

    #[test]
    fn file_is_not_directory() {
        let td = TempDir::new().unwrap();
        let f = td.path().join("file.txt");
        fs::write(&f, "hello").unwrap();
        let err = resolve_path(&f.to_string_lossy(), td.path()).unwrap_err();
        assert!(matches!(err, PathError::NotDirectory(p) if p == f));
    }

    #[test]
    fn nonexistent_path_errors() {
        let td = TempDir::new().unwrap();
        let p = td.path().join("no-such-dir");
        let err = resolve_path(&p.to_string_lossy(), td.path()).unwrap_err();
        assert!(matches!(err, PathError::NotFound(q) if q == p));
    }
}
