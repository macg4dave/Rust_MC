use std::fs;
use std::io;
use std::path::Path;
use fs_extra::file::copy as file_copy;
use fs_extra::dir::{copy as dir_copy, CopyOptions};
#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt, symlink as unix_symlink};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(windows)]
use std::os::windows::fs::{symlink_dir as windows_symlink_dir, symlink_file as windows_symlink_file};

/// Copy the contents of a directory recursively from `src` into `dst`.
///
/// This helper will:
/// - create `dst` (and parents) if necessary;
/// - copy the contents of `src` into `dst` (i.e. the children of `src`, not
///   the `src` directory itself) using `fs_extra`'s directory copy;
/// - attempt to preserve metadata (permissions/timestamps) by delegating to
///   `crate::fs_op::metadata::preserve_all_metadata` after a successful copy.
///
/// Behaviour notes and guarantees:
/// - Existing files in `dst` are not overwritten (the copy uses
///   non-overwrite semantics).
/// - Symlink semantics and some special file types (FIFOs, device nodes) are
///   preserved when possible. On Unix this helper will recreate symlinks and
///   named pipes (FIFOs) using best-effort system calls; device node creation
///   may require privileges and will be attempted but may fail with a
///   permission error.
///
/// # Errors
/// Returns an `io::Error` for any underlying filesystem or copy errors.
/// Errors coming from `fs_extra` are mapped into `io::ErrorKind::Other`.
pub(crate) fn copy_recursive(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    // Ensure the destination directory exists before starting.
    fs::create_dir_all(dst)?;


    // Iterate top-level entries under `src` and copy them into `dst` using
    // `fs_extra` so we get predictable behaviour (each child of `src` is
    // copied into `dst` rather than possibly nesting the source directory).
    for entry in fs::read_dir(src).map_err(|e| io::Error::new(io::ErrorKind::Other, e))? {
        let entry = entry.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let path = entry.path();
        let file_name = match entry.file_name().into_string() {
            Ok(s) => s,
            Err(_) => continue, // skip non-UTF8 names in tests/complex scenarios
        };

        // Use symlink_metadata so we can detect symlinks and special file types
        // without following the link.
        let meta = fs::symlink_metadata(&path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        if meta.file_type().is_dir() {
            // If the destination directory already exists, copy the contents
            // of `path` into it (preserving existing files). Otherwise copy
            // the directory itself into `dst`.
            let dest_dir = dst.join(&file_name);
            let mut dir_opts = CopyOptions::new();
            dir_opts.overwrite = false;
            dir_opts.buffer_size = 64 * 1024;

            if dest_dir.exists() {
                // copy contents into existing dest_dir
                dir_opts.copy_inside = true;
                dir_copy(&path, &dest_dir, &dir_opts).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            } else {
                // copy directory as a child of dst
                dir_opts.copy_inside = false;
                dir_copy(&path, dst, &dir_opts).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            }
            continue;
        }

        if meta.file_type().is_file() {
            // Copy the file into `dst/<file_name>` using fs_extra file copy.
            let dest_file = dst.join(&file_name);
            if dest_file.exists() {
                // Respect non-overwrite semantics: skip existing files.
                continue;
            }
            let mut file_opts = fs_extra::file::CopyOptions::new();
            file_opts.overwrite = false;
            file_opts.buffer_size = 64 * 1024;
            file_copy(&path, &dest_file, &file_opts).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            continue;
        }

        // Handle symlinks and some special file types.
        if meta.file_type().is_symlink() {
            // Recreate the symlink at the destination with the same target.
            let target = fs::read_link(&path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let dest_link = dst.join(&file_name);
            // If destination exists, do not overwrite.
            if dest_link.exists() {
                continue;
            }
            #[cfg(unix)]
            {
                unix_symlink(&target, &dest_link).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            }
            #[cfg(windows)]
            {
                if meta.file_type().is_dir() {
                    windows_symlink_dir(&target, &dest_link).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                } else {
                    windows_symlink_file(&target, &dest_link).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                }
            }
            continue;
        }

        // Unix-only: try to preserve FIFOs (named pipes) and device nodes where possible.
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            use std::ffi::CString;

            let dest_path = dst.join(&file_name);

            if meta.file_type().is_fifo() {
                // Create a FIFO at dest with the same mode bits as source (best-effort).
                let mode = meta.permissions().mode() & 0o777;
                let cstr = CString::new(dest_path.as_os_str().as_bytes()).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                let res = unsafe { libc::mkfifo(cstr.as_ptr(), mode as libc::mode_t) };
                if res != 0 {
                    return Err(io::Error::last_os_error());
                }
                continue;
            }

            if meta.file_type().is_char_device() || meta.file_type().is_block_device() {
                // Attempt to recreate device node. This usually requires privileges;
                // we attempt it and propagate any errors.
                use std::os::unix::fs::MetadataExt;
                let mode = meta.permissions().mode();
                let rdev = meta.rdev();
                let cstr = CString::new(dest_path.as_os_str().as_bytes()).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                let kind = if meta.file_type().is_char_device() { libc::S_IFCHR } else { libc::S_IFBLK };
                let m: libc::mode_t = (mode & 0o7777) as libc::mode_t | kind as libc::mode_t;
                let dev = rdev as libc::dev_t;
                let res = unsafe { libc::mknod(cstr.as_ptr(), m, dev) };
                if res != 0 {
                    return Err(io::Error::last_os_error());
                }
                continue;
            }
        }

        // Other special types (sockets, unknown) are currently ignored.
    }

    // Attempt to preserve metadata for the whole tree (best-effort).
    crate::fs_op::metadata::preserve_all_metadata(src, dst)?;

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::{Read, Write};

    /// Helper to write a file with `content` at `path`.
    fn write_file(path: &Path, content: &str) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut f = File::create(path)?;
        f.write_all(content.as_bytes())?;
        Ok(())
    }

    #[test]
    fn copies_directory_contents() -> io::Result<()> {
        let src = tempfile::tempdir()?;
        let dst = tempfile::tempdir()?;

        // src structure: file1.txt and nested/inner.txt
        let file1 = src.path().join("file1.txt");
        write_file(&file1, "hello")?;

        let nested = src.path().join("nested");
        fs::create_dir_all(&nested)?;
        write_file(&nested.join("inner.txt"), "inner")?;

        // Debug: list src/dst before copy to help diagnose failures.
        #[cfg(test)]
        fn list_dir(path: &std::path::Path) -> Vec<String> {
            let mut out = Vec::new();
            for entry in walkdir::WalkDir::new(path).min_depth(0) {
                if let Ok(e) = entry {
                    out.push(e.path().display().to_string());
                }
            }
            out
        }

        println!("src before copy: {:#?}", list_dir(src.path()));
        println!("dst before copy: {:#?}", list_dir(dst.path()));

        copy_recursive(src.path(), dst.path())?;

        println!("dst after copy: {:#?}", list_dir(dst.path()));

        // After copy, dst should contain file1.txt and nested/inner.txt
        let copy_file1 = dst.path().join("file1.txt");
        let mut contents = String::new();
        File::open(copy_file1)?.read_to_string(&mut contents)?;
        assert_eq!(contents, "hello");

        let mut inner_contents = String::new();
        File::open(dst.path().join("nested").join("inner.txt"))?
            .read_to_string(&mut inner_contents)?;
        assert_eq!(inner_contents, "inner");

        Ok(())
    }

    #[test]
    fn errors_on_missing_source() {
        let dst = tempfile::tempdir().unwrap();
        let nonexistent = Path::new("/this/path/should/not/exist/hopefully");
        let res = copy_recursive(nonexistent, dst.path());
        assert!(res.is_err(), "expected error copying from nonexistent source");
    }

    #[test]
    fn does_not_overwrite_existing_file() -> io::Result<()> {
        let src = tempfile::tempdir()?;
        let dst = tempfile::tempdir()?;

        // src has keep.txt with "new"
        write_file(&src.path().join("keep.txt"), "new")?;

        // dst has keep.txt with "old"
        write_file(&dst.path().join("keep.txt"), "old")?;

        copy_recursive(src.path(), dst.path())?;

        // After copy, dst/keep.txt should still be "old" because overwrite=false
        let mut contents = String::new();
        File::open(dst.path().join("keep.txt"))?.read_to_string(&mut contents)?;
        assert_eq!(contents, "old");

        Ok(())
    }

    #[test]
    fn preserves_symlinks_when_possible() -> io::Result<()> {
        use std::fs::File;
        use std::io::Read;

        let src = tempfile::tempdir()?;
        let dst = tempfile::tempdir()?;

        // Create a target file and a symlink to it.
        let target = src.path().join("target.txt");
        write_file(&target, "symlink target")?;

        let link = src.path().join("the_link");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, &link)?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&target, &link)?;

        copy_recursive(src.path(), dst.path())?;

        // Destination should contain a symlink with same link target (as a relative
        // or absolute path depending on creation). We verify we can read the
        // link and that following it gives the expected content.
        let dest_link = dst.path().join("the_link");
        let read_target = fs::read_link(&dest_link)?;
        // Ensure the target (safely) resolves to a file we can read.
        let mut buf = String::new();
        let mut f = File::open(dst.path().join(read_target))?;
        f.read_to_string(&mut buf)?;
        assert_eq!(buf, "symlink target");

        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn preserves_fifo_named_pipe() -> io::Result<()> {
        use std::os::unix::fs::FileTypeExt;
        use std::ffi::CString;

        let src = tempfile::tempdir()?;
        let dst = tempfile::tempdir()?;

        let fifo = src.path().join("mypipe");
        let cstr = CString::new(fifo.as_os_str().as_bytes()).unwrap();
        let res = unsafe { libc::mkfifo(cstr.as_ptr(), 0o644) };
        assert_eq!(res, 0, "mkfifo failed in test");

        copy_recursive(src.path(), dst.path())?;

        let metadata = fs::symlink_metadata(dst.path().join("mypipe"))?;
        assert!(metadata.file_type().is_fifo(), "expected FIFO at destination");

        Ok(())
    }
}
