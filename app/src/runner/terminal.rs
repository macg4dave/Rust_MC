use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::cursor::{Hide, Show};
use crossterm::queue;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::fmt;
use std::io;
use std::io::Stdout;
use std::io::Write;
use std::ops::{Deref, DerefMut};

/// Errors returned by terminal initialization/restore helpers.
#[derive(Debug)]
pub enum TerminalError {
    Io(io::Error),
    /// Fallback for other kinds of errors (crossterm error kinds, misc).
    Other(String),
}

impl fmt::Display for TerminalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TerminalError::Io(e) => write!(f, "IO error: {}", e),
            TerminalError::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for TerminalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TerminalError::Io(e) => Some(e),
            TerminalError::Other(_) => None,
        }
    }
}

impl From<io::Error> for TerminalError {
    fn from(e: io::Error) -> Self {
        TerminalError::Io(e)
    }
}

impl From<anyhow::Error> for TerminalError {
    fn from(e: anyhow::Error) -> Self {
        TerminalError::Other(format!("error: {}", e))
    }
}

// Note: `Terminal::new` returns an `io::Error` on failure in current `tui`.
// If this changes, add a dedicated variant and `From` impl.

/// RAII wrapper around a `Terminal` which restores the terminal state on Drop
/// (leave alternate screen, disable mouse capture, disable raw mode).
pub struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    restored: bool,
}

impl Deref for TerminalGuard {
    type Target = Terminal<CrosstermBackend<Stdout>>;
    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for TerminalGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl TerminalGuard {
    /// Create a new terminal guard. This will enter the alternate screen,
    /// enable mouse capture and enable raw mode. If creation fails, the
    /// terminal is not left in raw mode.
    pub fn new() -> Result<Self, TerminalError> {
        let mut stdout = io::stdout();
        // Enter alternate screen and enable mouse capture (queued then flushed).
        // Also hide the cursor and enable bracketed paste if available.
        queue!(stdout, EnterAlternateScreen, EnableMouseCapture, Hide)
            .map_err(TerminalError::from)?;
        stdout.flush().map_err(TerminalError::from)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).map_err(TerminalError::from)?;
        // Only enable raw mode after Terminal::new succeeds so failures don't leave raw mode enabled.
        enable_raw_mode().map_err(TerminalError::from)?;
        Ok(TerminalGuard {
            terminal,
            restored: false,
        })
    }

    /// Consume the guard and explicitly restore terminal state. This is
    /// equivalent to letting the guard be dropped but returns any IO error.
    pub fn restore(mut self) -> Result<(), TerminalError> {
        if !self.restored {
            // Try to disable raw mode first; ignore errors on subsequent steps but return if raw mode disable fails.
            disable_raw_mode().map_err(TerminalError::from)?;
            // Leave alternate screen and disable mouse capture (queued then flushed).
            queue!(
                self.terminal.backend_mut(),
                DisableMouseCapture,
                LeaveAlternateScreen,
                Show
            )
            .map_err(TerminalError::from)?;
            // flush backend if possible
            if let Err(e) = self.terminal.backend_mut().flush() {
                // best effort: report as Io error
                return Err(e.into());
            }
            self.terminal.show_cursor().map_err(TerminalError::from)?;
            self.restored = true;
        }
        Ok(())
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if self.restored {
            return;
        }
        // Best-effort restore on drop. Errors are ignored here to avoid panics during unwinding.
        let _ = disable_raw_mode();
        let _ = queue!(
            self.terminal.backend_mut(),
            DisableMouseCapture,
            LeaveAlternateScreen,
            Show
        );
        let _ = self.terminal.backend_mut().flush();
        let _ = self.terminal.show_cursor();
        self.restored = true;
    }
}

/// Initialize terminal and return a RAII `TerminalGuard`.
pub fn init_terminal() -> Result<TerminalGuard, TerminalError> {
    TerminalGuard::new()
}

/// Enable mouse capture on an existing terminal instance.
pub fn enable_mouse_capture_on_terminal(terminal: &mut TerminalGuard) -> Result<(), TerminalError> {
    queue!(terminal.backend_mut(), EnableMouseCapture).map_err(TerminalError::from)?;
    terminal
        .backend_mut()
        .flush()
        .map_err(TerminalError::from)?;
    Ok(())
}

/// Disable mouse capture on an existing terminal instance.
pub fn disable_mouse_capture_on_terminal(
    terminal: &mut TerminalGuard,
) -> Result<(), TerminalError> {
    queue!(terminal.backend_mut(), DisableMouseCapture).map_err(TerminalError::from)?;
    terminal
        .backend_mut()
        .flush()
        .map_err(TerminalError::from)?;
    Ok(())
}

/// Restore terminal state (leave alternate screen + disable raw mode) and show cursor.
pub fn restore_terminal(terminal: TerminalGuard) -> Result<(), TerminalError> {
    terminal.restore()
}

/// Best-effort force restore of the terminal state without owning a `TerminalGuard`.
/// This is intended for use from signal handlers or panic hooks where ownership of the
/// application's `TerminalGuard` is not available. It performs the same steps as
/// `TerminalGuard::restore` but uses `stdout` directly and ignores most errors.
pub fn force_restore() {
    // Attempt to disable raw mode first.
    let _ = disable_raw_mode();
    // Try to leave alternate screen, disable mouse capture and show cursor.
    let mut stdout = io::stdout();
    let _ = queue!(stdout, DisableMouseCapture, LeaveAlternateScreen, Show);
    let _ = stdout.flush();
    let _ = crossterm::execute!(io::stdout(), crossterm::cursor::Show);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn force_restore_runs() {
        // Should not panic and should be idempotent.
        force_restore();
        force_restore();
    }
}
