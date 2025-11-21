use ratatui::{layout::{Constraint, Direction, Layout}, Terminal};
use ratatui::backend::Backend;
use crate::ui::{UIState, Theme};
use ratatui::Frame;
use crate::app::core::App as CoreApp;

/// Draw one frame using the provided Terminal and view model.
pub fn draw_frame<B: Backend>(terminal: &mut Terminal<B>, state: &UIState, theme: &Theme) -> std::io::Result<()> {
    terminal.draw(|f| {
        let size = f.area();
        // menu (min 1, ideally 3), header (3), main (min), footer (2)
        // The main menu uses a bordered Paragraph which needs vertical
        // space for a top border, a content row, and a bottom border.
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            // Make the top menu a flexible area (min 1) so very small
            // terminals still render the menu content line even when total
            // available height is low. The bordered rendering is used only
            // when area.height >= 3.
            .constraints([Constraint::Min(1), Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
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
pub fn ui(f: &mut Frame, app: &CoreApp) {
    // Build a UIState view-model from the live Core App so the runner
    // reflects the real runtime state (menu focus, selected index, preview, etc.).
    let state = UIState::from_core(app);

    // Choose a reasonable Theme matching the app settings string so
    // `draw_frame` can render headers/file lists consistently with the
    // configured theme. Default to dark if an unknown value is present.
    let theme = match app.settings.theme.as_str() {
        "light" => Theme::light(),
        _ => Theme::dark(),
    };

    let size = f.area();
    // Make the top menu flexible so tiny terminals still get a content row.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
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
