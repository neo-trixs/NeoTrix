//! Hermes 社区插件生态 — 插件注册与发现 (NT-IO)
//!
//! 吸收源: github.com/kaishi00/hermes-community-plugins
//! 成熟度: C1 (unit-tested stub, 无外部插件仓库集成)
//!
//! 核心能力: 维护社区插件的注册表, 支持按名称/标签查找与启用状态跟踪。
//! 注: 与 nt_io_hermes_quota.rs 同属 Hermes 生态, 共享 nt_io_hermes 前缀。

use crate::core::nt_core_self_test::SelfTest;
use std::collections::HashMap;

/// 社区插件元数据。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CommunityPlugin {
    pub name: String,
    pub tags: Vec<String>,
    pub enabled: bool,
}

/// 插件注册表 trait — 注册、查找与启用管理。
pub trait PluginRegistry: Send + Sync {
    /// 注册一个插件。
    fn register(&mut self, plugin: CommunityPlugin);
    /// 按名称查找插件。
    fn lookup(&self, name: &str) -> Option<&CommunityPlugin>;
    /// 返回所有启用中的插件名。
    fn enabled_names(&self) -> Vec<String>;
}

/// 默认实现: 基于名称索引的注册表。
#[derive(Default)]
pub(crate) struct HermesCommunityRegistry {
    plugins: HashMap<String, CommunityPlugin>,
}

impl PluginRegistry for HermesCommunityRegistry {
    fn register(&mut self, plugin: CommunityPlugin) {
        self.plugins.insert(plugin.name.clone(), plugin);
    }

    fn lookup(&self, name: &str) -> Option<&CommunityPlugin> {
        self.plugins.get(name)
    }

    fn enabled_names(&self) -> Vec<String> {
        self.plugins
            .values()
            .filter(|p| p.enabled)
            .map(|p| p.name.clone())
            .collect()
    }
}

/// T1 SelfTest: 验证注册表存在并能查找/过滤启用插件。
#[derive(Default)]
pub(crate) struct HermesCommunitySelfTest;

impl SelfTest for HermesCommunitySelfTest {
    fn name(&self) -> &str {
        "nt_io_hermes_community"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut r = HermesCommunityRegistry::default();
        r.register(CommunityPlugin {
            name: "translator".into(),
            tags: vec!["nlp".into()],
            enabled: true,
        });
        r.register(CommunityPlugin {
            name: "sleepy".into(),
            tags: vec!["util".into()],
            enabled: false,
        });
        if r.lookup("translator").is_none() {
            return Err(vec!["nt_io_hermes_community: lookup failed".into()]);
        }
        if r.enabled_names() != vec!["translator".to_string()] {
            return Err(vec!["nt_io_hermes_community: enabled filter wrong".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_lookup() {
        let mut r = HermesCommunityRegistry::default();
        r.register(CommunityPlugin { name: "x".into(), tags: vec![], enabled: true });
        assert_eq!(r.lookup("x").unwrap().name, "x");
        assert!(r.lookup("missing").is_none());
    }

    #[test]
    fn test_enabled_filter() {
        let mut r = HermesCommunityRegistry::default();
        r.register(CommunityPlugin { name: "a".into(), tags: vec![], enabled: true });
        r.register(CommunityPlugin { name: "b".into(), tags: vec![], enabled: true });
        r.register(CommunityPlugin { name: "c".into(), tags: vec![], enabled: false });
        assert_eq!(r.enabled_names().len(), 2);
    }

    #[test]
    fn test_lookup_returns_metadata() {
        let mut r = HermesCommunityRegistry::default();
        r.register(CommunityPlugin {
            name: "t".into(),
            tags: vec!["vision".into()],
            enabled: true,
        });
        let p = r.lookup("t").unwrap();
        assert!(p.tags.contains(&"vision".to_string()));
    }
}
