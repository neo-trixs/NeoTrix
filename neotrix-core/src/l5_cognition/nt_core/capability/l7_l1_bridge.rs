//! L7 → L1 Orchestrator Bridge
//!
//! 将 L7 编排模式 (Supervisor/Swarm/Pipeline) 桥接到 L1 统一 trait
//! 适配器模式: L7 trait 的 execute(task) → L1 trait 的 plan(goal) + execute(plan)

use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::nt_core_platform::{Agent, AgentError as PlatformAgentError, AgentMetrics as PlatformAgentMetrics, AgentStatus as PlatformAgentStatus};
use crate::l6_meta::nt_core_capability::{Layer, Domain, UnifiedCapability, CapabilityMeta, CapabilityHealth as PlatformCapabilityHealth, CapabilityState, CapabilityInput, CapabilityOutput, CapabilityError as PlatformCapabilityError};
use crate::l1_action::traits::{
    L1Capability, Orchestrator as L1Orchestrator, CapabilityCategory, ConstellationLevel,
    CapabilityHealth, CapabilityStats, CapabilityError,
    Plan, PlanStep, PlanResult,
};
use super::nt_act_orch_patterns::{
    Orchestrator as L7Orchestrator, AgentError as L7AgentError,
};
use async_trait::async_trait;
#[cfg(test)]
use super::nt_act_orch_patterns::{AgentOutput, OrchestratorStats};

// ════════════════════════════════════════════════════════════════
// 适配器: L7 Orchestrator → L1 Orchestrator
// ════════════════════════════════════════════════════════════════

/// SupervisorOrchestrator → L1 桥接
pub struct SupervisorL1Bridge {
    inner: Box<dyn L7Orchestrator>,
    id: String,
}

impl SupervisorL1Bridge {
    pub fn new(inner: Box<dyn L7Orchestrator>) -> Self {
        let id = format!("l7_bridge_{}", inner.pattern_name());
        Self { inner, id }
    }
}

impl L1Capability for SupervisorL1Bridge {
    fn capability_id(&self) -> &str { &self.id }
    fn category(&self) -> CapabilityCategory { CapabilityCategory::Execution }
    fn constellation(&self) -> ConstellationLevel { ConstellationLevel::C2Integration }
    fn health_check(&self) -> CapabilityHealth {
        CapabilityHealth {
            healthy: true,
            latency_ms: None,
            error_rate: 0.0,
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            message: Some(format!("pattern={}", self.inner.pattern_name())),
        }
    }
    fn description(&self) -> &str { "L7 orchestrator bridged to L1 trait" }
    fn stats(&self) -> CapabilityStats { CapabilityStats::default() }
}

impl L1Orchestrator for SupervisorL1Bridge {
    fn plan(&self, goal: &str) -> Result<Plan, CapabilityError> {
        // L7 execute 单步, L1 plan 返回步骤列表
        Ok(Plan {
            steps: vec![PlanStep {
                id: "step_0".into(),
                action: goal.to_string(),
                depends_on: vec![],
            }],
            estimated_duration_ms: 0,
        })
    }

    fn execute(&self, plan: &Plan) -> Result<PlanResult, CapabilityError> {
        let goal = plan.steps.first()
            .map(|s| s.action.as_str())
            .unwrap_or("empty");
        let output = self.inner.execute(goal)
            .map_err(|e| match e {
                L7AgentError::AgentNotFound(msg) => CapabilityError::NotAvailable(msg),
                L7AgentError::ExecutionFailed(msg) => CapabilityError::ExecutionFailed(msg),
                L7AgentError::Timeout(msg) => CapabilityError::Timeout(msg),
                L7AgentError::HandoffFailed(msg) => CapabilityError::ExecutionFailed(msg),
                L7AgentError::InvalidConfig(msg) => CapabilityError::InvalidInput(msg),
            })?;
        Ok(PlanResult {
            success: true,
            steps_completed: plan.steps.len(),
            output: Some(serde_json::json!({
                "agent": output.agent_name,
                "content": output.content,
                "confidence": output.confidence,
            })),
        })
    }
}

