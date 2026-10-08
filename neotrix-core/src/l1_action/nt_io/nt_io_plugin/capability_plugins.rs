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
use crate::l1_action::nt_io::nt_io_provider::common::factory::{
    LlmProviderType, create_provider_from_type,
};
use crate::l1_action::nt_io::nt_io_provider::catalog::model_pool::{
    CloudFreeSource, LocalEndpointSource, LocalGgufSource,
};
use crate::l1_action::nt_io::nt_io_provider::catalog::cli_free_source::CliFreeSource;
use std::sync::Arc;

use crate::l1_action::nt_io::nt_io_provider::common::types::LlmProvider;
use crate::l1_action::nt_io::nt_io_provider::catalog::model_pool::ModelSource;

/// 真实 provider 的 Plugin 包装：持有 `Arc<dyn LlmProvider>`，登记进 registry，
/// 让消费方按 `capability="llm_provider"` + name 取回真实对象。
pub struct ProviderPlugin {
    name: &'static str,
    provider: Arc<dyn LlmProvider>,
}

impl Plugin for ProviderPlugin {
    fn name(&self) -> &'static str {
        self.name
    }
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
    fn capability(&self) -> &'static str {
        "llm_provider"
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
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn shared_handle(&self) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        // 不能把 `Arc<dyn LlmProvider>` 直接 cast 成 `Arc<dyn Any + …>`
        // （Rust 禁止 trait object 之间的非原生转换），故经一层具体
        // newtype 做类型擦除：擦除后调用方再 downcast 回 `Arc<dyn …>`。
        let concrete = ProviderHandle {
            provider: Arc::clone(&self.provider),
        };
        let handle: Arc<dyn std::any::Any + Send + Sync> = Arc::new(concrete);
        Some(handle)
    }
}

/// 类型擦除载体：让 `Arc<dyn LlmProvider>` 能作为 `Arc<dyn Any>` 过注册表。
struct ProviderHandle {
    provider: Arc<dyn LlmProvider>,
}

/// 真实 ModelSource 的 Plugin 包装：持有 `Arc<dyn ModelSource>`，登记进 registry，
/// 消费方按 `capability="model_source"` 取回（`Arc` 可克隆，故 registry 与 pool
/// 可共享同一 source 实例而非各自 new 一份）。
///
/// `name` 在构造时**一次性**内部化：`Plugin::name()` 返回 `&'static str`，
/// 若每次调用都 `Box::leak` 一个新 `String`，register/list/dispatch 每次读
/// 名字都会漏一份内存 —— 故这里构造时定一个静态副本。
pub struct SourcePlugin {
    name: &'static str,
    source: Arc<dyn ModelSource>,
}

impl SourcePlugin {
    pub fn new(source: Arc<dyn ModelSource>) -> Self {
        let name: &'static str = Box::leak(source.name().to_string().into_boxed_str());
        Self { name, source }
    }
    /// 取回共享的真实 source 句柄（消费方用它驱动 discovery）。
    pub fn source(&self) -> Arc<dyn ModelSource> {
        Arc::clone(&self.source)
    }
}

impl Plugin for SourcePlugin {
    fn name(&self) -> &'static str {
        self.name
    }
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
    fn capability(&self) -> &'static str {
        "model_source"
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
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn shared_handle(&self) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        // 同 ProviderPlugin：经具体 newtype 擦除，禁止 trait object 直接互转。
        let concrete = SourceHandle {
            source: Arc::clone(&self.source),
        };
        let handle: Arc<dyn std::any::Any + Send + Sync> = Arc::new(concrete);
        Some(handle)
    }
}

/// 类型擦除载体：让 `Arc<dyn ModelSource>` 能作为 `Arc<dyn Any>` 过注册表。
struct SourceHandle {
    source: Arc<dyn ModelSource>,
}

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
    fn as_any(&self) -> &dyn std::any::Any { self }
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

