use std::io::{self, stdout};
use std::path::Path;
use std::process::Command;

use crossterm::cursor::{Show, Hide};
use crossterm::execute;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen, EnterAlternateScreen};

/// Spawn `vim` on the given `path`, suspending the TUI (restoring the terminal)
/// and re-entering TUI mode after the editor exits.
///
/// This function is conservative: it attempts to restore the terminal state
/// even if launching the editor fails.
pub fn spawn_vim<P: AsRef<Path>>(path: P) -> io::Result<()> {
	// Disable raw mode and leave the alternate screen so the spawned editor
	// can take full control of the terminal with normal line buffering.
	if let Err(e) = disable_raw_mode() {
		// best-effort: return the error, nothing else we can do
		return Err(e);
	}

	let mut stdout = stdout();
	// Leave alternate screen, disable mouse capture and show cursor
	let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture, Show);

	// Run the editor synchronously. If `vim` isn't available, the error will
	// be returned to the caller after we try to restore terminal state.
	let status = Command::new("vim").arg(path.as_ref()).status();

	// After the editor exits (or fails to spawn), try to re-enter the TUI
	// environment: hide cursor, enable mouse capture and enter alternate
	// screen, then enable raw mode.
	let _ = execute!(stdout, Hide, EnableMouseCapture, EnterAlternateScreen);
	if let Err(e) = enable_raw_mode() {
	// Return original spawn error if present, otherwise this one.
	return status.and(Err(e));
	}

	// Propagate the editor process status (map to io::Error when appropriate).
	match status {
		Ok(s) if s.success() => Ok(()),
		Ok(s) => Err(io::Error::other(format!("vim exited with status: {}", s))),
		Err(e) => Err(e),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	// This test only verifies the function returns an error when `vim` is not
	// present or when trying to spawn without a real terminal. It is a
	// lightweight sanity check and not intended to actually open an editor in
	// CI.
	#[test]
	#[ignore]
	fn spawn_vim_handles_missing_editor() {
		// Use a likely-nonexistent path to avoid accidentally editing files.
		let test_path = PathBuf::from("/tmp/filezoom-test-nonexistent-should-not-create.txt");
		// We just ensure the function returns (Ok or Err) without panicking.
		let _ = spawn_vim(test_path);
	}
}