use std::fs;
use std::io;
use std::path::PathBuf;
use fs_extra::file::{copy as fs_extra_copy, CopyOptions};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::sync::atomic::{AtomicU64, Ordering};

/// Resolve destination path for an operation: if `dst` looks like a directory
/// (exists or ends with a separator) then target becomes `dst.join(src_name)`.
///
/// Kept as a small, dependency-free helper in `fs_op` so filesystem helpers
/// live together and can be tested independently of `App`.
pub fn resolve_target(dst: &std::path::Path, src_name: &str) -> PathBuf {
    if dst.is_dir() || dst.to_string_lossy().ends_with('/') {
        dst.join(src_name)
    } else {
        dst.to_path_buf()
    }
}

/// Ensure parent directory exists for a path.
pub fn ensure_parent_exists(p: &std::path::Path) -> io::Result<()> {
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Write `data` to `target` atomically by writing to a temporary file in the
/// same directory and then renaming into place. This avoids partial writes
/// being observed by other processes.
pub fn atomic_write(target: &std::path::Path, data: &[u8]) -> io::Result<()> {
    if let Some(dir) = target.parent() {
        fs::create_dir_all(dir)?;
        let mut tmp = dir.join(".tmp_atomic_write");
        // Use a time+pid-based suffix to avoid adding a dependency or
        // depending on `rand` internals. This is sufficiently unique for
        // temporary filenames in tests and small concurrent runs.
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let pid = std::process::id() as u128;
        let raw = format!("{:x}{:x}", pid, nanos);
        let suffix: String = raw
            .chars()
            .rev()
            .take(8)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        tmp.set_file_name(format!(".tmp_atomic_write.{}", suffix));
        // Write to the temp file first. If any step fails we attempt to
        // remove the temp file to avoid leaving artifacts.
        match fs::write(&tmp, data) {
            Ok(()) => {
                // In tests we may want to force the rename to fail to ensure
                // temp-file cleanup is exercised.
                {
                    if crate::fs_op::helpers::tests::should_force_rename_fail_in_write() {
                        let _ = fs::remove_file(&tmp);
                        return Err(io::Error::other("forced rename failure (write)"));
                    }
                }
                match fs::rename(&tmp, target) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        let _ = fs::remove_file(&tmp);
                        Err(e)
                    }
                }
            }
            Err(e) => {
                let _ = fs::remove_file(&tmp);
                Err(e)
            }
        }
    } else {
        // no parent: write directly
        fs::write(target, data)
    }
}

/// Copy a single file atomically: copy into a temp file in the destination
/// directory then rename into place.
pub fn atomic_copy_file(src: &std::path::Path, dst: &std::path::Path) -> io::Result<u64> {
    if let Some(dir) = dst.parent() {
        fs::create_dir_all(dir)?;
        let mut tmp = dir.join(".tmp_atomic_copy");
        // Use a time+pid+thread+seq-based suffix to avoid collisions when many
        // copies run concurrently on multiple threads.
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let pid = std::process::id() as u128;
        // Hash the current thread id so the suffix includes a per-thread
        // component without relying on unstable ThreadId APIs.
        let thread_id = format!("{:?}", std::thread::current().id());
        let mut hasher = DefaultHasher::new();
        thread_id.hash(&mut hasher);
        let thread_hash = hasher.finish();
        // Global sequential id to guarantee uniqueness across rapid calls
        // even if timestamp resolution were insufficient.
        static NEXT_COPY_ID: AtomicU64 = AtomicU64::new(0);
        let seq = NEXT_COPY_ID.fetch_add(1, Ordering::Relaxed) as u128;
        let raw = format!("{:x}{:x}{:x}{:x}", pid, nanos, thread_hash, seq);
        let suffix: String = raw
            .chars()
            .rev()
            .take(12)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        tmp.set_file_name(format!(".tmp_atomic_copy.{}", suffix));
        // Copy into temp file then rename. Clean up temp file on error.
        // Use fs_extra file copy for potentially better performance/options
        let mut options = CopyOptions::new();
        options.overwrite = false;
        // Use a 64 KiB buffer for file copies to balance throughput and memory.
        options.buffer_size = 64 * 1024;
        match fs_extra_copy(src, &tmp, &options).map_err(|e| io::Error::new(io::ErrorKind::Other, e)) {
            Ok(n) => {
                // In tests we may force a rename failure to verify cleanup.
                {
                    if crate::fs_op::helpers::tests::should_force_rename_fail_in_copy() {
                        let _ = fs::remove_file(&tmp);
                        return Err(io::Error::other("forced rename failure (copy)"));
                    }
                }
                match fs::rename(&tmp, dst) {
                    Ok(()) => {
                        // Preserve metadata (permissions/timestamps) from src -> dst.
                        let _ = crate::fs_op::metadata::preserve_all_metadata(src, dst);
                        Ok(n)
                    }
                    Err(e) => {
                        let _ = fs::remove_file(&tmp);
                        Err(e)
                    }
                }
            }
            Err(e) => {
                let _ = fs::remove_file(&tmp);
                Err(e)
            }
        }
    } else {
        let mut options = CopyOptions::new();
        options.overwrite = false;
        options.buffer_size = 64 * 1024;
        let res = fs_extra_copy(src, dst, &options).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        // Best-effort: preserve timestamps and permissions on the final dst
        let _ = crate::fs_op::metadata::preserve_all_metadata(src, dst);
        Ok(res)
    }
}