// ════════════════════════════════════════════════════════════════
// L7 编排能力注册中心
// ════════════════════════════════════════════════════════════════

/// L7 编排注册中心 — 注册所有 L7 编排模式的 L1 桥接
pub struct L7OrchestratorRegistry {
    bridges: Vec<Box<dyn L1Orchestrator>>,
}

impl Default for L7OrchestratorRegistry {
    fn default() -> Self { Self::new() }
}

impl L7OrchestratorRegistry {
    pub fn new() -> Self { Self { bridges: Vec::new() } }

    pub fn register_bridge(&mut self, bridge: Box<dyn L1Orchestrator>) {
        self.bridges.push(bridge);
    }

    pub fn optimal(&self) -> Option<&dyn L1Orchestrator> {
        self.bridges.iter()
            .filter(|b| b.health_check().healthy)
            .max_by(|a, b| {
                let a_s = 1.0 - a.health_check().error_rate;
                let b_s = 1.0 - b.health_check().error_rate;
                a_s.partial_cmp(&b_s).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|b| b.as_ref())
    }

    pub fn health_check_all(&self) -> Vec<(String, CapabilityHealth)> {
        self.bridges.iter().map(|b| (b.capability_id().to_string(), b.health_check())).collect()
    }
}

/// L7 编排路由器
pub struct L7OrchestratorRouter {
    registry: L7OrchestratorRegistry,
}

impl L7OrchestratorRouter {
    pub fn new(registry: L7OrchestratorRegistry) -> Self { Self { registry } }
    pub fn plan(&self, goal: &str) -> Result<Plan, CapabilityError> {
        self.registry.optimal()
            .ok_or_else(|| CapabilityError::NotAvailable("No L7 orchestrator".into()))?
            .plan(goal)
    }
    pub fn execute(&self, plan: &Plan) -> Result<PlanResult, CapabilityError> {
        self.registry.optimal()
            .ok_or_else(|| CapabilityError::NotAvailable("No L7 orchestrator".into()))?
            .execute(plan)
    }
}

/// L7 编排桥接
pub struct L7OrchestratorBridge {
    router: L7OrchestratorRouter,
}

impl L7OrchestratorBridge {
    pub fn new(router: L7OrchestratorRouter) -> Self { Self { router } }
    pub fn plan(&self, goal: &str) -> Result<Plan, CapabilityError> { self.router.plan(goal) }
    pub fn execute(&self, plan: &Plan) -> Result<PlanResult, CapabilityError> { self.router.execute(plan) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l7_bridge_trait() {
        // 使用 mock orchestrator
        struct MockL7Orch;
        impl L7Orchestrator for MockL7Orch {
            fn execute(&self, task: &str) -> Result<AgentOutput, L7AgentError> {
                Ok(AgentOutput::new("mock", &format!("done: {}", task)))
            }
            fn pattern_name(&self) -> &str { "mock" }
            fn stats(&self) -> OrchestratorStats { OrchestratorStats::default() }
        }

        let bridge = SupervisorL1Bridge::new(Box::new(MockL7Orch));
        assert_eq!(bridge.category(), CapabilityCategory::Execution);
        assert!(L1Capability::health_check(&bridge).healthy);

        let plan = bridge.plan("test goal").unwrap();
        assert_eq!(plan.steps.len(), 1);

        let result = L1Orchestrator::execute(&bridge, &plan).unwrap();
        assert!(result.success);
    }
}

// ════════════════════════════════════════════════════════════════
// Agent trait 实现 — 统一到 nt_core_platform
// ════════════════════════════════════════════════════════════════

impl UnifiedCapability for SupervisorL1Bridge {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: self.id.clone(),
            name: "SupervisorL1Bridge".to_string(),
            layer: Layer::L5Cognition,
            domain: Domain::NtAct,
            version: "0.1.0".to_string(),
            description: "L7 orchestrator bridged to L1 trait".to_string(),
            tags: vec!["bridge".to_string(), "l7".to_string(), "l1".to_string()],
            status: crate::l6_meta::nt_core_capability::CapabilityStatus::Healthy,
            metrics: crate::l6_meta::nt_core_capability::CapabilityMetrics::default(),
            cost_weight: 0.3,
            priority: 1.0,
        }
    }

