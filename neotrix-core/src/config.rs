use neotrix::neotrix::nt_shield::key_encryption;
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
                Ok(content) => match toml::from_str::<Self>(&content) {
                    Ok(mut cfg) => {
                        // Auto-decrypt any securely-stored API keys
                        if let Some(ref key) = cfg.api_key.clone() {
                            if key_encryption::is_encrypted(key) {
                                match key_encryption::decrypt(key) {
                                    Ok(plain) => {
                                        cfg.api_key = Some(plain);
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "[config] warning: failed to decrypt api_key: {}",
                                            e
                                        );
                                    }
                                }
                            }
                        }
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
