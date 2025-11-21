// Centralised keybind predicates for the application.
//
// This file exposes small, well-named helper functions like `is_quit` and
// `is_down` so the rest of the codebase can refer to key actions rather
// than raw `KeyCode` patterns. The functions are intentionally simple
// wrappers today so they can later be wired to user-configurable
// settings (XML/JSON) without touching the handlers.

use crate::input::KeyCode;
use crate::app::settings::runtime_keybinds;

fn is_bound(action: &str, code: &KeyCode) -> bool {
	runtime_keybinds::get().is_bound(action, code)
}

pub fn is_quit(code: &KeyCode) -> bool {
	is_bound("quit", code)
}

pub fn is_down(code: &KeyCode) -> bool {
	is_bound("down", code)
}

pub fn is_up(code: &KeyCode) -> bool {
	is_bound("up", code)
}

pub fn is_page_down(code: &KeyCode) -> bool {
	is_bound("page_down", code)
}

pub fn is_page_up(code: &KeyCode) -> bool {
	is_bound("page_up", code)
}

pub fn is_enter(code: &KeyCode) -> bool {
	is_bound("enter", code)
}

pub fn is_backspace(code: &KeyCode) -> bool {
	is_bound("backspace", code)
}

pub fn is_refresh(code: &KeyCode) -> bool {
	is_bound("refresh", code)
}

pub fn is_delete(code: &KeyCode) -> bool {
	is_bound("delete", code)
}

pub fn is_copy(code: &KeyCode) -> bool {
	is_bound("copy", code)
}

pub fn is_move(code: &KeyCode) -> bool {
	is_bound("mv", code)
}

pub fn is_new_file(code: &KeyCode) -> bool {
	is_bound("new_file", code)
}

pub fn is_new_dir(code: &KeyCode) -> bool {
	is_bound("new_dir", code)
}

pub fn is_rename(code: &KeyCode) -> bool {
	is_bound("rename", code)
}

pub fn is_sort(code: &KeyCode) -> bool {
	is_bound("sort", code)
}

pub fn is_toggle_sort_direction(code: &KeyCode) -> bool {
	is_bound("toggle_sort_direction", code)
}

pub fn is_toggle_selection(code: &KeyCode) -> bool {
	is_bound("toggle_selection", code)
}

pub fn is_tab(code: &KeyCode) -> bool {
	is_bound("tab", code)
}

pub fn is_f5(code: &KeyCode) -> bool {
	is_bound("f5", code)
}

pub fn is_f6(code: &KeyCode) -> bool {
	is_bound("f6", code)
}

pub fn is_left(code: &KeyCode) -> bool {
	is_bound("left", code)
}

pub fn is_right(code: &KeyCode) -> bool {
	is_bound("right", code)
}

pub fn is_esc(code: &KeyCode) -> bool {
	is_bound("esc", code)
}

pub fn is_char(code: &KeyCode, want: char) -> bool {
	matches!(code, &KeyCode::Char(c) if c == want)
}

// Keep helpers thin: they delegate to runtime-configured bindings.