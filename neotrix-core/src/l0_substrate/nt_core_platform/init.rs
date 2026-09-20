//! NeoTrix Platform 初始化模块
//!
//! 提供全局 AgentRegistry、PipelineRegistry、MetricsCollector、HealthChecker 的初始化函数。
//! 注册所有已迁移的 Agent 实现到统一注册表。
//!
//! ## 待注册 Agent（TODO Tracking）
//!
//! 以下 Agent 尚未实现 `Agent` trait，待实现后取消注释即可注册：
//!
//! | Agent ID | 层级 | 说明 |
//! |---|---|---|
//! | `trade.orchestrator_v1` | L1 Action | 26 阶段贸易编排 |
//! | `trade.orchestrator_v2` | L1 Action | 下一代贸易编排 |
//! | `act.orchestrator` | L1 Action | 通用动作编排 |
//! | `act.production_orchestrator` | L1 Action | 生产环境编排 |
//! | `act.security_guard` | L1 Action | 安全防护管理 |
//! | `memory.lead_manager` | L1 Action | 线索管理 |
//! | `memory.kb_search` | L1 Action | 知识库搜索引擎 |
//! | `memory.elastic_orchestrator` | L1 Action | 弹性记忆编排 |
//! | `world.nlp` | L2 Perception | 自然语言处理 |
//! | `world.ocr` | L2 Perception | OCR 文字识别 |
//! | `shield.ztnet` | L3 Embodiment | 零信任网络安全 |
//! | `mind.memory_orchestrator` | L5 Cognition | 记忆管理 |
//! | `memory_agent` | L5 Cognition | 记忆 Agent |
//! | `dgm_edit_orchestrator` | L5 Cognition | DGM 编辑编排 |
//! | `file.pdf_enhance` | File Ability | PDF 增强能力 |
//!
//! ## 已注册 Agent（运行时）
//!
//! - `act.agent_orchestrator` — Agent 协议编排
//! - `l7_orchestrator_registry` — L7 编排注册表
//! - `core.recovery_orchestrator` — 错误恢复编排
//! - `mind.consciousness_orchestrator` — 意识循环编排（OnceLock 模式，运行时初始化）
//! - `supervisor.l1_bridge` — L1 桥接监督（需要具体 L7Orchestrator 实现，运行时初始化）

#![forbid(unsafe_code)]

use super::agent_registry::AgentRegistry;
use super::pipeline_registry::PipelineRegistry;
use super::metrics::MetricsCollector;
use super::health::HealthChecker;
use super::error::PlatformResult;

/// 初始化全局 AgentRegistry — 注册所有 Agent
///
/// 注册顺序: L0 Error Recovery → L1 Action → L2 Perception → L3 Embodiment → L5 Cognition
///
/// Note: L1 agents (e.g. AgentOrchestrator) are registered by their own layer's
/// init function, not here, to enforce the substrate invariant (L0 must not import L1).
pub async fn init_agent_registry() -> PlatformResult<AgentRegistry> {
    let registry = AgentRegistry::new();

    // ── L0 Error Recovery ─────────────────────────────────────

    // RecoveryOrchestrator — 错误恢复编排
    {
        use crate::l0_substrate::nt_core_error::recovery::{RecoveryOrchestrator, RecoveryConfig};
        let agent = RecoveryOrchestrator::new(RecoveryConfig::default());
        registry.register(Box::new(agent)).await.map_err(|e| {
            super::error::PlatformError::Agent(format!("core.recovery_orchestrator: {}", e))
        })?;
    }

    // ── L1 Action Agents (registered via deferred init) ───────
    // AgentOrchestrator registration moved to L1 to enforce substrate invariant.
    // Callers should invoke l1_action::nt_act::agent_protocol::register_l1_agents(&registry)
    // after calling init_agent_registry().

    Ok(registry)
}

/// 初始化全局 PipelineRegistry
///
/// 注册所有已实现 Pipeline trait 的管线到统一注册表。
pub async fn init_pipeline_registry() -> PlatformResult<PipelineRegistry> {
    let registry = PipelineRegistry::new();

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
        assert!(registry.len().await >= 1);
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

        // L1 AgentOrchestrator is registered by L1, not L0 (substrate invariant)
        assert!(ids.contains(&"core.recovery_orchestrator".to_string()));
    }
}
