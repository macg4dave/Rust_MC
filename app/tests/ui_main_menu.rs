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
        // Give the main menu area a little vertical space so borders and
        // content can both be rendered. Using height 3 yields a top
        // border, the content line, and a bottom border.
        let area = Rect::new(0, 0, 80, 3);
        main_menu::render(f, area, state.menu_selected, state.menu_focused);
    }).unwrap();

    // Inspect the backend buffer to ensure the menu label is visible
    // in the rendered top row (the menu content should include "[File]").
    let buf = term.backend_mut().buffer();
    // The content line is the second row (index 1) when bordered block
    // with height 3 is rendered.
    let width = buf.area().width as u16;
    let mut content_row = String::new();
    for x in 0..width {
        if let Some(c) = buf.cell((x, 1)) { content_row.push_str(c.symbol()); }
    }
    assert!(content_row.contains("[File]"), "menu content row did not contain expected label: {}", content_row);
}
