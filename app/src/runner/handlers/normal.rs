use crate::app::{Action, App, InputKind, Mode, Side};
use crate::errors;
use crate::input::KeyCode;
use crate::runner::progress::{OperationDecision, ProgressUpdate};
use std::path::PathBuf;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions as FsCopyOptions;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

/// Handle keys when the application is in the normal (default) mode.
///
/// Returns `Ok(true)` when the caller should exit the application.
pub fn handle_normal(app: &mut App, code: KeyCode, page_size: usize) -> anyhow::Result<bool> {
    // Route to command line if active
    if app.command_line.is_some() {
        return crate::ui::command_line::handle_input(app, code);
    }

    match code {
        KeyCode::Char('q') => return Ok(true),
        KeyCode::Down => app.select_next(page_size),
        KeyCode::Up => app.select_prev(page_size),
        KeyCode::PageDown => app.select_page_down(page_size),
        KeyCode::PageUp => app.select_page_up(page_size),
        KeyCode::Enter if !app.menu_focused => handle_enter(app)?,
        KeyCode::Backspace => handle_go_up(app)?,
        KeyCode::Char('r') => handle_refresh(app)?,
        KeyCode::Char('d') => handle_delete_prompt(app),
        KeyCode::Char('c') => handle_copy_prompt(app),
        KeyCode::Char('m') => handle_move_prompt(app),
        KeyCode::Char('n') => {
            app.mode = Mode::Input { prompt: "New file name:".to_string(), buffer: String::new(), kind: InputKind::NewFile };
        }
        KeyCode::Char('N') => {
            app.mode = Mode::Input { prompt: "New dir name:".to_string(), buffer: String::new(), kind: InputKind::NewDir };
        }
        KeyCode::Char('R') => handle_rename_prompt(app),
        KeyCode::Char('s') => { app.sort = app.sort.next(); app.refresh()?; }
        KeyCode::Char('S') => { use crate::app::types::SortOrder::*; app.sort_order = match app.sort_order { Ascending => Descending, Descending => Ascending }; app.refresh()?; }
        KeyCode::Char(' ') => app.active_panel_mut().toggle_selection(),
        KeyCode::Tab => { app.active = match app.active { Side::Left => Side::Right, Side::Right => Side::Left }; }
        KeyCode::F(5) => handle_operation_start(app, Operation::Copy)?,
        KeyCode::F(6) => handle_operation_start(app, Operation::Move)?,
        KeyCode::F(1) => app.menu_focused = !app.menu_focused,
        KeyCode::Left if app.menu_focused => app.menu_prev(),
        KeyCode::Right if app.menu_focused => app.menu_next(),
        KeyCode::Enter if app.menu_focused => { app.menu_activate(); app.menu_focused = false; }
        KeyCode::Esc if app.menu_focused => app.menu_focused = false,
        KeyCode::Home => app.active_panel_mut().selected = 0,
        KeyCode::End => handle_end_key(app),
        KeyCode::Char('p') => app.toggle_preview(),
        KeyCode::F(3) => handle_context_actions(app),
        KeyCode::Char('t') => crate::ui::colors::toggle(),
        KeyCode::Char('?') => {
            let content = "Keys:\n\nq: quit\nF1: toggle menu focus\nLeft/Right: menu navigation when focused\nEnter: open/activate\nBackspace: up\nd: delete\nc: copy\nm: move\nn/N: new file/dir\nR: rename\ns/S: sort (toggle desc)\nTab: switch panels\n?: show this help\n".to_string();
            app.mode = Mode::Message { title: "Help".to_string(), content, buttons: vec!["OK".to_string()], selected: 0, actions: None };
        }
        KeyCode::Char('>') => app.active_panel_mut().preview_offset = app.active_panel_mut().preview_offset.saturating_add(5),
        KeyCode::Char('<') => app.active_panel_mut().preview_offset = app.active_panel_mut().preview_offset.saturating_sub(5),
        _ => {}
    }

    Ok(false)
}

// ----- Helpers & small refactors -----

/// Small enum to choose operation behaviour when starting F5/F6.
///
/// Used by `handle_operation_start` to decide whether the background
/// worker should perform a copy (F5) or a move (F6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Operation { Copy, Move }

/// Helper to construct a simple `Mode::Message` with an OK button.
///
/// This keeps message construction concise in the handlers.
fn make_message_mode(title: &str, content: String) -> Mode {
    Mode::Message { title: title.to_string(), content, buttons: vec!["OK".to_string()], selected: 0, actions: None }
}

