use serde::Serialize;

/// Thin view model passed to renderers — keeps widget code testable and small.
#[derive(Clone, Debug, Serialize, Default)]
pub struct UIState {
    pub left_list: Vec<String>,
    pub left_selected: usize,
    pub right_list: Vec<String>,
    pub right_selected: usize,
    pub menu_selected: usize,
    pub menu_focused: bool,
    /// Whether the top menu is open and showing a submenu
    pub menu_open: bool,
    /// When a submenu is open this is the index of the selected submenu entry
    pub menu_sub_selected: Option<usize>,
    pub preview_text: Option<String>,
    pub progress: u16,
}

impl UIState {
    pub fn sample() -> Self {
        Self {
            left_list: vec!["left-a".into(), "left-b".into(), "left-c".into()],
            left_selected: 0,
            right_list: vec!["right-x".into(), "right-y".into(), "right-z".into()],
            right_selected: 1,
            menu_selected: 0,
            menu_focused: true,
            menu_open: false,
            menu_sub_selected: None,
            preview_text: Some("preview".into()),
            progress: 25,
        }
    }

    /// Build a UIState view-model from the core App so UI rendering shows real data.
    pub fn from_core(app: &crate::app::core::App) -> Self {
        let left_list = app.left.entries.iter().map(|e| e.name.clone()).collect();
        let right_list = app.right.entries.iter().map(|e| e.name.clone()).collect();
        Self {
            left_list,
            left_selected: app.left.selected,
            right_list,
            right_selected: app.right.selected,
            preview_text: {
                let lp = app.left.preview.clone();
                if !lp.is_empty() {
                    Some(lp)
                } else {
                    let rp = app.right.preview.clone();
                    if !rp.is_empty() { Some(rp) } else { None }
                }
            },
            progress: 0,
            menu_selected: app.menu_index,
            menu_focused: app.menu_focused,
            menu_open: app.menu_state.open,
            menu_sub_selected: app.menu_state.submenu_index,
        }
    }
}
