use std::collections::HashMap;
use std::path::{Path, PathBuf};
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

/// 全局共享注册表 — CLI 命令与后台循环/第三方 CLI 调用共享同一实例，
/// 保证热插拔 (load/unload) 真实作用于运行中的注册表（R-P42: 拒绝平行适配器）。
pub fn global_registry() -> PluginRegistry {
    use std::sync::OnceLock;
    static GLOBAL: OnceLock<PluginRegistry> = OnceLock::new();
    GLOBAL.get_or_init(PluginRegistry::new).clone()
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
                match crate::neotrix::nt_io_plugin::wasm::WasmPluginWrapper::new(&full_path) {
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

    /// 目录热插拔监视 — 监听插件目录:
    /// - 新增 `.wasm`/`.dylib`/`.so`/`.dll` 文件 → 异步 load_from_dir
    /// - 删除文件 → 尝试按文件名去掉扩展名 unregister 对应插件
    ///
    /// 纯安全实现（R-P1: forbid unsafe_code），基于 notify 文件事件。
    /// 返回 JoinHandle；drop/abort 即停止监视。
    pub fn watch_dir(&self, path: PathBuf) -> Result<tokio::task::JoinHandle<()>, String> {
        if !path.is_dir() {
            return Err(format!("not a directory: {}", path.display()));
        }
        use notify::{RecommendedWatcher, Watcher, RecursiveMode};
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<notify::Result<notify::Event>>();
        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                let _ = tx.send(res);
            },
            notify::Config::default(),
        )
        .map_err(|e| format!("notify init failed: {}", e))?;
        watcher
            .watch(&path, RecursiveMode::NonRecursive)
            .map_err(|e| format!("watch failed: {}", e))?;

        let registry = self.clone();
        let dir = path.to_path_buf();
        let handle = tokio::spawn(async move {
            let _watcher = watcher;
            while let Some(ev) = rx.recv().await {
                let ev = match ev {
                    Ok(ev) => ev,
                    Err(e) => {
                        log::warn!("[plugin] watch event error: {}", e);
                        continue;
                    }
                };
                // 只关心文件增删（Create/Remove），忽略内容改写。
                let is_create = matches!(ev.kind, notify::EventKind::Create(_));
                let is_remove = matches!(ev.kind, notify::EventKind::Remove(_));
                if !is_create && !is_remove {
                    continue;
                }
                let ext_ok = |p: &Path| {
                    ["wasm", "dylib", "so", "dll"]
                        .iter()
                        .any(|e| p.extension().map(|x| x == *e).unwrap_or(false))
                };
                for p in ev.paths.iter().filter(|p| ext_ok(p)) {
                    if is_create {
                        match registry.load_from_dir(&dir).await {
                            Ok(_) => log::info!("[plugin] hot-plug: rescanned {}", dir.display()),
                            Err(e) => log::warn!("[plugin] hot-plug rescan failed: {}", e),
                        }
                        // load_from_dir 已扫描整个目录，避免逐文件重复处理。
                        break;
                    }
                    if is_remove {
                        let name = p
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        if !name.is_empty() {
                            match registry.unregister(&name).await {
                                Ok(()) => log::info!("[plugin] hot-unplug: unregistered {}", name),
                                Err(_) => log::debug!("[plugin] {} not registered, skip unplug", name),
                            }
                        }
                    }
                }
            }
        });
        log::info!("[plugin] watching dir {} for hot-plug", path.display());
        Ok(handle)
    }
}
