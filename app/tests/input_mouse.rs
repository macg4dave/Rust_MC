use crossterm::event::{
    KeyModifiers, MouseButton as CtBtn, MouseEvent as CtME, MouseEventKind as CtKind,
};
use fileZoom::input::mouse::{MouseButton, MouseEvent, MouseEventKind};

#[test]
fn convert_left_down() {
    let ct = CtME {
        kind: CtKind::Down(CtBtn::Left),
        column: 10,
        row: 4,
        modifiers: KeyModifiers::NONE,
    };
    let me: MouseEvent = ct.into();
    assert_eq!(me.column, 10);
    assert_eq!(me.row, 4);
    assert!(matches!(me.kind, MouseEventKind::Down(MouseButton::Left)));
}

#[test]
fn convert_scrolls() {
    let up = CtME {
        kind: CtKind::ScrollUp,
        column: 0,
        row: 0,
        modifiers: KeyModifiers::NONE,
    };
    let down = CtME {
        kind: CtKind::ScrollDown,
        column: 1,
        row: 2,
        modifiers: KeyModifiers::NONE,
    };
    let up_conv: MouseEvent = up.into();
    let down_conv: MouseEvent = down.into();
    assert!(matches!(up_conv.kind, MouseEventKind::ScrollUp));
    assert!(matches!(down_conv.kind, MouseEventKind::ScrollDown));
}
