use std::fmt;
use std::path::PathBuf;

/// User decision sent from the UI to a background worker when a
/// conflicting target is reported during a file operation.
///
/// Typical usage: the worker reports a conflict by sending a
/// `ProgressUpdate` with `conflict = Some(path)`. The UI prompts the
/// user and sends one of these variants back on the decision channel.
///
/// Variants:
/// - `Overwrite`: overwrite this target.
/// - `Skip`: skip this item.
/// - `OverwriteAll`: overwrite this and all subsequent conflicts.
/// - `SkipAll`: skip this and all subsequent conflicts.
/// - `Cancel`: abort the whole operation.
// (see `OperationDecision` above)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OperationDecision {
    /// Overwrite the conflicting target for this single item.
    Overwrite,

    /// Skip this single item and continue.
    Skip,

    /// Overwrite this and all subsequent conflicts.
    OverwriteAll,

    /// Skip this and all subsequent conflicts.
    SkipAll,

    /// Cancel the whole operation immediately.
    Cancel,
}

impl fmt::Display for OperationDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use OperationDecision::*;
        let s = match self {
            Overwrite => "Overwrite",
            Skip => "Skip",
            OverwriteAll => "OverwriteAll",
            SkipAll => "SkipAll",
            Cancel => "Cancel",
        };
        write!(f, "{}", s)
    }
}

/// ProgressUpdate is sent by background workers to the UI to report
/// progress and to request conflict resolution.
///
/// Protocol summary:
/// - `processed` / `total`: progress counters updated as items complete.
/// - `message`: optional human-friendly status text.
/// - `done`: true when the worker finished (either successfully or
///   due to an error/cancellation).
/// - `error`: optional error message when `done == true` and an error
///   occurred.
/// - `conflict`: when `Some(path)`, the worker is blocked waiting for
///   an `OperationDecision` from the UI for that `path`.
///
/// Example sequence:
/// 1. Worker -> ProgressUpdate { processed:0, total:N, message:Some("Starting"), done:false, conflict:None }
/// 2. Worker -> ProgressUpdate { processed:i, total:N, message:Some("Copied ..."), done:false, conflict:None }
/// 3. Worker -> ProgressUpdate { processed:i, total:N, message:Some("Conflict"), done:false, conflict:Some(path) }
/// 4. UI -> OperationDecision::Skip (sent via decision channel)
/// 5. Worker continues, eventually sending ProgressUpdate { processed:N, total:N, done:true }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressUpdate {
    /// How many items have been processed so far.
    pub processed: usize,

    /// Total number of items in the operation. When unknown this may be 0.
    pub total: usize,

    /// Optional human-friendly status text intended for display in the UI.
    pub message: Option<String>,

    /// Whether the operation has finished (either successfully, cancelled or
    /// due to an error).
    pub done: bool,

    /// Optional error message when `done == true` and an error occurred.
    pub error: Option<String>,

    /// If present, the worker has hit a conflict for this `PathBuf` and is
    /// waiting for an `OperationDecision` from the UI thread.
    pub conflict: Option<PathBuf>,
}

impl ProgressUpdate {
    /// Create a new progress update with minimal state.
    #[must_use]
    pub fn new(processed: usize, total: usize) -> Self {
        Self { processed, total, message: None, done: false, error: None, conflict: None }
    }

    /// Create a progress update that marks the operation done with an optional
    /// error message.
    #[must_use]
    pub fn done_with_error(processed: usize, total: usize, error: Option<String>) -> Self {
        Self { processed, total, message: error.clone(), done: true, error, conflict: None }
    }

    /// Convenience constructor for a conflict update. The returned struct has
    /// `done == false` and `error == None`.
    #[must_use]
    pub fn conflict(path: PathBuf, processed: usize, total: usize, message: Option<String>) -> Self {
        Self { processed, total, message, done: false, error: None, conflict: Some(path) }
    }

    /// Returns true if the operation is finished.
    #[must_use]
    pub fn is_done(&self) -> bool { self.done }

    /// Returns true when this update indicates an active conflict that
    /// requires a decision from the UI.
    #[must_use]
    pub fn is_conflict(&self) -> bool { self.conflict.is_some() }

    /// Returns the operation completion percentage in range 0.0..=100.0. If
    /// `total == 0` the function returns None.
    #[must_use]
    pub fn percent(&self) -> Option<f64> {
        if self.total == 0 { None } else { Some((self.processed as f64 / self.total as f64) * 100.0) }
    }
}

impl fmt::Display for ProgressUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.done {
            if let Some(err) = &self.error {
                write!(f, "done {}/{} (error: {})", self.processed, self.total, err)
            } else {
                write!(f, "done {}/{} ({})", self.processed, self.total, self.message.as_deref().unwrap_or(""))
            }
        } else if let Some(path) = &self.conflict {
            write!(f, "{}/{} - conflict: {} ({})", self.processed, self.total, path.display(), self.message.as_deref().unwrap_or(""))
        } else {
            write!(f, "{}/{} ({})", self.processed, self.total, self.message.as_deref().unwrap_or(""))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{OperationDecision, ProgressUpdate};
    use std::path::PathBuf;

    #[test]
    fn decision_is_copy_and_display() {
        let d = OperationDecision::OverwriteAll;
        // Copy semantics, eq and display should work
        let d2 = d; // copy
        assert_eq!(d, d2);
        assert_eq!(format!("{}", d), "OverwriteAll");
    }

    #[test]
    fn progress_update_helpers_and_display() {
        let p = ProgressUpdate::new(3, 10);
        assert!(!p.is_done());
        assert_eq!(p.percent().unwrap(), 30.0);

        let mut q = ProgressUpdate::conflict(PathBuf::from("/tmp/foo"), 4, 10, Some("blocked".to_string()));
        assert!(q.is_conflict());
        assert_eq!(format!("{}", q), "4/10 - conflict: /tmp/foo (blocked)");

        q.done = true;
        q.error = Some("oh no".to_string());
        assert_eq!(format!("{}", q), "done 4/10 (error: oh no)");
    }
}
