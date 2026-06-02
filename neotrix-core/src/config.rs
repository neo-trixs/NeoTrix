use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
#[allow(dead_code)]
pub struct NeoTrixConfig {
    pub default_llm_provider: Option<String>,
    pub provider: Option<String>,
    pub api_key: Option<String>,
    pub default_model: Option<String>,
    pub custom_endpoint: Option<String>,
    pub color_mode: Option<String>,
    pub log_level: Option<String>,
}

impl NeoTrixConfig {
    pub fn path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".config")
            .join("neotrix")
            .join("config.toml")
    }

    pub fn load() -> Self {
        let p = Self::path();
        if p.exists() {
            match std::fs::read_to_string(&p) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(cfg) => {
                        eprintln!("[config] loaded from {}", p.display());
                        cfg
                    }
                    Err(e) => {
                        eprintln!("[config] parse error in {}: {}", p.display(), e);
                        Self::default()
                    }
                },
                Err(e) => {
                    eprintln!("[config] read error: {}", e);
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }
}
