use std::fs;
use std::io;
use std::path::PathBuf;

impl crate::app::core::App {
    pub fn enter(&mut self) -> io::Result<()> {
        let sel = self.selected_index();
        let panel = self.active_panel_mut();
        if let Some(e) = panel.entries.get(sel) {
            if e.is_dir {
                panel.cwd = e.path.clone();
                self.refresh_active()?;
            }
        }
        Ok(())
    }

    pub fn go_up(&mut self) -> io::Result<()> {
        let panel = self.active_panel_mut();
        if let Some(parent) = panel.cwd.parent() {
            panel.cwd = parent.to_path_buf();
            self.refresh_active()?;
        }
        Ok(())
    }

    pub fn delete_selected(&mut self) -> io::Result<()> {
        let sel = self.selected_index();
        let panel = self.active_panel_mut();
        if let Some(e) = panel.entries.get(sel) {
            if e.is_dir {
                fs::remove_dir_all(&e.path)?;
            } else {
                fs::remove_file(&e.path)?;
            }
            self.refresh_active()?;
        }
        Ok(())
    }

    pub fn copy_selected_to(&mut self, dst: PathBuf) -> io::Result<()> {
        let sel = self.selected_index();
        let panel = self.active_panel_mut();
        if let Some(src) = panel.entries.get(sel) {
            let src_path = src.path.clone();
            let src_name = src.name.clone();
            let is_dir = src.is_dir;
            let target = crate::fs_op::helpers::resolve_target(&dst, &src_name);
            if is_dir {
                self.copy_recursive(&src_path, &target)?;
            } else {
                crate::fs_op::helpers::ensure_parent_exists(&target)?;
                // Use atomic copy for files
                crate::fs_op::helpers::atomic_copy_file(&src_path, &target)?;
            }
            self.refresh_active()?;
        }
        Ok(())
    }

    pub fn move_selected_to(&mut self, dst: PathBuf) -> io::Result<()> {
        let sel = self.selected_index();
        let panel = self.active_panel_mut();
        if let Some(src) = panel.entries.get(sel) {
            let src_path = src.path.clone();
            let src_name = src.name.clone();
            let target = crate::fs_op::helpers::resolve_target(&dst, &src_name);
            crate::fs_op::helpers::ensure_parent_exists(&target)?;
            // Prefer atomic rename; fall back to copy+remove if necessary
            crate::fs_op::helpers::atomic_rename_or_copy(&src_path, &target)?;
            self.refresh_active()?;
        }
        Ok(())
    }

    pub fn rename_selected_to(&mut self, name: String) -> io::Result<()> {
        let sel = self.selected_index();
        let panel = self.active_panel_mut();
        if let Some(src) = panel.entries.get(sel) {
            let src_path = src.path.clone();
            let target = panel.cwd.join(name);
            // Prefer atomic rename; fall back to copy+remove when needed
            crate::fs_op::helpers::atomic_rename_or_copy(&src_path, &target)?;
            self.refresh_active()?;
        }
        Ok(())
    }

    pub fn new_file(&mut self, name: String) -> io::Result<()> {
        let panel = self.active_panel_mut();
        let p = panel.cwd.join(name);
        // Create parent and write an empty file atomically to avoid races.
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        crate::fs_op::helpers::atomic_write(&p, &[])?;
        self.refresh_active()?;
        Ok(())
    }

    pub fn new_dir(&mut self, name: String) -> io::Result<()> {
        let panel = self.active_panel_mut();
        let p = panel.cwd.join(name);
        fs::create_dir_all(p)?;
        self.refresh_active()?;
        Ok(())
    }

    fn copy_recursive(&self, src: &std::path::Path, dst: &std::path::Path) -> io::Result<()> {
        // Delegate to shared helper in `fs_op::copy` so the logic is
        // reusable and unit-testable without constructing a full `App`.
        crate::fs_op::copy::copy_recursive(src, dst)
    }
}
