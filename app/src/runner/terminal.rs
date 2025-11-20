use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::fmt;
use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;

/// Errors returned by terminal initialization/restore helpers.
#[derive(Debug)]
pub enum TerminalError {
    Io(io::Error),
}

impl fmt::Display for TerminalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TerminalError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for TerminalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TerminalError::Io(e) => Some(e),
        }
    }
}

impl From<io::Error> for TerminalError {
    fn from(e: io::Error) -> Self {
        TerminalError::Io(e)
    }
}

// Note: `Terminal::new` returns an `io::Error` on failure in current `tui`.
// If this changes, add a dedicated variant and `From` impl.

/// Initialize terminal (enter alternate screen + enable raw mode) and return a TUI Terminal.
pub fn init_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, TerminalError> {
    enable_raw_mode().map_err(TerminalError::Io)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(TerminalError::Io)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend).map_err(TerminalError::Io)?;
    Ok(terminal)
}

/// Restore terminal state (leave alternate screen + disable raw mode) and show cursor.
pub fn restore_terminal(
    mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<(), TerminalError> {
    disable_raw_mode().map_err(TerminalError::Io)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(TerminalError::Io)?;
    terminal.show_cursor().map_err(TerminalError::Io)?;
    Ok(())
}
