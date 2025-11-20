use anyhow::{Context, Result};
use std::fs;
use super::write_settings::{config_file_path, Settings};

/// Load settings from disk. If file doesn't exist, returns Default::default().
pub fn load_settings() -> Result<Settings> {
    let path = config_file_path()?;
    if !path.exists() {
        return Ok(Settings::default());
    }
    let bytes = fs::read_to_string(&path)
        .with_context(|| format!("failed to read settings file {}", path.display()))?;
    let s: Settings = toml::from_str(&bytes)
        .with_context(|| format!("failed to parse settings TOML in {}", path.display()))?;
    Ok(s)
}