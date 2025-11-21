/// Small manual demo runner for UI scaffolding.
/// This is not used by automated tests — run locally to preview the layout.
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};
use crate::ui::{Theme, UIState};
use crate::ui::ui_main::draw_frame;

pub fn run_demo() -> anyhow::Result<()> {
    // Lightweight static example — use Terminal in alternate screen to draw once.
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let state = UIState::sample();
    let theme = Theme::dark();
    let _ = draw_frame(&mut terminal, &state, &theme);
    Ok(())
}

#[cfg(test)]
mod tests { use super::*; #[test] fn demo_runs_ok() { let _ = run_demo(); } }
