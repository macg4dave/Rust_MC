use anyhow::{Context, Result};
use quick_xml::de::from_str;
use std::fs;

use super::write_settings::{config_file_path, Settings};

/// Validate settings XML by deserializing into Settings.
pub fn validate_settings_xml() -> Result<()> {
    let path = config_file_path()?;
    if !path.exists() {
        anyhow::bail!("settings file not found: {}", path.display());
    }
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("failed to read settings file {}", path.display()))?;
    let _s: Settings = from_str(&contents)
        .with_context(|| format!("failed to parse settings XML in {}", path.display()))?;
    Ok(())
}