use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AppConfig {
    pub theme: String,
    pub auto_start: bool,
    pub daemon_port: u16,
    pub log_level: String,
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".neotrix")
        .join("desktop-config.json")
}

#[tauri::command]
pub async fn get_config() -> Result<AppConfig, String> {
    let path = config_path();
    if path.exists() {
        let data = fs::read_to_string(&path).map_err(|e| format!("Read config: {e}"))?;
        serde_json::from_str(&data).map_err(|e| format!("Parse config: {e}"))
    } else {
        Ok(AppConfig::default())
    }
}

#[tauri::command]
pub async fn set_config(config: AppConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Create config dir: {e}"))?;
    }
    let data = serde_json::to_string_pretty(&config).map_err(|e| format!("Serialize config: {e}"))?;
    fs::write(&path, data).map_err(|e| format!("Write config: {e}"))
}
