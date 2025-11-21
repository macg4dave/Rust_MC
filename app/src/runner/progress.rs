use std::fmt;
use std::path::PathBuf;

/// Decision made by the user when a conflict is presented.
#[derive(Clone, Debug)]
pub enum OperationDecision {
    Overwrite,
    Skip,
    OverwriteAll,
    SkipAll,
    Cancel,
}

/// Small struct used to report progress from background file operations.
#[derive(Clone, Debug)]
pub struct ProgressUpdate {
    pub processed: usize,
    pub total: usize,
    /// Optional human-friendly message
    pub message: Option<String>,
    /// Whether the operation completed (successful or with error)
    pub done: bool,
    /// Optional error message if done==true and an error occurred
    pub error: Option<String>,
    /// If present, worker is reporting a conflict for this `PathBuf` and
    /// is waiting for a decision from the UI thread.
    pub conflict: Option<PathBuf>,
}

impl fmt::Display for ProgressUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{} ({})",
            self.processed,
            self.total,
            self.message.as_deref().unwrap_or("")
        )
    }
}
