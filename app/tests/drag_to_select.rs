use fileZoom::app::App;
use fileZoom::runner::handlers;
use fileZoom::Entry;
use fileZoom::input::mouse::{MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use std::path::PathBuf;

#[test]
fn drag_to_select_left_panel() {
    let cwd = PathBuf::from("/");
    let mut app = App {
        left: fileZoom::app::core::panel::Panel::new(cwd.clone()),
        right: fileZoom::app::core::panel::Panel::new(cwd.clone()),
        active: fileZoom::app::types::Side::Left,
        mode: fileZoom::app::types::Mode::Normal,
        sort: fileZoom::app::types::SortKey::Name,
        sort_desc: false,
        menu_index: 0,
        menu_focused: false,
        preview_visible: false,
        command_line: None,
        settings: fileZoom::app::settings::write_settings::Settings::default(),
        op_progress_rx: None,
        op_cancel_flag: None,
        op_decision_tx: None,
        last_mouse_click_time: None,
        last_mouse_click_pos: None,
        drag_active: false,
        drag_start: None,
        drag_current: None,
        drag_button: None,
    };
    // populate left entries
    app.left.entries = (0..6)
        .map(|i| Entry::directory(format!("f{}", i), PathBuf::from(format!("/f{}", i)), None))
        .collect();
    app.left.selected = 0;
    app.left.clear_selections();

    let term = Rect::new(0, 0, 80, 24);
    // Start drag at the first visible entry (row 4), drag to the third entry (row 6)
    let down = MouseEvent { column: 2, row: 4, kind: MouseEventKind::Down(fileZoom::input::mouse::MouseButton::Left) };
    handlers::handle_mouse(&mut app, down, term).unwrap();
    let drag = MouseEvent { column: 2, row: 6, kind: MouseEventKind::Drag(fileZoom::input::mouse::MouseButton::Left) };
    handlers::handle_mouse(&mut app, drag, term).unwrap();
    let up = MouseEvent { column: 2, row: 6, kind: MouseEventKind::Up(fileZoom::input::mouse::MouseButton::Left) };
    handlers::handle_mouse(&mut app, up, term).unwrap();

    // Expect selections for domain indices 0..=2
    for i in 0..=2usize {
        assert!(app.left.selections.contains(&i), "expected selection to contain {}", i);
    }
}
