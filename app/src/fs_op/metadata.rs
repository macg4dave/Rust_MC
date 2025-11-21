use std::fs;
use std::io;
use std::path::Path;
use filetime::{FileTime, set_file_times};
use walkdir::WalkDir;

// Minimal metadata preservation helpers.
//
// These helpers provide small utilities to copy common metadata (permissions)
// from `src` to `dst`. They are intentionally conservative and focus on
// permissions because ownership/UID/GID aren't portable across systems.

/// Copy file permission bits from `src` to `dst` when possible.
#[allow(dead_code)]
pub(crate) fn copy_permissions(src: &Path, dst: &Path) -> io::Result<()> {
    let perms = fs::metadata(src)?.permissions();
    fs::set_permissions(dst, perms)
}

/// Placeholder for future metadata copying (timestamps, ownership).
/// Implement platform-specific behavior here when needed.
#[allow(dead_code)]
pub(crate) fn preserve_all_metadata(_src: &Path, _dst: &Path) -> io::Result<()> {
    // Default behavior: preserve permission bits and timestamps for the
    // single `src` -> `dst` path. If `src` is a directory this will copy
    // metadata recursively for entries under `src` to the corresponding
    // paths under `dst`.
    //
    // This is intentionally conservative: ownership (UID/GID) is not
    // touched because it is not generally portable across environments.
    if !_src.exists() || !_dst.exists() {
        return Ok(());
    }

    if _src.is_file() {
        // copy permissions
        let _ = copy_permissions(_src, _dst);
        // copy timestamps
        if let Ok(meta) = fs::metadata(_src) {
            if let (Ok(m), Ok(a)) = (meta.modified(), meta.accessed()) {
                let m_ft = FileTime::from_system_time(m);
                let a_ft = FileTime::from_system_time(a);
                let _ = set_file_times(_dst, a_ft, m_ft);
            }
        }
        return Ok(());
    }

    // For directories walk the tree and apply permissions + timestamps
    // to each matching path under dst. We tolerate missing targets and
    // treat metadata copying as best-effort to avoid hard failures during
    // bulk copies.
    for entry in WalkDir::new(_src).follow_links(false) {
        if let Ok(e) = entry {
            let rel = match e.path().strip_prefix(_src) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let target = _dst.join(rel);
            if !target.exists() {
                continue;
            }
            // copy permissions
            let _ = copy_permissions(e.path(), &target);
            // copy timestamps
            if let Ok(meta) = fs::metadata(e.path()) {
                if let (Ok(m), Ok(a)) = (meta.modified(), meta.accessed()) {
                    let m_ft = FileTime::from_system_time(m);
                    let a_ft = FileTime::from_system_time(a);
                    let _ = set_file_times(&target, a_ft, m_ft);
                }
            }
        }
    }
    Ok(())
}
