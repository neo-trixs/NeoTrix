//! 能力标签插件 —— 把 provider / model-source 登记进统一 PluginRegistry。
//!
//! 现有工厂方式：`LlmProviderType` 枚举 + `from_name` 匹配 + `ModelSource`
//! 的多个 impl 各自被 catalog/pool 直接消费。这让「外部能力在哪」散落
//! 在 3 处之上。统一做法：把每一种外部能力作为 `Plugin` 的capability
//! 标签挂进同一个 registry，ntcode/后台循环只枚举 registry by capability
//! 即可拿到「可用能力集合」的真相源；具体调用仍走各自的 trait 对象。
//!
//! 这一版实现「以 capability 标签统一列示」；把 factory/model_pool 的
//! 每个调用方全部改挂到 registry 是后续增量。
#![forbid(unsafe_code)]

use super::{Plugin, PluginEvent};
use crate::l1_action::nt_io::nt_io_provider::common::factory::LlmProviderType;

/// 以能力标签为主体的轻量 Plugin：只负责「登记存在、分类」，不构造对象。
pub struct CapabilityPlugin {
    name: &'static str,
    version: &'static str,
    capability: &'static str,
}

impl CapabilityPlugin {
    pub fn new(name: &'static str, capability: &'static str) -> Self {
        Self {
            name,
            version: env!("CARGO_PKG_VERSION"),
            capability,
        }
    }
}

impl Plugin for CapabilityPlugin {
    fn name(&self) -> &'static str {
        self.name
    }
    fn version(&self) -> &'static str {
        self.version
    }
    fn capability(&self) -> &'static str {
        self.capability
    }
    fn on_load(&self) -> Result<(), String> {
        Ok(())
    }
    fn on_unload(&self) -> Result<(), String> {
        Ok(())
    }
    fn on_event(&self, _event: &PluginEvent) -> Result<(), String> {
        Ok(())
    }
}

/// 已收录的 provider 名称 -> (LlmProviderType 名称, 标签)。
fn provider_names() -> Vec<(&'static str, &'static str)> {
    vec![
        ("openai", "llm_provider"),
        ("anthropic", "llm_provider"),
        ("gemini", "llm_provider"),
        ("ollama", "llm_provider"),
        ("openrouter", "llm_provider"),
        ("opencode-zen", "llm_provider"),
    ]
}

/// 已收录的免费/发现源名称 + 标签。
fn model_source_names() -> Vec<(&'static str, &'static str)> {
    vec![
        ("gguf", "model_source"),
        ("free", "model_source"),
        ("local-endpoint", "model_source"),
        ("cli-free", "model_source"),
    ]
}

/// 生产可用的內建能力插件全集。
pub fn builtin_capability_plugins() -> Vec<Box<dyn Plugin>> {
    let mut out: Vec<Box<dyn Plugin>> = Vec::new();
    for (name, cap) in provider_names() {
        let _ = LlmProviderType::from_name(name); // 触发一次静态收录路径
        out.push(Box::new(CapabilityPlugin::new(name, cap)) as Box<dyn Plugin>);
    }
    for (name, cap) in model_source_names() {
        out.push(Box::new(CapabilityPlugin::new(name, cap)) as Box<dyn Plugin>);
    }
    out
}

/// 按 capability 汇总当前已登记的 capability plugin 名字。
pub async fn capabilities_with(
    registry: &super::PluginRegistry,
    capability: &str,
) -> Vec<String> {
    registry
        .list()
        .await
        .into_iter()
        .filter(|info| info.capability == capability)
        .map(|info| info.name.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_plugins_cover_two_externals() {
        let list = builtin_capability_plugins();
        let caps: std::collections::HashSet<&str> =
            list.iter().map(|p| p.capability()).collect();
        assert!(caps.contains("llm_provider"));
        assert!(caps.contains("model_source"));
        let names: std::collections::HashSet<&str> =
            list.iter().map(|p| p.name()).collect();
        assert!(names.contains("opencode-zen"));
        assert!(names.contains("cli-free"));
    }

    #[test]
    fn every_plugin_load_and_unload_ok() {
        for p in builtin_capability_plugins() {
            assert!(p.on_load().is_ok());
            assert!(p.on_unload().is_ok());
        }
    }

    #[test]
    fn provider_names_match_factory_variants() {
        use crate::l1_action::nt_io::nt_io_provider::common::factory::LlmProviderType;
        for (name, _cap) in provider_names() {
            assert!(LlmProviderType::from_name(name).is_some(), "{}", name);
        }
    }
}
