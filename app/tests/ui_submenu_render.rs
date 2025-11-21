use ratatui::backend::TestBackend;
use ratatui::Terminal;
use fileZoom::ui::{UIState, ui_main, Theme};

#[test]
fn submenu_renders_inside_header() {
    let backend = TestBackend::new(80, 4);
    let mut term = Terminal::new(backend).unwrap();

    let mut state = UIState::sample();
    // Select the 'New' top menu (index 3) which has two submenu items
    state.menu_selected = 3;
    state.menu_open = true;
    state.menu_sub_selected = Some(1);

    ui_main::draw_frame(&mut term, &state, &Theme::dark()).unwrap();
}
