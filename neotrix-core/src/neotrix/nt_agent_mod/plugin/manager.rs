use std::path::PathBuf;
use crate::neotrix::nt_agent::plugin::manifest::PluginManifest;

/// Represents a loaded plugin.
#[derive(Debug, Clone)]
pub struct Plugin {
    pub manifest: PluginManifest,
    pub path: PathBuf,
    pub is_active: bool,
}

impl Plugin {
    pub fn new(manifest: PluginManifest, path: PathBuf) -> Self {
        Self {
            manifest,
            path,
            is_active: true,
        }
    }
}

/// Manages plugin discovery, loading, and lifecycle.
#[derive(Debug)]
pub struct PluginManager {
    plugins: Vec<Plugin>,
    scan_dirs: Vec<PathBuf>,
}

impl PluginManager {
    pub fn new() -> Self {
        let mut scan_dirs = Vec::new();
        if let Some(config_dir) = dirs::config_dir() {
            scan_dirs.push(config_dir.join("neotrix").join("plugins"));
        }
        if let Some(data_dir) = dirs::data_dir() {
            scan_dirs.push(data_dir.join("neotrix").join("plugins"));
        }
        Self {
            plugins: Vec::new(),
            scan_dirs,
        }
    }

    pub fn with_scan_dir(mut self, dir: PathBuf) -> Self {
        self.scan_dirs.push(dir);
        self
    }

    /// Scan all configured directories for plugins.
    pub fn scan(&mut self) -> Vec<Plugin> {
        let mut discovered = Vec::new();
        for dir in &self.scan_dirs {
            if !dir.exists() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let plugin_dir = entry.path();
                    if !plugin_dir.is_dir() {
                        continue;
                    }
                    let manifest_path = plugin_dir.join("plugin.json");
                    if !manifest_path.exists() {
                        continue;
                    }
                    if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                        match PluginManifest::from_json(&content) {
                            Ok(manifest) => {
                                let plugin = Plugin::new(manifest, plugin_dir);
                                let name = plugin.manifest.name.clone();
                                // Avoid duplicates
                                if !self.plugins.iter().any(|p| p.manifest.name == name) {
                                    self.plugins.push(plugin.clone());
                                    discovered.push(plugin);
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to load plugin from {:?}: {}", plugin_dir, e);
                            }
                        }
                    }
                }
            }
        }
        discovered
    }

    /// Get all loaded plugins.
    pub fn plugins(&self) -> &[Plugin] {
        &self.plugins
    }

    /// Get active plugins (those that are enabled).
    pub fn active_plugins(&self) -> Vec<&Plugin> {
        self.plugins.iter().filter(|p| p.is_active).collect()
    }

    /// Enable a plugin by name.
    pub fn enable(&mut self, name: &str) -> Result<(), String> {
        self.plugins.iter_mut()
            .find(|p| p.manifest.name == name)
            .map(|p| { p.is_active = true; })
            .ok_or_else(|| format!("Plugin '{}' not found", name))
    }

    /// Disable a plugin by name.
    pub fn disable(&mut self, name: &str) -> Result<(), String> {
        self.plugins.iter_mut()
            .find(|p| p.manifest.name == name)
            .map(|p| { p.is_active = false; })
            .ok_or_else(|| format!("Plugin '{}' not found", name))
    }

    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::fs;

    fn setup_test_plugin(dir: &Path, name: &str) {
        let plugin_dir = dir.join(name);
        fs::create_dir_all(&plugin_dir).expect("failed to create plugin test dir");
        let manifest = serde_json::json!({
            "name": name,
            "version": "1.0.0",
            "description": "test",
            "hooks": ["PreToolUse"],
            "tools": [],
            "commands": [],
        });
        fs::write(plugin_dir.join("plugin.json"), serde_json::to_string_pretty(&manifest).expect("failed to serialize plugin manifest")).expect("failed to write plugin.json");
    }

    #[test]
    fn test_plugin_manager_creation() {
        let mgr = PluginManager::new();
        assert!(mgr.is_empty());
    }

    #[test]
    fn test_scan_discovers_plugins() {
        let dir = std::env::temp_dir().join(format!("neotrix-plugin-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("failed to create scan test dir");

        setup_test_plugin(&dir, "plugin-a");
        setup_test_plugin(&dir, "plugin-b");

        let mut mgr = PluginManager::new()
            .with_scan_dir(dir.clone());
        let discovered = mgr.scan();
        assert_eq!(discovered.len(), 2);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_enable_disable() {
        let mut mgr = PluginManager::new();

        // Manually add a plugin
        let manifest = PluginManifest {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "test".to_string(),
            author: None,
            hooks: vec![],
            tools: vec![],
            commands: vec![],
            min_core_version: None,
        };
        mgr.plugins.push(Plugin::new(manifest, PathBuf::from("/tmp/test")));

        assert_eq!(mgr.active_plugins().len(), 1);
        mgr.disable("test").expect("disable should succeed for active plugin");
        assert_eq!(mgr.active_plugins().len(), 0);
        mgr.enable("test").expect("enable should succeed for disabled plugin");
        assert_eq!(mgr.active_plugins().len(), 1);
    }

    #[test]
    fn test_enable_nonexistent_error() {
        let mut mgr = PluginManager::new();
        assert!(mgr.enable("nonexistent").is_err());
    }

    #[test]
    fn test_plugin_count() {
        let mut mgr = PluginManager::new();
        assert_eq!(mgr.len(), 0);

        let manifest = PluginManifest {
            name: "p1".to_string(),
            version: "1.0.0".to_string(),
            description: "".to_string(),
            author: None,
            hooks: vec![],
            tools: vec![],
            commands: vec![],
            min_core_version: None,
        };
        mgr.plugins.push(Plugin::new(manifest, PathBuf::from("/tmp/p1")));
        assert_eq!(mgr.len(), 1);
    }
}