/// Handle an Enter key press when not focused on the top menu.
///
/// Behaviour:
/// - If the selected row is the header (index 0), enters a change-path input mode.
/// - If the selected row points to the parent entry and `go_up` is available, attempt to go up.
/// - Otherwise attempt to `enter` the selected entry (open directory or preview file).
///
/// Any filesystem errors are rendered via `errors::render_fsop_error` and shown
/// to the user in a `Mode::Message`.
fn handle_enter(app: &mut App) -> anyhow::Result<()> {
    let panel = app.active_panel_mut();
    if panel.selected == 0 {
        let prompt = format!("Change path (current: {}):", panel.cwd.display());
        app.mode = Mode::Input { prompt, buffer: String::new(), kind: InputKind::ChangePath };
        return Ok(());
    }

    let parent_count = if panel.cwd.parent().is_some() { 1usize } else { 0usize };
    if panel.selected == 1 && parent_count == 1 {
        if let Err(err) = app.go_up() {
            let msg = errors::render_fsop_error(&err, None, None, None);
            app.mode = make_message_mode("Error", msg);
        }
    } else if let Some(e) = panel.selected_entry().cloned() {
        if let Err(err) = app.enter() {
            let path_s = e.path.display().to_string();
            let msg = errors::render_fsop_error(&err, Some(&path_s), None, None);
            app.mode = make_message_mode("Error", msg);
        }
    }
    Ok(())
}

/// Attempt to move the active panel up one directory.
///
/// On error the function will render an error message into `app.mode` so the
/// user sees what went wrong.
fn handle_go_up(app: &mut App) -> anyhow::Result<()> {
    if let Err(err) = app.go_up() {
        let msg = errors::render_fsop_error(&err, None, None, None);
        app.mode = make_message_mode("Error", msg);
    }
    Ok(())
}

/// Refresh the active panels, showing an error message on failure.
fn handle_refresh(app: &mut App) -> anyhow::Result<()> {
    if let Err(err) = app.refresh() {
        let msg = errors::render_io_error(&err, None, None, None);
        app.mode = make_message_mode("Error", msg);
    }
    Ok(())
}

/// Prompt the user to confirm deletion of the currently selected entry.
///
/// If there is no selected entry this is a no-op.
fn handle_delete_prompt(app: &mut App) {
    let panel = app.active_panel_mut();
    if let Some(e) = panel.selected_entry() {
        let msg = format!("Delete {}? (y/n)", e.name);
        app.mode = Mode::Confirm { msg, on_yes: Action::DeleteSelected, selected: 0 };
    }
}

/// Prompt the user for a destination path to copy the currently selected entry.
fn handle_copy_prompt(app: &mut App) {
    let panel = app.active_panel_mut();
    if let Some(e) = panel.selected_entry() {
        let prompt = format!("Copy {} to:", e.name);
        app.mode = Mode::Input { prompt, buffer: String::new(), kind: InputKind::Copy };
    }
}

/// Prompt the user for a destination path to move the currently selected entry.
fn handle_move_prompt(app: &mut App) {
    let panel = app.active_panel_mut();
    if let Some(e) = panel.selected_entry() {
        let prompt = format!("Move {} to:", e.name);
        app.mode = Mode::Input { prompt, buffer: String::new(), kind: InputKind::Move };
    }
}

/// Prompt the user to rename the currently selected entry.
fn handle_rename_prompt(app: &mut App) {
    let panel = app.active_panel_mut();
    if let Some(e) = panel.entries.get(panel.selected) {
        let prompt = format!("Rename {} to:", e.name);
        app.mode = Mode::Input { prompt, buffer: String::new(), kind: InputKind::Rename };
    }
}

/// Move selection to the last entry in the active panel (End key behaviour).
fn handle_end_key(app: &mut App) {
    let panel = app.active_panel_mut();
    if !panel.entries.is_empty() {
        let header_count = 1usize;
        let parent_count = if panel.cwd.parent().is_some() { 1usize } else { 0usize };
        panel.selected = header_count + parent_count + panel.entries.len().saturating_sub(1);
    }
}

