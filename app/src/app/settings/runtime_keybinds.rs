use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use once_cell::sync::Lazy;
use crate::app::settings::config_dirs::project_config_dir;
use crate::input::KeyCode;
use std::fs;

/// Runtime-configured keybindings. Loads `keybinds.xml` from the
/// project config dir or current working directory if present. Always
/// provides a sensible default mapping so callers can simply consult
/// bindings via `KEYBINDS.is_bound("action", &code)`.
pub struct Keybinds {
    map: HashMap<String, Vec<KeyCode>>,
}

impl Keybinds {
    pub fn is_bound(&self, action: &str, code: &KeyCode) -> bool {
        self.map
            .get(action)
            .map(|v| v.iter().any(|k| k == code))
            .unwrap_or(false)
    }

    fn insert(&mut self, action: &str, kc: KeyCode) {
        self.map
            .entry(action.to_string())
            .or_default()
            .push(kc);
    }

    fn default() -> Self {
        use KeyCode::*;
        let mut m = HashMap::new();
        m.insert("quit".to_string(), vec![Char('q')]);
        m.insert("down".to_string(), vec![Down]);
        m.insert("up".to_string(), vec![Up]);
        m.insert("page_down".to_string(), vec![PageDown]);
        m.insert("page_up".to_string(), vec![PageUp]);
        m.insert("enter".to_string(), vec![Enter]);
        m.insert("backspace".to_string(), vec![Backspace]);
        m.insert("refresh".to_string(), vec![Char('r')]);
        m.insert("delete".to_string(), vec![Char('d')]);
        m.insert("copy".to_string(), vec![Char('c')]);
        m.insert("mv".to_string(), vec![Char('m')]);
        m.insert("new_file".to_string(), vec![Char('n')]);
        m.insert("new_dir".to_string(), vec![Char('N')]);
        m.insert("rename".to_string(), vec![Char('R')]);
        m.insert("sort".to_string(), vec![Char('s')]);
        m.insert("toggle_sort_direction".to_string(), vec![Char('S')]);
        m.insert("toggle_selection".to_string(), vec![Char(' ')]);
        m.insert("tab".to_string(), vec![Tab]);
        m.insert("f5".to_string(), vec![F(5)]);
        m.insert("f6".to_string(), vec![F(6)]);
        m.insert("left".to_string(), vec![Left]);
        m.insert("right".to_string(), vec![Right]);
        m.insert("esc".to_string(), vec![Esc]);

        Keybinds { map: m }
    }

    fn parse_keycode(s: &str) -> Option<KeyCode> {
        // Accept patterns like "Enter", "Backspace", "Esc", "Left",
        // "Char x" (or single char strings), "F5".
        use KeyCode::*;
        let t = s.trim();
        if t.eq_ignore_ascii_case("enter") {
            return Some(Enter);
        }
        if t.eq_ignore_ascii_case("backspace") {
            return Some(Backspace);
        }
        if t.eq_ignore_ascii_case("esc") || t.eq_ignore_ascii_case("escape") {
            return Some(Esc);
        }
        if t.eq_ignore_ascii_case("tab") {
            return Some(Tab);
        }
        if t.eq_ignore_ascii_case("left") {
            return Some(Left);
        }
        if t.eq_ignore_ascii_case("right") {
            return Some(Right);
        }
        if t.eq_ignore_ascii_case("up") {
            return Some(Up);
        }
        if t.eq_ignore_ascii_case("down") {
            return Some(Down);
        }
        if t.eq_ignore_ascii_case("pageup") || t.eq_ignore_ascii_case("page_up") {
            return Some(PageUp);
        }
        if t.eq_ignore_ascii_case("pagedown") || t.eq_ignore_ascii_case("page_down") {
            return Some(PageDown);
        }
        if let Some(rest) = t.strip_prefix('F') {
            if let Ok(n) = rest.parse::<u8>() {
                return Some(F(n));
            }
        }
        // Char syntax: either `Char x` or just a single character string
        if let Some(rest) = t.strip_prefix("Char ") {
            let ch = rest.chars().next()?;
            return Some(Char(ch));
        }
        if t.len() == 1 {
            return Some(Char(t.chars().next().unwrap()));
        }
        None
    }

    fn load_from_path(path: PathBuf) -> Result<Self> {
        // Simple, tolerant XML-ish parser: look for `<bind action="...">VALUE</bind>`
        let raw = fs::read_to_string(path)?;
        let mut kb = Keybinds { map: HashMap::new() };

        let mut rest = raw.as_str();
        while let Some(start) = rest.find("<bind") {
            rest = &rest[start..];
            // find action attribute
            let action_attr = rest.find("action=").and_then(|i| {
                let s = &rest[i + 7..]; // skip `action="`
                s.find('"').map(|j| &s[..j])
            });

            // more robust: search for action="..."
            let action = if let Some(a_start) = rest.find("action=\"") {
                let s = &rest[a_start + 8..];
                s.find('"').map(|endq| s[..endq].to_string())
            } else {
                None
            };

            // find end of start tag
            if let Some(gt) = rest.find('>') {
                let after = &rest[gt + 1..];
                if let Some(end) = after.find("</bind>") {
                    let inner = &after[..end].trim();
                    if let Some(action) = action.or_else(|| action_attr.map(|s| s.to_string())) {
                        if let Some(kc) = Keybinds::parse_keycode(inner) {
                            kb.insert(&action, kc);
                        }
                    }
                    rest = &after[end + 7..]; // move past </bind>
                    continue;
                }
            }
            break;
        }

        if kb.map.is_empty() {
            Ok(Keybinds::default())
        } else {
            let mut def = Keybinds::default();
            for (k, v) in kb.map.into_iter() {
                def.map.insert(k, v);
            }
            Ok(def)
        }
    }
}

static KEYBINDS: Lazy<Keybinds> = Lazy::new(|| {
    // Look for `keybinds.xml` first in the project config dir, then the cwd
    let mut candidates = Vec::new();
    let mut pc = project_config_dir();
    pc.push("keybinds.xml");
    candidates.push(pc);
    let mut cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    cwd.push("keybinds.xml");
    candidates.push(cwd);

    for p in candidates {
        if p.exists() {
            if let Ok(k) = Keybinds::load_from_path(p) {
                return k;
            }
        }
    }

    Keybinds::default()
});

/// Expose a reference to the global keybinds.
pub fn get() -> &'static Keybinds {
    &KEYBINDS
}
