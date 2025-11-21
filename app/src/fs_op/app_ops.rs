//! High-level filesystem operations implemented on `App`.
//!
//! This module provides convenient, UI-aware file operations (entering
//! directories, navigation, creating/removing files and directories,
//! and copying/moving/renaming selected entries). The actual filesystem
//! primitives and atomic helpers live in `crate::fs_op::helpers` and
//! `crate::fs_op::copy` so the behaviour is reusable and easily tested.

use std::fs;
use std::path::{Path, PathBuf};
use crate::fs_op::error::FsOpError;

impl crate::app::core::App {
    /// Enter the selected directory (if any) by updating the active
    /// panel's `cwd` and refreshing the panel listing.
    pub fn enter(&mut self) -> Result<(), FsOpError> {
        if let Some(sel) = self.selected_index() {
            let panel = self.active_panel_mut();
            if let Some(entry) = panel.entries.get(sel) {
                if entry.is_dir {
                    panel.cwd = entry.path.clone();
                    self.refresh_active()?;
                }
            }
        }
        Ok(())
    }

    /// Move the active panel up to its parent directory (if any) and
    /// refresh the listing.
    pub fn go_up(&mut self) -> Result<(), FsOpError> {
        let panel = self.active_panel_mut();
        if let Some(parent) = panel.cwd.parent() {
            panel.cwd = parent.to_path_buf();
            self.refresh_active()?;
        }
        Ok(())
    }

    /// Delete the currently selected entry (file or directory).
    pub fn delete_selected(&mut self) -> Result<(), FsOpError> {
        if let Some(sel) = self.selected_index() {
            let panel = self.active_panel_mut();
            if let Some(entry) = panel.entries.get(sel) {
                if entry.is_dir {
                    fs::remove_dir_all(&entry.path)?;
                } else {
                    fs::remove_file(&entry.path)?;
                }
                self.refresh_active()?;
            }
        }
        Ok(())
    }

    /// Copy the selected entry to `dst`.
    ///
    /// If the selection is a directory, performs a recursive copy. For
    /// regular files an atomic file-copy helper is used.
    pub fn copy_selected_to(&mut self, dst: PathBuf) -> Result<(), FsOpError> {
        if let Some(sel) = self.selected_index() {
            let panel = self.active_panel_mut();
            if let Some(src_entry) = panel.entries.get(sel) {
                let src_path: &Path = &src_entry.path;
                let src_name: &str = &src_entry.name;
                let target = crate::fs_op::helpers::resolve_target(&dst, src_name);
                if src_entry.is_dir {
                    // Call into the shared copy implementation directly so we
                    // avoid borrowing `self` immutably while a mutable borrow
                    // to the panel is still active.
                    crate::fs_op::copy::copy_recursive(src_path, &target)?;
                } else {
                    crate::fs_op::helpers::ensure_parent_exists(&target)?;
                    crate::fs_op::helpers::atomic_copy_file(src_path, &target)?;
                }
                self.refresh_active()?;
            }
        }
        Ok(())
    }

    /// Move (rename) the selected entry to `dst`.
    ///
    /// Attempts an atomic rename and falls back to copy+remove when
    /// required (e.g. cross-filesystem moves).
    pub fn move_selected_to(&mut self, dst: PathBuf) -> Result<(), FsOpError> {
        if let Some(sel) = self.selected_index() {
            let panel = self.active_panel_mut();
            if let Some(src_entry) = panel.entries.get(sel) {
                let src_path: &Path = &src_entry.path;
                let src_name: &str = &src_entry.name;
                let target = crate::fs_op::helpers::resolve_target(&dst, src_name);
                crate::fs_op::helpers::ensure_parent_exists(&target)?;
                crate::fs_op::helpers::atomic_rename_or_copy(src_path, &target)?;
                self.refresh_active()?;
            }
        }
        Ok(())
    }

    /// Rename the selected entry to `name` within the same directory.
    pub fn rename_selected_to(&mut self, name: String) -> Result<(), FsOpError> {
        if let Some(sel) = self.selected_index() {
            let panel = self.active_panel_mut();
            if let Some(src_entry) = panel.entries.get(sel) {
                let src_path: &Path = &src_entry.path;
                let target = panel.cwd.join(name);
                crate::fs_op::helpers::atomic_rename_or_copy(src_path, &target)?;
                self.refresh_active()?;
            }
        }
        Ok(())
    }

    /// Create a new empty file with `name` in the active panel.
    ///
    /// The file is written atomically to avoid races with concurrent
    /// readers; parent directories are created as needed.
    pub fn new_file(&mut self, name: String) -> Result<(), FsOpError> {
        let panel = self.active_panel_mut();
        let path = panel.cwd.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        crate::fs_op::helpers::atomic_write(&path, &[])?;
        self.refresh_active()?;
        Ok(())
    }

    /// Create a new directory named `name` inside the active panel's cwd.
    pub fn new_dir(&mut self, name: String) -> Result<(), FsOpError> {
        let panel = self.active_panel_mut();
        let path = panel.cwd.join(name);
        fs::create_dir_all(path)?;
        self.refresh_active()?;
        Ok(())
    }

