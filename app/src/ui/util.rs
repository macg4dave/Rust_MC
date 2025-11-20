use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::buffer::Buffer;

/// Compute column rectangles for a typical file list layout: name, size, modified, perms.
///
/// The helper uses Ratatui's Layout & `Constraint`s to compute the column Rects so
/// callers don't need to do manual width math. It returns a `Vec<Rect>` for each column.
///
/// Parameters:
/// - `area` - the area that should be split into columns
/// - `name_fill` - a `Ratio` pair indicating how to allocate remaining space (name column)
/// - `size_len`, `modified_len`, `perms_len` - fixed lengths for these columns in characters
pub fn columns_for_file_list(area: Rect, name_fill: (u32, u32), size_len: u16, modified_len: u16, perms_len: u16) -> Vec<Rect> {
    let (rnum, rden) = name_fill;
    // Use `Ratio` for the name column so it scales with available space. The other columns are `Length`.
    let constraints = [
        Constraint::Ratio(rnum, rden),
        Constraint::Length(size_len),
        Constraint::Length(modified_len),
        Constraint::Length(perms_len),
    ];

    Layout::default().direction(Direction::Horizontal).constraints(constraints.as_ref()).split(area).to_vec()
}

/// Render content into a temporary `Buffer` to enable splicing for scrollable content.
///
/// This helper renders `render_fn` into a buffer of `render_area`, then copies a rectangular
/// slice determined by `scroll_offset` and `viewport` into the final `buf` at `target_area`.
///
/// - `render_fn` must draw into the provided Frame-like object which is not available here, so we simply
/// render into Buffer using common ratatui widgets via the provided code path (this helper builds the Buffer
/// and expects a function that performs drawing into a `Buffer`).
pub fn splice_buffer_with_offset(demo_buf: &Buffer, render_area: Rect, target_area: Rect, scroll_offset: u16, out_buf: &mut Buffer) {
    // Calculate whether we need to scroll vertically and copy visible rows from demo_buf into out_buf
    let rows_to_skip = (scroll_offset as usize) * (render_area.width as usize);
    let visible_cells = demo_buf
        .content
        .iter()
        .skip(rows_to_skip)
        .take(target_area.area() as usize)
        .cloned();

    for (i, cell) in visible_cells.enumerate() {
        let x = i as u16 % target_area.width;
        let y = i as u16 / target_area.width;
        out_buf[(target_area.x + x, target_area.y + y)] = cell;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn columns_fill_area() {
        let area = Rect::new(0, 0, 80, 10);
        let size = 10u16;
        let modified = 16u16;
        let perms = 4u16;
        let cols = columns_for_file_list(area, (1, 1), size, modified, perms);
        assert_eq!(cols.len(), 4);
        let total_width: u16 = cols.iter().map(|r| r.width).sum();
        assert_eq!(total_width, area.width);
        // Name width should be area.width - the sum of fixed columns
        let name_width = area.width - (size + modified + perms);
        assert_eq!(cols[0].width, name_width);
    }
}
