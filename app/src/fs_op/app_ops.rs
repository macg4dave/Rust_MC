use std::fs;
use std::io;
use std::path::PathBuf;

impl crate::app::core::App {
    pub fn enter(&mut self) -> io::Result<()> {
        let panel = self.active_panel_mut();
        let sel = panel.selected;
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
        let panel = self.active_panel_mut();
        let sel = panel.selected;
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
        let panel = self.active_panel_mut();
        let sel = panel.selected;
        if let Some(src) = panel.entries.get(sel) {
            let src_path = src.path.clone();
            let src_name = src.name.clone();
            let is_dir = src.is_dir;
            let target = crate::app::core::App::resolve_target(&dst, &src_name);
            if is_dir {
                self.copy_recursive(&src_path, &target)?;
            } else {
                crate::app::core::App::ensure_parent_exists(&target)?;
                fs::copy(&src_path, &target)?;
            }
            self.refresh_active()?;
        }
        Ok(())
    }

    pub fn move_selected_to(&mut self, dst: PathBuf) -> io::Result<()> {
        let panel = self.active_panel_mut();
        let sel = panel.selected;
        if let Some(src) = panel.entries.get(sel) {
            let src_path = src.path.clone();
            let src_name = src.name.clone();
            let target = crate::app::core::App::resolve_target(&dst, &src_name);
            crate::app::core::App::ensure_parent_exists(&target)?;
            fs::rename(&src_path, &target)?;
            self.refresh_active()?;
        }
        Ok(())
    }

    pub fn rename_selected_to(&mut self, name: String) -> io::Result<()> {
        let panel = self.active_panel_mut();
        let sel = panel.selected;
        if let Some(src) = panel.entries.get(sel) {
            let src_path = src.path.clone();
            let target = panel.cwd.join(name);
            fs::rename(&src_path, &target)?;
            self.refresh_active()?;
        }
        Ok(())
    }

    pub fn new_file(&mut self, name: String) -> io::Result<()> {
        let panel = self.active_panel_mut();
        let p = panel.cwd.join(name);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::File::create(p)?;
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

    fn copy_recursive(&self, src: &PathBuf, dst: &PathBuf) -> io::Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let child_src = entry.path();
            let child_dst = dst.join(file_name);
            if child_src.is_dir() {
                self.copy_recursive(&child_src, &child_dst)?;
            } else {
                if let Some(p) = child_dst.parent() {
                    fs::create_dir_all(p)?;
                }
                fs::copy(&child_src, &child_dst)?;
            }
        }
        Ok(())
    }
}
