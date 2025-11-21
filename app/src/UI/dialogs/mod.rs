use crate::app::Action;
use ratatui::{layout::Rect, widgets::{Block, Borders, Paragraph}, Frame};

/// Map a selected button index to a runner Action, if provided.
pub fn selection_to_action(selected: usize, actions: Option<&[Action]>) -> Option<Action> {
    actions.and_then(|s| s.get(selected)).cloned()
}

/// Small Dialog presentation used by UI tests.
pub struct Dialog<'a> { title: &'a str, body: &'a str, buttons: Vec<&'a str>, selected: usize }
impl<'a> Dialog<'a> {
    pub fn new(title: &'a str, body: &'a str, buttons: &[&'a str], selected: usize) -> Self { Self { title, body, buttons: buttons.to_vec(), selected } }
    pub fn draw(&self, f: &mut Frame, area: Rect, _focused: bool) {
        let mut txt = self.body.to_string();
        if !self.buttons.is_empty() {
            txt.push_str("\n\n");
            let mut parts: Vec<String> = Vec::new();
            for (i, b) in self.buttons.iter().enumerate() {
                if i == self.selected { parts.push(format!("[{}]", b)); } else { parts.push(b.to_string()); }
            }
            txt.push_str(&parts.join(" "));
        }
        let p = Paragraph::new(txt).block(Block::default().borders(Borders::ALL).title(self.title));
        f.render_widget(p, area);
    }
}
