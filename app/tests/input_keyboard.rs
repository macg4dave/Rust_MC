use crossterm::event::{KeyCode as CtKeyCode, KeyEvent as CtKeyEvent, KeyModifiers};
use fileZoom::input::keyboard::{is_printable_key, KeyCode as AppKeyCode};

#[test]
fn printable_char_detected() {
    let ev = CtKeyEvent::new(CtKeyCode::Char('a'), KeyModifiers::NONE);
    let app_k: AppKeyCode = ev.into();
    assert!(is_printable_key(&app_k));
}

#[test]
fn non_printable_keys_not_detected() {
    let ev = CtKeyEvent::new(CtKeyCode::Enter, KeyModifiers::NONE);
    let app_k: AppKeyCode = ev.into();
    assert!(!is_printable_key(&app_k));
    let ev2 = CtKeyEvent::new(CtKeyCode::Up, KeyModifiers::NONE);
    let app_k2: AppKeyCode = ev2.into();
    assert!(!is_printable_key(&app_k2));
}

#[test]
fn modifier_ctrl_char_is_printable() {
    let ev = CtKeyEvent::new(CtKeyCode::Char('c'), KeyModifiers::CONTROL);
    let app_k: AppKeyCode = ev.into();
    assert!(is_printable_key(&app_k));
}

#[test]
fn modifier_alt_char_is_printable() {
    let ev = CtKeyEvent::new(CtKeyCode::Char('x'), KeyModifiers::ALT);
    let app_k: AppKeyCode = ev.into();
    assert!(is_printable_key(&app_k));
}

#[test]
fn function_key_not_printable() {
    let ev = CtKeyEvent::new(CtKeyCode::F(1), KeyModifiers::NONE);
    let app_k: AppKeyCode = ev.into();
    assert!(!is_printable_key(&app_k));
}
