use ratatui::layout::Rect;

/// Create a centered rectangle occupying `pct_x`% x `pct_y`% of `r`.
pub fn centered_rect(r: Rect, pct_x: u16, pct_y: u16) -> Rect {
    // pct_x and pct_y are desired width/height in units; clamp to available area
    let pw = std::cmp::min(pct_x, r.width);
    let ph = std::cmp::min(pct_y, r.height);
    let x = r.x + (r.width.saturating_sub(pw) / 2);
    let y = r.y + (r.height.saturating_sub(ph) / 2);
    Rect::new(x, y, pw.max(1), ph.max(1))
}
