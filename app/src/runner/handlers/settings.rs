use crate::app::Mode;
use crate::input::KeyCode;
use crate::app::settings::keybinds;

use crate::app::App;

/// Handle keys while in the Settings modal.
pub fn handle_settings(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    // Selected indices: 0 = mouse_enabled, 1 = double_click_ms, 2 = Save, 3 = Cancel
    if let Mode::Settings { selected } = &mut app.mode {
        if keybinds::is_esc(&code) {
            app.mode = Mode::Normal;
        } else if keybinds::is_up(&code) {
            if *selected > 0 {
                *selected -= 1
            } else {
                *selected = 3
            }
        } else if keybinds::is_down(&code) {
            *selected = (*selected + 1) % 4
        } else if keybinds::is_left(&code) {
            if *selected == 1 {
                let cur = &mut app.settings.mouse_double_click_ms;
                if *cur > 100 {
                    *cur = cur.saturating_sub(50)
                }
            }
        } else if keybinds::is_right(&code) {
            if *selected == 1 {
                let cur = &mut app.settings.mouse_double_click_ms;
                *cur = (*cur + 50).min(5000);
            }
        } else if keybinds::is_char(&code, '+') {
            if *selected == 1 {
                let cur = &mut app.settings.mouse_double_click_ms;
                *cur = (*cur + 50).min(5000);
            }
        } else if keybinds::is_char(&code, '-') {
            if *selected == 1 {
                let cur = &mut app.settings.mouse_double_click_ms;
                if *cur > 100 {
                    *cur = cur.saturating_sub(50)
                }
            }
        } else if keybinds::is_enter(&code) || keybinds::is_toggle_selection(&code) {
            match *selected {
                0 => {
                    app.settings.mouse_enabled = !app.settings.mouse_enabled;
                }
                1 => { /* noop on enter for numeric field */ }
                2 => {
                    // Save settings
                    match crate::app::settings::save_settings(&app.settings) {
                        Ok(_) => {
                            app.mode = Mode::Message {
                                title: "Settings Saved".to_string(),
                                content: "Settings persisted".to_string(),
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                                actions: None,
                            };
                        }
                        Err(e) => {
                            app.mode = Mode::Message {
                                title: "Error".to_string(),
                                content: format!("Failed to save settings: {}", e),
                                buttons: vec!["OK".to_string()],
                                selected: 0,
                                actions: None,
                            };
                        }
                    }
                }
                3 => {
                    app.mode = Mode::Normal;
                }
                _ => {}
            }
        }
    }
    Ok(false)
}
