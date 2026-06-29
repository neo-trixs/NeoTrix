use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct NeoTrixConfig {
    #[allow(dead_code)]
    pub default_llm_provider: Option<String>,
    pub provider: Option<String>,
    pub api_key: Option<String>,
    #[allow(dead_code)]
    pub default_model: Option<String>,
    #[allow(dead_code)]
    pub custom_endpoint: Option<String>,
    pub color_mode: Option<String>,
    pub log_level: Option<String>,
}

impl Default for NeoTrixConfig {
    fn default() -> Self {
        Self::auto_detect()
    }
}

impl NeoTrixConfig {
    pub fn path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".config")
            .join("neotrix")
            .join("config.toml")
    }

    /// Auto-detect all configuration from environment variables.
    /// No manual configuration needed — everything has a working default.
    pub fn auto_detect() -> Self {
        Self {
            default_llm_provider: Self::detect_llm_provider(),
            provider: None,
            api_key: None,
            default_model: None,
            custom_endpoint: None,
            color_mode: Self::detect_color_mode(),
            log_level: std::env::var("RUST_LOG").ok(),
        }
    }

    pub(crate) fn detect_llm_provider() -> Option<String> {
        for (name, provider) in &[
            ("ANTHROPIC_API_KEY", "anthropic"),
            ("OPENAI_API_KEY", "openai"),
            ("GEMINI_API_KEY", "gemini"),
            ("OLLAMA_HOST", "ollama"),
        ] {
            if std::env::var(name).is_ok() {
                return Some(provider.to_string());
            }
        }
        None
    }

    fn detect_color_mode() -> Option<String> {
        if std::env::var("NO_COLOR").is_ok() {
            return Some("none".into());
        }
        match std::env::var("TERM").as_deref() {
            Ok(t) if t.contains("256") || t == "xterm-kitty" || t == "alacritty" => {
                Some("auto".into())
            }
            _ => Some("auto".into()),
        }
    }

    pub fn load() -> Self {
        let p = Self::path();
        if p.exists() {
            match std::fs::read_to_string(&p) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(cfg) => {
                        log::info!("[config] loaded from {}", p.display());
                        cfg
                    }
                    Err(e) => {
                        log::error!("[config] parse error in {}: {}", p.display(), e);
                        Self::auto_detect()
                    }
                },
                Err(e) => {
                    log::error!("[config] read error: {}", e);
                    Self::auto_detect()
                }
            }
        } else {
            Self::auto_detect()
        }
    }
}
