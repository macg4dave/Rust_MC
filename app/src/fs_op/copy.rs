use std::fs;
use std::io;
use std::path::Path;
use walkdir::WalkDir;

/// Recursively copy `src` directory into `dst`.
///
/// The function creates `dst` (and nested directories) as needed and copies
/// regular files using `atomic_copy_file` from `crate::fs_op::helpers` so
/// callers won't observe partially-written files. Symlink handling and
/// metadata (permissions/timestamps) are intentionally out of scope for this
/// small helper; use a richer copy implementation if you need full fidelity.
pub(crate) fn copy_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    // Ensure the destination directory exists before starting.
    fs::create_dir_all(dst)?;

    // Walk the source directory recursively and mirror structure into dst.
    // We intentionally avoid following symlinks here to match prior behaviour.
    for entry in WalkDir::new(src).min_depth(1).follow_links(false) {
        let entry = entry.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let path = entry.path();

        let rel = path
            .strip_prefix(src)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let target = dst.join(rel);

        let ft = entry.file_type();
        if ft.is_dir() {
            fs::create_dir_all(&target)?;
        } else if ft.is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            crate::fs_op::helpers::atomic_copy_file(path, &target)?;
        } else {
            // Skip other file types (symlinks, device nodes) for this helper.
        }
    }

    Ok(())
}