    fn health(&self) -> PlatformCapabilityHealth {
        PlatformCapabilityHealth {
            state: CapabilityState::Ready,
            success_rate: 1.0,
            avg_latency_ms: 0.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, _input: CapabilityInput) -> Result<CapabilityOutput, PlatformCapabilityError> {
        Ok(CapabilityOutput::Text(format!(
            "SupervisorL1Bridge: pattern={}",
            self.inner.pattern_name()
        )))
    }

    fn supports(&self, _input: &CapabilityInput) -> bool {
        true
    }
}

#[async_trait]
impl Agent for SupervisorL1Bridge {
    fn agent_id(&self) -> &str { &self.id }
    fn agent_name(&self) -> &str { "SupervisorL1Bridge" }
    fn agent_layer(&self) -> Layer { Layer::L5Cognition }
    fn agent_domain(&self) -> Domain { Domain::NtAct }

    async fn initialize(&mut self) -> Result<(), PlatformAgentError> { Ok(()) }
    async fn start(&self) -> Result<(), PlatformAgentError> { Ok(()) }
    async fn stop(&self) -> Result<(), PlatformAgentError> { Ok(()) }
    fn status(&self) -> PlatformAgentStatus { PlatformAgentStatus::Running }
    fn metrics(&self) -> PlatformAgentMetrics { PlatformAgentMetrics::default() }
}

impl UnifiedCapability for L7OrchestratorRegistry {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "l7_orchestrator_registry".to_string(),
            name: "L7OrchestratorRegistry".to_string(),
            layer: Layer::L5Cognition,
            domain: Domain::NtAct,
            version: "0.1.0".to_string(),
            description: "L7 orchestrator registration center".to_string(),
            tags: vec!["registry".to_string(), "l7".to_string(), "orchestrator".to_string()],
            status: crate::l6_meta::nt_core_capability::CapabilityStatus::Healthy,
            metrics: crate::l6_meta::nt_core_capability::CapabilityMetrics::default(),
            cost_weight: 0.3,
            priority: 1.0,
        }
    }

    fn health(&self) -> PlatformCapabilityHealth {
        PlatformCapabilityHealth {
            state: CapabilityState::Ready,
            success_rate: 1.0,
            avg_latency_ms: 0.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, _input: CapabilityInput) -> Result<CapabilityOutput, PlatformCapabilityError> {
        Ok(CapabilityOutput::Text(format!(
            "L7OrchestratorRegistry: {} bridges",
            self.bridges.len()
        )))
    }

    fn supports(&self, _input: &CapabilityInput) -> bool {
        true
    }
}

#[async_trait]
impl Agent for L7OrchestratorRegistry {
    fn agent_id(&self) -> &str { "l7_orchestrator_registry" }
    fn agent_name(&self) -> &str { "L7OrchestratorRegistry" }
    fn agent_layer(&self) -> Layer { Layer::L5Cognition }
    fn agent_domain(&self) -> Domain { Domain::NtAct }

    async fn initialize(&mut self) -> Result<(), PlatformAgentError> { Ok(()) }
    async fn start(&self) -> Result<(), PlatformAgentError> { Ok(()) }
    async fn stop(&self) -> Result<(), PlatformAgentError> { Ok(()) }
    fn status(&self) -> PlatformAgentStatus { PlatformAgentStatus::Running }
    fn metrics(&self) -> PlatformAgentMetrics { PlatformAgentMetrics::default() }
}
