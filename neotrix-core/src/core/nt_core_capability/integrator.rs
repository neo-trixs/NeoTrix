//! 意识核心能力集成
//!
//! 将统一能力接口集成到意识核心

use super::discovery::{DiscoveryConfig, DiscoveryManager, DiscoveryResult, DistributedDiscovery};
use super::factory::CapabilityFactory;
use super::versioning::{SemanticVersion, UpgradeType, VersionManager};
use super::{
    CapabilityError, CapabilityInput, CapabilityOutput, CapabilityRegistry, CapabilityRouter,
};
use std::sync::Arc;

/// 意识核心能力集成器
pub struct ConsciousnessCapabilityIntegrator {
    /// 能力注册中心
    registry: CapabilityRegistry,
    /// 路由器
    router: CapabilityRouter,
    /// 版本管理器
    version_manager: VersionManager,
    /// 发现管理器
    discovery_manager: DiscoveryManager,
}

impl ConsciousnessCapabilityIntegrator {
    /// 创建新的集成器
    pub fn new() -> Self {
        let registry = CapabilityRegistry::new();
        let router = CapabilityRouter::new(Arc::new(registry.clone()));
        let discovery =
            DistributedDiscovery::new(DiscoveryConfig::default(), Arc::new(registry.clone()));
        let discovery_manager = DiscoveryManager::new(discovery);

        Self {
            registry,
            router,
            version_manager: VersionManager::new(),
            discovery_manager,
        }
    }

    /// 初始化所有内置能力
    pub fn init_builtin_capabilities(&mut self) {
        // 注册所有内置能力
        for cap in CapabilityFactory::create_all() {
            let meta = cap.meta();
            let version =
                SemanticVersion::parse(&meta.version).unwrap_or(SemanticVersion::new(1, 0, 0));

            self.registry.register(cap);

            self.version_manager
                .register_version(&meta.id, version, "system", "内置能力");
        }
    }

    /// 注册NT-MIND能力
    pub fn init_mind_capabilities(&mut self) {
        // TODO: implement when mind_capability module is created
    }

    /// 注册NT-MEMORY能力
    pub fn init_memory_capabilities(&mut self) {
        // TODO: implement when memory_capability module is created
    }

    /// 注册NT-ACT能力
    pub fn init_act_capabilities(&mut self) {
        // TODO: implement when act_capability module is created
    }

    /// 执行路由
    pub fn route(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        self.router.route(input)
    }

    /// 获取版本信息
    pub fn get_version(&self, capability_id: &str) -> Option<&SemanticVersion> {
        self.version_manager.get_current_version(capability_id)
    }

    /// 检查版本兼容性
    pub fn check_compatibility(&self, capability_id: &str, required: &SemanticVersion) -> bool {
        self.version_manager
            .check_compatibility(capability_id, required)
    }

    /// 升级能力版本
    pub fn upgrade(
        &mut self,
        capability_id: &str,
        upgrade_type: UpgradeType,
    ) -> Option<SemanticVersion> {
        self.version_manager.upgrade(capability_id, upgrade_type)
    }

    /// 回滚能力版本
    pub fn rollback(&mut self, capability_id: &str) -> Option<SemanticVersion> {
        self.version_manager.rollback(capability_id)
    }

    /// 执行分布式发现
    pub fn discover(&mut self) -> DiscoveryResult {
        self.discovery_manager.discovery.discover()
    }

    /// 获取注册中心
    pub fn registry(&self) -> &CapabilityRegistry {
        &self.registry
    }

    /// 获取路由器
    pub fn router(&self) -> &CapabilityRouter {
        &self.router
    }

    /// 获取版本管理器
    pub fn version_manager(&self) -> &VersionManager {
        &self.version_manager
    }
}

impl Default for ConsciousnessCapabilityIntegrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integrator_creation() {
        let mut integrator = ConsciousnessCapabilityIntegrator::new();
        integrator.init_builtin_capabilities();

        assert!(!integrator.registry.list_all().is_empty());
    }

    #[test]
    fn version_check() {
        let mut integrator = ConsciousnessCapabilityIntegrator::new();
        integrator.init_builtin_capabilities();

        let version = integrator.get_version("nt-world-nlp");
        assert!(version.is_some());
    }

    #[test]
    fn route_execution() {
        let mut integrator = ConsciousnessCapabilityIntegrator::new();
        integrator.init_builtin_capabilities();

        let input = CapabilityInput::Text("测试路由".into());
        let result = integrator.route(input);
        assert!(result.is_ok());
    }
}
