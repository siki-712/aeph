use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

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

/// Save the last opened document index
pub fn save_last_doc(index: usize) -> Result<()> {
    let path = get_session_path()?;
    fs::write(&path, index.to_string())?;
    Ok(())
}

/// Load the last opened document index (returns 0 if not found)
pub fn load_last_doc() -> usize {
    get_session_path()
        .ok()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}
