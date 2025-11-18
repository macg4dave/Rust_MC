use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, Paragraph};
use tui::Frame;

use crate::app::App;
use crate::app::Mode;

pub mod menu;
pub mod modal;
pub mod panels;
pub mod header;
pub mod colors;

pub use menu::*;
pub use modal::*;
pub use panels::*;
pub use header::*;

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    panels::draw_list(
        f,
        main_chunks[0],
        &app.left,
        app.active == crate::app::Side::Left,
    );
    panels::draw_list(
        f,
        main_chunks[1],
        &app.right,
        app.active == crate::app::Side::Right,
    );

    // bottom help bar
    let theme = crate::ui::colors::current();
    let help = Paragraph::new("↑/↓:navigate  PgUp/PgDn:page  Enter:open  Backspace:up  d:delete  c:copy  m:move  R:rename  n:new file  N:new dir  s:sort  q:quit")
        .block(Block::default().borders(Borders::ALL).style(theme.help_block_style));
    f.render_widget(help, chunks[2]);

    // top menu bar
    menu::draw_menu(f, chunks[0], app);

    // Modal
    match &app.mode {
        Mode::Confirm { msg, .. } => modal::draw_modal(f, f.size(), "Confirm", msg),
        Mode::Input { prompt, buffer, .. } => modal::draw_modal(f, f.size(), prompt, buffer),
        Mode::Normal => {}
    }
}
