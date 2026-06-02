use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLock;

use super::{Plugin, PluginEvent, PluginInfo, PluginSource, PluginStatus};

/// Thread-safe singleton registry for all plugins.
#[derive(Default)]
pub struct InnerRegistry {
    plugins: HashMap<&'static str, RegisteredPlugin>,
}

struct RegisteredPlugin {
    plugin: Box<dyn Plugin>,
    info: PluginInfo,
}

impl InnerRegistry {
    fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<(), String> {
        let name = plugin.name();
        let version = plugin.version();
        if self.plugins.contains_key(name) {
            return Err(format!("plugin '{}' already registered", name));
        }
        plugin.on_load()?;
        self.plugins.insert(name, RegisteredPlugin {
            info: PluginInfo {
                name,
                version,
                source: PluginSource::BuiltIn,
                loaded_at: Instant::now(),
                status: PluginStatus::Loaded,
            },
            plugin,
        });
        log::info!("[plugin] registered: {} v{}", name, version);
        Ok(())
    }

    fn unregister(&mut self, name: &str) -> Result<(), String> {
        match self.plugins.remove(name) {
            Some(rp) => {
                rp.plugin.on_unload()?;
                log::info!("[plugin] unregistered: {}", name);
                Ok(())
            }
            None => Err(format!("plugin '{}' not found", name)),
        }
    }

    fn list(&self) -> Vec<PluginInfo> {
        self.plugins.values().map(|rp| rp.info.clone()).collect()
    }

    fn dispatch(&self, event: &PluginEvent) {
        for rp in self.plugins.values() {
            if let Err(e) = rp.plugin.on_event(event) {
                log::warn!("[plugin] {}/on_event({}): {}", rp.info.name, event, e);
            }
        }
    }
}

/// Public handle to the shared plugin registry.
#[derive(Clone, Default)]
pub struct PluginRegistry {
    inner: Arc<RwLock<InnerRegistry>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn register(&self, plugin: Box<dyn Plugin>) -> Result<(), String> {
        self.inner.write().await.register(plugin)
    }

    pub async fn unregister(&self, name: &str) -> Result<(), String> {
        self.inner.write().await.unregister(name)
    }

    pub async fn list(&self) -> Vec<PluginInfo> {
        self.inner.read().await.list()
    }

    pub async fn dispatch(&self, event: &PluginEvent) {
        self.inner.read().await.dispatch(event)
    }

    /// Load all `.wasm` / `.so` / `.dll` files from the given directory.
    /// WASM files are loaded via wasmtime when the `sandbox` feature is enabled.
    #[cfg(feature = "sandbox")]
    pub async fn load_from_dir(&self, path: &Path) -> Result<Vec<&'static str>, String> {
        if !path.is_dir() {
            return Err(format!("not a directory: {}", path.display()));
        }
        let mut loaded: Vec<&'static str> = Vec::new();
        let mut entries = match std::fs::read_dir(path) {
            Ok(e) => e,
            Err(e) => return Err(format!("read_dir failed: {}", e)),
        };
        while let Some(entry) = entries.next().transpose().map_err(|e| e.to_string())? {
            let fname = entry.file_name().to_string_lossy().to_string();
            if fname.ends_with(".wasm") {
                let full_path = entry.path();
                match crate::neotrix::plugin::wasm::WasmPluginWrapper::new(&full_path) {
                    Ok(wrapper) => {
                        let name = wrapper.name().to_string();
                        if let Err(e) = self.register(Box::new(wrapper)).await {
                            log::warn!("[plugin] failed to register wasm plugin '{}': {}", name, e);
                        } else {
                            log::info!("[plugin] loaded wasm plugin: {}", name);
                            loaded.push(Box::leak(name.into_boxed_str()));
                        }
                    }
                    Err(e) => log::warn!("[plugin] invalid wasm plugin '{}': {}", fname, e),
                }
            } else if fname.ends_with(".so") || fname.ends_with(".dll") || fname.ends_with(".dylib") {
                log::info!("[plugin] discovered dynamic plugin: {} (loading not yet implemented)", fname);
            }
        }
        Ok(loaded)
    }

    /// Non-sandbox fallback: log discovery only.
    #[cfg(not(feature = "sandbox"))]
    pub async fn load_from_dir(&self, path: &Path) -> Result<Vec<&'static str>, String> {
        if !path.is_dir() {
            return Err(format!("not a directory: {}", path.display()));
        }
        let loaded = Vec::new();
        let mut entries = match std::fs::read_dir(path) {
            Ok(e) => e,
            Err(e) => return Err(format!("read_dir failed: {}", e)),
        };
        while let Some(entry) = entries.next().transpose().map_err(|e| e.to_string())? {
            let fname = entry.file_name().to_string_lossy().to_string();
            if fname.ends_with(".wasm") {
                log::info!("[plugin] discovered wasm plugin: {} (enable 'sandbox' feature to load)", fname);
            } else if fname.ends_with(".so") || fname.ends_with(".dll") || fname.ends_with(".dylib") {
                log::info!("[plugin] discovered dynamic plugin: {} (loading not yet implemented)", fname);
            }
        }
        Ok(loaded)
    }
}
