use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use super::{Plugin, PluginEvent, PluginInfo, PluginSource, PluginStatus};
use super::discovery::discover_skills_on_disk;

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
    inner: Arc<Mutex<InnerRegistry>>,
    discovered_count: Arc<AtomicUsize>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Synchronous disk scan for skills/plugins
    pub fn discover(&self) {
        let skills = discover_skills_on_disk();
        self.discovered_count.store(skills.len(), Ordering::Relaxed);
        if !skills.is_empty() {
            log::info!("[plugin] discovered {} skill(s): {:?}",
                skills.len(),
                skills.iter().map(|s| s.name.as_str()).collect::<Vec<_>>());
        }
    }

    /// Number of discovered skills/plugins
    pub fn count(&self) -> usize {
        self.discovered_count.load(Ordering::Relaxed)
    }

    pub fn register(&self, plugin: Box<dyn Plugin>) -> Result<(), String> {
        self.inner.lock().map_err(|e| e.to_string())?.register(plugin)
    }

    pub fn unregister(&self, name: &str) -> Result<(), String> {
        self.inner.lock().map_err(|e| e.to_string())?.unregister(name)
    }

    pub fn list(&self) -> Vec<PluginInfo> {
        self.inner.lock().map(|g| g.list()).unwrap_or_default()
    }

    pub fn dispatch(&self, event: &PluginEvent) {
        if let Ok(guard) = self.inner.lock() {
            guard.dispatch(event)
        }
    }

    /// Load all `.wasm` / `.so` / `.dll` files from the given directory.
    /// WASM files are loaded via wasmtime when the `sandbox` feature is enabled.
    #[cfg(feature = "sandbox")]
    pub fn load_from_dir(&self, path: &Path) -> Result<Vec<&'static str>, String> {
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
                match crate::neotrix::nt_io_plugin::wasm::WasmPluginWrapper::new(&full_path) {
                    Ok(wrapper) => {
                        let name = wrapper.name().to_string();
                        if let Err(e) = self.register(Box::new(wrapper)) {
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
    pub fn load_from_dir(&self, path: &Path) -> Result<Vec<&'static str>, String> {
        if !path.is_dir() {
            return Err(format!("not a directory: {}", path.display()));
        }
        let _ = std::fs::read_dir(path).map_err(|e| e.to_string())?;
        Ok(Vec::new())
    }
}
