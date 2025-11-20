use std::fs;
use std::io;
use std::path::Path;

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

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let child_src = entry.path();
        let child_dst = dst.join(&file_name);

        // Use the DirEntry file_type when possible to avoid an extra `stat`.
        // Note: `file_type()` may follow platform semantics for symlinks; if
        // symlink support is required change this logic accordingly.
        let ft = entry.file_type()?;
        if ft.is_dir() {
            // Recurse into directories (creates child_dst inside recursive call).
            copy_recursive(&child_src, &child_dst)?;
        } else {
            // Use atomic copy for files to avoid observing partial files.
            // Parent directories are created at the start of recursion and
            // by recursive directory copies; an extra create here was
            // defensive but unnecessary and removed to avoid redundant
            // syscalls and to keep ordering explicit.
            crate::fs_op::helpers::atomic_copy_file(&child_src, &child_dst)?;
        }
    }

    Ok(())
}
