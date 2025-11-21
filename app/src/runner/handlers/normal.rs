use crate::app::{Action, App, InputKind, Mode, Side};
use crate::errors;
use crate::input::KeyCode;
use crate::runner::progress::{OperationDecision, ProgressUpdate};
use std::path::PathBuf;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions as FsCopyOptions;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

pub fn handle_normal(app: &mut App, code: KeyCode, page_size: usize) -> anyhow::Result<bool> {
    // If command-line is active, route keys there first.
    if app.command_line.is_some() {
        return crate::ui::command_line::handle_input(app, code);
    }
    match &code {
        &KeyCode::Char('q') => return Ok(true),
        &KeyCode::Down => app.select_next(page_size),
        &KeyCode::Up => app.select_prev(page_size),
        &KeyCode::PageDown => app.select_page_down(page_size),
        &KeyCode::PageUp => app.select_page_up(page_size),
        &KeyCode::Enter if !app.menu_focused => {
            let panel = app.active_panel_mut();
            if panel.selected == 0 {
                let prompt = format!("Change path (current: {}):", panel.cwd.display());
                app.mode = Mode::Input {
                    prompt,
                    buffer: String::new(),
                    kind: InputKind::ChangePath,
                };
            } else {
                let parent_count = if panel.cwd.parent().is_some() {
                    1usize
                } else {
                    0usize
                };
                if panel.selected == 1 && parent_count == 1 {
                    if let Err(err) = app.go_up() {
                        let msg = errors::render_fsop_error(&err, None, None, None);
                        app.mode = Mode::Message {
                            title: "Error".to_string(),
                            content: msg,
                            buttons: vec!["OK".to_string()],
                            selected: 0,
                            actions: None,
                        };
                    }
                } else if let Some(e) = panel.selected_entry().cloned() {
                    if let Err(err) = app.enter() {
                        let path_s = e.path.display().to_string();
                        let msg = errors::render_fsop_error(&err, Some(&path_s), None, None);
                        app.mode = Mode::Message {
                            title: "Error".to_string(),
                            content: msg,
                            buttons: vec!["OK".to_string()],
                            selected: 0,
                            actions: None,
                        };
                    }
                }
            }
        }
        &KeyCode::Backspace => {
            if let Err(err) = app.go_up() {
                let msg = errors::render_fsop_error(&err, None, None, None);
                app.mode = Mode::Message {
                    title: "Error".to_string(),
                    content: msg,
                    buttons: vec!["OK".to_string()],
                    selected: 0,
                    actions: None,
                };
            }
        }
        &KeyCode::Char('r') => {
            if let Err(err) = app.refresh() {
                let msg = errors::render_io_error(&err, None, None, None);
                app.mode = Mode::Message {
                    title: "Error".to_string(),
                    content: msg,
                    buttons: vec!["OK".to_string()],
                    selected: 0,
                    actions: None,
                };
            }
        }
        &KeyCode::Char('d') => {
            let panel = app.active_panel_mut();
            let e_opt = panel.selected_entry();
            if let Some(e) = e_opt {
                let msg = format!("Delete {}? (y/n)", e.name);
                app.mode = Mode::Confirm {
                    msg,
                    on_yes: Action::DeleteSelected,
                    selected: 0,
                };
            }
        }
        &KeyCode::Char('c') => {
            let panel = app.active_panel_mut();
            let e_opt = panel.selected_entry();
            if let Some(e) = e_opt {
                let prompt = format!("Copy {} to:", e.name);
                app.mode = Mode::Input {
                    prompt,
                    buffer: String::new(),
                    kind: InputKind::Copy,
                };
            }
        }
        &KeyCode::Char('m') => {
            let panel = app.active_panel_mut();
            let e_opt = panel.selected_entry();
            if let Some(e) = e_opt {
                let prompt = format!("Move {} to:", e.name);
                app.mode = Mode::Input {
                    prompt,
                    buffer: String::new(),
                    kind: InputKind::Move,
                };
            }
        }
        &KeyCode::Char('n') => {
            app.mode = Mode::Input {
                prompt: "New file name:".to_string(),
                buffer: String::new(),
                kind: InputKind::NewFile,
            };
        }
        &KeyCode::Char('N') => {
            app.mode = Mode::Input {
                prompt: "New dir name:".to_string(),
                buffer: String::new(),
                kind: InputKind::NewDir,
            };
        }
        &KeyCode::Char('R') => {
            let panel = app.active_panel_mut();
            let e_opt = panel.entries.get(panel.selected);
            if let Some(e) = e_opt {
                let prompt = format!("Rename {} to:", e.name);
                app.mode = Mode::Input {
                    prompt,
                    buffer: String::new(),
                    kind: InputKind::Rename,
                };
            }
        }
        &KeyCode::Char('s') => {
            app.sort = app.sort.next();
            app.refresh()?;
        }
        &KeyCode::Char('S') => {
            app.sort_order = match app.sort_order {
                crate::app::types::SortOrder::Ascending => crate::app::types::SortOrder::Descending,
                crate::app::types::SortOrder::Descending => crate::app::types::SortOrder::Ascending,
            };
            app.refresh()?;
        }
        &KeyCode::Char(' ') => {
            app.active_panel_mut().toggle_selection();
        }
        &KeyCode::Tab => {
            app.active = match app.active {
                Side::Left => Side::Right,
                Side::Right => Side::Left,
            };
        }
        &KeyCode::F(5) => {
            let src_paths: Vec<PathBuf> = {
                let panel = app.active_panel();
                let mut v = Vec::new();
                if !panel.selections.is_empty() {
                    for &idx in panel.selections.iter() {
                        if let Some(e) = panel.entries.get(idx) {
                            v.push(e.path.clone());
                        }
                    }
                } else if let Some(si) = app.selected_index() {
                    if let Some(e) = panel.entries.get(si) {
                        v.push(e.path.clone());
                    }
                }
                v
            };
            if src_paths.is_empty() {
                return Ok(false);
            }
            let dst_dir = match app.active {
                Side::Left => app.right.cwd.clone(),
                Side::Right => app.left.cwd.clone(),
            };

            let (tx, rx) = mpsc::channel();
            let (dec_tx, dec_rx) = mpsc::channel::<OperationDecision>();
            app.op_decision_tx = Some(dec_tx.clone());
            app.op_progress_rx = Some(rx);
            let total = src_paths.len();
            app.mode = Mode::Progress {
                title: "Copying".to_string(),
                processed: 0,
                total,
                message: "Starting".to_string(),
                cancelled: false,
            };

            let cancel_flag = Arc::new(AtomicBool::new(false));
            app.op_cancel_flag = Some(cancel_flag.clone());

            std::thread::spawn(move || {
                // Fast-path: if none of the targets already exist, we can copy
                // all items in one batch using `fs_extra::copy_items`, which is
                // typically faster. If any target exists we must keep the
                // interactive per-item conflict handling below.
                let mut any_conflict = false;
                for src in &src_paths {
                    if let Some(fname) = src.file_name() {
                        let target = dst_dir.join(fname);
                        if target.exists() {
                            any_conflict = true;
                            break;
                        }
                    }
                }
                if !any_conflict {
                    let mut options = FsCopyOptions::new();
                    options.copy_inside = false; // copy the items themselves into dst
                    options.overwrite = false;
                    // Use a 64 KiB buffer for batch copies to balance throughput and memory.
                    options.buffer_size = 64 * 1024;
                    match copy_items(&src_paths, &dst_dir, &options) {
                        Ok(_) => {
                            // After a successful batch copy, attempt to preserve
                            // permissions/timestamps for each copied item. This is
                            // best-effort and will not abort the operation on
                            // failures; if preserving metadata fails we'll send
                            // an error update instead.
                            for src in &src_paths {
                                if let Some(fname) = src.file_name() {
                                    let target = dst_dir.join(fname);
                                    let _ = crate::fs_op::metadata::preserve_all_metadata(src, &target);
                                }
                            }
                            // send progress updates for each item
                            for (i, src) in src_paths.into_iter().enumerate() {
                                let _ = tx.send(ProgressUpdate {
                                    processed: i + 1,
                                    total,
                                    message: Some(format!("Copied {}", src.display())),
                                    done: false,
                                    error: None,
                                    conflict: None,
                                });
                            }
                            let _ = tx.send(ProgressUpdate {
                                processed: total,
                                total,
                                message: Some("Completed".to_string()),
                                done: true,
                                error: None,
                                conflict: None,
                            });
                            return;
                        }
                        Err(e) => {
                            let _ = tx.send(ProgressUpdate {
                                processed: 0,
                                total,
                                message: Some(format!("Error: {}", e)),
                                done: true,
                                error: Some(format!("{}", e)),
                                conflict: None,
                            });
                            return;
                        }
                    }
                }

                let mut overwrite_all = false;
                let mut skip_all = false;
                for (i, src) in src_paths.into_iter().enumerate() {
                    if cancel_flag.load(Ordering::SeqCst) {
                        let _ = tx.send(ProgressUpdate {
                            processed: i,
                            total,
                            message: Some("Cancelled".to_string()),
                            done: true,
                            error: Some("Cancelled".to_string()),
                            conflict: None,
                        });
                        return;
                    }
                    let file_name = src.file_name().map(|s| s.to_os_string());
                    let target = if let Some(fname) = &file_name {
                        dst_dir.join(fname)
                    } else {
                        dst_dir.clone()
                    };

                    if target.exists() {
                        if skip_all {
                            let _ = tx.send(ProgressUpdate {
                                processed: i,
                                total,
                                message: Some(format!("Skipped {}", src.display())),
                                done: false,
                                error: None,
                                conflict: None,
                            });
                            continue;
                        }
                        if !overwrite_all {
                            let _ = tx.send(ProgressUpdate {
                                processed: i,
                                total,
                                message: Some("Conflict".to_string()),
                                done: false,
                                error: None,
                                conflict: Some(target.clone()),
                            });
                            match dec_rx.recv() {
                                Ok(OperationDecision::Cancel) => {
                                    let _ = tx.send(ProgressUpdate {
                                        processed: i,
                                        total,
                                        message: Some("Cancelled by user".to_string()),
                                        done: true,
                                        error: Some("Cancelled by user".to_string()),
                                        conflict: None,
                                    });
                                    return;
                                }
                                Ok(OperationDecision::Skip) => {
                                    let _ = tx.send(ProgressUpdate {
                                        processed: i,
                                        total,
                                        message: Some(format!("Skipped {}", src.display())),
                                        done: false,
                                        error: None,
                                        conflict: None,
                                    });
                                    continue;
                                }
                                Ok(OperationDecision::SkipAll) => {
                                    skip_all = true;
                                    let _ = tx.send(ProgressUpdate {
                                        processed: i,
                                        total,
                                        message: Some(format!("Skipped {} (all)", src.display())),
                                        done: false,
                                        error: None,
                                        conflict: None,
                                    });
                                    continue;
                                }
                                Ok(OperationDecision::OverwriteAll) => {
                                    overwrite_all = true;
                                }
                                Ok(OperationDecision::Overwrite) => {}
                                Err(_) => {
                                    let _ = tx.send(ProgressUpdate {
                                        processed: i,
                                        total,
                                        message: Some("Decision channel closed".to_string()),
                                        done: true,
                                        error: Some("Decision channel closed".to_string()),
                                        conflict: None,
                                    });
                                    return;
                                }
                            }
                        }
                        if target.is_dir() {
                            let _ = std::fs::remove_dir_all(&target);
                        } else {
                            let _ = std::fs::remove_file(&target);
                        }
                    }

                    let res = if src.is_dir() {
                        crate::fs_op::copy::copy_recursive(&src, &target)
                    } else {
                        if let Err(e) = crate::fs_op::helpers::ensure_parent_exists(&target) {
                            Err(e)
                        } else {
                            crate::fs_op::helpers::atomic_copy_file(&src, &target).map(|_| ())
                        }
                    };
                    if let Err(e) = res {
                        let _ = tx.send(ProgressUpdate {
                            processed: i,
                            total,
                            message: Some(format!("Error: {}", e)),
                            done: true,
                            error: Some(format!("{}", e)),
                            conflict: None,
                        });
                        return;
                    }
                    let _ = tx.send(ProgressUpdate {
                        processed: i + 1,
                        total,
                        message: Some(format!("Copied {}", src.display())),
                        done: false,
                        error: None,
                        conflict: None,
                    });
                }
                let _ = tx.send(ProgressUpdate {
                    processed: total,
                    total,
                    message: Some("Completed".to_string()),
                    done: true,
                    error: None,
                    conflict: None,
                });
            });
        }
        &KeyCode::F(6) => {
            let src_paths: Vec<PathBuf> = {
                let panel = app.active_panel();
                let mut v = Vec::new();
                if !panel.selections.is_empty() {
                    for &idx in panel.selections.iter() {
                        if let Some(e) = panel.entries.get(idx) {
                            v.push(e.path.clone());
                        }
                    }
                } else if let Some(si) = app.selected_index() {
                    if let Some(e) = panel.entries.get(si) {
                        v.push(e.path.clone());
                    }
                }
                v
            };
            if src_paths.is_empty() {
                return Ok(false);
            }
            let dst_dir = match app.active {
                Side::Left => app.right.cwd.clone(),
                Side::Right => app.left.cwd.clone(),
            };

            let (tx, rx) = mpsc::channel();
            let (dec_tx, dec_rx) = mpsc::channel::<OperationDecision>();
            app.op_decision_tx = Some(dec_tx.clone());
            app.op_progress_rx = Some(rx);
            let total = src_paths.len();
            app.mode = Mode::Progress {
                title: "Moving".to_string(),
                processed: 0,
                total,
                message: "Starting".to_string(),
                cancelled: false,
            };

            let cancel_flag = Arc::new(AtomicBool::new(false));
            app.op_cancel_flag = Some(cancel_flag.clone());

            std::thread::spawn(move || {
                let mut overwrite_all = false;
                let mut skip_all = false;
                for (i, src) in src_paths.into_iter().enumerate() {
                    if cancel_flag.load(Ordering::SeqCst) {
                        let _ = tx.send(ProgressUpdate {
                            processed: i,
                            total,
                            message: Some("Cancelled".to_string()),
                            done: true,
                            error: Some("Cancelled".to_string()),
                            conflict: None,
                        });
                        return;
                    }
                    let file_name = src.file_name().map(|s| s.to_os_string());
                    let target = if let Some(fname) = &file_name {
                        dst_dir.join(fname)
                    } else {
                        dst_dir.clone()
                    };

                    if target.exists() {
                        if skip_all {
                            let _ = tx.send(ProgressUpdate {
                                processed: i,
                                total,
                                message: Some(format!("Skipped {}", src.display())),
                                done: false,
                                error: None,
                                conflict: None,
                            });
                            continue;
                        }
                        if !overwrite_all {
                            let _ = tx.send(ProgressUpdate {
                                processed: i,
                                total,
                                message: Some("Conflict".to_string()),
                                done: false,
                                error: None,
                                conflict: Some(target.clone()),
                            });
                            match dec_rx.recv() {
                                Ok(OperationDecision::Cancel) => {
                                    let _ = tx.send(ProgressUpdate {
                                        processed: i,
                                        total,
                                        message: Some("Cancelled by user".to_string()),
                                        done: true,
                                        error: Some("Cancelled by user".to_string()),
                                        conflict: None,
                                    });
                                    return;
                                }
                                Ok(OperationDecision::Skip) => {
                                    let _ = tx.send(ProgressUpdate {
                                        processed: i,
                                        total,
                                        message: Some(format!("Skipped {}", src.display())),
                                        done: false,
                                        error: None,
                                        conflict: None,
                                    });
                                    continue;
                                }
                                Ok(OperationDecision::SkipAll) => {
                                    skip_all = true;
                                    let _ = tx.send(ProgressUpdate {
                                        processed: i,
                                        total,
                                        message: Some(format!("Skipped {} (all)", src.display())),
                                        done: false,
                                        error: None,
                                        conflict: None,
                                    });
                                    continue;
                                }
                                Ok(OperationDecision::OverwriteAll) => {
                                    overwrite_all = true;
                                }
                                Ok(OperationDecision::Overwrite) => {}
                                Err(_) => {
                                    let _ = tx.send(ProgressUpdate {
                                        processed: i,
                                        total,
                                        message: Some("Decision channel closed".to_string()),
                                        done: true,
                                        error: Some("Decision channel closed".to_string()),
                                        conflict: None,
                                    });
                                    return;
                                }
                            }
                        }
                        if target.is_dir() {
                            let _ = std::fs::remove_dir_all(&target);
                        } else {
                            let _ = std::fs::remove_file(&target);
                        }
                    }

                    let res = if let Err(e) = crate::fs_op::helpers::ensure_parent_exists(&target) {
                        Err(e)
                    } else {
                        crate::fs_op::helpers::atomic_rename_or_copy(&src, &target).map(|_| ())
                    };
                    if let Err(e) = res {
                        let _ = tx.send(ProgressUpdate {
                            processed: i,
                            total,
                            message: Some(format!("Error: {}", e)),
                            done: true,
                            error: Some(format!("{}", e)),
                            conflict: None,
                        });
                        return;
                    }
                    let _ = tx.send(ProgressUpdate {
                        processed: i + 1,
                        total,
                        message: Some(format!("Moved {}", src.display())),
                        done: false,
                        error: None,
                        conflict: None,
                    });
                }
                let _ = tx.send(ProgressUpdate {
                    processed: total,
                    total,
                    message: Some("Completed".to_string()),
                    done: true,
                    error: None,
                    conflict: None,
                });
            });
        }
        &KeyCode::F(1) => {
            app.menu_focused = !app.menu_focused;
        }
        KeyCode::Left if app.menu_focused => {
            app.menu_prev();
        }
        KeyCode::Right if app.menu_focused => {
            app.menu_next();
        }
        KeyCode::Enter if app.menu_focused => {
            app.menu_activate();
            app.menu_focused = false;
        }
        KeyCode::Esc if app.menu_focused => {
            app.menu_focused = false;
        }
        KeyCode::Home => {
            app.active_panel_mut().selected = 0;
        }
        KeyCode::End => {
            let panel = app.active_panel_mut();
            if !panel.entries.is_empty() {
                let header_count = 1usize;
                let parent_count = if panel.cwd.parent().is_some() {
                    1usize
                } else {
                    0usize
                };
                panel.selected =
                    header_count + parent_count + panel.entries.len().saturating_sub(1);
            }
        }
        KeyCode::Char('p') => {
            app.toggle_preview();
        }
        KeyCode::F(3) => {
            let panel = app.active_panel();
            if let Some(e) = panel.selected_entry() {
                let options = if app.settings.context_actions.is_empty() {
                    vec![
                        "View".to_string(),
                        "Edit".to_string(),
                        "Permissions".to_string(),
                        "Cancel".to_string(),
                    ]
                } else {
                    app.settings.context_actions.clone()
                };
                app.mode = Mode::ContextMenu {
                    title: format!("Actions: {}", e.name),
                    options,
                    selected: 0,
                    path: e.path.clone(),
                };
            } else {
                app.mode = Mode::Message {
                    title: "Actions".to_string(),
                    content: "No entry selected".to_string(),
                    buttons: vec!["OK".to_string()],
                    selected: 0,
                    actions: None,
                };
            }
        }
        KeyCode::Char('t') => {
            crate::ui::colors::toggle();
        }
        KeyCode::Char('?') => {
            let content = "Keys:\n\nq: quit\nF1: toggle menu focus\nLeft/Right: menu navigation when focused\nEnter: open/activate\nBackspace: up\nd: delete\nc: copy\nm: move\nn/N: new file/dir\nR: rename\ns/S: sort (toggle desc)\nTab: switch panels\n?: show this help\n".to_string();
            app.mode = Mode::Message {
                title: "Help".to_string(),
                content,
                buttons: vec!["OK".to_string()],
                selected: 0,
                actions: None,
            };
        }
        KeyCode::Char('>') => {
            let panel = app.active_panel_mut();
            panel.preview_offset = panel.preview_offset.saturating_add(5);
        }
        KeyCode::Char('<') => {
            let panel = app.active_panel_mut();
            panel.preview_offset = panel.preview_offset.saturating_sub(5);
        }
        _ => {}
    }
    Ok(false)
}
