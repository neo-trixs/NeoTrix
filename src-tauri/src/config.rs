use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Layered configuration for NeoTrix desktop app.
/// Priority: CLI args > env vars > config file > defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    #[serde(default = "default_provider")]
    pub provider: String,

    #[serde(default)]
    pub api_key: Option<String>,

    #[serde(default = "default_base_url")]
    pub base_url: String,

    #[serde(default = "default_model")]
    pub model: String,

    #[serde(default)]
    pub embedding_api_key: Option<String>,

    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default = "default_max_connections")]
    pub max_connections: usize,

    #[serde(default = "default_request_timeout_secs")]
    pub request_timeout_secs: u64,

    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
}

fn default_provider() -> String {
    "openai".into()
}
fn default_base_url() -> String {
    "https://api.openai.com/v1".into()
}
fn default_model() -> String {
    "gpt-4o".into()
}
fn default_log_level() -> String {
    "info".into()
}
fn default_max_connections() -> usize {
    8
}
fn default_request_timeout_secs() -> u64 {
    30
}
fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or(PathBuf::from("."))
        .join("neotrix")
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            api_key: None,
            base_url: default_base_url(),
            model: default_model(),
            embedding_api_key: None,
            log_level: default_log_level(),
            max_connections: default_max_connections(),
            request_timeout_secs: default_request_timeout_secs(),
            data_dir: default_data_dir(),
        }
    }
}

impl AppConfig {
    /// Load config with layered priority:
    /// 1. Start with defaults
    /// 2. Overlay config file (if exists)
    /// 3. Overlay env vars (NEOTRIX_*)
    /// 4. Validate all required fields
    pub fn load() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // Layer 1: Config file
        if let Some(file_config) = Self::load_config_file()? {
            config = file_config;
        }

        // Layer 2: Environment variables override
        config.apply_env_overrides();

        // Layer 3: Validate
        config.validate()?;

        Ok(config)
    }

    fn load_config_file() -> Result<Option<Self>, ConfigError> {
        let config_paths = Self::config_search_paths();

        for path in &config_paths {
            if path.exists() {
                let content = std::fs::read_to_string(path)
                    .map_err(|e| ConfigError::FileRead(path.display().to_string(), e))?;
                let parsed: Self = toml::from_str(&content)
                    .map_err(|e| ConfigError::Parse(path.display().to_string(), e))?;
                tracing::info!(path = %path.display(), "Loaded config file");
                return Ok(Some(parsed));
            }
        }

        Ok(None)
    }

    fn config_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Project-local .neotrix/
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd.join(".neotrix").join("config.toml"));
        }

        // User config dir
        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("neotrix").join("config.toml"));
        }

        // Home dir fallback
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".neotrix").join("config.toml"));
        }

        paths
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("NEOTRIX_PROVIDER") {
            self.provider = val;
        }
        if let Ok(val) = std::env::var("NEOTRIX_API_KEY") {
            self.api_key = Some(val);
        }
        if let Ok(val) = std::env::var("NEOTRIX_BASE_URL") {
            self.base_url = val;
        }
        if let Ok(val) = std::env::var("NEOTRIX_MODEL") {
            self.model = val;
        }
        if let Ok(val) = std::env::var("NEOTRIX_EMBEDDING_API_KEY") {
            self.embedding_api_key = Some(val);
        }
        if let Ok(val) = std::env::var("NEOTRIX_LOG_LEVEL") {
            self.log_level = val;
        }
    }

    /// Fail-fast validation. Called at startup.
    fn validate(&self) -> Result<(), ConfigError> {
        if self.provider.is_empty() {
            return Err(ConfigError::Validation("provider cannot be empty".into()));
        }
        if self.base_url.is_empty() {
            return Err(ConfigError::Validation("base_url cannot be empty".into()));
        }
        if self.model.is_empty() {
            return Err(ConfigError::Validation("model cannot be empty".into()));
        }
        if self.max_connections == 0 {
            return Err(ConfigError::Validation(
                "max_connections must be > 0".into(),
            ));
        }
        if self.request_timeout_secs == 0 {
            return Err(ConfigError::Validation(
                "request_timeout_secs must be > 0".into(),
            ));
        }
        if !["trace", "debug", "info", "warn", "error"].contains(&self.log_level.as_str()) {
            return Err(ConfigError::Validation(format!(
                "invalid log_level: {} (expected trace/debug/info/warn/error)",
                self.log_level
            )));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum ConfigError {
    FileRead(String, std::io::Error),
    Parse(String, toml::de::Error),
    Validation(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileRead(path, e) => write!(f, "Failed to read config at {path}: {e}"),
            Self::Parse(path, e) => write!(f, "Failed to parse config at {path}: {e}"),
            Self::Validation(msg) => write!(f, "Config validation failed: {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}
