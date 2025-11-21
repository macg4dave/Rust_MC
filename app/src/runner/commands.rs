use crate::app::{Action, App};
use crate::fs_op::error::FsOpError;
use anyhow::Result;

/// Perform an Action on the given app instance.
pub fn perform_action(app: &mut App, action: Action) -> Result<(), FsOpError> {
    match action {
        Action::DeleteSelected => app.delete_selected(),
        Action::CopyTo(p) => app.copy_selected_to(p),
        Action::MoveTo(p) => app.move_selected_to(p),
        Action::RenameTo(name) => app.rename_selected_to(name),
        Action::NewFile(name) => app.new_file(name),
        Action::NewDir(name) => app.new_dir(name),
    }
}

/// Parse and execute a simple textual command from the command-line input.
/// Returns Ok(true) if a command matched and was executed, Ok(false) if
/// the command was unrecognized.
pub fn execute_command(app: &mut App, input: &str) -> Result<bool> {
    let cmd = input.trim();
    if cmd.is_empty() {
        return Ok(false);
    }
    match cmd {
        "toggle-preview" => {
            app.toggle_preview();
            Ok(true)
        }
        "menu-next" => {
            app.menu_next();
            Ok(true)
        }
        "menu-prev" => {
            app.menu_prev();
            Ok(true)
        }
        "menu-activate" => {
            app.menu_activate();
            Ok(true)
        }
        _ => Ok(false),
    }
}
