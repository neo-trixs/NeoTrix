//! plugin_harness — everything-is-a-plugin harness + Cordis 时空可组合范式。
//!
//! 吸收 deepseek-harness 机制: 一切皆插件 (PluginSpec 描述, 运行时动态注册),
//! 模块可动态加载/卸载, 生命周期由 PluginLifecycle 统一管理 (load → tick → unload)。
//! NT-ACT 域 (行动执行者) 的插件编排层, 用于在 Orchestrator 之上组合原子能力。

/// 插件运行时状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginStatus {
    Loaded,
    Idle,
    Running,
    Disabled,
    Failed,
}

/// 插件描述符 — everything-is-a-plugin 的注册单元。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginSpec {
    pub id: String,
    pub name: String,
    pub version: String,
    pub entry: String,
    pub enabled: bool,
}

impl Default for PluginSpec {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            version: String::from("0.1.0"),
            entry: String::new(),
            enabled: true,
        }
    }
}

/// 插件生命周期契约 — 模块可动态加载/卸载。
pub trait PluginLifecycle: Send + Sync {
    fn load(&mut self) -> Result<(), String>;
    fn unload(&mut self) -> Result<(), String>;
    fn tick(&mut self) -> Result<(), String>;
}

/// 最简插件实现 — 用于 harness 与生命周期测试。
#[derive(Debug, Clone)]
pub struct SimplePlugin {
    pub spec: PluginSpec,
    pub status: PluginStatus,
    pub tick_count: u64,
}

impl SimplePlugin {
    pub fn from_spec(spec: PluginSpec) -> Self {
        let status = if spec.enabled {
            PluginStatus::Idle
        } else {
            PluginStatus::Disabled
        };
        Self {
            spec,
            status,
            tick_count: 0,
        }
    }
}

impl PluginLifecycle for SimplePlugin {
    fn load(&mut self) -> Result<(), String> {
        if self.spec.enabled {
            self.status = PluginStatus::Loaded;
            Ok(())
        } else {
            Err(format!("plugin '{}' is disabled, cannot load", self.spec.id))
        }
    }

    fn unload(&mut self) -> Result<(), String> {
        self.status = PluginStatus::Disabled;
        Ok(())
    }

    fn tick(&mut self) -> Result<(), String> {
        self.tick_count = self.tick_count.saturating_add(1);
        self.status = PluginStatus::Running;
        Ok(())
    }
}

/// 插件编排 harness — 动态注册/加载/卸载的组合容器。
#[derive(Debug, Clone)]
pub struct PluginHarness {
    pub plugins: Vec<SimplePlugin>,
    pub max_plugins: usize,
}

impl Default for PluginHarness {
    fn default() -> Self {
        Self::new(32)
    }
}

impl PluginHarness {
    pub fn new(max_plugins: usize) -> Self {
        Self {
            plugins: Vec::new(),
            max_plugins,
        }
    }

    pub fn register(&mut self, spec: PluginSpec) -> Result<(), String> {
        if self.plugins.len() >= self.max_plugins {
            return Err(format!(
                "cannot register plugin '{}': max_plugins {} reached",
                spec.id, self.max_plugins
            ));
        }
        if self.plugins.iter().any(|p| p.spec.id == spec.id) {
            return Err(format!("duplicate plugin id '{}'", spec.id));
        }
        self.plugins.push(SimplePlugin::from_spec(spec));
        Ok(())
    }

    pub fn load_all(&mut self) -> usize {
        let mut loaded = 0;
        for p in self.plugins.iter_mut() {
            if p.status != PluginStatus::Loaded && p.load().is_ok() {
                loaded += 1;
            }
        }
        loaded
    }

    pub fn tick_all(&mut self) -> usize {
        let mut ticked = 0;
        for p in self.plugins.iter_mut() {
            if p.tick().is_ok() {
                ticked += 1;
            }
        }
        ticked
    }

    pub fn unload_all(&mut self) -> usize {
        let mut unloaded = 0;
        for p in self.plugins.iter_mut() {
            if p.status != PluginStatus::Disabled && p.unload().is_ok() {
                unloaded += 1;
            }
        }
        unloaded
    }

