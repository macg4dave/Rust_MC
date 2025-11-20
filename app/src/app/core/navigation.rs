use super::*;

impl App {
    /// Ensure the currently selected entry is visible within the active
    /// panel's viewport. Delegates to the panel-level helper so the scrolling
    /// behaviour is defined in one place and can be unit-tested independently.
    pub fn ensure_selection_visible(&mut self, list_height: usize) {
        self.active_panel_mut().ensure_selected_visible(list_height);
    }

    /// Move active selection down by one entry.
    pub fn next(&mut self, list_height: usize) {
        let panel = self.active_panel_mut();
        if !panel.entries.is_empty() {
            panel.select_next();
        }
        self.ensure_selection_visible(list_height);
        self.update_preview_for(self.active);
    }

    /// Move active selection up by one entry.
    pub fn previous(&mut self, list_height: usize) {
        let panel = self.active_panel_mut();
        if !panel.entries.is_empty() {
            panel.select_prev();
        }
        self.ensure_selection_visible(list_height);
        self.update_preview_for(self.active);
    }

    /// Move active selection down by `list_height` (page down) with clamping.
    pub fn page_down(&mut self, list_height: usize) {
        let panel = self.active_panel_mut();
        if !panel.entries.is_empty() {
            let new = std::cmp::min(
                panel.selected.saturating_add(list_height),
                panel.entries.len().saturating_sub(1),
            );
            panel.selected = new;
        }
        self.ensure_selection_visible(list_height);
        self.update_preview_for(self.active);
    }

    /// Move active selection up by `list_height` (page up) with saturating
    /// subtraction so it never underflows.
    pub fn page_up(&mut self, list_height: usize) {
        let panel = self.active_panel_mut();
        if !panel.entries.is_empty() {
            panel.selected = panel.selected.saturating_sub(list_height);
        }
        self.ensure_selection_visible(list_height);
        self.update_preview_for(self.active);
    }
}
