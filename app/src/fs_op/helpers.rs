use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use fs_extra::file::{copy as fs_extra_copy, CopyOptions};
use super::test_helpers as tests;

/// Resolve destination path for an operation: if `dst` looks like a directory
/// (exists or ends with a separator) then target becomes `dst.join(src_name)`.
///
/// Kept as a small, dependency-free helper in `fs_op` so filesystem helpers
/// live together and can be tested independently of `App`.
/// Resolve a destination path for an operation.
///
/// If `dst` is a directory (exists as directory) or syntactically ends
/// with a trailing `/`, the returned path will be `dst.join(src_name)`.
/// Otherwise `dst` is returned as-is.
pub fn resolve_target(dst: &Path, src_name: &str) -> PathBuf {
    if dst.is_dir() || dst.to_string_lossy().ends_with('/') {
        dst.join(src_name)
    } else {
        dst.to_path_buf()
    }
}

/// Ensure parent directory exists for a path.
/// Ensure the parent directory of `p` exists.
pub fn ensure_parent_exists(p: &Path) -> io::Result<()> {
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Write `data` to `target` atomically by writing to a temporary file in the
/// same directory and then renaming into place. This avoids partial writes
/// being observed by other processes.
/// Atomically write `data` to `target` by writing a temp file then
/// renaming into place. Temp files are created in the same directory as
/// `target` to ensure the rename is atomic on the same filesystem.
pub fn atomic_write(target: &Path, data: &[u8]) -> io::Result<()> {
    if let Some(dir) = target.parent() {
        fs::create_dir_all(dir)?;
        let mut tmp = dir.join(".tmp_atomic_write");

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            .as_nanos();
        let pid = process::id() as u128;
        let raw = format!("{:x}{:x}", pid, nanos);
        let suffix = raw.chars().rev().take(8).collect::<String>().chars().rev().collect::<String>();
        tmp.set_file_name(format!(".tmp_atomic_write.{}", suffix));

        // Ensure the temp file is removed on any early return.
        if let Err(e) = fs::write(&tmp, data) {
            let _ = fs::remove_file(&tmp);
            return Err(e);
        }

        // test hook may force a failure to exercise cleanup paths
        if tests::should_force_rename_fail_in_write() {
            let _ = fs::remove_file(&tmp);
            return Err(io::Error::new(io::ErrorKind::Other, "forced rename failure (write)"));
        }

        fs::rename(&tmp, target).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            e
        })
    } else {
        // No parent directory — write directly.
        fs::write(target, data)
    }
}

/// Copy a single file atomically: copy into a temp file in the destination
/// directory then rename into place.
/// Atomically copy a single file by copying into a temp file in the
/// destination directory and renaming into place. Returns number of bytes
/// copied on success.
pub fn atomic_copy_file(src: &Path, dst: &Path) -> io::Result<u64> {
    // Prepare copy options used in both branches.
    let mut options = CopyOptions::new();
    options.overwrite = false;
    options.buffer_size = 64 * 1024;

    if let Some(dir) = dst.parent() {
        fs::create_dir_all(dir)?;
        let mut tmp = dir.join(".tmp_atomic_copy");

        // Build a reasonably unique suffix from pid, time, thread and a
        // monotonic sequence counter to avoid collisions in concurrent runs.
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            .as_nanos();
        let pid = process::id() as u128;
        let thread_id = format!("{:?}", std::thread::current().id());
        let mut hasher = DefaultHasher::new();
        thread_id.hash(&mut hasher);
        let thread_hash = hasher.finish();
        static NEXT_COPY_ID: AtomicU64 = AtomicU64::new(0);
        let seq = NEXT_COPY_ID.fetch_add(1, Ordering::Relaxed) as u128;
        let raw = format!("{:x}{:x}{:x}{:x}", pid, nanos, thread_hash, seq);
        let suffix = raw.chars().rev().take(12).collect::<String>().chars().rev().collect::<String>();
        tmp.set_file_name(format!(".tmp_atomic_copy.{}", suffix));

        let n = fs_extra_copy(src, &tmp, &options).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // test hook may force a failure to exercise cleanup
        if tests::should_force_rename_fail_in_copy() {
            let _ = fs::remove_file(&tmp);
            return Err(io::Error::new(io::ErrorKind::Other, "forced rename failure (copy)"));
        }

        fs::rename(&tmp, dst).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            e
        })?;

        let _ = crate::fs_op::metadata::preserve_all_metadata(src, dst);
        Ok(n)
    } else {
        let res = fs_extra_copy(src, dst, &options).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let _ = crate::fs_op::metadata::preserve_all_metadata(src, dst);
        Ok(res)
    }
}

