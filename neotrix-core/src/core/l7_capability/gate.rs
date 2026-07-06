use serde::{Deserialize, Serialize};
use crate::core::l7_capability::registry::{
    Capability,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateCheck { Permission, Budget, Circuit, Humility }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GateResult {
    Pass,
    Blocked { reason: String, check: GateCheck },
}

impl GateResult { pub fn is_pass(&self) -> bool { matches!(self, Self::Pass) } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetState {
    pub tokens_remaining: u64,
    pub ms_remaining: u64,
    pub memory_kb_remaining: u64,
}

impl Default for BudgetState {
    fn default() -> Self { Self { tokens_remaining: 100_000, ms_remaining: 30_000, memory_kb_remaining: 1024 } }
}

pub struct GreatFilterGate {
    pub humility_threshold: f64,
    pub max_exploration_bias: f64,
}

impl Default for GreatFilterGate { fn default() -> Self { Self::new(0.3, 0.5) } }

impl GreatFilterGate {
    pub fn new(humility_threshold: f64, max_exploration_bias: f64) -> Self {
        Self { humility_threshold, max_exploration_bias }
    }

    pub fn verify(&self, capability: &Capability, budget: &BudgetState, illusion_risk: f64) -> GateResult {
        let perm = self.check_permission(capability);
        if !perm.is_pass() { return perm; }
        let bgt = self.check_budget(capability, budget);
        if !bgt.is_pass() { return bgt; }
        let humility = self.check_humility(illusion_risk);
        if !humility.is_pass() { return humility; }
        GateResult::Pass
    }

    pub fn verify_promotion(&self, _cap: &Capability, illusion_risk: f64, p_value: f64) -> GateResult {
        if illusion_risk > self.humility_threshold {
            return GateResult::Blocked {
                reason: format!("Illusion risk {:.2} exceeds threshold {:.2}", illusion_risk, self.humility_threshold),
                check: GateCheck::Humility,
            };
        }
        if p_value < 0.05 {
            return GateResult::Blocked {
                reason: format!("p={:.3} below 0.05", p_value),
                check: GateCheck::Humility,
            };
        }
        GateResult::Pass
    }

    fn check_permission(&self, _capability: &Capability) -> GateResult { GateResult::Pass }
    fn check_budget(&self, capability: &Capability, budget: &BudgetState) -> GateResult {
        if capability.cost.estimated_tokens > budget.tokens_remaining {
            return GateResult::Blocked {
                reason: format!("Need {} tokens, have {}", capability.cost.estimated_tokens, budget.tokens_remaining),
                check: GateCheck::Budget,
            };
        }
        if capability.cost.estimated_ms > budget.ms_remaining {
            return GateResult::Blocked { reason: "Time budget exceeded".into(), check: GateCheck::Budget };
        }
        GateResult::Pass
    }
    fn check_humility(&self, illusion_risk: f64) -> GateResult {
        if illusion_risk > self.humility_threshold {
            GateResult::Blocked { reason: format!("Illusion risk {:.2}", illusion_risk), check: GateCheck::Humility }
        } else { GateResult::Pass }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::l7_capability::registry::{capability_id_from_name, CapabilityCost, CapabilityKind, CapabilityStats, CapabilityVector, CapabilityTier, CapabilityRuntime, CapabilityStability, DomainCategory, MaturityLevel};

    fn test_cap() -> Capability {
        Capability {
            id: capability_id_from_name("t"), name: "t".into(), tags: vec!["t".into()],
            kind: CapabilityKind::Cognitive, maturity: MaturityLevel::Candidate,
            vector: CapabilityVector::default(), e8_triggers: vec![],
            context_requirements: vec![],
            cost: CapabilityCost { estimated_tokens: 500, estimated_ms: 200, estimated_memory_kb: 64 },
            stats: CapabilityStats::default(), version: "0.1.0".into(), layer: 4,
            tier: CapabilityTier::Core, runtime: CapabilityRuntime::Local,
            stability: CapabilityStability::Production,
            fallback_chain: vec![], provider: None,
            domain: DomainCategory::General,
            input_schema: None, output_schema: None,
            resource_cpu: 1.0, resource_ram_mb: 64.0, resource_vram_mb: 0.0,
            dependencies: vec![],
        }
    }

    #[test] fn test_gate_pass() {
        let g = GreatFilterGate::default();
        assert_eq!(g.verify(&test_cap(), &BudgetState::default(), 0.1), GateResult::Pass);
    }

    #[test] fn test_gate_blocked_budget() {
        let g = GreatFilterGate::default();
        let b = BudgetState { tokens_remaining: 100, ms_remaining: 30_000, memory_kb_remaining: 1024 };
        let r = g.verify(&test_cap(), &b, 0.1);
        assert!(matches!(r, GateResult::Blocked { check: GateCheck::Budget, .. }));
    }

    #[test] fn test_gate_blocked_humility() {
        let g = GreatFilterGate::default();
        let r = g.verify(&test_cap(), &BudgetState::default(), 0.5);
        assert!(matches!(r, GateResult::Blocked { check: GateCheck::Humility, .. }));
    }

    #[test] fn test_promotion_gate() {
        let g = GreatFilterGate::default();
        assert_eq!(g.verify_promotion(&test_cap(), 0.1, 0.5), GateResult::Pass);
        assert!(matches!(g.verify_promotion(&test_cap(), 0.5, 0.01), GateResult::Blocked { .. }));
        assert!(matches!(g.verify_promotion(&test_cap(), 0.1, 0.01), GateResult::Blocked { .. }));
    }
}
