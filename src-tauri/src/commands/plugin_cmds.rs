use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub homepage: String,
    #[serde(default)]
    pub entry_points: Vec<String>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStatus {
    pub id: String,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub loaded: bool,
    pub load_time_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEvent {
    pub timestamp: u64,
    pub kind: String,
    pub plugin_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub plugins_dir: String,
    pub auto_load: bool,
    pub allow_unverified: bool,
    pub max_plugins: usize,
}

impl Default for PluginConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        Self {
            plugins_dir: format!("{}/.config/neotrix/plugins", home),
            auto_load: true,
            allow_unverified: false,
            max_plugins: 50,
        }
    }
}

struct PluginState {
    manifests: Vec<PluginManifest>,
    statuses: Vec<PluginStatus>,
    config: PluginConfig,
    events: VecDeque<PluginEvent>,
}

impl PluginState {
    fn new() -> Self {
        Self {
            manifests: Vec::new(),
            statuses: Vec::new(),
            config: PluginConfig::default(),
            events: VecDeque::with_capacity(100),
        }
    }

    fn push_event(&mut self, kind: &str, plugin_id: &str, message: &str) {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        if self.events.len() >= 100 {
            self.events.pop_front();
        }
        self.events.push_back(PluginEvent {
            timestamp: ts,
            kind: kind.to_string(),
            plugin_id: plugin_id.to_string(),
            message: message.to_string(),
        });
    }

    fn upsert_status(&mut self, id: &str, enabled: bool) {
        if let Some(s) = self.statuses.iter_mut().find(|s| s.id == id) {
            s.enabled = enabled;
        }
    }

    fn rebuild_statuses(&mut self) {
        self.statuses = self
            .manifests
            .iter()
            .map(|m| {
                let existing = self.statuses.iter().find(|s| s.id == m.id);
                PluginStatus {
                    id: m.id.clone(),
                    name: m.name.clone(),
                    version: m.version.clone(),
                    enabled: existing.map_or(true, |s| s.enabled),
                    loaded: existing.map_or(true, |s| s.loaded),
                    load_time_ms: existing.map_or(0, |s| s.load_time_ms),
                    error: None,
                }
            })
            .collect();
    }
}

static PLUGIN_STATE: LazyLock<Mutex<PluginState>> = LazyLock::new(|| Mutex::new(PluginState::new()));

#[tauri::command]
pub fn plugin_list() -> Vec<PluginStatus> {
    let mut state = PLUGIN_STATE.lock().unwrap_or_else(|e| e.into_inner());
    let dir = &state.config.plugins_dir;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(manifest) = serde_json::from_str::<PluginManifest>(&content) {
                        if !state.manifests.iter().any(|m| m.id == manifest.id) {
                            state.manifests.push(manifest);
                        }
                    }
                }
            }
        }
    }
    state.rebuild_statuses();
    state.statuses.clone()
}

#[tauri::command]
pub fn plugin_install(path: String) -> Result<PluginStatus, String> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read plugin file: {}", e))?;
    let manifest: PluginManifest = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid plugin manifest: {}", e))?;
    if manifest.id.is_empty() {
        return Err("Plugin manifest must have a non-empty 'id'".into());
    }
    if manifest.name.is_empty() {
        return Err("Plugin manifest must have a non-empty 'name'".into());
    }
    let mut state = PLUGIN_STATE.lock().unwrap_or_else(|e| e.into_inner());
    if state.manifests.iter().any(|m| m.id == manifest.id) {
        return Err(format!("Plugin '{}' is already installed", manifest.id));
    }
    if state.manifests.len() >= state.config.max_plugins {
        return Err(format!(
            "Max plugins ({}) reached",
            state.config.max_plugins
        ));
    }
    let id = manifest.id.clone();
    let name = manifest.name.clone();
    let version = manifest.version.clone();
    state.manifests.push(manifest);
    let status = PluginStatus {
        id: id.clone(),
        name,
        version,
        enabled: true,
        loaded: true,
        load_time_ms: 0,
        error: None,
    };
    state.statuses.push(status.clone());
    state.push_event("loaded", &id, "Plugin installed successfully");
    Ok(status)
}

#[tauri::command]
pub fn plugin_uninstall(id: String) -> Result<(), String> {
    let mut state = PLUGIN_STATE.lock().unwrap_or_else(|e| e.into_inner());
    let before = state.manifests.len();
    state.manifests.retain(|m| m.id != id);
    state.statuses.retain(|s| s.id != id);
    if state.manifests.len() == before {
        return Err(format!("Plugin '{}' not found", id));
    }
    state.push_event("unloaded", &id, "Plugin uninstalled");
    Ok(())
}

#[tauri::command]
pub fn plugin_enable(id: String) -> Result<(), String> {
    let mut state = PLUGIN_STATE.lock().unwrap_or_else(|e| e.into_inner());
    if !state.manifests.iter().any(|m| m.id == id) {
        return Err(format!("Plugin '{}' not found", id));
    }
    state.upsert_status(&id, true);
    state.push_event("config_change", &id, "Plugin enabled");
    Ok(())
}

