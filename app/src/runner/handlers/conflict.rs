use crate::app::{App, Mode};
use crate::input::KeyCode;
use crate::runner::progress::OperationDecision;

pub fn handle_conflict(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    match &mut app.mode {
        Mode::Conflict { path: _, selected, apply_all } => {
            match code {
                KeyCode::Left => {
                    if *selected > 0 { *selected -= 1; }
                }
                KeyCode::Right => {
                    if *selected < 2 { *selected += 1; }
                }
                KeyCode::Char(' ') => {
                    *apply_all = !*apply_all;
                }
                KeyCode::Enter => {
                    let decision = match *selected {
                        0 => if *apply_all { OperationDecision::OverwriteAll } else { OperationDecision::Overwrite },
                        1 => if *apply_all { OperationDecision::SkipAll } else { OperationDecision::Skip },
                        _ => OperationDecision::Cancel,
                    };
                    if let Some(tx) = &app.op_decision_tx {
                        let _ = tx.send(decision);
                    }
                    app.mode = Mode::Progress { title: "Resolving".to_string(), processed: 0, total: 0, message: "Applying decision".to_string(), cancelled: false };
                }
                KeyCode::Esc => {
                    if let Some(tx) = &app.op_decision_tx {
                        let _ = tx.send(OperationDecision::Cancel);
                    }
                    app.mode = Mode::Progress { title: "Resolving".to_string(), processed: 0, total: 0, message: "Cancelling".to_string(), cancelled: true };
                }
                KeyCode::Char('o') | KeyCode::Char('O') => {
                    let decision = if *apply_all { OperationDecision::OverwriteAll } else { OperationDecision::Overwrite };
                    if let Some(tx) = &app.op_decision_tx { let _ = tx.send(decision); }
                    app.mode = Mode::Progress { title: "Resolving".to_string(), processed: 0, total: 0, message: "Applying decision".to_string(), cancelled: false };
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    let decision = if *apply_all { OperationDecision::SkipAll } else { OperationDecision::Skip };
                    if let Some(tx) = &app.op_decision_tx { let _ = tx.send(decision); }
                    app.mode = Mode::Progress { title: "Resolving".to_string(), processed: 0, total: 0, message: "Applying decision".to_string(), cancelled: false };
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    *apply_all = !*apply_all;
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    if let Some(tx) = &app.op_decision_tx { let _ = tx.send(OperationDecision::Cancel); }
                    app.mode = Mode::Progress { title: "Resolving".to_string(), processed: 0, total: 0, message: "Cancelling".to_string(), cancelled: true };
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(false)
}
