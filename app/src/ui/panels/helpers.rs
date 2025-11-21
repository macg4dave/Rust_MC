/// Compute the scrollbar thumb start position and size for a given viewport height
/// and scrollable content characteristics.
pub fn compute_scrollbar_thumb(
    height: usize,
    total: usize,
    visible: usize,
    offset: usize,
) -> (usize, usize) {
    if total == 0 || visible == 0 || total <= visible || height == 0 {
        return (0, 0);
    }
    let thumb_size = std::cmp::max(1, (visible * height) / total);
    let mut start = if total > 0 {
        (offset * height) / total
    } else {
        0
    };
    if start + thumb_size > height {
        if thumb_size >= height {
            start = 0;
        } else {
            start = height - thumb_size;
        }
    }
    (start, thumb_size)
}
