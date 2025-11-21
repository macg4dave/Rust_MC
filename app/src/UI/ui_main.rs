use ratatui::{layout::{Constraint, Direction, Layout}, Terminal};
use ratatui::backend::Backend;
use crate::ui::{UIState, Theme};
use ratatui::Frame;
use crate::app::core::App as CoreApp;

/// Draw one frame using the provided Terminal and view model.
pub fn draw_frame<B: Backend>(terminal: &mut Terminal<B>, state: &UIState, theme: &Theme) -> std::io::Result<()> {
    terminal.draw(|f| {
        let size = f.area();
        // menu (1), header (3), main (min), footer (2)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
            .split(size);

        let main = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(chunks[2]);

        crate::ui::widgets::main_menu::render(f, chunks[0], state.menu_selected, state.menu_focused);
        crate::ui::widgets::header::render(f, chunks[1], state, theme);
        crate::ui::widgets::file_list::render(f, main[0], &state.left_list, state.left_selected, theme);
        crate::ui::widgets::file_list::render(f, main[1], &state.right_list, state.right_selected, theme);
        crate::ui::widgets::footer::render(f, chunks[3], state, theme);
    }).map(|_| ())
}

/// Legacy UI entrypoint used by the runner: draw directly into a Frame
pub fn ui(f: &mut Frame, _app: &CoreApp) {
    // Construct a tiny UIState view-model for scaffold draws.
    let state = UIState::sample();
    let theme = Theme::dark();

    let size = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
        .split(size);
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(chunks[2]);

    crate::ui::widgets::main_menu::render(f, chunks[0], state.menu_selected, state.menu_focused);
    crate::ui::widgets::header::render(f, chunks[1], &state, &theme);
    crate::ui::widgets::file_list::render(f, main[0], &state.left_list, state.left_selected, &theme);
    crate::ui::widgets::file_list::render(f, main[1], &state.right_list, state.right_selected, &theme);
    crate::ui::widgets::footer::render(f, chunks[3], &state, &theme);
}
