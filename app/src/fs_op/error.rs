use thiserror::Error;
use std::path::PathBuf;

/// Errors produced by filesystem operation helpers and `App`-level ops.
#[derive(Error, Debug)]
pub enum FsOpError {
    /// Wrapper for underlying IO errors.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error with context message.
    #[error("Filesystem operation failed: {0}")]
    Message(String),

    /// Contextual error that includes source and destination paths.
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
