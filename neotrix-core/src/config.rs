use crate::neotrix::nt_shield::key_encryption;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
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
                        if std::env::var("NEOTRIX_QUIET").is_err() {
                            eprintln!("[config] loaded from {}", p.display());
                        }
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

    /// 将指定字段写回配置文件（保留已有字段，缺失的以 None 保存）。
    /// 支持全部 `NeoTrixConfig` 字段；未知 key 返回 false。
    pub fn save_field(&self, field: &str, value: &str) -> bool {
        let mut cfg = Self::load();
        match field {
            "default_llm_provider" => cfg.default_llm_provider = Some(value.to_string()),
            "provider" => cfg.provider = Some(value.to_string()),
            "api_key" => cfg.api_key = Some(value.to_string()),
            "default_model" => cfg.default_model = Some(value.to_string()),
            "custom_endpoint" => cfg.custom_endpoint = Some(value.to_string()),
            "color_mode" => cfg.color_mode = Some(value.to_string()),
            "log_level" => cfg.log_level = Some(value.to_string()),
            _ => return false,
        }
        if let Some(dir) = Self::path().parent() {
            if !dir.exists() {
                let _ = std::fs::create_dir_all(dir);
            }
        }
        let toml_str = match toml::to_string(&cfg) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[config] serialize error: {}", e);
                return false;
            }
        };
        if let Err(e) = std::fs::write(Self::path(), toml_str) {
            eprintln!("[config] write error: {}", e);
            false
        } else {
            eprintln!("[config] saved {}={} to {}", field, value, Self::path().display());
            true
        }
    }
}
