// Small helpers related to filesystem watcher events used by the runner.
// Kept feature-gated so the crate does not require watcher types when
// `fs-watch` is disabled.
#![cfg(feature = "fs-watch")]

/// Map a filesystem watcher [`FsEvent`] to the set of `Side` values that
/// should be refreshed.
///
/// This helper is intentionally small and pure so it can be unit-tested
/// independently of the event loop that consumes its results.
pub(crate) fn affected_sides_from_fs_event(
    evt: &crate::fs_op::watcher::FsEvent,
    left: &std::path::Path,
    right: &std::path::Path,
) -> Vec<crate::app::Side> {
    use crate::fs_op::watcher::FsEvent;
    use crate::app::Side;

    let mut affected = Vec::new();
    match evt {
        FsEvent::Create(p) | FsEvent::Modify(p) | FsEvent::Remove(p) => {
            if p.starts_with(left) { affected.push(Side::Left); }
            if p.starts_with(right) { affected.push(Side::Right); }
        }
        FsEvent::Rename(a, b) => {
            if a.starts_with(left) || b.starts_with(left) { affected.push(Side::Left); }
            if a.starts_with(right) || b.starts_with(right) { affected.push(Side::Right); }
        }
        FsEvent::Other => {}
    }
    affected.sort_by_key(|s| match s { crate::app::Side::Left => 0, crate::app::Side::Right => 1 });
    affected.dedup();
    affected
}

#[cfg(all(test, feature = "fs-watch"))]
mod tests {
    use super::affected_sides_from_fs_event;
    use crate::fs_op::watcher::FsEvent;
    use crate::app::Side;

    #[test]
    fn affected_sides_create_left() {
        let left = std::path::Path::new("/tmp/left");
        let right = std::path::Path::new("/tmp/right");
        let ev = FsEvent::Create(std::path::PathBuf::from("/tmp/left/file.txt"));
        let sides = affected_sides_from_fs_event(&ev, left, right);
        assert_eq!(sides, vec![Side::Left]);
    }

    #[test]
    fn affected_sides_rename_both() {
        let left = std::path::Path::new("/tmp/left");
        let right = std::path::Path::new("/tmp/right");
        let ev = FsEvent::Rename(std::path::PathBuf::from("/tmp/left/a"), std::path::PathBuf::from("/tmp/right/b"));
        let mut sides = affected_sides_from_fs_event(&ev, left, right);
        sides.sort_by_key(|s| match s { Side::Left => 0, Side::Right => 1 });
        assert_eq!(sides, vec![Side::Left, Side::Right]);
    }
}
