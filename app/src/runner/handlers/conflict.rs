use crate::app::{App, Mode};
use crate::app::settings::keybinds;
use crate::input::KeyCode;
use crate::runner::progress::OperationDecision;

const RESOLVING_TITLE: &str = "Resolving";
const APPLYING_MSG: &str = "Applying decision";
const CANCELLING_MSG: &str = "Cancelling";

/// Map the user's current selection and the `apply_all` toggle to an
/// `OperationDecision` value.
fn map_selection_to_decision(selected: usize, apply_all: bool) -> OperationDecision {
    match selected {
        0 => {
            if apply_all {
                OperationDecision::OverwriteAll
            } else {
                OperationDecision::Overwrite
            }
        }
        1 => {
            if apply_all {
                OperationDecision::SkipAll
            } else {
                OperationDecision::Skip
            }
        }
        _ => OperationDecision::Cancel,
    }
}

/// Helper to send a decision to the worker (if present) and transition the
/// UI into a `Mode::Progress` state with the provided message and cancel flag.
fn send_decision_and_enter_progress(app: &mut App, decision: OperationDecision, message: &str, cancelled: bool) {
    if let Some(tx) = &app.op_decision_tx {
        let _ = tx.send(decision);
    }
    app.mode = Mode::Progress {
        title: RESOLVING_TITLE.to_string(),
        processed: 0,
        total: 0,
        message: message.to_string(),
        cancelled,
    };
}

/// Handle key events when the application is in a conflict resolution mode.
///
/// Returns `Ok(false)` currently (keeps existing behaviour). The function
/// mutates `app.mode` and may send an `OperationDecision` to a background
/// worker via `app.op_decision_tx`.
pub fn handle_conflict(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    if let Mode::Conflict { path: _, selected, apply_all } = &mut app.mode {
            if keybinds::is_left(&code) {
                *selected = (*selected).saturating_sub(1);
            } else if keybinds::is_right(&code) {
                *selected = (*selected + 1).min(2);
            } else if keybinds::is_toggle_selection(&code) || keybinds::is_char(&code, 'a') || keybinds::is_char(&code, 'A') {
                *apply_all = !*apply_all;
            } else if keybinds::is_enter(&code)
                || keybinds::is_char(&code, 'o') || keybinds::is_char(&code, 'O')
                || keybinds::is_char(&code, 's') || keybinds::is_char(&code, 'S')
            {
                // Determine decision based on the selection and toggle.
                let decision = if keybinds::is_enter(&code) {
                    map_selection_to_decision(*selected, *apply_all)
                } else if keybinds::is_char(&code, 'o') || keybinds::is_char(&code, 'O') {
                    if *apply_all { OperationDecision::OverwriteAll } else { OperationDecision::Overwrite }
                } else {
                    // 's' / 'S'
                    if *apply_all { OperationDecision::SkipAll } else { OperationDecision::Skip }
                };

                send_decision_and_enter_progress(app, decision, APPLYING_MSG, false);
            } else if keybinds::is_esc(&code) || keybinds::is_char(&code, 'c') || keybinds::is_char(&code, 'C') {
                send_decision_and_enter_progress(app, OperationDecision::Cancel, CANCELLING_MSG, true);
            }
    }

    Ok(false)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_selection_overwrite() {
        assert!(matches!(map_selection_to_decision(0, false), OperationDecision::Overwrite));
        assert!(matches!(map_selection_to_decision(0, true), OperationDecision::OverwriteAll));
    }

    #[test]
    fn map_selection_skip() {
        assert!(matches!(map_selection_to_decision(1, false), OperationDecision::Skip));
        assert!(matches!(map_selection_to_decision(1, true), OperationDecision::SkipAll));
    }

    #[test]
    fn map_selection_cancel() {
        assert!(matches!(map_selection_to_decision(2, false), OperationDecision::Cancel));
        assert!(matches!(map_selection_to_decision(99, true), OperationDecision::Cancel));
    }
}
