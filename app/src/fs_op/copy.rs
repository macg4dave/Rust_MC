use std::fs;
use std::io;
use std::path::Path;

/// Recursively copy `src` directory to `dst` directory.
///
/// This helper ensures target directories are created as needed and will copy
/// files preserving content. It's intentionally small and uses only the
/// standard library so it stays dependency-free and easy to test.
pub(crate) fn copy_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let child_src = entry.path();
        let child_dst = dst.join(file_name);
        if child_src.is_dir() {
            copy_recursive(&child_src, &child_dst)?;
            } else {
                if let Some(p) = child_dst.parent() {
                    fs::create_dir_all(p)?;
                }
                // Use atomic copy for file to avoid observing partial files
                crate::fs_op::helpers::atomic_copy_file(&child_src, &child_dst)?;
            }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use std::fs;

    fn find_tmp_files(dir: &std::path::Path, prefix: &str) -> Vec<std::path::PathBuf> {
        let mut found = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with(prefix) {
                    found.push(e.path());
                }
            }
        }
        found
    }

    #[test]
    fn copies_directory_tree() {
        let _guard = crate::fs_op::helpers::tests::acquire_test_lock();
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();

        // Create nested structure: a/b/file.txt and a/c/file2.txt
        let a = src.child("a");
        a.create_dir_all().unwrap();
        a.child("b").create_dir_all().unwrap();
        a.child("b").child("file.txt").write_str("hello").unwrap();
        a.child("c").create_dir_all().unwrap();
        a.child("c").child("file2.txt").write_str("world").unwrap();

        let src_path = src.path().join("a");
        let dst_path = dst.path().join("a_copy");

        copy_recursive(&src_path, &dst_path).expect("copy failed");

        assert!(dst_path.join("b").is_dir());
        assert!(dst_path.join("b").join("file.txt").is_file());
        assert_eq!(fs::read_to_string(dst_path.join("b").join("file.txt")).unwrap(), "hello");
        assert!(dst_path.join("c").join("file2.txt").is_file());
        assert_eq!(fs::read_to_string(dst_path.join("c").join("file2.txt")).unwrap(), "world");

        src.close().unwrap();
        dst.close().unwrap();
    }

    #[test]
    fn atomic_copy_creates_no_tmp_files() {
        let _guard = crate::fs_op::helpers::tests::acquire_test_lock();
        let tmp = TempDir::new().unwrap();
        let src_file = tmp.child("file.txt");
        src_file.write_str("data").unwrap();

        let dst_dir = tmp.child("out");
        dst_dir.create_dir_all().unwrap();
        let _dst = dst_dir.path().join("file.txt");

        // Now do a direct atomic_copy_file test
        let src = tmp.path().join("file.txt");
        let dst = tmp.path().join("file_copy.txt");
        let _ = crate::fs_op::helpers::atomic_copy_file(&src, &dst).expect("atomic copy failed");

        // No `.tmp_atomic_copy.` files should remain in the tmp dir
        let tmps = find_tmp_files(tmp.path(), ".tmp_atomic_copy.");
        assert!(tmps.is_empty(), "found leftover tmp files: {:?}", tmps);
    }

    #[test]
    fn atomic_copy_cleanup_on_forced_rename_failure() {
        let _guard = crate::fs_op::helpers::tests::acquire_test_lock();
        let tmp = TempDir::new().unwrap();
        let src_file = tmp.child("file.txt");
        src_file.write_str("data").unwrap();

        let src = tmp.path().join("file.txt");
        let dst = tmp.path().join("file_copy_fail.txt");

        // Force the rename inside atomic_copy_file to fail and ensure the
        // temporary file is cleaned up.
        crate::fs_op::helpers::tests::set_force_rename_fail_in_copy(true);
        let res = crate::fs_op::helpers::atomic_copy_file(&src, &dst);
        crate::fs_op::helpers::tests::set_force_rename_fail_in_copy(false);

        assert!(res.is_err(), "expected atomic_copy_file to fail due to forced rename");

        // No `.tmp_atomic_copy.` files should remain in the tmp dir
        let tmps = find_tmp_files(tmp.path(), ".tmp_atomic_copy.");
        assert!(tmps.is_empty(), "found leftover tmp files after forced failure: {:?}", tmps);
    }

    #[test]
    fn atomic_write_cleanup_on_forced_rename_failure() {
        let _guard = crate::fs_op::helpers::tests::acquire_test_lock();
        let tmp = TempDir::new().unwrap();
        let dst = tmp.path().join("write_fail.txt");

        crate::fs_op::helpers::tests::set_force_rename_fail_in_write(true);
        let res = crate::fs_op::helpers::atomic_write(&dst, b"hello");
        crate::fs_op::helpers::tests::set_force_rename_fail_in_write(false);

        assert!(res.is_err(), "expected atomic_write to fail due to forced rename");

        // Ensure no .tmp_atomic_write.* files remain
        let tmps = find_tmp_files(tmp.path(), ".tmp_atomic_write.");
        assert!(tmps.is_empty(), "found leftover tmp files after forced write failure: {:?}", tmps);
    }

    #[test]
    fn atomic_rename_or_copy_fallback_and_cleanup() {
        let _guard = crate::fs_op::helpers::tests::acquire_test_lock();
        let tmp = TempDir::new().unwrap();
        let src_file = tmp.child("rfile.txt");
        src_file.write_str("payload").unwrap();

        let src = tmp.path().join("rfile.txt");
        let dst = tmp.path().join("rfile_moved.txt");

        // Force the initial rename to be treated as failing so fallback is used
        crate::fs_op::helpers::tests::set_force_rename_fail_in_rename_or_copy(true);
        // Also ensure the copy path succeeds (do not force copy rename failure)
        let res = crate::fs_op::helpers::atomic_rename_or_copy(&src, &dst);
        crate::fs_op::helpers::tests::set_force_rename_fail_in_rename_or_copy(false);

        assert!(res.is_ok(), "expected fallback copy+remove to succeed");

        // After fallback move, source should be removed and dest should exist
        assert!(!src.exists(), "source should have been removed after fallback");
        assert!(dst.exists(), "destination should exist after fallback copy");

        // Ensure no leftover tmp files
        let tmps_write = find_tmp_files(tmp.path(), ".tmp_atomic_write.");
        let tmps_copy = find_tmp_files(tmp.path(), ".tmp_atomic_copy.");
        assert!(tmps_write.is_empty() && tmps_copy.is_empty(), "found leftover tmp files after rename_or_copy fallback: {:?} {:?}", tmps_write, tmps_copy);
    }
}
