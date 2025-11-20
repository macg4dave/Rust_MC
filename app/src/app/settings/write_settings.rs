use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// User-editable settings persisted to a TOML file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub theme: String,
    pub show_hidden: bool,
    pub left_panel_width: u16,
    pub right_panel_width: u16,
    /// Ordered list of context actions shown in the context menu.
    pub context_actions: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: "default".into(),
            show_hidden: false,
            left_panel_width: 40,
            right_panel_width: 40,
            context_actions: vec!["View".to_string(), "Edit".to_string(), "Permissions".to_string(), "Cancel".to_string()],
        }
    }
}

/// Compute the config file path using XDG_CONFIG_HOME or fallback to $HOME/.config/fileZoom/settings.toml
pub fn config_file_path() -> Result<PathBuf> {
    if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
        let mut p = PathBuf::from(xdg);
        p.push("fileZoom");
        p.push("settings.toml");
        return Ok(p);
    }

    // fallback to $HOME/.config/fileZoom/settings.toml
    let home = env::var("HOME").context("HOME not set; cannot determine config directory")?;
    let mut p = PathBuf::from(home);
    p.push(".config");
    p.push("fileZoom");
    p.push("settings.toml");
    Ok(p)
}

/// Save settings to disk (creates parent directory if needed).
pub fn save_settings(settings: &Settings) -> Result<()> {
    let path = config_file_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config dir {}", parent.display()))?;
    }
    let s = toml::to_string_pretty(settings).context("failed to serialize settings to TOML")?;
    let mut file = fs::File::create(&path)
        .with_context(|| format!("failed to create settings file {}", path.display()))?;
    file.write_all(s.as_bytes())
        .with_context(|| format!("failed to write settings to {}", path.display()))?;
    Ok(())
}