//! Utilities for parsing and executing high-level UI actions.
//!
//! This module centralises small runner helpers used by the input layer.
//! It intentionally keeps a thin surface area: parsing/normalising short
//! textual commands (used by the command-line prompt) and dispatching
//! `Action` values to the associated `App` methods.

use crate::app::{Action, App};
use crate::fs_op::error::FsOpError;

/// Parseable, textual commands accepted by the command-line input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ParsedCommand {
    TogglePreview,
    MenuNext,
    MenuPrev,
    MenuActivate,
}

impl ParsedCommand {
    /// Execute this parsed command against the provided application.
    ///
    /// The method intentionally takes `&mut App` and performs only
    /// in-memory state changes or delegates to existing `App` helpers.
    pub(crate) fn execute(self, app: &mut App) {
        match self {
            ParsedCommand::TogglePreview => app.toggle_preview(),
            ParsedCommand::MenuNext => app.menu_next(),
            ParsedCommand::MenuPrev => app.menu_prev(),
            ParsedCommand::MenuActivate => app.menu_activate(),
        }
    }
}

/// Attempt to parse a short textual command from `input`.
///
/// Returns `Some(ParsedCommand)` when the input matches a known command
/// (ignoring surrounding whitespace), otherwise `None`.
pub(crate) fn parse_command(input: &str) -> Option<ParsedCommand> {
    match input.trim() {
        "toggle-preview" => Some(ParsedCommand::TogglePreview),
        "menu-next" => Some(ParsedCommand::MenuNext),
        "menu-prev" => Some(ParsedCommand::MenuPrev),
        "menu-activate" => Some(ParsedCommand::MenuActivate),
        _ => None,
    }
}

/// Perform an `Action` on the application.
///
/// This is a small dispatcher that maps high-level `Action` values to the
/// corresponding `App` methods. It preserves the original return type
/// from the underlying filesystem helpers (`FsOpError`).
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

/// Parse and execute a short textual command from the command-line input.
///
/// Returns `Ok(true)` if a known command matched and was executed, `Ok(false)`
/// if the input was empty or unrecognised. The function does not currently
/// report filesystem errors because the handled commands operate on in-memory
/// state only; the result error type is `FsOpError` for future-proofing.
pub fn execute_command(app: &mut App, input: &str) -> Result<bool, FsOpError> {
    if let Some(cmd) = parse_command(input) {
        cmd.execute(app);
        Ok(true)
    } else {
        Ok(false)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_known_commands() {
        assert_eq!(parse_command("toggle-preview"), Some(ParsedCommand::TogglePreview));
        assert_eq!(parse_command(" menu-next "), Some(ParsedCommand::MenuNext));
        assert_eq!(parse_command("menu-prev"), Some(ParsedCommand::MenuPrev));
        assert_eq!(parse_command("menu-activate"), Some(ParsedCommand::MenuActivate));
    }

    #[test]
    fn parse_unknown_or_empty() {
        assert_eq!(parse_command(""), None);
        assert_eq!(parse_command("unknown"), None);
        assert_eq!(parse_command("toggle_preview"), None);
    }
}
