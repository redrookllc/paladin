use crate::error::Result;
use crate::paths;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api_base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub default_symbol: String,
    pub default_interval: String,
    pub theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_base_url: "https://api.openai.com/v1".to_string(),
            api_key: None,
            model: "gpt-4o-mini".to_string(),
            default_symbol: "SPY".to_string(),
            default_interval: "1d".to_string(),
            theme: "dark".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = paths::config_path()?;
        let mut cfg = if path.exists() {
            let text = std::fs::read_to_string(&path)?;
            toml::from_str::<Config>(&text)?
        } else {
            Self::default()
        };
        if let Ok(v) = std::env::var("PALADIN_OPENAI_BASE_URL") {
            cfg.api_base_url = v;
        }
        if let Ok(v) = std::env::var("PALADIN_OPENAI_API_KEY") {
            cfg.api_key = Some(v);
        } else if cfg.api_key.is_none() {
            if let Ok(v) = std::env::var("OPENAI_API_KEY") {
                cfg.api_key = Some(v);
            }
        }
        if let Ok(v) = std::env::var("PALADIN_MODEL") {
            cfg.model = v;
        }
        Ok(cfg)
    }

    pub fn save(&self) -> Result<()> {
        let path = paths::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)?;
        std::fs::write(&path, text)?;
        Ok(())
    }

    pub fn effective_api_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var("PALADIN_OPENAI_API_KEY").ok())
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
    }
}