/// Open the context actions menu for the currently selected entry.
///
/// If custom context actions are configured in settings they are used,
/// otherwise a sensible default set is presented. If no entry is selected
/// a short message is shown.
fn handle_context_actions(app: &mut App) {
    let panel = app.active_panel();
    if let Some(e) = panel.selected_entry() {
        let options = if app.settings.context_actions.is_empty() {
            vec!["View".to_string(), "Edit".to_string(), "Permissions".to_string(), "Cancel".to_string()]
        } else {
            app.settings.context_actions.clone()
        };
        app.mode = Mode::ContextMenu { title: format!("Actions: {}", e.name), options, selected: 0, path: e.path.clone() };
    } else {
        app.mode = make_message_mode("Actions", "No entry selected".to_string());
    }
}

/// Collect the source paths that should be acted on for copy/move operations.
///
/// Preference order:
/// 1. If the panel has multi-selections, return all selected entries.
/// 2. Otherwise return the single selected entry (if any).
/// 3. Otherwise return an empty vector.
fn collect_src_paths(app: &App) -> Vec<PathBuf> {
    let panel = app.active_panel();
    if !panel.selections.is_empty() {
        panel.selections.iter().filter_map(|&idx| panel.entries.get(idx).map(|e| e.path.clone())).collect()
    } else if let Some(si) = app.selected_index() {
        panel.entries.get(si).map(|e| vec![e.path.clone()]).unwrap_or_default()
    } else {
        Vec::new()
    }
}

/// Start a background file operation (copy or move).
///
/// This function:
/// - collects source paths using `collect_src_paths`;
/// - determines the destination directory (the opposite panel's cwd);
/// - sets up channels for progress (`op_progress_rx`) and conflict/decision events (`op_decision_tx`);
/// - sets `op_cancel_flag` so the UI can cancel the running operation;
/// - updates `app.mode` to `Mode::Progress` so the UI shows progress;
/// - spawns the appropriate background worker thread (`spawn_copy_worker` or `spawn_move_worker`).
///
/// Progress protocol (the `ProgressUpdate` messages):
///
/// - `processed: usize` — how many items have been processed so far (0-based when reporting errors/cancelled, otherwise counts completed items).
/// - `total: usize` — total number of items in the operation.
/// - `message: Option<String>` — a short human-readable status message suitable for displaying in the progress UI.
/// - `done: bool` — whether the operation has finished (successfully, with error, or cancelled).
/// - `error: Option<String>` — when present and `done == true` indicates the operation ended with an error.
/// - `conflict: Option<PathBuf>` — when present this indicates the worker encountered an existing target and awaits a user decision.
///
/// Typical sequence (example):
///
/// ```text
/// { processed: 0, total: 3, message: "Starting", done: false }
/// { processed: 1, total: 3, message: "Copied /src/a.txt", done: false }
/// { processed: 1, total: 3, message: "Conflict", conflict: Some(/dst/b.txt), done: false }
/// <-- UI sends an OperationDecision (e.g. Overwrite) via `op_decision_tx` -->
/// { processed: 2, total: 3, message: "Copied /src/b.txt", done: false }
/// { processed: 3, total: 3, message: "Completed", done: true }
/// ```
///
/// The UI should display `message` for quick feedback, render a conflict dialog
/// when `conflict` is Some(path) and send an `OperationDecision` down the
/// decision channel. When `done == true` the UI should stop tracking progress
/// and show `error` if present.
fn handle_operation_start(app: &mut App, op: Operation) -> anyhow::Result<()> {
    let src_paths = collect_src_paths(app);
    if src_paths.is_empty() { return Ok(()); }

    let dst_dir = match app.active { Side::Left => app.right.cwd.clone(), Side::Right => app.left.cwd.clone() };

    let (tx, rx) = mpsc::channel();
    let (dec_tx, dec_rx) = mpsc::channel::<OperationDecision>();
    app.op_decision_tx = Some(dec_tx.clone());
    app.op_progress_rx = Some(rx);
    let total = src_paths.len();
    app.mode = Mode::Progress { title: match op { Operation::Copy => "Copying".to_string(), Operation::Move => "Moving".to_string() }, processed: 0, total, message: "Starting".to_string(), cancelled: false };

    let cancel_flag = Arc::new(AtomicBool::new(false));
    app.op_cancel_flag = Some(cancel_flag.clone());

    match op {
        Operation::Copy => spawn_copy_worker(src_paths, dst_dir, tx, dec_rx, cancel_flag),
        Operation::Move => spawn_move_worker(src_paths, dst_dir, tx, dec_rx, cancel_flag),
    }

    Ok(())
}

