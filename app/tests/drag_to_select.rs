use fileZoom::app::App;
use fileZoom::input::mouse::{MouseEvent, MouseEventKind};
use fileZoom::runner::handlers;
use fileZoom::Entry;
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
        sort_order: fileZoom::app::types::SortOrder::Ascending,
        menu_index: 0,
        menu_focused: false,
        menu_state: fileZoom::ui::menu_model::MenuState::default(),
            preview_visible: false,
            file_stats_visible: false,
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
        // Start drag at the first visible entry (compute dynamically to account for layout)
        let header_count = 1usize;
        let parent_count = if app.left.cwd.parent().is_some() { 1usize } else { 0usize };
        let first_domain_row = 4 + 1 + (header_count + parent_count) as u16;

        let down = MouseEvent {
            column: 2,
            row: first_domain_row,
        kind: MouseEventKind::Down(fileZoom::input::mouse::MouseButton::Left),
    };
    handlers::handle_mouse(&mut app, down, term).unwrap();
        let drag = MouseEvent {
            column: 2,
            row: first_domain_row + 2,
        kind: MouseEventKind::Drag(fileZoom::input::mouse::MouseButton::Left),
    };
    handlers::handle_mouse(&mut app, drag, term).unwrap();
    let up = MouseEvent {
        column: 2,
        row: first_domain_row + 2,
        kind: MouseEventKind::Up(fileZoom::input::mouse::MouseButton::Left),
    };
    handlers::handle_mouse(&mut app, up, term).unwrap();

    // Expect selections for domain indices 0..=2
    for i in 0..=2usize {
        assert!(
            app.left.selections.contains(&i),
            "expected selection to contain {}",
            i
        );
    }
}
