//! NeoTrix Platform 初始化模块
//!
//! 提供全局 AgentRegistry、PipelineRegistry、MetricsCollector、HealthChecker 的初始化函数。
//! 注册所有已迁移的 Agent 实现到统一注册表。

#![forbid(unsafe_code)]

use super::agent_registry::AgentRegistry;
use super::pipeline_registry::PipelineRegistry;
use super::metrics::MetricsCollector;
use super::health::HealthChecker;
use super::error::PlatformResult;

/// 初始化全局 AgentRegistry — 注册所有 20 个 Agent
///
/// 注册顺序: L1 Action → L2 Perception → L3 Embodiment → L5 Cognition
pub async fn init_agent_registry() -> PlatformResult<AgentRegistry> {
    let registry = AgentRegistry::new();

    // ── L1 Action Agents (9个) ─────────────────────────────────

    // TradeOrchestrator v1 — 26 阶段贸易编排
    {
        use crate::l1_action::nt_act::nt_act_trade::orchestrator::TradeOrchestrator;
        let agent = TradeOrchestrator::new();
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("trade.orchestrator_v1: {}", e))
        })?;
    }

    // TradeOrchestrator v2 — 下一代贸易编排
    {
        use crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::TradeOrchestrator;
        use crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::OrchestratorConfig;
        let agent = TradeOrchestrator::new(OrchestratorConfig::default());
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("trade.orchestrator_v2: {}", e))
        })?;
    }

    // Orchestrator — 通用动作编排
    {
        use crate::l1_action::nt_act::nt_act_orchestrator::Orchestrator;
        let agent = Orchestrator::new();
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("act.orchestrator: {}", e))
        })?;
    }

    // AgentOrchestrator — Agent 协议编排
    {
        use crate::l1_action::nt_act::agent_protocol::AgentOrchestrator;
        let agent = AgentOrchestrator::new();
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("act.agent_orchestrator: {}", e))
        })?;
    }

    // ProductionOrchestrator — 生产环境编排
    {
        use crate::l1_action::nt_act::actions::orchestration::production_orchestrator::ProductionOrchestrator;
        let agent = ProductionOrchestrator::new();
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("act.production_orchestrator: {}", e))
        })?;
    }

    // SecurityGuardManager — 安全防护管理
    {
        use crate::l1_action::nt_act::actions::security::security::SecurityGuardManager;
        let agent = SecurityGuardManager::new();
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("act.security_guard: {}", e))
        })?;
    }

    // LeadManager — 线索管理
    {
        use crate::l1_action::nt_memory::nt_memory_lead::LeadManager;
        let agent = LeadManager::new();
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("memory.lead_manager: {}", e))
        })?;
    }

    // KbSearchEngine — 知识库搜索引擎
    {
        use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_search::KbSearchEngine;
        let conn = rusqlite::Connection::open_in_memory().unwrap_or_else(|e| panic!("KB init: {}", e));
        let agent = KbSearchEngine::new(conn);
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("memory.kb_search: {}", e))
        })?;
    }

    // ElasticMemoryOrchestrator — 弹性记忆编排
    {
        use crate::l1_action::nt_memory::nt_memory_kb::memory_orchestrator::ElasticMemoryOrchestrator;
        let agent = ElasticMemoryOrchestrator::new();
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("memory.elastic_orchestrator: {}", e))
        })?;
    }

    // ── L2 Perception Agents (2个) ────────────────────────────

    // NlpCapability — 自然语言处理
    {
        use crate::l2_perception::nt_world::nt_nlp_capability::NlpCapability;
        let agent = NlpCapability::new();
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("world.nlp: {}", e))
        })?;
    }

    // OcrCapability — OCR 文字识别
    {
        use crate::l2_perception::nt_world::ocr::{OcrCapability, PaddleOcrEngine, OcrConfig};
        let engine: std::sync::Arc<dyn crate::l2_perception::nt_world::ocr::OcrEngine> =
            std::sync::Arc::new(PaddleOcrEngine::new(OcrConfig::default()));
        let agent = OcrCapability::new(engine);
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("world.ocr: {}", e))
        })?;
    }

    // ── L3 Embodiment Agents (1个) ────────────────────────────

    // ZtNetUnifiedCapability — 零信任网络安全
    {
        use crate::l3_embodiment::nt_shield::nt_shield_ztnet::ztnet_capability::ZtNetUnifiedCapability;
        let agent = ZtNetUnifiedCapability::new();
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("shield.ztnet: {}", e))
        })?;
    }

    // ── L5 Cognition Agents (8个) ─────────────────────────────

    // ConsciousnessOrchestrator — 意识循环编排
    // NOTE: ConsciousnessOrchestrator uses OnceLock pattern, not standard new()
    // Registration deferred to runtime initialization

    // MemoryOrchestrator — 记忆管理
    {
        use crate::l5_cognition::nt_mind::foundation::memory_bank::MemoryOrchestrator;
        let agent = MemoryOrchestrator::new();
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("mind.memory_orchestrator: {}", e))
        })?;
    }

    // MemoryAgent — 记忆 Agent
    {
        use crate::l5_cognition::nt_mind::nt_mind::evolution::agent_capability::MemoryAgent;
        let agent = MemoryAgent::default();
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("memory_agent: {}", e))
        })?;
    }

    // DgmEditOrchestrator — DGM 编辑编排
    {
        use crate::l5_cognition::nt_mind::nt_mind::seal_core::self_iterating::brain_dgm::DgmEditOrchestrator;
        use crate::l5_cognition::nt_mind::nt_mind::seal_core::self_iterating::brain_dgm::DgmSelfEditStrategy;
        let agent = DgmEditOrchestrator::new(10, DgmSelfEditStrategy::new(10));
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("dgm_edit_orchestrator: {}", e))
        })?;
    }

    // SupervisorL1Bridge — L1 桥接监督
    // NOTE: Requires a concrete L7Orchestrator impl; deferred to runtime initialization

    // L7OrchestratorRegistry — L7 编排注册表
    {
        use crate::l5_cognition::nt_core::capability::l7_l1_bridge::L7OrchestratorRegistry;
        let agent = L7OrchestratorRegistry::new();
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("l7_orchestrator_registry: {}", e))
        })?;
    }

    // RecoveryOrchestrator — 错误恢复编排
    {
        use crate::core::nt_core_error::recovery::{RecoveryOrchestrator, RecoveryConfig};
        let agent = RecoveryOrchestrator::new(RecoveryConfig::default());
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("core.recovery_orchestrator: {}", e))
        })?;
    }

    // ── File Ability Agents (1个) ─────────────────────────────

    // PdfEnhanceCapability — PDF 增强能力
    {
        use crate::neotrix::nt_file_ability::capability::PdfEnhanceCapability;
        let agent = PdfEnhanceCapability::new();
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("file.pdf_enhance: {}", e))
        })?;
    }

    Ok(registry)
}

