use serde::{Deserialize, Serialize};

/// Evolution Contract — Phase 0: Goal negotiation before growth cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionContract {
    pub cycle: u64,
    pub claim: String,              // What we intend to achieve this cycle
    pub evidence_plan: Vec<String>, // How we'll prove it (metrics, tests, artifacts)
    pub stop_rule: StopRule,        // Conditions to halt this evolution direction
    pub exploration_budget: f64,    // 0.0-1.0 fraction of resources for unconstrained exploration
    pub timestamp: u64,
}

/// Stop Rule — prevents runaway evolution in one direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopRule {
    pub max_generations_without_improvement: u64,
    pub min_quality_threshold: f64,
    pub max_resource_consumption: f64, // CPU/memory/time budget
    pub drift_tolerance: f64,          // How much deviation from contract before intervention
}

impl Default for StopRule {
    fn default() -> Self {
        Self {
            max_generations_without_improvement: 5,
            min_quality_threshold: 0.3,
            max_resource_consumption: 0.8,
            drift_tolerance: 0.2,
        }
    }
}

/// Contract Fulfillment — Phase 6: verified evidence of evolution contract completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractFulfillment {
    pub cycle: u64,
    pub claim: String,
    pub evidence_met: usize,
    pub evidence_total: usize,
    pub fulfilled: bool,
    pub quality_achieved: f64,
    pub timestamp: u64,
}

/// Drift Report — Phase 7: Post-cycle audit of evolution fidelity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub cycle: u64,
    pub contract_fulfilled: bool,
    pub claim_achieved: bool,
    pub evidence_collected: Vec<String>,
    pub quality_achieved: f64,
    pub resource_consumed: f64,
    pub drift_detected: bool,
    pub drift_magnitude: f64,
    pub stop_rule_triggered: bool,
    pub corrective_actions: Vec<String>,
    pub timestamp: u64,
}
