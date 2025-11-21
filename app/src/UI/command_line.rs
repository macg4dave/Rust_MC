use crate::app::core::App;
use crate::input::KeyCode;

#[derive(Clone, Debug, Default)]
pub struct CommandLineState { pub visible: bool, pub buffer: String, pub cursor: usize }

pub fn handle_input(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    if let Some(cmd) = &mut app.command_line {
        match code {
            KeyCode::Char(c) => { cmd.buffer.push(c); return Ok(false); }
            KeyCode::Enter => {
                let b = cmd.buffer.clone(); cmd.visible = false; cmd.buffer.clear();
                // delegate to runner commands to parse/execute
                let _ = crate::runner::commands::execute_command(app, &b);
                return Ok(true);
            }
            _ => {}
        }
    }
    Ok(false)
}
