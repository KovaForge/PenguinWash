//! Configuration loading and saving
//!
//! Config file: ~/.config/penguinwash.toml

use crate::Config;
use anyhow::Result;
use std::path::PathBuf;

/// Path to the config file
pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("penguinwash.toml")
}

/// Load config from ~/.config/penguinwash.toml
pub fn load_config() -> Result<Config> {
    let path = config_path();
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = std::fs::read_to_string(&path)?;
    toml::from_str(&content).map_err(Into::into)
}

/// Save config to ~/.config/penguinwash.toml
pub fn save_config(config: &Config) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}
