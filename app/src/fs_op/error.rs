use std::path::PathBuf;

use thiserror::Error;

/// Crate-wide error type for filesystem operations.
///
/// This error centralises common filesystem-related failure modes so
/// callers can use `?` and pattern-match on rich, structured variants.
/// It intentionally implements conversions from `std::io::Error` and
/// `String`/`&str` to make error propagation ergonomic.
#[derive(Debug, Error)]
pub enum FsOpError {
    /// Underlying I/O error. This variant preserves the original `std::io::Error`
    /// as the source so error chains remain informative.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error with a human-friendly message.
    #[error("Filesystem operation failed: {0}")]
    Message(String),

    /// Contextual error including source and destination paths. Use this when
    /// an operation involves two paths and you want both to appear in the
    /// error message for diagnostics.
    #[error("Operation failed from `{src}` to `{dst}`: {msg}")]
    PathContext {
        src: PathBuf,
        dst: PathBuf,
        msg: String,
    },
}

impl From<String> for FsOpError {
    fn from(s: String) -> Self {
        FsOpError::Message(s)
    }
}

impl From<&str> for FsOpError {
    fn from(s: &str) -> Self {
        FsOpError::Message(s.to_string())
    }
}

impl FsOpError {
    /// Convenience constructor for `PathContext` errors.
    pub fn path_context<S: Into<String>, P: Into<PathBuf>>(src: P, dst: P, msg: S) -> Self {
        FsOpError::PathContext {
            src: src.into(),
            dst: dst.into(),
            msg: msg.into(),
        }
    }
}
