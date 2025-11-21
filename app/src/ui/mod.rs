use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::app::Mode;

pub mod bar_ui;
pub mod colors;
pub mod command_line;
pub mod dialogs;
pub mod header;
pub mod menu;
pub mod modal;
pub mod panels;
pub mod util;
pub mod file_stats_ui;

pub use bar_ui::*;
pub use dialogs::*;
pub use header::*;
pub use menu::*;
pub use modal::*;
pub use panels::*;
pub use file_stats_ui::*;

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
            .constraints(
                [
                    Constraint::Percentage(40),
                    Constraint::Percentage(40),
                    Constraint::Percentage(20),
                ]
                .as_ref(),
            )
            .split(chunks[2])
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunks[2])
    };

    // Use nested vertical layouts for each panel: a small header row and the list below.
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(main_chunks[0]);
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(main_chunks[1]);

    // Draw panel headers (compact path display)
    crate::ui::header::draw_panel_header(f, left_chunks[0], &app.left.cwd.display().to_string());
    crate::ui::header::draw_panel_header(f, right_chunks[0], &app.right.cwd.display().to_string());

    // Draw lists into the remaining area below each header.
    panels::draw_list(
        f,
        left_chunks[1],
        &app.left,
        app.active == crate::app::Side::Left,
    );
    panels::draw_list(
        f,
        right_chunks[1],
        &app.right,
        app.active == crate::app::Side::Right,
    );

    if app.preview_visible {
        // Show preview for the active panel in the third column. Split preview into header + content too.
        let preview_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
            .split(main_chunks[2]);
        let active_panel = app.active_panel();
        crate::ui::header::draw_panel_header(
            f,
            preview_chunks[0],
            &active_panel.cwd.display().to_string(),
        );
        panels::draw_preview(f, preview_chunks[1], active_panel);
    }

    // bottom help bar or command-line if active
    let theme = crate::ui::colors::current();
    if let Some(cl) = &app.command_line {
        if cl.visible {
            crate::ui::command_line::draw_command_line(f, chunks[3], app);
        } else {
            let help = Paragraph::new("F1:menu  F3:actions/right-click  ?:help  ↑/↓:navigate  PgUp/PgDn:page  Enter:open  Backspace:up  Tab:switch panels  F5:copy  F6:move  d:delete  c:copy(to...)  m:move(to...)  R:rename  n:new file  N:new dir  s:sort  q:quit")
                .block(Block::default().borders(Borders::ALL).style(theme.help_block_style));
            f.render_widget(help, chunks[3]);
        }
    } else {
        let help = Paragraph::new("F1:menu  F3:actions/right-click  ?:help  ↑/↓:navigate  PgUp/PgDn:page  Enter:open  Backspace:up  Tab:switch panels  F5:copy  F6:move  d:delete  c:copy(to...)  m:move(to...)  R:rename  n:new file  N:new dir  s:sort  q:quit")
            .block(Block::default().borders(Borders::ALL).style(theme.help_block_style));
        f.render_widget(help, chunks[3]);
    }

    // top header (menu + status combined)
    // Combine the top two chunks (menu + status) into a single header rect
    let header_area = ratatui::layout::Rect::new(
        chunks[0].x,
        chunks[0].y,
        chunks[0].width,
        chunks[0].height + chunks[1].height,
    );
    let sort_order = if app.sort_order == crate::app::types::SortOrder::Descending { "(desc)" } else { "(asc)" };
    let status = format!(
        "Active: {}  |  Sort: {} {}  |  Menu: F1",
        app.active, app.sort, sort_order
    );
    menu::draw_menu(f, header_area, &status, app);

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
            ..
        } => {
            // Render as error if title contains "Error", otherwise info
            let btn_refs: Vec<&str> = buttons.iter().map(|s| s.as_str()).collect();
            if title.to_lowercase().contains("error") {
                dialogs::draw_error(f, f.area(), title, content, &btn_refs, *selected);
            } else {
                dialogs::draw_info(f, f.area(), title, content, &btn_refs, *selected);
            }
        }
        Mode::Progress {
            title,
            processed,
            total,
            message,
            cancelled,
        } => {
            crate::ui::bar_ui::draw_progress_modal(
                f,
                f.area(),
                title,
                *processed,
                *total,
                message,
                *cancelled,
            );
        }
        Mode::Conflict {
            path,
            selected,
            apply_all,
        } => {
            // Render a compact conflict dialog with a checkbox for "Apply to all"
            let checkbox = if *apply_all {
                "[x] Apply to all"
            } else {
                "[ ] Apply to all"
            };
            let content = format!(
                "Target exists: {}\n\n{}\n\nChoose an action:",
                path.display(),
                checkbox
            );
            let buttons = ["Overwrite", "Skip", "Cancel"];
            dialogs::draw_confirm(f, f.area(), "Conflict", &content, &buttons, *selected);
        }
        Mode::ContextMenu {
            title,
            options,
            selected,
            path,
        } => {
            // Reuse the confirm dialog for a small action menu. Convert options to &str slices.
            let btn_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();
            dialogs::draw_confirm(
                f,
                f.area(),
                title,
                &format!("{}", path.display()),
                &btn_refs,
                *selected,
            );
        }
        Mode::Settings { selected } => {
            dialogs::draw_settings(f, f.area(), app, *selected);
        }
        Mode::Normal => {}
    }
}
