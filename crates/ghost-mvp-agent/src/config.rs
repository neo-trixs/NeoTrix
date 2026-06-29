use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub agent_name: String,
    pub agent_version: String,
    pub http_port: u16,
    pub discovery_port: u16,
    pub data_dir: PathBuf,

    pub neotrix_a2a_endpoint: Option<String>,
    pub registry_url: Option<String>,

    pub schedule_analysis_interval_hours: u64,
    pub schedule_geo_audit_interval_days: u64,

    pub persona_path: Option<PathBuf>,
    pub skills_dir: Option<PathBuf>,

    pub geo_optimizer_command: Option<String>,

    pub hui_mei_enabled: bool,
    pub letmepost_api_key: Option<String>,
    pub devto_api_key: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            agent_name: "ghost-mvp".into(),
            agent_version: "1.0.0".into(),
            http_port: 8890,
            discovery_port: 42069,
            data_dir: home.join(".ghost-mvp"),

            neotrix_a2a_endpoint: None,
            registry_url: None,

            schedule_analysis_interval_hours: 168,
            schedule_geo_audit_interval_days: 30,

            persona_path: Some(home.join(".agents/skills/ghost-mvp/persona.yaml")),
            skills_dir: Some(home.join(".agents/skills/ghost-mvp")),

            geo_optimizer_command: Some("uvx --from geo-optimizer-skill geo".into()),

            hui_mei_enabled: false,
            letmepost_api_key: None,
            devto_api_key: None,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::config_path();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            let cfg = Config::default();
            let _ = std::fs::create_dir_all(cfg.data_dir.parent().unwrap_or(&cfg.data_dir));
            let content = toml::to_string_pretty(&cfg).unwrap_or_default();
            let _ = std::fs::write(&config_path, &content);
            cfg
        }
    }

    fn config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".config/ghost-mvp/config.toml")
    }
}
