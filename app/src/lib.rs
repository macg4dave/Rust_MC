pub mod app;
#[path = "ui/mod.rs"]
pub mod ui;

pub use crate::app::{App, Entry, SortKey, Mode, InputKind, Action, Side};

pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greet_returns_expected() {
        assert_eq!(greet("Alice"), "Hello, Alice!");
    }
}
