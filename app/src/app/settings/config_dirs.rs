use std::path::PathBuf;

use anyhow::Result;
use directories_next::ProjectDirs;

/// Helpers for locating and creating config/cache directories for fileZoom.
///
/// This follows platform conventions via `directories-next` and falls back
/// to `$HOME/.filezoom` when `ProjectDirs` is not available.
pub fn project_config_dir() -> PathBuf {
    if let Some(dirs) = ProjectDirs::from("com", "macg4dave", "fileZoom") {
        dirs.config_dir().to_path_buf()
    } else {
        let mut p = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        p.push(".filezoom");
        p
    }
}

/// Path for user cache directory for fileZoom.
pub fn user_cache_dir() -> PathBuf {
    if let Some(dirs) = ProjectDirs::from("com", "macg4dave", "fileZoom") {
        dirs.cache_dir().to_path_buf()
    } else {
        let mut p = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        p.push(".cache");
        p.push("filezoom");
        p
    }
}

/// Ensure config and cache directories exist. Creates any missing directories.
pub fn ensure_dirs_exist() -> Result<()> {
    let cfg = project_config_dir();
    std::fs::create_dir_all(&cfg)?;
    let cache = user_cache_dir();
    std::fs::create_dir_all(&cache)?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::env;
    use std::fs;

    #[test]
    fn ensure_dirs_creates_dirs_with_home_fallback() -> Result<(), Box<dyn std::error::Error>> {
        let td = tempdir()?;
        env::set_var("HOME", td.path());

        let cfg = project_config_dir();
        let cache = user_cache_dir();

        if cfg.exists() { fs::remove_dir_all(&cfg)?; }
        if cache.exists() { fs::remove_dir_all(&cache)?; }

        ensure_dirs_exist()?;

        assert!(cfg.exists(), "config dir should exist");
        assert!(cache.exists(), "cache dir should exist");

        Ok(())
    }
}