    // Note: We delegate recursive copy directly to `crate::fs_op::copy`
    // instead of forwarding through an `&self` method to avoid borrow
    // conflicts when a mutable borrow to a panel is active.
}

#[cfg(test)]
mod tests {
    use std::fs as stdfs;
    use tempfile::tempdir;

    #[test]
    fn new_file_creates_file_and_refreshes_panel() {
        let tmp = tempdir().expect("tempdir");
        let cwd = tmp.path().to_path_buf();
        let opts = crate::app::StartOptions { start_dir: Some(cwd.clone()), ..Default::default() };
        let mut app = crate::app::core::App::with_options(&opts).expect("with_options");
        // create a new file
        app.new_file("created.txt".to_string()).expect("new_file");
        let p = cwd.join("created.txt");
        assert!(p.exists(), "expected new file to exist");
        // cleanup
        let _ = stdfs::remove_file(p);
    }

    #[test]
    fn delete_selected_removes_file() {
        let tmp = tempdir().expect("tempdir");
        let cwd = tmp.path().to_path_buf();
        // create a file on disk
        let file_path = tmp.path().join("to_delete.txt");
        stdfs::write(&file_path, "x").expect("write file");

        let opts = crate::app::StartOptions { start_dir: Some(cwd.clone()), ..Default::default() };
        let mut app = crate::app::core::App::with_options(&opts).expect("with_options");
        // refresh so the entry appears in the panel listing
        app.refresh().expect("refresh");

        // Find the entry index in panel.entries
        let entry_index = app
            .active_panel()
            .entries
            .iter()
            .position(|e| e.name == "to_delete.txt")
            .expect("entry present");

        // Compute the UI-selected index (header + optional parent + entry_index)
        let parent_rows = app.active_panel().cwd.parent().is_some() as usize;
        let ui_index = 1 + parent_rows + entry_index;
        // Set selection and delete
        app.active_panel_mut().selected = ui_index;
        app.delete_selected().expect("delete_selected");
        assert!(!file_path.exists(), "expected file removed");
    }

    #[cfg(feature = "test-helpers")]
    #[test]
    fn move_falls_back_to_copy_and_remove_when_rename_forced_to_fail() {
        use crate::fs_op::test_helpers as helpers_tests;

        let _lock = helpers_tests::acquire_test_lock();
        // Force rename to fail so atomic_rename_or_copy performs copy+remove.
        helpers_tests::set_force_rename_fail_in_rename_or_copy(true);

        let tmp = tempdir().expect("tempdir");
        let cwd = tmp.path().to_path_buf();

        // create a source file
        let src = tmp.path().join("mv_force.txt");
        stdfs::write(&src, "content").expect("write src");

        let opts = crate::app::StartOptions { start_dir: Some(cwd.clone()), ..Default::default() };
        let mut app = crate::app::core::App::with_options(&opts).expect("with_options");
        app.refresh().expect("refresh");

        // select the created file
        let idx = app.left.entries.iter().position(|e| e.name == "mv_force.txt").expect("entry present");
        let parent_rows = app.left.cwd.parent().is_some() as usize;
        app.left.selected = 1 + parent_rows + idx;

        let dest_dir = tmp.path().join("dest_force");
        stdfs::create_dir_all(&dest_dir).expect("mkdir");

        // perform move (which will be forced to copy+remove)
        app.move_selected_to(dest_dir.clone()).expect("move_selected_to");

        // verify moved file exists at destination and not at source
        let dest_file = dest_dir.join("mv_force.txt");
        assert!(dest_file.exists(), "expected dest file present");
        assert!(!src.exists(), "expected source removed");

        // cleanup: reset test hook
        helpers_tests::set_force_rename_fail_in_rename_or_copy(false);
    }

    #[cfg(feature = "test-helpers")]
    #[test]
    fn rename_falls_back_to_copy_when_rename_forced_to_fail() {
        use crate::fs_op::test_helpers as helpers_tests;

        let _lock = helpers_tests::acquire_test_lock();
        helpers_tests::set_force_rename_fail_in_rename_or_copy(true);

        let tmp = tempdir().expect("tempdir");
        let cwd = tmp.path().to_path_buf();
        let src = tmp.path().join("rnm_force.txt");
        stdfs::write(&src, "x").expect("write");

        let opts = crate::app::StartOptions { start_dir: Some(cwd.clone()), ..Default::default() };
        let mut app = crate::app::core::App::with_options(&opts).expect("with_options");
        app.refresh().expect("refresh");

        let idx = app.left.entries.iter().position(|e| e.name == "rnm_force.txt").expect("entry");
        let parent_rows = app.left.cwd.parent().is_some() as usize;
        app.left.selected = 1 + parent_rows + idx;

        app.rename_selected_to("rnm_forced_renamed.txt".to_string()).expect("rename");

        assert!(tmp.path().join("rnm_forced_renamed.txt").exists());
        assert!(!src.exists());

        helpers_tests::set_force_rename_fail_in_rename_or_copy(false);
    }
}