/// Spawn a background thread that performs copy operations.
///
/// The worker sends `ProgressUpdate` messages over `tx` to report per-item
/// progress and final completion. Conflict resolution decisions are read
/// synchronously from `dec_rx` (the UI thread sends `OperationDecision`
/// values when the user chooses). A shared `cancel_flag` can be set by the
/// UI to request cancellation; the worker will observe it and abort.
///
/// Implementation notes:
/// - Attempts a fast-path batch copy with `fs_extra::copy_items` when no
///   destination names already exist, falling back to per-item handling if
///   conflicts are possible.
/// - Preserves metadata after a successful batch copy via
///   `crate::fs_op::metadata::preserve_all_metadata`.
fn spawn_copy_worker(src_paths: Vec<PathBuf>, dst_dir: PathBuf, tx: mpsc::Sender<ProgressUpdate>, dec_rx: mpsc::Receiver<OperationDecision>, cancel_flag: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let total = src_paths.len();
        // Fast-path: if none of the targets already exist, use batch copy.
        let any_conflict = src_paths.iter().any(|src| src.file_name().map(|fname| dst_dir.join(fname).exists()).unwrap_or(false));

        if !any_conflict {
            let mut options = FsCopyOptions::new();
            options.copy_inside = false;
            options.overwrite = false;
            options.buffer_size = 64 * 1024;
            match copy_items(&src_paths, &dst_dir, &options) {
                Ok(_) => {
                    for src in &src_paths {
                        if let Some(fname) = src.file_name() {
                            let target = dst_dir.join(fname);
                            let _ = crate::fs_op::metadata::preserve_all_metadata(src, &target);
                        }
                    }
                    for (i, src) in src_paths.iter().enumerate() {
                        let _ = tx.send(ProgressUpdate { processed: i + 1, total, message: Some(format!("Copied {}", src.display())), done: false, error: None, conflict: None });
                    }
                    let _ = tx.send(ProgressUpdate { processed: total, total, message: Some("Completed".to_string()), done: true, error: None, conflict: None });
                    return;
                }
                Err(e) => {
                    let _ = tx.send(ProgressUpdate { processed: 0, total, message: Some(format!("Error: {}", e)), done: true, error: Some(format!("{}", e)), conflict: None });
                    return;
                }
            }
        }

        // Per-item handling when conflicts may occur.
        let mut overwrite_all = false;
        let mut skip_all = false;
        for (i, src) in src_paths.into_iter().enumerate() {
            if cancel_flag.load(Ordering::SeqCst) {
                let _ = tx.send(ProgressUpdate { processed: i, total, message: Some("Cancelled".to_string()), done: true, error: Some("Cancelled".to_string()), conflict: None });
                return;
            }
            let target = src.file_name().map(|f| dst_dir.join(f)).unwrap_or_else(|| dst_dir.clone());

            if target.exists() {
                if skip_all {
                    let _ = tx.send(ProgressUpdate { processed: i, total, message: Some(format!("Skipped {}", src.display())), done: false, error: None, conflict: None });
                    continue;
                }
                if !overwrite_all {
                    let _ = tx.send(ProgressUpdate { processed: i, total, message: Some("Conflict".to_string()), done: false, error: None, conflict: Some(target.clone()) });
                    match dec_rx.recv() {
                        Ok(OperationDecision::Cancel) => { let _ = tx.send(ProgressUpdate { processed: i, total, message: Some("Cancelled by user".to_string()), done: true, error: Some("Cancelled by user".to_string()), conflict: None }); return; }
                        Ok(OperationDecision::Skip) => { let _ = tx.send(ProgressUpdate { processed: i, total, message: Some(format!("Skipped {}", src.display())), done: false, error: None, conflict: None }); continue; }
                        Ok(OperationDecision::SkipAll) => { skip_all = true; let _ = tx.send(ProgressUpdate { processed: i, total, message: Some(format!("Skipped {} (all)", src.display())), done: false, error: None, conflict: None }); continue; }
                        Ok(OperationDecision::OverwriteAll) => { overwrite_all = true; }
                        Ok(OperationDecision::Overwrite) => {}
                        Err(_) => { let _ = tx.send(ProgressUpdate { processed: i, total, message: Some("Decision channel closed".to_string()), done: true, error: Some("Decision channel closed".to_string()), conflict: None }); return; }
                    }
                }
                let _ = if target.is_dir() { std::fs::remove_dir_all(&target) } else { std::fs::remove_file(&target) };
            }

            let res = if src.is_dir() {
                crate::fs_op::copy::copy_recursive(&src, &target)
            } else if let Err(e) = crate::fs_op::helpers::ensure_parent_exists(&target) {
                Err(e)
            } else {
                crate::fs_op::helpers::atomic_copy_file(&src, &target).map(|_| ())
            };
            if let Err(e) = res { let _ = tx.send(ProgressUpdate { processed: i, total, message: Some(format!("Error: {}", e)), done: true, error: Some(format!("{}", e)), conflict: None }); return; }
            let _ = tx.send(ProgressUpdate { processed: i + 1, total, message: Some(format!("Copied {}", src.display())), done: false, error: None, conflict: None });
        }
        let _ = tx.send(ProgressUpdate { processed: total, total, message: Some("Completed".to_string()), done: true, error: None, conflict: None });
    });
}

