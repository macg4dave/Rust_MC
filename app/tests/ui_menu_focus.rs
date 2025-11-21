use ratatui::backend::TestBackend;
use ratatui::Terminal;
use fileZoom::ui::{UIState, Theme, ui_main};

#[test]
fn main_menu_focus_and_selection_render() {
    let backend = TestBackend::new(80, 4);
    let mut term = Terminal::new(backend).unwrap();

    let mut state = UIState::sample();
    state.menu_selected = 1;
    state.menu_focused = true;
    ui_main::draw_frame(&mut term, &state, &Theme::dark()).unwrap();

    // Toggle focus and selection and draw again to exercise both code paths
    state.menu_focused = false;
    state.menu_selected = 4;
    ui_main::draw_frame(&mut term, &state, &Theme::dark()).unwrap();
}
