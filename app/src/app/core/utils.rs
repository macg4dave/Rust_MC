use super::panel::Panel;

/// Number of always-present UI header rows.
const HEADER_ROWS: usize = 1;

/// Return the total number of UI rows that will be rendered for a panel.
///
/// The UI contains a synthetic header row (path/title) and may include a
/// synthetic parent row when the panel's `cwd` has a parent directory.
/// The remainder of the rows correspond to the domain `entries` stored in
/// the panel. This helper is intentionally tiny and pure to make unit
/// testing straightforward.
pub(super) fn ui_row_count(panel: &Panel) -> usize {
    HEADER_ROWS + (panel.cwd.parent().is_some() as usize) + panel.entries.len()
}

/// Map a UI-selected row index to the corresponding domain `entries` index.
///
/// The UI presents synthetic rows before the domain entries: the header and
/// optionally a parent row. If `selected_row` refers to one of those
/// synthetic rows or to an out-of-range index, `None` is returned.
///
/// This function performs bounds checking to avoid panics if callers pass
/// an index that is not currently clamped to the panel's UI row range.
pub(super) fn ui_to_entry_index(selected_row: usize, panel: &Panel) -> Option<usize> {
    let parent_rows = panel.cwd.parent().is_some() as usize;
    // Fast path using checked_sub to avoid underflow on subtraction.
    selected_row
        .checked_sub(HEADER_ROWS + parent_rows)
        .and_then(|idx| if idx < panel.entries.len() { Some(idx) } else { None })
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::types::Entry;
    use std::path::PathBuf;

    fn make_panel_with_entries(cwd: PathBuf, names: &[&str]) -> Panel {
        let mut panel = Panel::new(cwd);
        panel.entries = names
            .iter()
            .map(|n| Entry::file(n.to_string(), PathBuf::from(n), 0, None))
            .collect();
        panel
    }

    #[test]
    fn ui_row_count_includes_header_and_parent() {
        // Path with a parent: relative "foo/bar".
        let panel = make_panel_with_entries(PathBuf::from("foo/bar"), &["a", "b"]);
        assert_eq!(ui_row_count(&panel), HEADER_ROWS + 1 + 2);

        // Path without a parent (single-component path) should not add parent row.
        // Use a root path which does not have a parent component on Unix.
        let panel_no_parent = make_panel_with_entries(PathBuf::from("/"), &[]);
        assert_eq!(ui_row_count(&panel_no_parent), HEADER_ROWS + 0 + 0);
    }

    #[test]
    fn ui_to_entry_index_maps_correctly_and_checks_bounds() {
        // Two entries, path with parent -> header + parent + entries
        let panel = make_panel_with_entries(PathBuf::from("foo/bar"), &["e1", "e2"]);

        // header (0) and parent (1) map to None
        assert_eq!(ui_to_entry_index(0, &panel), None);
        assert_eq!(ui_to_entry_index(1, &panel), None);

        // entries start at 2
        assert_eq!(ui_to_entry_index(2, &panel), Some(0));
        assert_eq!(ui_to_entry_index(3, &panel), Some(1));

        // out-of-range selection should return None rather than panicking
        assert_eq!(ui_to_entry_index(4, &panel), None);

        // When there is no parent row, entries start immediately after header
        let panel_no_parent = make_panel_with_entries(PathBuf::from("/"), &["only"]);
        assert_eq!(ui_to_entry_index(0, &panel_no_parent), None); // header
        assert_eq!(ui_to_entry_index(1, &panel_no_parent), Some(0));
    }
}