    pub fn status_summary(&self) -> Vec<(String, PluginStatus)> {
        self.plugins
            .iter()
            .map(|p| (p.spec.id.clone(), p.status))
            .collect()
    }
}

impl crate::core::nt_core_self_test::SelfTest for PluginHarness {
    fn name(&self) -> &str {
        "nt_agent_orchestrator_plugin_harness"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut h = PluginHarness::new(4);
        for i in 0..3 {
            h.register(PluginSpec {
                id: format!("p{}", i),
                name: format!("plugin-{}", i),
                ..Default::default()
            })
            .map_err(|e| vec![format!("register failed: {}", e)])?;
        }
        if h.load_all() != 3 {
            return Err(vec!["load_all must load 3 enabled plugins".into()]);
        }
        if h.tick_all() != 3 {
            return Err(vec!["tick_all must tick 3 loaded plugins".into()]);
        }
        if h.plugins.iter().any(|p| p.tick_count != 1) {
            return Err(vec!["tick must increment tick_count".into()]);
        }
        if h.unload_all() != 3 {
            return Err(vec!["unload_all must unload 3 plugins".into()]);
        }
        if h.plugins.iter().any(|p| p.status != PluginStatus::Disabled) {
            return Err(vec!["unload must leave every plugin Disabled".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(id: &str, enabled: bool) -> PluginSpec {
        PluginSpec {
            id: id.to_string(),
            name: format!("plugin-{}", id),
            entry: format!("entry::{}", id),
            enabled,
            ..Default::default()
        }
    }

    #[test]
    fn register_over_limit_errors() {
        let mut h = PluginHarness::new(2);
        assert!(h.register(spec("a", true)).is_ok());
        assert!(h.register(spec("b", true)).is_ok());
        assert!(h.register(spec("c", true)).is_err());
    }

    #[test]
    fn register_duplicate_id_errors() {
        let mut h = PluginHarness::new(4);
        assert!(h.register(spec("dup", true)).is_ok());
        assert!(h.register(spec("dup", false)).is_err());
        assert_eq!(h.plugins.len(), 1);
    }

    #[test]
    fn load_all_counts_enabled_plugins() {
        let mut h = PluginHarness::new(4);
        h.register(spec("a", true)).unwrap();
        h.register(spec("b", true)).unwrap();
        h.register(spec("c", false)).unwrap();
        assert_eq!(h.load_all(), 2);
        let loaded = h
            .status_summary()
            .into_iter()
            .filter(|(_, s)| *s == PluginStatus::Loaded)
            .count();
        assert_eq!(loaded, 2);
    }

    #[test]
    fn tick_all_increments_tick_count() {
        let mut h = PluginHarness::new(4);
        h.register(spec("a", true)).unwrap();
        h.register(spec("b", true)).unwrap();
        h.load_all();
        assert_eq!(h.tick_all(), 2);
        assert!(h.plugins.iter().all(|p| p.tick_count == 1));
        assert_eq!(h.tick_all(), 2);
        assert!(h.plugins.iter().all(|p| p.tick_count == 2));
    }

    #[test]
    fn unload_all_disables_every_plugin() {
        let mut h = PluginHarness::new(4);
        h.register(spec("a", true)).unwrap();
        h.register(spec("b", true)).unwrap();
        h.load_all();
        assert_eq!(h.unload_all(), 2);
        assert!(h
            .status_summary()
            .into_iter()
            .all(|(_, s)| s == PluginStatus::Disabled));
    }

    #[test]
    fn simple_plugin_lifecycle() {
        let mut p = SimplePlugin::from_spec(spec("x", true));
        assert_eq!(p.status, PluginStatus::Idle);
        p.load().unwrap();
        assert_eq!(p.status, PluginStatus::Loaded);
        p.tick().unwrap();
        assert_eq!(p.status, PluginStatus::Running);
        assert_eq!(p.tick_count, 1);
        p.unload().unwrap();
        assert_eq!(p.status, PluginStatus::Disabled);
    }

    #[test]
    fn disabled_plugin_cannot_load() {
        let mut p = SimplePlugin::from_spec(spec("off", false));
        assert!(p.load().is_err());
        assert_eq!(p.status, PluginStatus::Disabled);
    }
}