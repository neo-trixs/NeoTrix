use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

// ═══════════════════════════════════════════════════════════════════
// Gateway Plugin System — trait + manager + hot reload
// ═══════════════════════════════════════════════════════════════════

/// Gateway Plugin trait — 所有网关插件必须实现此 trait
pub(crate) trait GatewayPlugin: Send + Sync {
    /// 插件名称
    fn name(&self) -> &str;
    /// 插件版本
    fn version(&self) -> &str;
    /// 插件优先级 (越小越先执行)
    fn priority(&self) -> i32;
    /// 是否启用
    fn is_enabled(&self) -> bool;
    /// 请求前钩子 — 可修改请求或拦截
    fn pre_request(&self, _ctx: &mut RequestContext) -> Result<(), PluginError> {
        Ok(())
    }
    /// 响应后钩子 — 可修改响应或记录指标
    fn post_response(&self, _ctx: &mut ResponseContext) -> Result<(), PluginError> {
        Ok(())
    }
    /// 错误处理钩子
    fn on_error(&self, _ctx: &mut ErrorContext) -> Result<(), PluginError> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub provider: String,
    pub model: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub(crate) struct ResponseContext {
    pub provider: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub latency: Duration,
}

#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub provider: String,
    pub error: String,
    pub retry_count: u32,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub(crate) enum PluginError {
    SkipRequest(String),
    OverrideResponse(Vec<u8>),
    Abort(String),
}

/// 插件管理器 — 管理插件生命周期和执行顺序
pub struct PluginManager {
    plugins: Vec<Box<dyn GatewayPlugin>>,
    enabled: RwLock<HashMap<String, bool>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            enabled: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn GatewayPlugin>) {
        let name = plugin.name().to_string();
        self.enabled
            .write()
            .unwrap()
            .insert(name, plugin.is_enabled());
        self.plugins.push(plugin);
        self.plugins.sort_by_key(|p| p.priority());
    }

    pub fn run_pre_request(&self, ctx: &mut RequestContext) -> Result<(), PluginError> {
        for plugin in &self.plugins {
            let name = plugin.name();
            let enabled = self.enabled.read().unwrap();
            if enabled.get(name).copied().unwrap_or(true) {
                plugin.pre_request(ctx)?;
            }
        }
        Ok(())
    }

    pub fn run_post_response(&self, ctx: &mut ResponseContext) -> Result<(), PluginError> {
        for plugin in &self.plugins {
            let name = plugin.name();
            let enabled = self.enabled.read().unwrap();
            if enabled.get(name).copied().unwrap_or(true) {
                plugin.post_response(ctx)?;
            }
        }
        Ok(())
    }

    pub fn run_on_error(&self, ctx: &mut ErrorContext) -> Result<(), PluginError> {
        for plugin in &self.plugins {
            let name = plugin.name();
            let enabled = self.enabled.read().unwrap();
            if enabled.get(name).copied().unwrap_or(true) {
                plugin.on_error(ctx)?;
            }
        }
        Ok(())
    }

    pub fn enable(&self, name: &str) {
        if let Some(v) = self.enabled.write().unwrap().get_mut(name) {
            *v = true;
        }
    }

    pub fn disable(&self, name: &str) {
        if let Some(v) = self.enabled.write().unwrap().get_mut(name) {
            *v = false;
        }
    }

    pub fn list_plugins(&self) -> Vec<(&str, bool)> {
        self.plugins
            .iter()
            .map(|p| {
                let enabled = self
                    .enabled
                    .read()
                    .unwrap()
                    .get(p.name())
                    .copied()
                    .unwrap_or(true);
                (p.name(), enabled)
            })
            .collect()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Plugin Hot Reload — 运行时加载/卸载插件
// ═══════════════════════════════════════════════════════════════════

/// 插件热重载管理器 — 运行时加载/卸载插件
pub(crate) struct PluginHotReload {
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
pub(crate) struct ReloadEvent {
    pub plugin: String,
    pub action: ReloadAction,
    pub timestamp: Instant,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub(crate) enum ReloadAction {
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

    struct TestPlugin {
        name: String,
        priority: i32,
    }

    impl GatewayPlugin for TestPlugin {
        fn name(&self) -> &str {
            &self.name
        }
        fn version(&self) -> &str {
            "0.1.0"
        }
        fn priority(&self) -> i32 {
            self.priority
        }
        fn is_enabled(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_plugin_registration() {
        let mut mgr = PluginManager::new();
        mgr.register(Box::new(TestPlugin {
            name: "a".into(),
            priority: 2,
        }));
        mgr.register(Box::new(TestPlugin {
            name: "b".into(),
            priority: 1,
        }));
        let names: Vec<_> = mgr.list_plugins().iter().map(|(n, _)| n.to_string()).collect();
        assert_eq!(names, vec!["b", "a"]);
    }

    #[test]
    fn test_plugin_enable_disable() {
        let mut mgr = PluginManager::new();
        mgr.register(Box::new(TestPlugin {
            name: "test".into(),
            priority: 0,
        }));
        mgr.disable("test");
        assert!(!mgr.enabled.read().unwrap()["test"]);
        mgr.enable("test");
        assert!(mgr.enabled.read().unwrap()["test"]);
    }

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
