//! NeoTrix 意识核心能力注册
//!
//! 自动注册所有内置能力到能力注册中心

use crate::core::nt_core_capability::*;
use std::sync::Arc;

/// 内置能力工厂
pub struct CapabilityFactory;

impl CapabilityFactory {
    /// 创建所有内置能力
    pub fn create_all() -> Vec<Arc<dyn UnifiedCapability>> {
        vec![
            // NT-WORLD NLP
            crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability(),
            // NT-SHIELD ZT-Net
            crate::l3_embodiment::nt_shield::nt_shield_ztnet::ztnet_capability::create_ztnet_capability(),
            // NT-WORLD Asset Map
            crate::l2_perception::nt_world::asset_map::asset_map_capability::create_asset_map_capability(),
            // NT-FILE-ABILITY PDF Enhance
            crate::neotrix::nt_file_ability::create_pdf_enhance_capability(),
        ]
    }

    /// 按域创建能力
    pub fn create_by_domain(domain: Domain) -> Vec<Arc<dyn UnifiedCapability>> {
        Self::create_all()
            .into_iter()
            .filter(|cap| cap.meta().domain == domain)
            .collect()
    }

    /// 按层级创建能力
    pub fn create_by_layer(layer: Layer) -> Vec<Arc<dyn UnifiedCapability>> {
        Self::create_all()
            .into_iter()
            .filter(|cap| cap.meta().layer == layer)
            .collect()
    }
}

/// 初始化全局能力注册中心
pub fn init_global_registry() -> CapabilityRegistry {
    let mut registry = CapabilityRegistry::new();

    for cap in CapabilityFactory::create_all() {
        registry.register(cap);
    }

    // 注册 PDF 增强能力
    registry.register(
        crate::neotrix::nt_file_ability::create_pdf_enhance_capability()
    );

    registry
}

/// 创建带默认路由的路由器
pub fn create_default_router() -> CapabilityRouter {
    let registry = Arc::new(init_global_registry());
    let mut router = CapabilityRouter::new(registry);

    // 添加默认路由规则
    router.add_rule(|input| match input {
        CapabilityInput::Nlp(_) => Some("nt-world-nlp".into()),
        CapabilityInput::Asset(_) => Some("nt-world-asset-map".into()),
        CapabilityInput::Network(_) => Some("nt-shield-ztnet".into()),
        CapabilityInput::Security(_) => Some("nt-shield-ztnet".into()),
        CapabilityInput::FileEnhance(_) => Some("nt-file-pdf-enhance".into()),
        _ => None,
    });

    router
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_creates_all() {
        let caps = CapabilityFactory::create_all();
        assert!(caps.len() >= 3);
    }

    #[test]
    fn factory_by_domain() {
        let caps = CapabilityFactory::create_by_domain(Domain::NtWorld);
        assert!(!caps.is_empty());
    }

    #[test]
    fn global_registry() {
        let registry = init_global_registry();
        assert!(registry.list_all().len() >= 4); // NLP + ZT-Net + Asset + PDF Enhance
    }

    #[test]
    fn router_creation() {
        let router = create_default_router();
        assert!(!router.registry().list_all().is_empty());
    }
}
