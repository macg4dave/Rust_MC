#[cfg(feature = "test-helpers")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "test-helpers")]
use std::sync::{Mutex, OnceLock};

/// Test-only hooks used by unit tests to force failure paths and ensure
/// temporary-file cleanup behaves correctly. These functions exist only
/// when the `test-helpers` feature is enabled; when disabled they are
/// provided as stable no-op fallbacks below.
#[cfg(feature = "test-helpers")]
pub fn set_force_rename_fail_in_copy(v: bool) {
    static FORCE_RENAME_FAIL_IN_COPY: AtomicBool = AtomicBool::new(false);
    FORCE_RENAME_FAIL_IN_COPY.store(v, Ordering::SeqCst);
}
#[cfg(feature = "test-helpers")]
pub fn should_force_rename_fail_in_copy() -> bool {
    static FORCE_RENAME_FAIL_IN_COPY: AtomicBool = AtomicBool::new(false);
    FORCE_RENAME_FAIL_IN_COPY.load(Ordering::SeqCst)
}

#[cfg(feature = "test-helpers")]
pub fn set_force_rename_fail_in_write(v: bool) {
    static FORCE_RENAME_FAIL_IN_WRITE: AtomicBool = AtomicBool::new(false);
    FORCE_RENAME_FAIL_IN_WRITE.store(v, Ordering::SeqCst);
}
#[cfg(feature = "test-helpers")]
pub fn should_force_rename_fail_in_write() -> bool {
    static FORCE_RENAME_FAIL_IN_WRITE: AtomicBool = AtomicBool::new(false);
    FORCE_RENAME_FAIL_IN_WRITE.load(Ordering::SeqCst)
}

#[cfg(feature = "test-helpers")]
pub fn set_force_rename_fail_in_rename_or_copy(v: bool) {
    static FORCE_RENAME_FAIL_IN_RENAME_OR_COPY: AtomicBool = AtomicBool::new(false);
    FORCE_RENAME_FAIL_IN_RENAME_OR_COPY.store(v, Ordering::SeqCst);
}
#[cfg(feature = "test-helpers")]
pub fn should_force_rename_fail_in_rename_or_copy() -> bool {
    static FORCE_RENAME_FAIL_IN_RENAME_OR_COPY: AtomicBool = AtomicBool::new(false);
    FORCE_RENAME_FAIL_IN_RENAME_OR_COPY.load(Ordering::SeqCst)
}

#[cfg(feature = "test-helpers")]
pub fn acquire_test_lock() -> std::sync::MutexGuard<'static, ()> {
    static TEST_HOOK_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_HOOK_MUTEX
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap()
}

// No-op fallbacks when the feature is not active.
#[cfg(not(feature = "test-helpers"))]
pub fn set_force_rename_fail_in_copy(_v: bool) {}
#[cfg(not(feature = "test-helpers"))]
pub fn should_force_rename_fail_in_copy() -> bool {
    false
}

#[cfg(not(feature = "test-helpers"))]
pub fn set_force_rename_fail_in_write(_v: bool) {}
#[cfg(not(feature = "test-helpers"))]
pub fn should_force_rename_fail_in_write() -> bool {
    false
}

#[cfg(not(feature = "test-helpers"))]
pub fn set_force_rename_fail_in_rename_or_copy(_v: bool) {}
#[cfg(not(feature = "test-helpers"))]
pub fn should_force_rename_fail_in_rename_or_copy() -> bool {
    false
}

#[cfg(not(feature = "test-helpers"))]
pub fn acquire_test_lock() -> std::sync::MutexGuard<'static, ()> {
    use std::sync::{Mutex, OnceLock};
    static DUMMY: OnceLock<Mutex<()>> = OnceLock::new();
    DUMMY.get_or_init(|| Mutex::new(())).lock().unwrap()
}
