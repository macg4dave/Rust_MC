use crate::app::{Action, App};

/// Perform an Action on the given app instance.
pub fn perform_action(app: &mut App, action: Action) -> std::io::Result<()> {
    match action {
        Action::DeleteSelected => app.delete_selected(),
        Action::CopyTo(p) => app.copy_selected_to(p),
        Action::MoveTo(p) => app.move_selected_to(p),
        Action::RenameTo(name) => app.rename_selected_to(name),
        Action::NewFile(name) => app.new_file(name),
        Action::NewDir(name) => app.new_dir(name),
    }
}
