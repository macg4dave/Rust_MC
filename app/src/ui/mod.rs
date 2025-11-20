use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::app::Mode;

pub mod colors;
pub mod dialogs;
pub mod header;
pub mod menu;
pub mod modal;
pub mod panels;
pub mod util;

pub use dialogs::*;
pub use header::*;
pub use menu::*;
pub use modal::*;
pub use panels::*;

pub fn ui(f: &mut Frame, app: &App) {
    // Top menu (1), status (1), main panes (min), bottom help (1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(f.area());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[2]);

    panels::draw_list(f, main_chunks[0], &app.left, app.active == crate::app::Side::Left);
    panels::draw_list(f, main_chunks[1], &app.right, app.active == crate::app::Side::Right);

    // bottom help bar
    let theme = crate::ui::colors::current();
    let help = Paragraph::new("F1:menu  ?:help  ↑/↓:navigate  PgUp/PgDn:page  Enter:open  Backspace:up  d:delete  c:copy  m:move  R:rename  n:new file  N:new dir  s:sort  q:quit")
        .block(Block::default().borders(Borders::ALL).style(theme.help_block_style));
    f.render_widget(help, chunks[3]);

    // top menu bar
    menu::draw_menu(f, chunks[0], app);

    // status bar under top menu
    let sort_order = if app.sort_desc { "(desc)" } else { "(asc)" };
    let status = format!(
        "Active: {}  |  Sort: {} {}  |  Menu: F1",
        app.active, app.sort, sort_order
    );
    let status_p = Paragraph::new(status)
        .block(Block::default().borders(Borders::BOTTOM).style(theme.help_block_style));
    f.render_widget(status_p, chunks[1]);

    // Modal
    match &app.mode {
        Mode::Confirm { msg, selected, .. } => {
            dialogs::draw_confirm(f, f.area(), "Confirm", msg, &["Yes", "No"], *selected)
        }
        Mode::Input { prompt, buffer, .. } => modal::draw_modal(f, f.area(), prompt, buffer),
        Mode::Message {
            title,
            content,
            buttons,
            selected,
        } => {
            // Render as error if title contains "Error", otherwise info
            let btn_refs: Vec<&str> = buttons.iter().map(|s| s.as_str()).collect();
            if title.to_lowercase().contains("error") {
                dialogs::draw_error(f, f.area(), title, content, &btn_refs, *selected);
            } else {
                dialogs::draw_info(f, f.area(), title, content, &btn_refs, *selected);
            }
        }
        Mode::Normal => {}
    }
}
