//! L7 → L1 Orchestrator Bridge
//!
//! 将 L7 编排模式 (Supervisor/Swarm/Pipeline) 桥接到 L1 统一 trait
//! 适配器模式: L7 trait 的 execute(task) → L1 trait 的 plan(goal) + execute(plan)

use std::time::{SystemTime, UNIX_EPOCH};

use crate::l1_action::traits::{
    L1Capability, Orchestrator as L1Orchestrator, CapabilityCategory, ConstellationLevel,
    CapabilityHealth, CapabilityStats, CapabilityError,
    Plan, PlanStep, PlanResult,
};
use super::nt_act_orch_patterns::{
    Orchestrator as L7Orchestrator, AgentError,
};

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
        Self { inner, id: format!("l7_bridge_{}", inner.pattern_name()) }
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
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
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
                AgentError::AgentNotFound(msg) => CapabilityError::NotAvailable(msg),
                AgentError::ExecutionFailed(msg) => CapabilityError::ExecutionFailed(msg),
                AgentError::Timeout(msg) => CapabilityError::Timeout(msg),
                AgentError::HandoffFailed(msg) => CapabilityError::ExecutionFailed(msg),
                AgentError::InvalidConfig(msg) => CapabilityError::InvalidInput(msg),
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
            fn execute(&self, task: &str) -> Result<AgentOutput, AgentError> {
                Ok(AgentOutput::new("mock", &format!("done: {}", task)))
            }
            fn pattern_name(&self) -> &str { "mock" }
            fn stats(&self) -> OrchestratorStats { OrchestratorStats::default() }
        }

        let bridge = SupervisorL1Bridge::new(Box::new(MockL7Orch));
        assert_eq!(bridge.category(), CapabilityCategory::Execution);
        assert!(bridge.health_check().healthy);

        let plan = bridge.plan("test goal").unwrap();
        assert_eq!(plan.steps.len(), 1);

        let result = bridge.execute(&plan).unwrap();
        assert!(result.success);
    }
}
