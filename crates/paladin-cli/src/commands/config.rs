use crate::ui;
use console::style;
use paladin_core::config::Config;
use paladin_core::error::{Error, Result};
use paladin_core::paths;

pub fn show() -> Result<()> {
    let cfg = Config::load()?;
    let redacted = Config {
        api_key: cfg.api_key.as_ref().map(|_| "***".to_string()),
        ..cfg.clone()
    };
    println!("{}", toml::to_string_pretty(&redacted)?);
    Ok(())
}

pub fn path() -> Result<()> {
    println!("{}", paths::config_path()?.display());
    Ok(())
}

pub fn set(key: &str, value: &str) -> Result<()> {
    let mut cfg = Config::load()?;
    match key {
        "api_base_url" => cfg.api_base_url = value.to_string(),
        "api_key" => cfg.api_key = Some(value.to_string()),
        "model" => cfg.model = value.to_string(),
        "default_symbol" => cfg.default_symbol = value.to_string(),
        "default_interval" => cfg.default_interval = value.to_string(),
        "theme" => cfg.theme = value.to_string(),
        other => return Err(Error::InvalidInput(format!("unknown key: {other}"))),
    }
    cfg.save()?;
    println!("  {} {} = {}", ui::ok(), key, style(value).cyan());
    Ok(())
}
