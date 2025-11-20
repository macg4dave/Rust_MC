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

    // If preview is enabled reserve space for a third pane for the preview.
    let main_chunks = if app.preview_visible {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(40),
                Constraint::Percentage(20),
            ]
            .as_ref())
            .split(chunks[2])
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunks[2])
    };

    panels::draw_list(f, main_chunks[0], &app.left, app.active == crate::app::Side::Left);
    panels::draw_list(f, main_chunks[1], &app.right, app.active == crate::app::Side::Right);

    if app.preview_visible {
        // Show preview for the active panel in the third column
        let active_panel = app.active_panel();
        panels::draw_preview(f, main_chunks[2], active_panel);
    }

    // bottom help bar
    let theme = crate::ui::colors::current();
    let help = Paragraph::new("F1:menu  F3:actions/right-click  ?:help  ↑/↓:navigate  PgUp/PgDn:page  Enter:open  Backspace:up  Tab:switch panels  F5:copy  F6:move  d:delete  c:copy(to...)  m:move(to...)  R:rename  n:new file  N:new dir  s:sort  q:quit")
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
        Mode::Progress { title, processed, total, message, .. } => {
            let content = format!("{}/{}\n{}\n\nPress Esc to cancel.", processed, total, message);
            modal::draw_popup(f, f.area(), 40, 20, title, &content);
        }
        Mode::Conflict { path, selected, apply_all } => {
            // Render a compact conflict dialog with a checkbox for "Apply to all"
            let checkbox = if *apply_all { "[x] Apply to all" } else { "[ ] Apply to all" };
            let content = format!("Target exists: {}\n\n{}\n\nChoose an action:", path.display(), checkbox);
            let buttons = ["Overwrite", "Skip", "Cancel"];
            dialogs::draw_confirm(f, f.area(), "Conflict", &content, &buttons, *selected);
        }
        Mode::ContextMenu { title, options, selected, path } => {
            // Reuse the confirm dialog for a small action menu. Convert options to &str slices.
            let btn_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();
            dialogs::draw_confirm(f, f.area(), title, &format!("{}", path.display()), &btn_refs, *selected);
        }
        Mode::Normal => {}
    }
}