#[tauri::command]
pub fn plugin_disable(id: String) -> Result<(), String> {
    let mut state = PLUGIN_STATE.lock().unwrap_or_else(|e| e.into_inner());
    if !state.manifests.iter().any(|m| m.id == id) {
        return Err(format!("Plugin '{}' not found", id));
    }
    state.upsert_status(&id, false);
    state.push_event("config_change", &id, "Plugin disabled");
    Ok(())
}

#[tauri::command]
pub fn plugin_get(id: String) -> Result<PluginStatus, String> {
    let state = PLUGIN_STATE.lock().unwrap_or_else(|e| e.into_inner());
    state
        .statuses
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or_else(|| format!("Plugin '{}' not found", id))
}

#[tauri::command]
pub fn plugin_config() -> Result<PluginConfig, String> {
    let state = PLUGIN_STATE.lock().unwrap_or_else(|e| e.into_inner());
    Ok(state.config.clone())
}

#[tauri::command]
pub fn plugin_set_config(config: PluginConfig) -> Result<(), String> {
    let mut state = PLUGIN_STATE.lock().unwrap_or_else(|e| e.into_inner());
    state.push_event("config_change", "system", "Plugin config updated");
    state.config = config;
    Ok(())
}

#[tauri::command]
pub fn plugin_event_log(count: usize) -> Vec<PluginEvent> {
    let state = PLUGIN_STATE.lock().unwrap_or_else(|e| e.into_inner());
    state.events.iter().rev().take(count).cloned().collect()
}

#[tauri::command]
pub fn plugin_run(
    id: String,
    entry_point: String,
    args: Option<Vec<String>>,
) -> Result<String, String> {
    let state = PLUGIN_STATE.lock().unwrap_or_else(|e| e.into_inner());
    if !state.manifests.iter().any(|m| m.id == id) {
        return Err(format!("Plugin '{}' not found", id));
    }
    if !state.statuses.iter().any(|s| s.id == id && s.enabled) {
        return Err(format!("Plugin '{}' is not enabled", id));
    }
    let args_summary = args
        .as_ref()
        .map(|a| a.join(" "))
        .unwrap_or_default();
    let msg = format!(
        "plugin_run: id={} entry_point={} args=[{}]",
        id, entry_point, args_summary
    );
    log::info!("{}", msg);
    drop(state);
    let mut s = PLUGIN_STATE.lock().unwrap_or_else(|e| e.into_inner());
    s.push_event("loaded", &id, &msg);
    Ok(msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset_state() {
        let mut state = PLUGIN_STATE.lock().unwrap_or_else(|e| e.into_inner());
        state.manifests.clear();
        state.statuses.clear();
        state.events.clear();
    }

    #[test]
    fn test_plugin_install() {
        reset_state();
        let tmp = std::env::temp_dir().join("test_plugin_install.json");
        std::fs::write(
            &tmp,
            r#"{"id":"p1","name":"Test Plugin","version":"1.0.0"}"#,
        )
        .unwrap();
        let status = plugin_install(tmp.to_string_lossy().to_string()).unwrap();
        assert_eq!(status.id, "p1");
        assert_eq!(status.name, "Test Plugin");
        assert!(status.enabled);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_plugin_list_after_install() {
        reset_state();
        let tmp = std::env::temp_dir().join("test_plugin_list.json");
        std::fs::write(
            &tmp,
            r#"{"id":"l1","name":"List Plugin"}"#,
        )
        .unwrap();
        plugin_install(tmp.to_string_lossy().to_string()).unwrap();
        let list = plugin_list();
        assert!(list.iter().any(|s| s.id == "l1"));
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_plugin_enable_disable() {
        reset_state();
        let tmp = std::env::temp_dir().join("test_enable_disable.json");
        std::fs::write(
            &tmp,
            r#"{"id":"ed1","name":"ED Plugin"}"#,
        )
        .unwrap();
        plugin_install(tmp.to_string_lossy().to_string()).unwrap();
        assert!(plugin_get("ed1".into()).unwrap().enabled);
        plugin_disable("ed1".into()).unwrap();
        assert!(!plugin_get("ed1".into()).unwrap().enabled);
        plugin_enable("ed1".into()).unwrap();
        assert!(plugin_get("ed1".into()).unwrap().enabled);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_plugin_event_log_bounded() {
        reset_state();
        let mut state = PLUGIN_STATE.lock().unwrap_or_else(|e| e.into_inner());
        for i in 0..150 {
            state.push_event("test", "p1", &format!("event {}", i));
        }
        assert_eq!(state.events.len(), 100);
        drop(state);
        let recent = plugin_event_log(5);
        assert_eq!(recent.len(), 5);
        assert!(recent[0].message.contains("event 149"));
    }
}
