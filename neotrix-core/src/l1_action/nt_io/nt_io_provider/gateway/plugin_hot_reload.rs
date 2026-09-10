use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// 插件热重载管理器 — 运行时加载/卸载插件
pub struct PluginHotReload {
    plugins: RwLock<HashMap<String, PluginEntry>>,
    event_log: RwLock<Vec<ReloadEvent>>,
}

#[derive(Debug, Clone)]
struct PluginEntry {
    name: String,
    version: String,
    loaded_at: Instant,
    checksum: u64,
    enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ReloadEvent {
    pub plugin: String,
    pub action: ReloadAction,
    pub timestamp: Instant,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum ReloadAction {
    Load,
    Unload,
    Reload,
    Update,
}

impl PluginHotReload {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            event_log: RwLock::new(Vec::new()),
        }
    }

    pub fn load(&self, name: &str, version: &str, checksum: u64) -> Result<(), String> {
        let mut plugins = self.plugins.write().unwrap();
        if plugins.contains_key(name) {
            return Err(format!("plugin {} already loaded", name));
        }
        plugins.insert(
            name.to_string(),
            PluginEntry {
                name: name.to_string(),
                version: version.to_string(),
                loaded_at: Instant::now(),
                checksum,
                enabled: true,
            },
        );
        self.log_event(name, ReloadAction::Load, true, "loaded");
        Ok(())
    }

    pub fn unload(&self, name: &str) -> Result<(), String> {
        let mut plugins = self.plugins.write().unwrap();
        if plugins.remove(name).is_some() {
            self.log_event(name, ReloadAction::Unload, true, "unloaded");
            Ok(())
        } else {
            Err(format!("plugin {} not found", name))
        }
    }

    pub fn reload(&self, name: &str, new_checksum: u64) -> Result<(), String> {
        let mut plugins = self.plugins.write().unwrap();
        match plugins.get_mut(name) {
            Some(entry) => {
                entry.checksum = new_checksum;
                entry.loaded_at = Instant::now();
                self.log_event(name, ReloadAction::Reload, true, "reloaded");
                Ok(())
            }
            None => Err(format!("plugin {} not found", name)),
        }
    }

    pub fn is_loaded(&self, name: &str) -> bool {
        self.plugins.read().unwrap().contains_key(name)
    }

    pub fn has_changed(&self, name: &str, current_checksum: u64) -> bool {
        self.plugins
            .read()
            .unwrap()
            .get(name)
            .map(|e| e.checksum != current_checksum)
            .unwrap_or(true)
    }

    pub fn list(&self) -> Vec<(String, String, bool)> {
        self.plugins
            .read()
            .unwrap()
            .values()
            .map(|e| (e.name.clone(), e.version.clone(), e.enabled))
            .collect()
    }

    pub fn get_events(&self, name: Option<&str>) -> Vec<ReloadEvent> {
        let log = self.event_log.read().unwrap();
        match name {
            Some(n) => log.iter().filter(|e| e.plugin == n).cloned().collect(),
            None => log.clone(),
        }
    }

    fn log_event(&self, plugin: &str, action: ReloadAction, success: bool, msg: &str) {
        let mut log = self.event_log.write().unwrap();
        log.push(ReloadEvent {
            plugin: plugin.to_string(),
            action,
            timestamp: Instant::now(),
            success,
            message: msg.to_string(),
        });
        if log.len() > 1000 {
            log.drain(0..500);
        }
    }
}

impl Default for PluginHotReload {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_unload() {
        let mgr = PluginHotReload::new();
        mgr.load("test", "1.0.0", 12345).unwrap();
        assert!(mgr.is_loaded("test"));
        mgr.unload("test").unwrap();
        assert!(!mgr.is_loaded("test"));
    }

    #[test]
    fn test_reload_detection() {
        let mgr = PluginHotReload::new();
        mgr.load("test", "1.0.0", 100).unwrap();
        assert!(!mgr.has_changed("test", 100));
        assert!(mgr.has_changed("test", 200));
    }

    #[test]
    fn test_event_logging() {
        let mgr = PluginHotReload::new();
        mgr.load("p1", "1.0", 1).unwrap();
        mgr.unload("p1").unwrap();
        let events = mgr.get_events(Some("p1"));
        assert_eq!(events.len(), 2);
    }
}
