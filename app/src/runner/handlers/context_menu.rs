use crate::app::{App, Mode};
use crate::input::KeyCode;
use crate::app::settings::keybinds;

/// Well-known labels for context-menu actions.
///
/// Using a small enum makes matching explicit and avoids repeated string
/// comparisons throughout the handler logic.
#[derive(Debug, Clone)]
enum ContextAction {
    View,
    Edit,
    Permissions,
    /// Any action label we don't specifically recognise.
    Other(String),
}

impl ContextAction {
    /// Parse a label shown in the UI into a known action when possible.
    fn from_label(label: &str) -> Self {
        match label {
            "View" | "Open" => ContextAction::View,
            "Edit" => ContextAction::Edit,
            "Permissions" | "Inspect Permissions" => ContextAction::Permissions,
            other => ContextAction::Other(other.to_string()),
        }
    }
}

/// Handle key events while the application is displaying a context menu.
///
/// Returns `Ok(false)` to indicate the event was handled; the boolean return
/// value is reserved for callers that may want to short-circuit further
/// processing in the future. This function intentionally does not return
/// application-level errors; any failures during auxiliary operations (for
/// example spawning an editor) are reported to the user via `Mode::Message`.
pub fn handle_context_menu(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    // Build a `Mode::Message` value without mutably borrowing `app` so we can
    // assign it after inspecting `app.mode` (avoids borrow conflicts).
    let build_message = |title: &str, content: String| -> Mode {
        Mode::Message {
            title: title.to_string(),
            content,
            buttons: vec!["OK".to_string()],
            selected: 0,
            actions: None,
        }
    };

    // If we need to change `app.mode`, we store the new mode here and assign
    // it after the match to avoid borrowing `app.mode` while it's being
    // inspected.
    let mut pending_mode: Option<Mode> = None;

    if let Mode::ContextMenu {
            title: _,
            options,
            selected,
            path: _,
        } = &mut app.mode {
            // Navigation: move selection left/up or right/down.
            if keybinds::is_left(&code) || keybinds::is_up(&code) {
                *selected = selected.saturating_sub(1);
            } else if keybinds::is_right(&code) || keybinds::is_down(&code) {
                // clamp at last option index
                if *selected + 1 < options.len() {
                    *selected += 1;
                }
            } else if keybinds::is_char(&code, 'q') || keybinds::is_esc(&code) {
                pending_mode = Some(Mode::Normal);
            } else if keybinds::is_enter(&code) {
                // Snapshot the chosen option before we replace the mode.
                let choice = options.get(*selected).cloned();
                // By default dismiss the context menu; specific actions may
                // replace this with a message dialog.
                pending_mode = Some(Mode::Normal);

                if let Some(ch) = choice {
                    // Parse the chosen label into a known action where possible.
                    match ContextAction::from_label(ch.as_str()) {
                        ContextAction::View => {
                            app.preview_visible = true;
                            app.update_preview_for(app.active);
                        }
                        ContextAction::Edit => {
                            if let Some(e) = app.active_panel().selected_entry() {
                                let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
                                let use_integrated = app.settings.prefer_integrated_vim
                                    || editor == "vi"
                                    || editor == "vim";

                                if use_integrated {
                                    pending_mode = match crate::app::text_editors::vim_support::spawn_vim(&e.path) {
                                        Ok(_) => Some(build_message("Edit", format!("Launched vim for: {}", e.name))),
                                        Err(_) => Some(build_message("Edit", "Failed to launch vim".to_string())),
                                    };
                                } else {
                                    let cmd = format!("{} \"{}\"", editor, e.path.display());
                                    pending_mode = match std::process::Command::new("sh").arg("-c").arg(cmd).spawn() {
                                        Ok(_) => Some(build_message("Edit", format!("Launched editor: {}", editor))),
                                        Err(_) => Some(build_message("Edit", "Failed to launch editor".to_string())),
                                    };
                                }
                            } else {
                                pending_mode = Some(build_message("Edit", "No entry selected".to_string()));
                            }
                        }
                        ContextAction::Permissions => {
                            if let Some(e) = app.active_panel().selected_entry() {
                                match std::fs::metadata(&e.path) {
                                    Ok(md) => {
                                        #[cfg(unix)]
                                        {
                                            use std::os::unix::fs::PermissionsExt;
                                            let mode = md.permissions().mode();
                                            pending_mode = Some(build_message("Permissions", format!("{}: {:o}", e.name, mode)));
                                        }
                                        #[cfg(not(unix))]
                                        {
                                            pending_mode = Some(build_message("Permissions", format!("{}: (platform-specific metadata)", e.name)));
                                        }
                                    }
                                    Err(_) => pending_mode = Some(build_message("Permissions", "Cannot read metadata".to_string())),
                                }
                            } else {
                                pending_mode = Some(build_message("Permissions", "No entry selected".to_string()));
                            }
                        }
                        ContextAction::Other(label) => pending_mode = Some(build_message("Action", format!("Action '{}' not implemented", label))),
                    }
                }
            }
    }

    if let Some(m) = pending_mode {
        app.mode = m;
    }

    Ok(false)
}
