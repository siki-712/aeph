use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Enable auto-pairing for brackets and quotes
    #[serde(default = "default_true")]
    pub auto_pair: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self { auto_pair: true }
    }
}

fn get_project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "ephe", "aeph")
}

fn get_data_dir() -> Result<PathBuf> {
    get_project_dirs()
        .map(|dirs| dirs.data_dir().to_path_buf())
        .context("Failed to determine data directory")
}

pub fn get_config_path() -> Option<PathBuf> {
    get_project_dirs().map(|dirs| dirs.config_dir().join("config.toml"))
}

/// Load configuration from file, or return default if not found
pub fn load_config() -> Config {
    get_config_path()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

/// Save configuration to file
#[allow(dead_code)]
pub fn save_config(config: &Config) -> Result<()> {
    let path = get_config_path().context("Failed to determine config path")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)?;
    fs::write(&path, content)?;
    Ok(())
}

pub fn get_paper_path(index: usize) -> Result<PathBuf> {
    let data_dir = get_data_dir()?;
    fs::create_dir_all(&data_dir)
        .with_context(|| format!("Failed to create data dir: {}", data_dir.display()))?;
    Ok(data_dir.join(format!("paper{}.md", index)))
}

fn get_session_path() -> Result<PathBuf> {
    let data_dir = get_data_dir()?;
    fs::create_dir_all(&data_dir)?;
    Ok(data_dir.join("session.json"))
}

/// Session data persisted between app launches
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Session {
    #[serde(default)]
    pub last_doc: usize,
    #[serde(default)]
    pub grid_style: u8,
    #[serde(default)]
    pub text_align: u8,
}

/// Save session data
pub fn save_session(session: &Session) -> Result<()> {
    let path = get_session_path()?;
    let content = serde_json::to_string(session)?;
    fs::write(&path, content)?;
    Ok(())
}

/// Load session data
pub fn load_session() -> Session {
    get_session_path()
        .ok()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

