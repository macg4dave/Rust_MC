use fileZoom::ui::widgets::main_menu;
use fileZoom::ui::UIState;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use ratatui::layout::Rect;

#[test]
fn main_menu_renders_smoke() {
    let backend = TestBackend::new(80, 3);
    let mut term = Terminal::new(backend).unwrap();
    let state = UIState::sample();

    term.draw(|f| {
        let area = Rect::new(0, 0, 80, 1);
        main_menu::render(f, area, state.menu_selected, state.menu_focused);
    }).unwrap();
}
