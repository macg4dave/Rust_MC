use crate::app::App;
use crate::input::{poll, read_event, InputEvent};
use crate::runner::handlers;
use crate::runner::terminal::{init_terminal, restore_terminal};
use crate::ui;

use std::time::Duration;

pub fn run_app() -> anyhow::Result<()> {
    let mut terminal = init_terminal()?;

    // Initialize app using the current working directory.
    let mut app = App::new()?;
    // Load persisted settings from disk if available and apply.
    if let Ok(s) = crate::app::settings::load_settings() {
        app.settings = s;
    }

    // Track current mouse capture state so we can toggle it at runtime when
    // user changes the `mouse_enabled` setting in the UI.
    let mut mouse_capture_enabled = app.settings.mouse_enabled;
    // If settings requested mouse disabled, turn off capture now (init enabled it).
    if !mouse_capture_enabled {
        let _ = crate::runner::terminal::disable_mouse_capture_on_terminal(&mut terminal);
    }

    // Main event loop
    loop {
        terminal.draw(|f| ui::ui(f, &app))?;

        // Precompute page size for navigation handlers.
        let page_size = (terminal.size()?.height as usize).saturating_sub(4);

        if poll(Duration::from_millis(100))? {
            let iev = read_event()?;
            match iev {
                InputEvent::Key(key) => {
                    let code = key.code;
                    // Delegate key handling to the refactored handlers module.
                    if handlers::handle_key(&mut app, code, page_size)? {
                        break;
                    }
                }
                InputEvent::Mouse(me) => {
                    // dispatch mouse events to the handlers module which will
                    // map coordinates to UI areas using the terminal size
                    let ts = terminal.size()?;
                    let term_rect = ratatui::layout::Rect::new(0, 0, ts.width, ts.height);
                    handlers::handle_mouse(&mut app, me, term_rect)?;
                }
                InputEvent::Resize(_, _) => { /* redraw on next loop */ }
                InputEvent::Other => {}
            }
            // If the user toggled the mouse setting in handlers, reflect this
            // by enabling/disabling mouse capture on the terminal instance.
            if app.settings.mouse_enabled != mouse_capture_enabled {
                mouse_capture_enabled = app.settings.mouse_enabled;
                if mouse_capture_enabled {
                    let _ = crate::runner::terminal::enable_mouse_capture_on_terminal(&mut terminal);
                } else {
                    let _ = crate::runner::terminal::disable_mouse_capture_on_terminal(&mut terminal);
                }
            }
        }
    }

    // Restore terminal state before exiting.
    restore_terminal(terminal)?;
    Ok(())
}
