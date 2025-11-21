use crate::app::{App, Mode};
use crate::input::KeyCode;
use crate::app::settings::keybinds;
use crate::runner::progress::OperationDecision;

pub fn handle_conflict(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    match &mut app.mode {
        Mode::Conflict {
            path: _,
            selected,
            apply_all,
        } => {
            if keybinds::is_left(&code) {
                if *selected > 0 {
                    *selected -= 1;
                }
            } else if keybinds::is_right(&code) {
                if *selected < 2 {
                    *selected += 1;
                }
            } else if keybinds::is_toggle_selection(&code) {
                *apply_all = !*apply_all;
            } else if keybinds::is_enter(&code) {
                let decision = match *selected {
                    0 => {
                        if *apply_all {
                            OperationDecision::OverwriteAll
                        } else {
                            OperationDecision::Overwrite
                        }
                    }
                    1 => {
                        if *apply_all {
                            OperationDecision::SkipAll
                        } else {
                            OperationDecision::Skip
                        }
                    }
                    _ => OperationDecision::Cancel,
                };
                if let Some(tx) = &app.op_decision_tx {
                    let _ = tx.send(decision);
                }
                app.mode = Mode::Progress {
                    title: "Resolving".to_string(),
                    processed: 0,
                    total: 0,
                    message: "Applying decision".to_string(),
                    cancelled: false,
                };
            } else if keybinds::is_esc(&code) {
                if let Some(tx) = &app.op_decision_tx {
                    let _ = tx.send(OperationDecision::Cancel);
                }
                app.mode = Mode::Progress {
                    title: "Resolving".to_string(),
                    processed: 0,
                    total: 0,
                    message: "Cancelling".to_string(),
                    cancelled: true,
                };
            } else if keybinds::is_char(&code, 'o') || keybinds::is_char(&code, 'O') {
                let decision = if *apply_all {
                    OperationDecision::OverwriteAll
                } else {
                    OperationDecision::Overwrite
                };
                if let Some(tx) = &app.op_decision_tx {
                    let _ = tx.send(decision);
                }
                app.mode = Mode::Progress {
                    title: "Resolving".to_string(),
                    processed: 0,
                    total: 0,
                    message: "Applying decision".to_string(),
                    cancelled: false,
                };
            } else if keybinds::is_char(&code, 's') || keybinds::is_char(&code, 'S') {
                let decision = if *apply_all {
                    OperationDecision::SkipAll
                } else {
                    OperationDecision::Skip
                };
                if let Some(tx) = &app.op_decision_tx {
                    let _ = tx.send(decision);
                }
                app.mode = Mode::Progress {
                    title: "Resolving".to_string(),
                    processed: 0,
                    total: 0,
                    message: "Applying decision".to_string(),
                    cancelled: false,
                };
            } else if keybinds::is_char(&code, 'a') || keybinds::is_char(&code, 'A') {
                *apply_all = !*apply_all;
            } else if keybinds::is_char(&code, 'c') || keybinds::is_char(&code, 'C') {
                if let Some(tx) = &app.op_decision_tx {
                    let _ = tx.send(OperationDecision::Cancel);
                }
                app.mode = Mode::Progress {
                    title: "Resolving".to_string(),
                    processed: 0,
                    total: 0,
                    message: "Cancelling".to_string(),
                    cancelled: true,
                };
            }
        },
        _ => {}
    }
    Ok(false)
}
