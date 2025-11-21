use crate::app::App;
use crate::input::{poll, read_event, InputEvent, MouseEvent, KeyCode};
use crate::runner::handlers;
use crate::runner::terminal::{restore_terminal, TerminalGuard};
use std::sync::mpsc::Receiver;
use crate::ui;
use std::time::Duration;
// path types are referenced behind feature gates where needed

#[cfg(feature = "fs-watch")]
use std::sync::mpsc::channel as mpsc_channel;

/// Map a filesystem watcher [`FsEvent`] to the set of `Side` values that
/// should be refreshed.
///
/// This function inspects the provided filesystem event and determines
/// whether it affects the left panel directory, the right panel
/// directory, or both. The returned vector is deduplicated and sorted
/// in canonical order (Left then Right) so callers can rely on a stable
/// iteration order when refreshing panels.
///
/// # Parameters
/// - `evt`: filesystem event emitted by the watcher.
/// - `left`: path of the left panel current working directory.
/// - `right`: path of the right panel current working directory.
///
/// # Returns
/// A `Vec<crate::app::Side>` containing zero, one, or both sides that need
/// to be refreshed.
///
/// # Note
/// This helper is feature-gated behind `fs-watch` so the crate does not
/// take a hard dependency on the watcher types when that feature is
/// disabled. Keeping it small and pure makes it easy to unit-test.
#[cfg(feature = "fs-watch")]
use crate::runner::watch_helpers::affected_sides_from_fs_event;

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
    // user changes the `mouse_enabled` setting in the UI. Use a small enum
    // for clearer intent instead of a raw boolean.
    /// Represents whether terminal mouse capture is currently enabled.
    ///
    /// This lightweight enum replaces a raw `bool` to make intent clearer
    /// when toggling terminal mouse capture at runtime. A `From<bool>`
    /// implementation and an `as_bool()` accessor are provided for
    /// ergonomic conversion to/from existing boolean settings.
    enum MouseCapture {
        Enabled,
        Disabled,
    }
    impl From<bool> for MouseCapture {
        fn from(b: bool) -> Self { if b { MouseCapture::Enabled } else { MouseCapture::Disabled } }
    }
    impl MouseCapture {
        fn as_bool(&self) -> bool { matches!(self, MouseCapture::Enabled) }
    }

    let mut mouse_capture = MouseCapture::from(app.settings.mouse_enabled);
    if !mouse_capture.as_bool() {
        let _ = crate::runner::terminal::disable_mouse_capture_on_terminal(&mut terminal);
    }

    // Spawn filesystem watchers for the initial panel directories when the
    // `fs-watch` feature is enabled. Watchers send `FsEvent` into the
    // receiver, and the event loop will refresh affected panels.
    #[cfg(feature = "fs-watch")]
    let (fs_tx, fs_rx) = mpsc_channel::<crate::fs_op::watcher::FsEvent>();
    #[cfg(feature = "fs-watch")]
    // Manage watcher join handles and stop senders per side so we can restart
    // watchers when the panel cwd changes during runtime.
    #[allow(unused_assignments)]
    let mut left_watcher: Option<(std::thread::JoinHandle<()>, std::sync::mpsc::Sender<()>)> = None;
    #[cfg(feature = "fs-watch")]
    #[allow(unused_assignments)]
    let mut right_watcher: Option<(std::thread::JoinHandle<()>, std::sync::mpsc::Sender<()>)> = None;
    #[cfg(feature = "fs-watch")]
    {
        let left_path = app.left.cwd.clone();
        let right_path = app.right.cwd.clone();
        let tx_left = fs_tx.clone();
        let tx_right = fs_tx.clone();
        // Left
        let (stop_tx_left, stop_rx_left) = std::sync::mpsc::channel::<()>();
        let h_left = crate::fs_op::watcher::spawn_watcher(left_path, tx_left, stop_rx_left);
        left_watcher = Some((h_left, stop_tx_left));
        // Right
        let (stop_tx_right, stop_rx_right) = std::sync::mpsc::channel::<()>();
        let h_right = crate::fs_op::watcher::spawn_watcher(right_path, tx_right, stop_rx_right);
        right_watcher = Some((h_right, stop_tx_right));
    }
    

    #[cfg(feature = "fs-watch")]
    let mut prev_left = app.left.cwd.clone();
    #[cfg(feature = "fs-watch")]
    let mut prev_right = app.right.cwd.clone();

    // Main event loop
    loop {
        // If watcher signalled a filesystem event, trigger a refresh and redraw.
        #[cfg(feature = "fs-watch")]
        if let Ok(evt) = fs_rx.try_recv() {
            let affected = affected_sides_from_fs_event(&evt, &app.left.cwd, &app.right.cwd);
            for side in affected {
                let _ = app.refresh_side(side);
            }
        }

        // If panel cwd changed since last loop, restart the corresponding watcher
        #[cfg(feature = "fs-watch")]
        {
            if app.left.cwd != prev_left {
                // stop previous left watcher
                if let Some((h, stop_tx)) = left_watcher.take() {
                    let _ = stop_tx.send(());
                    let _ = h.join();
                }
                // start new left watcher
                let (stop_tx_left, stop_rx_left) = std::sync::mpsc::channel::<()>();
                let tx_left = fs_tx.clone();
                let h_left = crate::fs_op::watcher::spawn_watcher(app.left.cwd.clone(), tx_left, stop_rx_left);
                left_watcher = Some((h_left, stop_tx_left));
                prev_left = app.left.cwd.clone();
            }
            if app.right.cwd != prev_right {
                if let Some((h, stop_tx)) = right_watcher.take() {
                    let _ = stop_tx.send(());
                    let _ = h.join();
                }
                let (stop_tx_right, stop_rx_right) = std::sync::mpsc::channel::<()>();
                let tx_right = fs_tx.clone();
                let h_right = crate::fs_op::watcher::spawn_watcher(app.right.cwd.clone(), tx_right, stop_rx_right);
                right_watcher = Some((h_right, stop_tx_right));
                prev_right = app.right.cwd.clone();
            }
        }
        // If a shutdown signal has been received (e.g. ctrl-c), break so
        // we can restore the terminal cleanly in the outer scope.
        if shutdown_rx.try_recv().is_ok() {
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
                // Removed unused alias for MouseEvent
                // use crate::input::MouseEvent as AppMouseEvent;

            let mut key_events: Vec<KeyCode> = Vec::new();
            let mut other_mouse: Vec<MouseEvent> = Vec::new();
            let mut last_mouse_move: Option<MouseEvent> = None;
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
            if app.settings.mouse_enabled != mouse_capture.as_bool() {
                mouse_capture = MouseCapture::from(app.settings.mouse_enabled);
                if mouse_capture.as_bool() {
                    let _ = crate::runner::terminal::enable_mouse_capture_on_terminal(&mut terminal);
                } else {
                    let _ = crate::runner::terminal::disable_mouse_capture_on_terminal(&mut terminal);
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

#[cfg(all(test, feature = "fs-watch"))]
mod tests {
    use super::affected_sides_from_fs_event;
    use crate::fs_op::watcher::FsEvent;
    use crate::app::Side;
    use std::path::PathBuf;

    #[test]
    fn affected_sides_create_left() {
        let left = std::path::Path::new("/tmp/left");
        let right = std::path::Path::new("/tmp/right");
        let ev = FsEvent::Create(PathBuf::from("/tmp/left/file.txt"));
        let sides = affected_sides_from_fs_event(&ev, left, right);
        assert_eq!(sides, vec![Side::Left]);
    }

    #[test]
    fn affected_sides_rename_both() {
        let left = std::path::Path::new("/tmp/left");
        let right = std::path::Path::new("/tmp/right");
        let ev = FsEvent::Rename(PathBuf::from("/tmp/left/a"), PathBuf::from("/tmp/right/b"));
        let mut sides = affected_sides_from_fs_event(&ev, left, right);
        sides.sort_by_key(|s| match s { Side::Left => 0, Side::Right => 1 });
        assert_eq!(sides, vec![Side::Left, Side::Right]);
    }
}
