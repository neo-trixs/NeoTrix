//! # nt_core_plugin — Plugin trait & runtime registry
//!
//! Defines the `Plugin` trait and `PluginRegistry` for native Rust plugins.
//! All plugins are stubs — WASM/dynamic loading lives in `neotrix::nt_io_plugin`.

pub mod types;

#[cfg(feature = "sandbox")]
pub mod wasm_loader;

use std::collections::HashMap;
use std::path::Path;
use types::{PluginInfo, PluginState};

/// Core trait every native plugin must implement.
pub trait Plugin: Send {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    fn state(&self) -> PluginState;
    fn load(&mut self) -> Result<(), String>;
    fn unload(&mut self) -> Result<(), String>;
}

/// Thread-local registry of loaded plugins.
///
/// Stores plugins by name. The CLI commands (`/plugin list / load / unload / info`)
/// delegate directly to this registry.
#[derive(Default)]
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register and load a plugin.
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<(), String> {
        let mut p = plugin;
        let name = p.name().to_string();
        if self.plugins.contains_key(&name) {
            return Err(format!("plugin '{}' already registered", name));
        }
        p.load()?;
        log::info!(
            "[plugin] loaded: {} v{} — {}",
            p.name(),
            p.version(),
            p.description()
        );
        self.plugins.insert(name, p);
        Ok(())
    }

    /// Unload and remove a plugin by name.
    pub fn unregister(&mut self, name: &str) -> Result<(), String> {
        match self.plugins.remove(name) {
            Some(mut p) => {
                p.unload()?;
                log::info!("[plugin] unloaded: {}", name);
                Ok(())
            }
            None => Err(format!("plugin '{}' not found", name)),
        }
    }

    /// List all registered plugin infos.
    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins
            .values()
            .map(|p| PluginInfo {
                name: p.name().to_string(),
                version: p.version().to_string(),
                description: p.description().to_string(),
                state: p.state(),
                source: "native".to_string(),
            })
            .collect()
    }

    /// Get info for a single plugin.
    pub fn info(&self, name: &str) -> Option<PluginInfo> {
        self.plugins.get(name).map(|p| PluginInfo {
            name: p.name().to_string(),
            version: p.version().to_string(),
            description: p.description().to_string(),
            state: p.state(),
            source: "native".to_string(),
        })
    }

    /// Load a WASM plugin from a `.wasm` file path.
    ///
    /// Creates a `WasmPlugin`, compiles the module, instantiates it,
    /// and registers it into the registry.
    #[cfg(feature = "sandbox")]
    pub fn load_wasm(&mut self, path: &Path) -> Result<(), String> {
        use wasm_loader::WasmPlugin;
        let mut plugin = WasmPlugin::from_file(path)?;
        let name = plugin.name().to_string();
        if self.plugins.contains_key(&name) {
            return Err(format!("plugin '{}' already registered", name));
        }
        plugin.load()?;
        log::info!(
            "[plugin] loaded WASM: {} v{} — {}",
            plugin.name(),
            plugin.version(),
            plugin.description()
        );
        self.plugins.insert(name, Box::new(plugin));
        Ok(())
    }

    /// Load a WASM plugin stub — friendly error when `sandbox` feature is off.
    #[cfg(not(feature = "sandbox"))]
    pub fn load_wasm(&mut self, _path: &Path) -> Result<(), String> {
        Err("WASM plugin support not compiled. Rebuild with --features sandbox".to_string())
    }

    /// Number of registered plugins.
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

    struct DummyPlugin {
        name: String,
        loaded: bool,
    }
    impl DummyPlugin {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                loaded: false,
            }
        }
    }
    impl Plugin for DummyPlugin {
        fn name(&self) -> &str {
            &self.name
        }
        fn version(&self) -> &str {
            "0.1.0"
        }
        fn description(&self) -> &str {
            "dummy test plugin"
        }
        fn state(&self) -> PluginState {
            if self.loaded {
                PluginState::Loaded
            } else {
                PluginState::Unloaded
            }
        }
        fn load(&mut self) -> Result<(), String> {
            self.loaded = true;
            Ok(())
        }
        fn unload(&mut self) -> Result<(), String> {
            self.loaded = false;
            Ok(())
        }
    }

    #[test]
    fn test_register_and_list() {
        let mut reg = PluginRegistry::new();
        assert!(reg.is_empty());
        reg.register(Box::new(DummyPlugin::new("test-a"))).unwrap();
        reg.register(Box::new(DummyPlugin::new("test-b"))).unwrap();
        assert_eq!(reg.len(), 2);
        let list = reg.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_register_duplicate_fails() {
        let mut reg = PluginRegistry::new();
        reg.register(Box::new(DummyPlugin::new("dup"))).unwrap();
        let err = reg.register(Box::new(DummyPlugin::new("dup"))).unwrap_err();
        assert!(err.contains("already registered"));
    }

    #[test]
    fn test_unregister() {
        let mut reg = PluginRegistry::new();
        reg.register(Box::new(DummyPlugin::new("tmp"))).unwrap();
        assert_eq!(reg.len(), 1);
        reg.unregister("tmp").unwrap();
        assert!(reg.is_empty());
    }

    #[test]
    fn test_unregister_missing_fails() {
        let mut reg = PluginRegistry::new();
        let err = reg.unregister("nope").unwrap_err();
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_info() {
        let mut reg = PluginRegistry::new();
        reg.register(Box::new(DummyPlugin::new("alpha"))).unwrap();
        let info = reg.info("alpha").unwrap();
        assert_eq!(info.name, "alpha");
        assert_eq!(info.version, "0.1.0");
        assert_eq!(info.state, PluginState::Loaded);
    }
}
