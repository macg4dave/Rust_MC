use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Block, Borders};
use ratatui::Frame;
use crate::input::KeyCode;
use crate::runner::commands;
use crate::app::types::Mode;

/// Lightweight command-line state stored on App when active.
#[derive(Clone, Debug)]
pub struct CommandLineState {
    pub visible: bool,
    pub buffer: String,
    pub cursor: usize,
}

impl CommandLineState {
    pub fn new() -> Self {
        CommandLineState { visible: false, buffer: String::new(), cursor: 0 }
    }
}

/// Draw the command line into the provided area.
pub fn draw_command_line(f: &mut Frame, area: Rect, app: &crate::app::core::App) {
    if let Some(cl) = &app.command_line {
        if cl.visible {
            let txt = format!(":{}", cl.buffer);
            let p = Paragraph::new(txt).block(Block::default().borders(Borders::ALL).title("Command"));
            f.render_widget(p, area);
        }
    }
}

/// Handle key input destined for the command line. Return Ok(true) if key consumed.
pub fn handle_input(app: &mut crate::app::core::App, key: KeyCode) -> anyhow::Result<bool> {
    if app.command_line.is_some() {
        // take ownership of the command-line state so we can mutate/consume it
        let mut cl = app.command_line.take().unwrap();
        match key {
            KeyCode::Char(c) => {
                cl.buffer.push(c);
                cl.cursor = cl.buffer.len();
                // put back
                app.command_line = Some(cl);
                return Ok(true);
            }
            KeyCode::Backspace => {
                if cl.cursor > 0 { cl.buffer.pop(); cl.cursor = cl.buffer.len(); }
                app.command_line = Some(cl);
                return Ok(true);
            }
            KeyCode::Enter => {
                // Execute commands using the runner registry
                let cmd = cl.buffer.trim();
                // consume command-line (do not put back)
                match commands::execute_command(app, cmd) {
                    Ok(true) => { /* executed */ }
                    Ok(false) => {
                        app.mode = Mode::Message { title: "Command".to_string(), content: format!("Unknown command: {}", cmd), buttons: vec!["OK".to_string()], selected: 0, actions: None };
                    }
                    Err(e) => {
                        app.mode = Mode::Message { title: "Error".to_string(), content: format!("Command error: {}", e), buttons: vec!["OK".to_string()], selected: 0, actions: None };
                    }
                }
                return Ok(true);
            }
            KeyCode::Esc => {
                // dismiss without executing
                return Ok(true);
            }
            _ => {
                // unhandled; put back and return consumed
                app.command_line = Some(cl);
                return Ok(true);
            }
        }
    }
    Ok(false)
}
