use std::fs;
use std::io;
use std::path::PathBuf;

/// Resolve destination path for an operation: if `dst` looks like a directory
/// (exists or ends with a separator) then target becomes `dst.join(src_name)`.
///
/// Kept as a small, dependency-free helper in `fs_op` so filesystem helpers
/// live together and can be tested independently of `App`.
pub fn resolve_target(dst: &PathBuf, src_name: &str) -> PathBuf {
    if dst.is_dir() || dst.to_string_lossy().ends_with('/') {
        dst.join(src_name)
    } else {
        dst.clone()
    }
}

/// Ensure parent directory exists for a path.
pub fn ensure_parent_exists(p: &PathBuf) -> io::Result<()> {
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Write `data` to `target` atomically by writing to a temporary file in the
/// same directory and then renaming into place. This avoids partial writes
/// being observed by other processes.
pub fn atomic_write(target: &PathBuf, data: &[u8]) -> io::Result<()> {
    if let Some(dir) = target.parent() {
        fs::create_dir_all(dir)?;
        let mut tmp = dir.join(".tmp_atomic_write");
        // use a randomized suffix to avoid collisions
        use rand::{distributions::Alphanumeric, Rng};
        let suffix: String = rand::thread_rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect();
        tmp.set_file_name(format!(".tmp_atomic_write.{}", suffix));
        // Write to the temp file first. If any step fails we attempt to
        // remove the temp file to avoid leaving artifacts.
        match fs::write(&tmp, data) {
            Ok(()) => {
                // In tests we may want to force the rename to fail to ensure
                // temp-file cleanup is exercised.
                #[cfg(test)]
                {
                    if crate::fs_op::helpers::tests::should_force_rename_fail_in_write() {
                        let _ = fs::remove_file(&tmp);
                        return Err(io::Error::new(io::ErrorKind::Other, "forced rename failure (write)"));
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
pub fn atomic_copy_file(src: &PathBuf, dst: &PathBuf) -> io::Result<u64> {
    if let Some(dir) = dst.parent() {
        fs::create_dir_all(dir)?;
        let mut tmp = dir.join(".tmp_atomic_copy");
        use rand::{distributions::Alphanumeric, Rng};
        let suffix: String = rand::thread_rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect();
        tmp.set_file_name(format!(".tmp_atomic_copy.{}", suffix));
        // Copy into temp file then rename. Clean up temp file on error.
        match fs::copy(src, &tmp) {
            Ok(n) => {
                // In tests we may force a rename failure to verify cleanup.
                #[cfg(test)]
                {
                    if crate::fs_op::helpers::tests::should_force_rename_fail_in_copy() {
                        let _ = fs::remove_file(&tmp);
                        return Err(io::Error::new(io::ErrorKind::Other, "forced rename failure (copy)"));
                    }
                }
                match fs::rename(&tmp, dst) {
                    Ok(()) => Ok(n),
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
        fs::copy(src, dst)
    }
}

/// Try to rename `src` to `dst`. If `rename` fails due to cross-filesystem
/// issues, fall back to an atomic copy+remove approach.
pub fn atomic_rename_or_copy(src: &PathBuf, dst: &PathBuf) -> io::Result<()> {
    // Allow tests to force the rename path to fail so we exercise the
    // fallback copy+remove behavior.
    #[cfg(test)]
    if crate::fs_op::helpers::tests::should_force_rename_fail_in_rename_or_copy() {
        let _ = atomic_copy_file(src, dst)?;
        fs::remove_file(src)?;
        return Ok(());
    }

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
/// temporary-file cleanup behaves correctly. These are only compiled in
/// test builds and do not affect production behavior.
#[cfg(test)]
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
        TEST_HOOK_MUTEX.get_or_init(|| Mutex::new(())).lock().unwrap()
    }
}
