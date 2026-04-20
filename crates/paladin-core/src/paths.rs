use crate::error::{Error, Result};
use std::path::PathBuf;

pub fn home_dir() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("PALADIN_HOME") {
        return Ok(PathBuf::from(p));
    }
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| Error::Config("$HOME not set".into()))?;
    Ok(base.join(".paladin"))
}

pub fn config_path() -> Result<PathBuf> {
    Ok(home_dir()?.join("config.toml"))
}

pub fn cache_dir() -> Result<PathBuf> {
    let p = home_dir()?.join("cache");
    std::fs::create_dir_all(&p).ok();
    Ok(p)
}

pub fn logs_dir() -> Result<PathBuf> {
    let p = home_dir()?.join("logs");
    std::fs::create_dir_all(&p).ok();
    Ok(p)
}
