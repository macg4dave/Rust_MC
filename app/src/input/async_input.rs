use anyhow::Result;
use crossterm::event::Event;
use crossterm::event::EventStream;
use futures_util::stream::StreamExt;

/// Async event listener using `crossterm::EventStream`.
///
/// Example usage (requires `--features async-input` and an async runtime):
///
/// ```ignore
/// fileZoom::input::async_input::event_listener(|ev| {
///     // handle event synchronously inside closure
/// });
/// ```
pub async fn event_listener<F>(mut on_event: F) -> Result<()>
where
    F: FnMut(Event) + Send + 'static,
{
    let mut stream = EventStream::new();
    while let Some(ev) = stream.next().await {
        match ev {
            Ok(event) => on_event(event),
            Err(_e) => {
                // best-effort: errors from the event stream are ignored here
                // but callers could be updated to receive an error callback.
            }
        }
    }
    Ok(())
}