/// Try to rename `src` to `dst`. If `rename` fails due to cross-filesystem
/// issues, fall back to an atomic copy+remove approach.
pub fn atomic_rename_or_copy(src: &std::path::Path, dst: &std::path::Path) -> io::Result<()> {
    // Allow tests to force the rename path to fail so we exercise the
    // fallback copy+remove behavior.
    if crate::fs_op::helpers::tests::should_force_rename_fail_in_rename_or_copy() {
        let _ = atomic_copy_file(src, dst)?;
        fs::remove_file(src)?;
        return Ok(());
    }

    // If the source is a directory, delegate to the move_path helper which
    // performs a recursive, fs_extra-backed copy+remove fallback when a
    // simple rename is not possible (cross-device moves, etc.). This keeps
    // semantics consistent between file and directory moves.
    if src.is_dir() {
        match fs::rename(src, dst) {
            Ok(_) => return Ok(()),
            Err(_e) => {
                // Use the mv helper which returns its own error type; map to io::Error
                return match crate::fs_op::mv::move_path(src, dst) {
                    Ok(()) => Ok(()),
                    Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
                };
            }
        }
    }

    // For regular files, try a rename first and fall back to an atomic copy
    // into the destination directory then remove the source file.
    match fs::rename(src, dst) {
        Ok(_) => Ok(()),
        Err(_e) => {
            // On cross-device link error or other errors, attempt copy then remove
            // If `atomic_copy_file` fails it will attempt to clean up its temp file.
            let _ = atomic_copy_file(src, dst)?;
            fs::remove_file(src)?;
            // ignore original error and return Ok
            Ok(())
        }
    }
}

/// Test-only hooks used by unit tests to force failure paths and ensure
/// temporary-file cleanup behaves correctly. These are gated behind the
/// non-default `test-helpers` cargo feature so production builds do not
/// include test-only code. A no-op fallback is provided when the feature
/// is not enabled so calls from the main code remain compilable.
#[cfg(feature = "test-helpers")]
pub mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Mutex, OnceLock};

    static FORCE_RENAME_FAIL_IN_COPY: AtomicBool = AtomicBool::new(false);
    static FORCE_RENAME_FAIL_IN_WRITE: AtomicBool = AtomicBool::new(false);
    static FORCE_RENAME_FAIL_IN_RENAME_OR_COPY: AtomicBool = AtomicBool::new(false);

    pub fn set_force_rename_fail_in_copy(v: bool) {
        FORCE_RENAME_FAIL_IN_COPY.store(v, Ordering::SeqCst);
    }
    pub fn should_force_rename_fail_in_copy() -> bool {
        FORCE_RENAME_FAIL_IN_COPY.load(Ordering::SeqCst)
    }

    pub fn set_force_rename_fail_in_write(v: bool) {
        FORCE_RENAME_FAIL_IN_WRITE.store(v, Ordering::SeqCst);
    }
    pub fn should_force_rename_fail_in_write() -> bool {
        FORCE_RENAME_FAIL_IN_WRITE.load(Ordering::SeqCst)
    }

    pub fn set_force_rename_fail_in_rename_or_copy(v: bool) {
        FORCE_RENAME_FAIL_IN_RENAME_OR_COPY.store(v, Ordering::SeqCst);
    }
    pub fn should_force_rename_fail_in_rename_or_copy() -> bool {
        FORCE_RENAME_FAIL_IN_RENAME_OR_COPY.load(Ordering::SeqCst)
    }

    static TEST_HOOK_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

    /// Acquire a global test lock to serialize tests that mutate the global
    /// test hooks. This returns a guard that will be dropped at the end of
    /// the test scope.
    pub fn acquire_test_lock() -> std::sync::MutexGuard<'static, ()> {
        TEST_HOOK_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap()
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

#[cfg(not(feature = "test-helpers"))]
pub mod tests {
    /// No-op setters and conservative getters so production builds do not
    /// include test-only behavior but internal calls remain valid.
    pub fn set_force_rename_fail_in_copy(_v: bool) {}
    pub fn should_force_rename_fail_in_copy() -> bool {
        false
    }

    pub fn set_force_rename_fail_in_write(_v: bool) {}
    pub fn should_force_rename_fail_in_write() -> bool {
        false
    }

    pub fn set_force_rename_fail_in_rename_or_copy(_v: bool) {}
    pub fn should_force_rename_fail_in_rename_or_copy() -> bool {
        false
    }

    pub fn acquire_test_lock() -> std::sync::MutexGuard<'static, ()> {
        use std::sync::{Mutex, OnceLock};
        static DUMMY: OnceLock<Mutex<()>> = OnceLock::new();
        DUMMY.get_or_init(|| Mutex::new(())).lock().unwrap()
    }
}
