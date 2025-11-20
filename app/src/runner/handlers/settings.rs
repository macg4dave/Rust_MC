use crate::app::Mode;
use crate::input::KeyCode;

use crate::app::App;

/// Handle keys while in the Settings modal.
pub fn handle_settings(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    // Selected indices: 0 = mouse_enabled, 1 = double_click_ms, 2 = Save, 3 = Cancel
    if let Mode::Settings { selected } = &mut app.mode {
        match code {
            KeyCode::Esc => { app.mode = Mode::Normal; }
            KeyCode::Up => { if *selected > 0 { *selected -= 1 } else { *selected = 3 } }
            KeyCode::Down => { *selected = (*selected + 1) % 4 }
            KeyCode::Left => {
                if *selected == 1 {
                    let cur = &mut app.settings.mouse_double_click_ms;
                    if *cur > 100 { *cur = cur.saturating_sub(50) }
                }
            }
            KeyCode::Right => {
                if *selected == 1 {
                    let cur = &mut app.settings.mouse_double_click_ms;
                    *cur = (*cur + 50).min(5000);
                }
            }
            KeyCode::Char('+') => {
                if *selected == 1 {
                    let cur = &mut app.settings.mouse_double_click_ms;
                    *cur = (*cur + 50).min(5000);
                }
            }
            KeyCode::Char('-') => {
                if *selected == 1 {
                    let cur = &mut app.settings.mouse_double_click_ms;
                    if *cur > 100 { *cur = cur.saturating_sub(50) }
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                match *selected {
                    0 => { app.settings.mouse_enabled = !app.settings.mouse_enabled; }
                    1 => { /* noop on enter for numeric field */ }
                    2 => {
                        // Save settings
                        match crate::app::settings::save_settings(&app.settings) {
                            Ok(_) => {
                                app.mode = Mode::Message { title: "Settings Saved".to_string(), content: "Settings persisted".to_string(), buttons: vec!["OK".to_string()], selected: 0, actions: None };
                            }
                            Err(e) => {
                                app.mode = Mode::Message { title: "Error".to_string(), content: format!("Failed to save settings: {}", e), buttons: vec!["OK".to_string()], selected: 0, actions: None };
                            }
                        }
                    }
                    3 => { app.mode = Mode::Normal; }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    Ok(false)
}
