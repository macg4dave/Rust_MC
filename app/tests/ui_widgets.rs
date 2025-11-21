use fileZoom::ui::{UIState, Theme};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use ratatui::layout::Rect;

#[test]
fn header_and_panels_render() {
    let backend = TestBackend::new(80, 10);
    let mut t = Terminal::new(backend).unwrap();
    let state = UIState::sample();
    let theme = Theme::dark();

    t.draw(|f| {
        let area = Rect::new(0, 0, 80, 3);
        fileZoom::ui::widgets::header::render(f, area, &state, &theme);
    }).unwrap();

    t.draw(|f| {
        let area = Rect::new(0, 3, 40, 6);
        fileZoom::ui::widgets::file_list::render(f, area, &state.left_list, state.left_selected, &theme);
    }).unwrap();
}