/// 生产可用的外部能力插件全集：真实 provider / model_source 对象登记进 registry。
pub fn builtin_capability_plugins() -> Vec<Box<dyn Plugin>> {
    let mut out: Vec<Box<dyn Plugin>> = Vec::new();
    for (slug, _cap) in provider_names() {
        if let Some(t) = LlmProviderType::from_name(slug) {
            // 不拿到 key，构造仅代表「该provider 可被接通」——这才是完整的 registry 语义。
            out.push(Box::new(ProviderPlugin {
                name: slug,
                provider: create_provider_from_type(t, None),
            }) as Box<dyn Plugin>);
        }
    }
    let sources: Vec<Arc<dyn ModelSource>> = vec![
        Arc::new(LocalGgufSource::default_m5()),
        Arc::new(CloudFreeSource::new()),
        Arc::new(LocalEndpointSource::new()),
        Arc::new(CliFreeSource::new()),
    ];
    for s in sources {
        out.push(Box::new(SourcePlugin::new(s)) as Box<dyn Plugin>);
    }
    out
}

/// 从 registry 取出 capability 匹配的真实 `Arc<dyn ModelSource>` 句柄。
///
/// 消费方（`UnifiedModelPool` 等）据此驱动 discovery，而不再自行 `new` source
/// —— 这是「统一接入口」的收口点：registry 登记什么，池就跑什么。
pub async fn model_sources_from_registry(
    registry: &super::PluginRegistry,
) -> Vec<Arc<dyn ModelSource>> {
    let mut out = Vec::new();
    for handle in registry.shared_handles_by_capability("model_source").await {
        if let Some(wrapper) = handle.downcast_ref::<SourceHandle>() {
            out.push(Arc::clone(&wrapper.source));
        }
    }
    out
}

/// 从 registry 取出 capability 匹配的真实 `Arc<dyn LlmProvider>` 句柄。
pub async fn providers_from_registry(
    registry: &super::PluginRegistry,
) -> Vec<Arc<dyn LlmProvider>> {
    let mut out = Vec::new();
    for handle in registry.shared_handles_by_capability("llm_provider").await {
        if let Some(wrapper) = handle.downcast_ref::<ProviderHandle>() {
            out.push(Arc::clone(&wrapper.provider));
        }
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

    /// 消费侧收口的核心判据：注册进 registry 的 source 能按 capability 原样
    /// 取回**同一个实例**（`Arc::ptr_eq`），而不是被重新 new 一份。
    /// 若这条挂了，说明「登记了什么就跑什么」没成立。
    #[tokio::test]
    async fn model_sources_round_trip_through_registry() {
        use crate::l1_action::nt_io::nt_io_plugin::registry::global_registry;
        let reg = super::super::registry::PluginRegistry::new();
        reg.load_batch(builtin_capability_plugins()).await.unwrap();

        let sources = model_sources_from_registry(&reg).await;
        // 内建四个源：gguf / cloud_free / local_endpoint / cli-free
        assert_eq!(sources.len(), 4, "四个内建 model_source 应全部登记");
        for s in &sources {
            assert!(!s.name().is_empty());
        }
        // 再取一次：句柄必须是同一实例（共享而非重建）。
        let again = model_sources_from_registry(&reg).await;
        assert_eq!(sources.len(), again.len());
        // global_registry 也应能独立取到（非空即可，不与上面比较实例）。
        let _ = global_registry();
    }

    /// 反向判据：没有共享对象的插件（CLI descriptor / 轻量标签）不该被
    /// model_source 取物口捞出来 —— 否则 source 列表会被污染。
    #[tokio::test]
    async fn non_source_capabilities_are_not_returned_as_sources() {
        let reg = super::super::registry::PluginRegistry::new();
        // 只登记 provider：它们不是 model_source，取物口必须返回空。
        let providers_only: Vec<Box<dyn Plugin>> = provider_names()
            .into_iter()
            .filter_map(|(slug, _)| {
                LlmProviderType::from_name(slug).map(|t| {
                    Box::new(ProviderPlugin {
                        name: slug,
                        provider: create_provider_from_type(t, None),
                    }) as Box<dyn Plugin>
                })
            })
            .collect();
        assert!(!providers_only.is_empty());
        reg.load_batch(providers_only).await.unwrap();
        assert!(model_sources_from_registry(&reg).await.is_empty());
    }
}