/// Try to rename `src` to `dst`. If `rename` fails due to cross-filesystem
/// issues, fall back to an atomic copy+remove approach.
/// Rename `src` to `dst`, falling back to copy+remove on failure (for
/// example cross-filesystem moves). Directories are delegated to the
/// `mv::move_path` helper which handles recursive semantics.
pub fn atomic_rename_or_copy(src: &Path, dst: &Path) -> io::Result<()> {
    // test hook: force fallback path
    if tests::should_force_rename_fail_in_rename_or_copy() {
        atomic_copy_file(src, dst)?;
        fs::remove_file(src)?;
        return Ok(());
    }

    if src.is_dir() {
        if fs::rename(src, dst).is_ok() {
            return Ok(());
        }
        return crate::fs_op::mv::move_path(src, dst)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()));
    }

    if fs::rename(src, dst).is_ok() {
        Ok(())
    } else {
        atomic_copy_file(src, dst)?;
        fs::remove_file(src)?;
        Ok(())
    }
}

#[cfg(test)]
mod parallel_tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs as stdfs;
    use rayon::prelude::*;

    #[test]
    fn atomic_copy_file_parallel_no_temp_collision() {
        // Create source and destination directories
        let sdir = tempdir().expect("temp src");
        let ddir = tempdir().expect("temp dst");
        // Create many small source files
        let n = 64;
        for i in 0..n {
            let p = sdir.path().join(format!("file_{}.txt", i));
            stdfs::write(&p, format!("hello {}", i)).expect("write src");
        }

        // Gather source paths and copy in parallel into dst
        let srcs: Vec<_> = (0..n)
            .map(|i| sdir.path().join(format!("file_{}.txt", i)))
            .collect();

        srcs.into_par_iter().for_each(|src| {
            let dst = ddir.path().join(src.file_name().unwrap());
            atomic_copy_file(&src, &dst).expect("copy");
        });

        // Ensure all destination files are present and no temp files remain
        let mut found = 0;
        for entry in stdfs::read_dir(ddir.path()).expect("read dst") {
            let e = entry.expect("entry");
            let name = e.file_name().to_string_lossy().to_string();
            assert!(!name.starts_with(".tmp_atomic_copy."), "temp file left behind: {}", name);
            found += 1;
        }
        assert_eq!(found, n);
    }

    #[test]
    fn atomic_copy_file_stress_many_concurrent_copies() {
        // Stress test: many concurrent copies targeting a smaller set of
        // destination names to force collisions and exercise temp-file
        // uniqueness and cleanup.
        let sdir = tempdir().expect("temp src");
        let ddir = tempdir().expect("temp dst");

        // Single source file used by all copy tasks.
        let src = sdir.path().join("shared_src.txt");
        stdfs::write(&src, "stress-test-content").expect("write src");

        // Many tasks but few distinct destination names to force collisions.
        let tasks = 1024usize;
        let dest_names = 16usize;

        let dsts: Vec<std::path::PathBuf> = (0..tasks)
            .map(|i| ddir.path().join(format!("dst_{}.txt", i % dest_names)))
            .collect();

        // Run copies in parallel; we ignore individual copy errors because
        // races on rename may cause some copies to fail — the important
        // assertion is that no temp files remain.
        dsts.into_par_iter().for_each(|dst| {
            let _ = atomic_copy_file(&src, &dst);
        });

        // After workload finishes ensure there are no leftover temp files.
        for entry in stdfs::read_dir(ddir.path()).expect("read dst") {
            let e = entry.expect("entry");
            let name = e.file_name().to_string_lossy().to_string();
            assert!(
                !name.starts_with(".tmp_atomic_copy."),
                "temp file left behind: {}",
                name
            );
        }
    }
}
// test hooks have been moved to `app/src/fs_op/test_helpers.rs` and are
// imported above as the `tests` alias so the existing call sites remain
// unchanged (e.g. `tests::should_force_rename_fail_in_copy()`).
