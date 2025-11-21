use crate::app::{App, Mode};
use crate::input::KeyCode;
use std::sync::atomic::Ordering;

pub fn handle_progress(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    match code {
        KeyCode::Esc => {
            if let Some(flag) = app.op_cancel_flag.take() {
                flag.store(true, Ordering::SeqCst);
            }
            if let Mode::Progress {
                title,
                processed,
                total,
                message,
                ..
            } = &mut app.mode
            {
                *message = "Cancelling...".to_string();
                app.mode = Mode::Progress {
                    title: title.clone(),
                    processed: *processed,
                    total: *total,
                    message: message.clone(),
                    cancelled: true,
                };
            }
        }
        _ => {}
    }
    Ok(false)
}
