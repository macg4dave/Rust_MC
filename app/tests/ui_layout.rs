use fileZoom::ui::{draw_frame, UIState, Theme};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn draw_frame_smoke_test() {
    let backend = TestBackend::new(80, 24);
    let mut term = Terminal::new(backend).unwrap();
    let state = UIState::sample();
    let theme = Theme::dark();

    // Ensure draw_frame completes without panic
    draw_frame(&mut term, &state, &theme).expect("draw failed");
}
