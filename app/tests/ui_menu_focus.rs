use ratatui::backend::TestBackend;
use ratatui::Terminal;
use fileZoom::ui::UIState;
use fileZoom::ui::widgets::main_menu;

#[test]
fn main_menu_focus_and_selection_render() {
    let backend = TestBackend::new(80, 4);
    let mut term = Terminal::new(backend).unwrap();

    let mut state = UIState::sample();
    state.menu_selected = 1;
    state.menu_focused = true;
    // Render the main menu directly with enough height so content is visible.
    term.draw(|f| {
        let area = ratatui::layout::Rect::new(0, 0, 80, 3);
        main_menu::render(f, area, state.menu_selected, state.menu_focused);
    }).unwrap();

    // After initial draw the buffer should include the active (bracketed)
    // label for the selected index (1 => "Copy"). Search the entire
    // backend display for better robustness across small heights.
    let view = format!("{}", term.backend_mut());
    assert!(view.contains("[Copy]"), "expected backend to include [Copy], got:\n{}", view);

    // Toggle focus and selection and draw again to exercise both code paths
    state.menu_focused = false;
    state.menu_selected = 4;
    term.draw(|f| {
        let area = ratatui::layout::Rect::new(0, 0, 80, 3);
        main_menu::render(f, area, state.menu_selected, state.menu_focused);
    }).unwrap();

    // Verify the new active label is present (index 4 => "Sort")
    let view2 = format!("{}", term.backend_mut());
    assert!(view2.contains("[Sort]"), "expected backend to include [Sort], got:\n{}", view2);
}