/// Spawn a background thread that performs move (rename) operations.
///
/// The worker semantics mirror `spawn_copy_worker` but use
/// `atomic_rename_or_copy` to attempt a rename and fall back to copying
/// when necessary. Progress, conflict decisions, and cancellation behave
/// the same as for the copy worker.
fn spawn_move_worker(src_paths: Vec<PathBuf>, dst_dir: PathBuf, tx: mpsc::Sender<ProgressUpdate>, dec_rx: mpsc::Receiver<OperationDecision>, cancel_flag: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let mut overwrite_all = false;
        let mut skip_all = false;
        let total = src_paths.len();
        for (i, src) in src_paths.into_iter().enumerate() {
            if cancel_flag.load(Ordering::SeqCst) { let _ = tx.send(ProgressUpdate { processed: i, total, message: Some("Cancelled".to_string()), done: true, error: Some("Cancelled".to_string()), conflict: None }); return; }
            let target = src.file_name().map(|f| dst_dir.join(f)).unwrap_or_else(|| dst_dir.clone());

            if target.exists() {
                if skip_all { let _ = tx.send(ProgressUpdate { processed: i, total, message: Some(format!("Skipped {}", src.display())), done: false, error: None, conflict: None }); continue; }
                if !overwrite_all {
                    let _ = tx.send(ProgressUpdate { processed: i, total, message: Some("Conflict".to_string()), done: false, error: None, conflict: Some(target.clone()) });
                    match dec_rx.recv() {
                        Ok(OperationDecision::Cancel) => { let _ = tx.send(ProgressUpdate { processed: i, total, message: Some("Cancelled by user".to_string()), done: true, error: Some("Cancelled by user".to_string()), conflict: None }); return; }
                        Ok(OperationDecision::Skip) => { let _ = tx.send(ProgressUpdate { processed: i, total, message: Some(format!("Skipped {}", src.display())), done: false, error: None, conflict: None }); continue; }
                        Ok(OperationDecision::SkipAll) => { skip_all = true; let _ = tx.send(ProgressUpdate { processed: i, total, message: Some(format!("Skipped {} (all)", src.display())), done: false, error: None, conflict: None }); continue; }
                        Ok(OperationDecision::OverwriteAll) => { overwrite_all = true; }
                        Ok(OperationDecision::Overwrite) => {}
                        Err(_) => { let _ = tx.send(ProgressUpdate { processed: i, total, message: Some("Decision channel closed".to_string()), done: true, error: Some("Decision channel closed".to_string()), conflict: None }); return; }
                    }
                }
                let _ = if target.is_dir() { std::fs::remove_dir_all(&target) } else { std::fs::remove_file(&target) };
            }

            let res = if let Err(e) = crate::fs_op::helpers::ensure_parent_exists(&target) {
                Err(e)
            } else {
                crate::fs_op::helpers::atomic_rename_or_copy(&src, &target).map(|_| ())
            };
            if let Err(e) = res { let _ = tx.send(ProgressUpdate { processed: i, total, message: Some(format!("Error: {}", e)), done: true, error: Some(format!("{}", e)), conflict: None }); return; }
            let _ = tx.send(ProgressUpdate { processed: i + 1, total, message: Some(format!("Moved {}", src.display())), done: false, error: None, conflict: None });
        }
        let _ = tx.send(ProgressUpdate { processed: total, total, message: Some("Completed".to_string()), done: true, error: None, conflict: None });
    });
}
