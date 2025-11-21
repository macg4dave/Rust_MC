use crate::app::App;
use crate::input::{poll, read_event, InputEvent};
use crate::runner::handlers;
use crate::runner::terminal::{restore_terminal, TerminalGuard};
use std::sync::mpsc::Receiver;
use crate::ui;

use std::time::Duration;

pub fn run_app(
    mut terminal: TerminalGuard,
    shutdown_rx: Receiver<()>,
    start_opts: crate::app::StartOptions,
) -> anyhow::Result<()> {

    // Initialize app using provided start options (may include a start
    // directory or initial mouse setting).
    let mut app = App::with_options(&start_opts)?;
    // Load persisted settings from disk if available and apply.
    if let Ok(s) = crate::app::settings::load_settings() {
        app.settings = s;
    }

    // Re-apply CLI-provided startup overrides (CLI should win over persisted settings).
    if let Some(m) = start_opts.mouse_enabled {
        app.settings.mouse_enabled = m;
    }
    if let Some(s) = start_opts.show_hidden {
        app.settings.show_hidden = s;
    }
    if let Some(ref theme) = start_opts.theme {
        app.settings.theme = theme.clone();
        crate::ui::colors::set_theme(theme.as_str());
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
        // If a shutdown signal has been received (e.g. ctrl-c), break so
        // we can restore the terminal cleanly in the outer scope.
        if let Ok(_) = shutdown_rx.try_recv() {
            break;
        }

        // Draw once at the top of the loop. Resize events will also trigger
        // an immediate redraw below when detected in the aggregated events.
        terminal.draw(|f| ui::ui(f, &app))?;

        // Precompute page size for navigation handlers.
        let page_size = (terminal.size()?.height as usize).saturating_sub(4);

        // Poll for any input for up to 100ms. Use `poll` to avoid blocking
        // indefinitely and to allow aggregation of bursts of events.
        if poll(Duration::from_millis(100))? {
            // Collect one or more available events. After the first event
            // arrives, poll briefly to coalesce follow-up events (e.g. many
            // Mouse::Moved events) so we can debounce them.
            let mut events = Vec::new();
            // Read first event, logging any transient errors and skipping them
            // so the loop can continue and the RAII guard will restore if
            // an error forces early return later.
            match read_event() {
                Ok(ev) => events.push(ev),
                Err(e) => {
                    tracing::error!("failed to read input event: {:#}", e);
                }
            }

            // Short window to collect additional immediate events. Errors from
            // `read_event` are logged and skipped so the application remains
            // resilient to transient input errors.
            while poll(Duration::from_millis(5))? {
                match read_event() {
                    Ok(ev) => events.push(ev),
                    Err(e) => tracing::error!("failed to read input event: {:#}", e),
                }
            }

            // Safety: avoid unbounded growth if input is being flooded.
            const MAX_EVENTS: usize = 1024;
            if events.len() > MAX_EVENTS {
                events.truncate(MAX_EVENTS);
            }

            // Coalesce collected events:
            // - keep all key events (processed in order)
            // - keep non-move mouse events in order
            // - coalesce multiple Mouse::Moved into the last one
            // - remember last resize and trigger an immediate redraw
            use crate::input::MouseEvent as AppMouseEvent;

            let mut key_events = Vec::new();
            let mut other_mouse = Vec::new();
            let mut last_mouse_move: Option<AppMouseEvent> = None;
            let mut last_resize: Option<(u16, u16)> = None;

            for ev in events {
                match ev {
                    InputEvent::Key(k) => key_events.push(k),
                    InputEvent::Mouse(m) => {
                        // `m` is the crate-local MouseEvent; coalesce Move kinds.
                        use crate::input::MouseEventKind as AppMouseKind;
                        match m.kind {
                            AppMouseKind::Move => last_mouse_move = Some(m),
                            _ => other_mouse.push(m),
                        }
                    }
                    InputEvent::Resize(w, h) => last_resize = Some((w, h)),
                    InputEvent::Other => {}
                }
            }

            // Process key events in order. Keys may cause the app to request
            // exit; honor that.
            // Track whether handlers requested exit so we can break the outer loop
            // and run the normal restore path once.
            let mut should_exit = false;
            for code in key_events {
                if handlers::handle_key(&mut app, code, page_size)? {
                    should_exit = true;
                    break;
                }
            }

            // Process non-move mouse events in order.
            if !other_mouse.is_empty() {
                let ts = terminal.size()?;
                let term_rect = ratatui::layout::Rect::new(0, 0, ts.width, ts.height);
                for m in other_mouse {
                    handlers::handle_mouse(&mut app, m, term_rect)?;
                }
            }

            // Process a single, coalesced mouse-move event (if any).
            if let Some(m) = last_mouse_move {
                let ts = terminal.size()?;
                let term_rect = ratatui::layout::Rect::new(0, 0, ts.width, ts.height);
                handlers::handle_mouse(&mut app, m, term_rect)?;
            }

            // If resize occurred in the burst, trigger an immediate redraw so
            // `ratatui` can update layout before the next loop iteration.
            if let Some((_w, _h)) = last_resize {
                terminal.draw(|f| ui::ui(f, &app))?;
            }

            // If the user toggled the mouse setting in handlers, reflect this
            // by enabling/disabling mouse capture on the terminal instance.
            if app.settings.mouse_enabled != mouse_capture_enabled {
                mouse_capture_enabled = app.settings.mouse_enabled;
                if mouse_capture_enabled {
                    let _ =
                        crate::runner::terminal::enable_mouse_capture_on_terminal(&mut terminal);
                } else {
                    let _ =
                        crate::runner::terminal::disable_mouse_capture_on_terminal(&mut terminal);
                }
            }
            if should_exit {
                break;
            }
        }
    }

    // Restore terminal state before exiting.
    restore_terminal(terminal)?;
    Ok(())
}