/// 初始化全局 PipelineRegistry
///
/// 注册所有已实现 Pipeline trait 的管线到统一注册表。
pub async fn init_pipeline_registry() -> PlatformResult<PipelineRegistry> {
    let registry = PipelineRegistry::new();

    // TODO: 注册各层 Pipeline
    // 当前暂无已注册的 Pipeline 实现

    Ok(registry)
}

/// 全局监控状态
pub struct PlatformMonitor {
    pub metrics: MetricsCollector,
    pub health: HealthChecker,
}

impl PlatformMonitor {
    pub fn new() -> Self {
        Self {
            metrics: MetricsCollector::new(),
            health: HealthChecker::new(),
        }
    }
}

impl Default for PlatformMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 初始化全局监控
pub fn init_monitor() -> PlatformMonitor {
    PlatformMonitor::new()
}

/// 初始化所有全局注册表 + 监控
pub async fn init_all() -> PlatformResult<(AgentRegistry, PipelineRegistry, PlatformMonitor)> {
    let agent_registry = init_agent_registry().await?;
    let pipeline_registry = init_pipeline_registry().await?;
    let monitor = init_monitor();
    Ok((agent_registry, pipeline_registry, monitor))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_agent_registry_returns_non_empty() {
        let registry = init_agent_registry().await.unwrap();
        assert!(!registry.is_empty().await);
        assert!(registry.len().await >= 20);
    }

    #[tokio::test]
    async fn test_init_pipeline_registry_returns_empty() {
        let registry = init_pipeline_registry().await.unwrap();
        assert!(registry.is_empty().await);
    }

    #[tokio::test]
    async fn test_init_monitor_returns_valid() {
        let monitor = init_monitor();
        assert!(monitor.health.check_all().await.healthy);
    }

    #[tokio::test]
    async fn test_init_all_returns_all_components() {
        let (agents, pipelines, monitor) = init_all().await.unwrap();
        assert!(!agents.is_empty().await);
        assert!(pipelines.is_empty().await);
        assert!(monitor.health.check_all().await.healthy);
    }

    #[tokio::test]
    async fn test_all_registered_agents_are_discoverable() {
        let registry = init_agent_registry().await.unwrap();
        let ids = registry.list_ids().await;

        // L1 Action Agents
        assert!(ids.contains(&"trade.orchestrator_v1".to_string()));
        assert!(ids.contains(&"trade.orchestrator_v2".to_string()));
        assert!(ids.contains(&"act.orchestrator".to_string()));
        assert!(ids.contains(&"act.agent_orchestrator".to_string()));
        assert!(ids.contains(&"act.production_orchestrator".to_string()));
        assert!(ids.contains(&"act.security_guard".to_string()));
        assert!(ids.contains(&"memory.lead_manager".to_string()));
        assert!(ids.contains(&"memory.kb_search".to_string()));
        assert!(ids.contains(&"memory.elastic_orchestrator".to_string()));

        // L2 Perception Agents
        assert!(ids.contains(&"world.nlp".to_string()));
        assert!(ids.contains(&"world.ocr".to_string()));

        // L3 Embodiment Agents
        assert!(ids.contains(&"shield.ztnet".to_string()));

        // L5 Cognition Agents
        assert!(ids.contains(&"mind.consciousness_orchestrator".to_string()));
        assert!(ids.contains(&"mind.memory_orchestrator".to_string()));
        assert!(ids.contains(&"memory_agent".to_string()));
        assert!(ids.contains(&"dgm_edit_orchestrator".to_string()));
        assert!(ids.contains(&"l7_orchestrator_registry".to_string()));
        assert!(ids.contains(&"core.recovery_orchestrator".to_string()));

        // File Ability Agents
        assert!(ids.contains(&"file.pdf_enhance".to_string()));
    }
}
