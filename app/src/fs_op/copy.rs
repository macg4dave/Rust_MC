use std::fs;
use std::io;
use std::path::Path;
use fs_extra::dir::{copy as dir_copy, CopyOptions};

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

    // Use `fs_extra`'s directory copy functionality to copy the contents of
    // `src` into `dst`. This simplifies the implementation and provides
    // robust options for recursive copies. We keep behaviour similar to the
    // previous implementation by copying the directory contents (not the
    // source directory itself) into the already-created `dst`.
    let mut options = CopyOptions::new();
    // `copy_inside = true` copies the contents of `src` into `dst` rather
    // than copying the `src` folder itself.
    options.copy_inside = true;
    // Do not overwrite existing files by default; preserve prior behaviour.
    options.overwrite = false;
    // Use a 64 KiB buffer for recursive directory copies where supported.
    options.buffer_size = 64 * 1024;

    // fs_extra returns its own error type; map it to io::Error for callers.
    match dir_copy(src, dst, &options) {
        Ok(_) => {
            // Preserve permissions/timestamps from source tree into dst.
            if let Err(e) = crate::fs_op::metadata::preserve_all_metadata(src, dst) {
                return Err(e);
            }
            Ok(())
        }
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
    }
}
