use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLock;

use super::{Plugin, PluginEvent, PluginInfo, PluginSource, PluginStatus};
use crate::core::nt_core_context::revertible::{ClosureEffect, RevertibleContext};

/// HMR 事务性热替换的结果分类 (§5.2.2 classify 不动点判定)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotReloadOutcome {
    /// 插件为全新安装 (此前未注册)。
    Installed,
    /// 同名插件已注册且版本不同 — 已事务性替换 (旧版被新版本接管)。
    Replaced,
    /// 同名同版本 — 不动点, 拒绝重复重载 (无变更)。
    NoChange,
}

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

    /// 事务化批量装载 (∂Γ RevertibleContext 回滚日志):
    /// 任一批内插件 on_load 失败 → 整体回滚, 此前已注册的同一批插件全部移除
    /// (all-or-nothing, 语义对齐 dsh load_batch)。成功时逐一向用户 on_unload
    /// 仅发生在 rollback (on_unload 的失败不阻断回滚)。
    fn load_batch(&mut self, plugins: Vec<Box<dyn Plugin>>) -> Result<Vec<&'static str>, String> {
        let mut ctx = RevertibleContext::new(&mut *self);
        let mut loaded: Vec<&'static str> = Vec::new();
        for plugin in plugins {
            let name = plugin.name();
            let version = plugin.version();
            if ctx.state().plugins.contains_key(name) {
                ctx.recover();
                return Err(format!("plugin '{}' already registered", name));
            }
            if let Err(e) = plugin.on_load() {
                ctx.recover();
                return Err(format!("plugin '{}' on_load failed: {}", name, e));
            }
            let cell = std::sync::Mutex::new(Some(plugin));
            let fwd_name = name;
            let fwd_version = version;
            ctx.track(ClosureEffect::new(
                format!("load:{}", name),
                move |reg: &mut &mut InnerRegistry| {
                    let p = cell.lock().expect("plugin registry cell poisoned").take().expect("forward invoked once");
                    let info = PluginInfo {
                        name: fwd_name,
                        version: fwd_version,
                        source: PluginSource::BuiltIn,
                        loaded_at: Instant::now(),
                        status: PluginStatus::Loaded,
                    };
                    reg.plugins.insert(fwd_name, RegisteredPlugin { info, plugin: p });
                },
                move |reg: &mut &mut InnerRegistry| {
                    reg.plugins.remove(name);
                },
            ));
            loaded.push(name);
        }
        Ok(loaded)
    }

    /// HMR 事务性热替换 (§5.2.2 三阶段):
    /// 1. **Classify** 不动点判定: 同名同版本 → NoChange (拒绝重复重载);
    ///    同名不同版本 → 替换; 未注册 → 安装。
    /// 2. **Stale 清除**: 替换时旧插件被移出并暂存 (inverse 可恢复)。
    /// 3. **事务重载** (∂Γ RevertibleContext): 新插件 `on_load` 失败 → 整批
    ///    recover, 已替换的恢复旧版, 已安装的移除 — 无半重载状态 (no-half-reload)。
    fn hot_reload_batch(
        &mut self,
        plugins: Vec<Box<dyn Plugin>>,
    ) -> Result<Vec<(&'static str, HotReloadOutcome)>, String> {
        let mut ctx = RevertibleContext::new(&mut *self);
        let mut outcomes: Vec<(&'static str, HotReloadOutcome)> = Vec::new();
        for plugin in plugins {
            let name = plugin.name();
            let version = plugin.version();
            let existing = ctx.state().plugins.get(name).map(|rp| rp.info.version);
            match existing {
                // ── 不动点: 同名同版本, 无变更 ──
                Some(ver) if ver == version => {
                    outcomes.push((name, HotReloadOutcome::NoChange));
                    log::info!("[plugin] hot-reload: {} v{} is a fixed point, skip", name, version);
                }
                // ── 替换: 同名不同版本 ──
                Some(_) => {
                    if let Err(e) = plugin.on_load() {
                        ctx.recover();
                        return Err(format!("hot-reload '{}' on_load failed: {}", name, e));
                    }
                    let cell = std::sync::Mutex::new(Some((
                        PluginInfo {
                            name,
                            version,
                            source: PluginSource::BuiltIn,
                            loaded_at: Instant::now(),
                            status: PluginStatus::Loaded,
                        },
                        plugin,
                    )));
                    let old_cell: std::sync::Arc<std::sync::Mutex<Option<RegisteredPlugin>>> =
                        std::sync::Arc::new(std::sync::Mutex::new(None));
                    let old_cell_fwd = old_cell.clone();
                    ctx.track(ClosureEffect::new(
                        format!("replace:{}", name),
                        move |reg: &mut &mut InnerRegistry| {
                            let (info, new_p) = cell.lock().expect("plugin registry cell poisoned").take().expect("forward invoked once");
                            let stale = reg.plugins.remove(name);
                            *old_cell_fwd.lock().expect("plugin registry old_cell poisoned") = stale;
                            reg.plugins.insert(name, RegisteredPlugin { info: info.clone(), plugin: new_p });
                        },
                        move |reg: &mut &mut InnerRegistry| {
                            reg.plugins.remove(name);
                            if let Some(old_p) = old_cell.lock().expect("plugin registry cell poisoned").take() {
                                reg.plugins.insert(name, old_p);
                            }
                        },
                    ));
                    outcomes.push((name, HotReloadOutcome::Replaced));
                    log::info!("[plugin] hot-reload: replaced {} v{}", name, version);
                }
                // ── 安装: 未注册 ──
                None => {
                    if let Err(e) = plugin.on_load() {
                        ctx.recover();
                        return Err(format!("hot-reload '{}' on_load failed: {}", name, e));
                    }
                    let cell = std::sync::Mutex::new(Some((
                        PluginInfo {
                            name,
                            version,
                            source: PluginSource::BuiltIn,
                            loaded_at: Instant::now(),
                            status: PluginStatus::Loaded,
                        },
                        plugin,
                    )));
                    ctx.track(ClosureEffect::new(
                        format!("load:{}", name),
                        move |reg: &mut &mut InnerRegistry| {
                            let (info, p) = cell.lock().expect("plugin registry cell poisoned").take().expect("forward invoked once");
                            reg.plugins.insert(name, RegisteredPlugin { info: info.clone(), plugin: p });
                        },
                        move |reg: &mut &mut InnerRegistry| {
                            reg.plugins.remove(name);
                        },
                    ));
                    outcomes.push((name, HotReloadOutcome::Installed));
                    log::info!("[plugin] hot-reload: installed {} v{}", name, version);
                }
            }
        }
        Ok(outcomes)
    }

    /// 单插件 HMR — 委托批量事务, 返回单个分类。
    fn hot_reload(&mut self, plugin: Box<dyn Plugin>) -> Result<HotReloadOutcome, String> {
        let mut outcomes = self.hot_reload_batch(vec![plugin])?;
        match outcomes.len() {
            0 => Err("hot-reload produced no outcome".to_string()),
            _ => Ok(outcomes.remove(0).1),
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

    /// 事务化批量装载 — 任一批内失败即整体回滚。
    pub async fn load_batch(&self, plugins: Vec<Box<dyn Plugin>>) -> Result<Vec<&'static str>, String> {
        self.inner.write().await.load_batch(plugins)
    }

    /// HMR 事务性热替换 (批量) — 任一 on_load 失败整批回滚。
    pub async fn hot_reload_batch(
        &self,
        plugins: Vec<Box<dyn Plugin>>,
    ) -> Result<Vec<(&'static str, HotReloadOutcome)>, String> {
        self.inner.write().await.hot_reload_batch(plugins)
    }

    /// HMR 事务性热替换 (单插件)。
    pub async fn hot_reload(&self, plugin: Box<dyn Plugin>) -> Result<HotReloadOutcome, String> {
        self.inner.write().await.hot_reload(plugin)
    }

    /// HMR 目录重载 (sandbox): 扫描目录并事务性热替换同名插件,
    /// 新版本接管, 旧版本被替换 — 目录内任一同名插件 on_load 失败 → 整批回滚。
    #[cfg(feature = "sandbox")]
    pub async fn hot_reload_from_dir(&self, path: &Path) -> Result<Vec<(&'static str, HotReloadOutcome)>, String> {
        if !path.is_dir() {
            return Err(format!("not a directory: {}", path.display()));
        }
        let mut batch: Vec<Box<dyn Plugin>> = Vec::new();
        let mut entries = match std::fs::read_dir(path) {
            Ok(e) => e,
            Err(e) => return Err(format!("read_dir failed: {}", e)),
        };
        while let Some(entry) = entries.next().transpose().map_err(|e| e.to_string())? {
            let fname = entry.file_name().to_string_lossy().to_string();
            if fname.ends_with(".wasm") {
                let full_path = entry.path();
                match crate::neotrix::nt_io_plugin::wasm::WasmPluginWrapper::new(&full_path) {
                    Ok(wrapper) => batch.push(Box::new(wrapper)),
                    Err(e) => log::warn!("[plugin] hot-reload invalid wasm '{}': {}", fname, e),
                }
            }
        }
        if batch.is_empty() {
            return Ok(Vec::new());
        }
        self.hot_reload_batch(batch).await
    }

    /// HMR 目录重载 (non-sandbox): 仅记录发现。
    #[cfg(not(feature = "sandbox"))]
    pub async fn hot_reload_from_dir(&self, path: &Path) -> Result<Vec<(&'static str, HotReloadOutcome)>, String> {
        if !path.is_dir() {
            return Err(format!("not a directory: {}", path.display()));
        }
        Ok(Vec::new())
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
        let mut batch: Vec<Box<dyn Plugin>> = Vec::new();
        let mut entries = match std::fs::read_dir(path) {
            Ok(e) => e,
            Err(e) => return Err(format!("read_dir failed: {}", e)),
        };
        while let Some(entry) = entries.next().transpose().map_err(|e| e.to_string())? {
            let fname = entry.file_name().to_string_lossy().to_string();
            if fname.ends_with(".wasm") {
                let full_path = entry.path();
                match crate::neotrix::nt_io_plugin::wasm::WasmPluginWrapper::new(&full_path) {
                    Ok(wrapper) => batch.push(Box::new(wrapper)),
                    Err(e) => log::warn!("[plugin] invalid wasm plugin '{}': {}", fname, e),
                }
            } else if fname.ends_with(".so") || fname.ends_with(".dll") || fname.ends_with(".dylib") {
                log::info!("[plugin] discovered dynamic plugin: {} (loading not yet implemented)", fname);
            }
        }
        if batch.is_empty() {
            return Ok(Vec::new());
        }
        // 事务化批量装载: 任一批内失败 → 整体回滚 (∂Γ RevertibleContext)。
        self.load_batch(batch).await
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
                // 只关心文件增删改（Create/Remove/Modify）。
                let is_create = matches!(ev.kind, notify::EventKind::Create(_));
                let is_remove = matches!(ev.kind, notify::EventKind::Remove(_));
                let is_modify = matches!(ev.kind, notify::EventKind::Modify(_));
                if !is_create && !is_remove && !is_modify {
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
                    if is_modify {
                        // HMR 事务性热替换: 同名新版本接管, 旧版本被替换。
                        match registry.hot_reload_from_dir(&dir).await {
                            Ok(outcomes) => {
                                let n_replaced = outcomes.iter().filter(|(_, o)| *o == HotReloadOutcome::Replaced).count();
                                let n_installed = outcomes.iter().filter(|(_, o)| *o == HotReloadOutcome::Installed).count();
                                log::info!(
                                    "[plugin] hot-reload: {} replaced, {} installed, {} unchanged",
                                    n_replaced, n_installed,
                                    outcomes.len() - n_replaced - n_installed
                                );
                            }
                            Err(e) => log::warn!("[plugin] hot-reload failed (rolled back): {}", e),
                        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct MockPlugin {
        name: &'static str,
        version: &'static str,
        fail_load: bool,
        load_calls: Arc<AtomicU32>,
        unload_calls: Arc<AtomicU32>,
        fail_unload: bool,
    }

    impl Plugin for MockPlugin {
        fn name(&self) -> &'static str { self.name }
        fn version(&self) -> &'static str { self.version }
        fn on_load(&self) -> Result<(), String> {
            self.load_calls.fetch_add(1, Ordering::SeqCst);
            if self.fail_load {
                Err("mock on_load failure".into())
            } else {
                Ok(())
            }
        }
        fn on_unload(&self) -> Result<(), String> {
            self.unload_calls.fetch_add(1, Ordering::SeqCst);
            if self.fail_unload {
                Err("mock on_unload failure".into())
            } else {
                Ok(())
            }
        }
        fn on_event(&self, _e: &PluginEvent) -> Result<(), String> { Ok(()) }
    }

    fn mock(name: &'static str, fail_load: bool, load_calls: &Arc<AtomicU32>) -> Box<dyn Plugin> {
        mock_ver(name, "1.0", fail_load, load_calls)
    }

    fn mock_ver(name: &'static str, version: &'static str, fail_load: bool, load_calls: &Arc<AtomicU32>) -> Box<dyn Plugin> {
        Box::new(MockPlugin {
            name,
            version,
            fail_load,
            load_calls: load_calls.clone(),
            unload_calls: Arc::new(AtomicU32::new(0)),
            fail_unload: false,
        })
    }

    fn version_of(reg: &InnerRegistry, name: &str) -> &'static str {
        reg.plugins.get(name).map(|rp| rp.info.version).unwrap_or("")
    }

    #[test]
    fn test_load_batch_all_ok() {
        let mut reg = InnerRegistry::default();
        let calls = Arc::new(AtomicU32::new(0));
        let plugins = vec![mock("a", false, &calls), mock("b", false, &calls), mock("c", false, &calls)];
        let loaded = reg.load_batch(plugins).unwrap();
        assert_eq!(loaded, vec!["a", "b", "c"]);
        assert_eq!(reg.plugins.len(), 3);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_load_batch_rolls_back_on_failure() {
        let mut reg = InnerRegistry::default();
        let calls = Arc::new(AtomicU32::new(0));
        // a 成功, b 失败 → 整体回滚, a 不得残留 (all-or-nothing)
        let plugins = vec![mock("a", false, &calls), mock("b", true, &calls), mock("c", false, &calls)];
        let err = reg.load_batch(plugins).unwrap_err();
        assert!(err.contains("b"));
        assert!(err.contains("on_load failed"));
        assert!(reg.plugins.is_empty(), "rollback 后注册表应为空");
        assert_eq!(calls.load(Ordering::SeqCst), 2); // a 与 b 各 on_load 一次
    }

    #[test]
    fn test_load_batch_duplicate_rolls_back_previous() {
        let mut reg = InnerRegistry::default();
        let calls = Arc::new(AtomicU32::new(0));
        // 先单独注册 "a"
        reg.register(mock("a", false, &calls)).unwrap();
        // 批量 [b, a] → a 重复, 整批回滚, b 不得残留
        let plugins = vec![mock("b", false, &calls), mock("a", false, &calls)];
        let err = reg.load_batch(plugins).unwrap_err();
        assert!(err.contains("already registered"));
        assert!(reg.plugins.len() == 1, "仅保留批次前的 a, b 应被回滚");
        assert!(reg.plugins.contains_key("a"));
        assert!(!reg.plugins.contains_key("b"));
    }

    #[test]
    fn test_load_batch_empty_is_ok() {
        let mut reg = InnerRegistry::default();
        let loaded = reg.load_batch(Vec::new()).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_hot_reload_installs_new_plugin() {
        let mut reg = InnerRegistry::default();
        let calls = Arc::new(AtomicU32::new(0));
        let outcome = reg.hot_reload(mock("alpha", false, &calls)).unwrap();
        assert_eq!(outcome, HotReloadOutcome::Installed);
        assert_eq!(version_of(&reg, "alpha"), "1.0");
    }

    #[test]
    fn test_hot_reload_fixed_point_no_change() {
        let mut reg = InnerRegistry::default();
        let calls = Arc::new(AtomicU32::new(0));
        reg.register(mock("beta", false, &calls)).unwrap();
        // 同名同版本 → 不动点, 不重复重载
        let outcome = reg.hot_reload(mock_ver("beta", "1.0", false, &calls)).unwrap();
        assert_eq!(outcome, HotReloadOutcome::NoChange);
        // 版本未变, on_load 不应再被调用
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_hot_reload_replaces_newer_version() {
        let mut reg = InnerRegistry::default();
        let calls = Arc::new(AtomicU32::new(0));
        reg.register(mock_ver("gamma", "1.0", false, &calls)).unwrap();
        // 同名 v2.0 → 事务性替换
        let outcome = reg.hot_reload(mock_ver("gamma", "2.0", false, &calls)).unwrap();
        assert_eq!(outcome, HotReloadOutcome::Replaced);
        assert_eq!(version_of(&reg, "gamma"), "2.0");
    }

    #[test]
    fn test_hot_reload_rolls_back_on_load_failure() {
        let mut reg = InnerRegistry::default();
        let calls = Arc::new(AtomicU32::new(0));
        reg.register(mock_ver("delta", "1.0", false, &calls)).unwrap();
        // 新版本 on_load 失败 → 事务回滚, 旧版 1.0 保留
        let err = reg.hot_reload(mock_ver("delta", "2.0", true, &calls)).unwrap_err();
        assert!(err.contains("on_load failed"));
        assert_eq!(version_of(&reg, "delta"), "1.0", "回滚后旧版本应保留");
    }

    #[test]
    fn test_hot_reload_batch_atomic_rollback() {
        let mut reg = InnerRegistry::default();
        let calls = Arc::new(AtomicU32::new(0));
        reg.register(mock_ver("zeta", "1.0", false, &calls)).unwrap();
        // 批: zeta→2.0 (替换成功), omega (安装), theta (安装失败)
        // theta on_load 失败 → 整批回滚: zeta 恢复 1.0, omega 移除
        let plugins = vec![
            mock_ver("zeta", "2.0", false, &calls),
            mock("omega", false, &calls),
            mock_ver("theta", "3.0", true, &calls),
        ];
        let err = reg.hot_reload_batch(plugins).unwrap_err();
        assert!(err.contains("theta"));
        assert_eq!(version_of(&reg, "zeta"), "1.0", "已替换的 zeta 应恢复旧版");
        assert!(!reg.plugins.contains_key("omega"), "已安装的 omega 应被回滚移除");
        assert!(!reg.plugins.contains_key("theta"));
    }
}
