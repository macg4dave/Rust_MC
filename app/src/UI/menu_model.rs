// reserved for future mapping between menu items and runner Actions

#[derive(Copy, Clone, Debug)]
pub enum MenuAction {
    Settings,
    NewFile,
    NewDir,
    Copy,
    Move,
    Sort,
    Help,
    Quit,
    About,
    Noop,
}

#[derive(Clone, Debug)]
pub struct MenuItem { pub label: String, pub action: Option<MenuAction> }

#[derive(Clone, Debug)]
pub struct MenuTop { pub label: String, pub action: Option<MenuAction>, pub submenu: Option<Vec<MenuItem>> }

#[derive(Clone, Debug)]
pub struct MenuState { pub open: bool, pub top_index: usize, pub submenu_index: Option<usize> }

impl Default for MenuState { fn default() -> Self { Self { open: false, top_index: 0, submenu_index: None } } }

pub struct MenuModel;

impl MenuModel {
    pub fn default_model() -> (Vec<MenuTop>, ()) {
        let tops = vec![
            MenuTop { label: "File".into(), action: None, submenu: Some(vec![MenuItem{label:"Open".into(), action: Some(MenuAction::Noop)}]) },
            MenuTop { label: "Copy".into(), action: Some(MenuAction::Copy), submenu: None },
            MenuTop { label: "Move".into(), action: Some(MenuAction::Move), submenu: None },
            MenuTop { label: "New".into(), action: None, submenu: Some(vec![MenuItem{label:"New File".into(), action: Some(MenuAction::NewFile)}, MenuItem{label:"New Dir".into(), action: Some(MenuAction::NewDir)}])},
            MenuTop { label: "Sort".into(), action: Some(MenuAction::Sort), submenu: None },
            MenuTop { label: "Settings".into(), action: Some(MenuAction::Settings), submenu: None },
            MenuTop { label: "Help".into(), action: Some(MenuAction::Help), submenu: None },
        ];
        (tops, ())
    }
}

impl MenuState {
    pub fn selected_action(&self, model: &(Vec<MenuTop>, ())) -> Option<MenuAction> {
        model.0.get(self.top_index).and_then(|top| {
            if let Some(si) = self.submenu_index { top.submenu.as_ref().and_then(|s| s.get(si)).and_then(|it| it.action.clone()) } else { None }
        })
    }

    pub fn index_for_x(col: u16, width: u16, labels: &Vec<&str>) -> usize {
        let n = labels.len();
        if n == 0 { return 0; }
        let step = (width as usize).saturating_div(n.max(1));
        let idx = (col as usize).saturating_div(step.max(1));
        std::cmp::min(idx, n.saturating_sub(1))
    }

    pub fn close(&mut self) { self.open = false; self.submenu_index = None; }

    pub fn toggle_top(&mut self, idx: usize) { if self.open && self.top_index == idx { self.close(); } else { self.open = true; self.top_index = idx; self.submenu_index = Some(0); } }

    pub fn open_top(&mut self, idx: usize) { self.open = true; self.top_index = idx; self.submenu_index = Some(0); }

    pub fn select_next(&mut self, total: usize) { if let Some(i) = self.submenu_index { self.submenu_index = Some((i + 1) % total); } }

    pub fn select_prev(&mut self, total: usize) { if let Some(i) = self.submenu_index { self.submenu_index = Some((i + total - 1) % total); } }
}
