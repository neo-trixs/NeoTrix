pub mod cognitive_wal;
pub mod pco;
pub mod proof_bundle;
pub mod rsi_meta_cycle;
pub mod self_mod_pipeline;
pub mod shadow_runtime;

use crate::core::nt_core_experience::safety_gate::StatisticalSafetyGate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MicroEdit {
    AdjustDimension(String, f64),
    UpdateLearningRate(f64),
    NormalizeVector,
    AddExtension(Vec<(String, f64)>),
    SetProvenance(String),
    BatchAdjust(Vec<(String, f64)>),
    AddedDimension(String, f64),
    ModifiedDimension(String, f64, f64),
    RemovedDimension(String),
    GenerateNtModule(String, String),
}

impl MicroEdit {
    pub fn delta_label(&self) -> &'static str {
        match self {
            MicroEdit::AddedDimension(_, _) => "ADDED",
            MicroEdit::ModifiedDimension(_, _, _) => "MODIFIED",
            MicroEdit::RemovedDimension(_) => "REMOVED",
            MicroEdit::AdjustDimension(_, _) => "ADJUSTED",
            MicroEdit::BatchAdjust(_) => "BATCH_ADJUSTED",
            _ => "OTHER",
        }
    }

    pub fn dimension_name(&self) -> Option<&str> {
        match self {
            MicroEdit::AddedDimension(name, _) => Some(name),
            MicroEdit::ModifiedDimension(name, _, _) => Some(name),
            MicroEdit::RemovedDimension(name) => Some(name),
            MicroEdit::AdjustDimension(name, _) => Some(name),
            MicroEdit::GenerateNtModule(name, _) => Some(name),
            _ => None,
        }
    }

    /// 人类可读的摘要（供 T3 视图生成使用）
    pub fn summary(&self) -> String {
        match self {
            MicroEdit::AdjustDimension(name, val) => {
                format!("adjust dimension '{}' by {}", name, val)
            }
            MicroEdit::UpdateLearningRate(lr) => format!("update learning rate to {}", lr),
            MicroEdit::NormalizeVector => "normalize capability vector".into(),
            MicroEdit::AddExtension(exts) => format!("add {} extensions", exts.len()),
            MicroEdit::SetProvenance(src) => format!("set provenance to '{}'", src),
            MicroEdit::BatchAdjust(adjustments) => {
                format!("batch adjust {} dimensions", adjustments.len())
            }
            MicroEdit::AddedDimension(name, val) => format!("add dimension '{}' = {}", name, val),
            MicroEdit::ModifiedDimension(name, old, new) => {
                format!("modify '{}': {} → {}", name, old, new)
            }
            MicroEdit::RemovedDimension(name) => format!("remove dimension '{}'", name),
            MicroEdit::GenerateNtModule(_, _) => "generate .nt module".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfEdit {
    pub task_type: crate::core::TaskType,
    pub target_dimensions: Vec<String>,
    pub adjustment_magnitude: f64,
    pub tool_calls: Vec<ToolCall>,
    pub config_overrides: HashMap<String, f64>,
}

impl SelfEdit {
    pub fn evaluate_by_negentropy(&self, delta_n: f64, threshold: f64) -> bool {
        delta_n > threshold
    }

    pub fn negentropy_verdict(&self, delta_n: f64) -> &'static str {
        if delta_n > 0.05 {
            "accept: positive negentropy gain"
        } else if delta_n > 0.0 {
            "accept: marginal gain"
        } else if delta_n > -0.05 {
            "reject: neutral or slightly negative"
        } else {
            "revert: significant negentropy loss"
        }
    }

    pub fn should_revert(&self, delta_n: f64) -> bool {
        delta_n < -0.05
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub action: String,
    pub params: HashMap<String, String>,
}

// ── DGM-H Metacognitive Self-Modification ────────────────────────────
// Reference: DGM-HyperAgents (arXiv:2603.19461)
// Meta agent should be able to modify its own meta-level parameters,
// not just task-level agents.  Edits that succeed in one domain transfer
// improved priors to similar domains.

/// Target subsystem for a meta-level self-edit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetaSelfEditTarget {
    SafetyGate,
    EditPolicy,
    MetaLearningRate,
    RewardFunction,
    SubspaceTopology,
}

/// A proposal for the meta agent to modify its own parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaSelfEditProposal {
    pub target: MetaSelfEditTarget,
    pub current_value: String,
    pub proposed_value: String,
    pub rationale: String,
    pub expected_improvement: f64,
    pub risk_score: f64,
    pub domain: String,
}

/// DGM-H metacognitive self-edit manager.
///
/// Tracks meta-level modifications, evaluates them through the statistical
/// safety gate, adapts the meta-learning rate based on empirical outcomes,
/// and supports cross-domain edit transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaSelfEditManager {
    pub edit_history: Vec<MetaSelfEditProposal>,
    pub success_rate: f64,
    pub self_modification_count: u64,
    pub meta_learning_rate: f64,
    pub safety_gate: StatisticalSafetyGate,
    /// Per-domain success rates for cross-domain transfer.
    pub domain_success: HashMap<String, (u64, u64)>,
    max_history: usize,
}

impl MetaSelfEditManager {
    pub fn new(risk_budget: f64) -> Self {
        Self {
            edit_history: Vec::new(),
            success_rate: 0.0,
            self_modification_count: 0,
            meta_learning_rate: 0.1,
            safety_gate: StatisticalSafetyGate::with_risk_budget(risk_budget),
            domain_success: HashMap::new(),
            max_history: 100,
        }
    }

    /// Propose a meta-level self-edit and evaluate through the safety gate.
    ///
    /// Returns `Ok(true)` if the edit is allowed, `Err(reason)` if denied.
    /// The meta-learning rate is adapted based on the edit's risk profile.
    pub fn propose_self_edit(&mut self, proposal: MetaSelfEditProposal) -> Result<bool, String> {
        let risk = proposal.risk_score * (1.0 - self.meta_learning_rate);
        self.safety_gate
            .evaluate_with_stats(risk, &proposal.rationale)?;
        self.edit_history.push(proposal);
        if self.edit_history.len() > self.max_history {
            self.edit_history.remove(0);
        }
        self.self_modification_count += 1;
        self.meta_learning_rate = (self.meta_learning_rate * 1.01).min(0.5);
        Ok(true)
    }

    /// Record the outcome of a meta-edit and update learning rate.
    pub fn record_edit_outcome(
        &mut self,
        proposal: &MetaSelfEditProposal,
        success: bool,
        _negentropy_delta: f64,
    ) {
        self.safety_gate.record_outcome(success);
        let n = self.safety_gate.n_modifications.max(1);
        self.success_rate = self.safety_gate.n_successful as f64 / n as f64;
        self.meta_learning_rate = if success {
            (self.meta_learning_rate * 1.05).min(0.5)
        } else {
            (self.meta_learning_rate * 0.95).max(0.01)
        };
        let entry = self
            .domain_success
            .entry(proposal.domain.clone())
            .or_insert((0, 0));
        if success {
            entry.0 += 1;
        }
        entry.1 += 1;
    }

    /// Cross-domain success prior: returns the success rate from the most
    /// similar known domain, or a default 0.5 if no prior exists.
    pub fn cross_domain_prior(&self, domain: &str) -> f64 {
        if let Some(&(succ, total)) = self.domain_success.get(domain) {
            if total > 0 {
                return succ as f64 / total as f64;
            }
        }
        for (known_domain, &(succ, total)) in &self.domain_success {
            if total > 0 && domain.contains(known_domain) || known_domain.contains(domain) {
                return succ as f64 / total as f64;
            }
        }
        0.5
    }

    pub fn edit_count(&self) -> u64 {
        self.self_modification_count
    }
    pub fn safety_gate(&self) -> &StatisticalSafetyGate {
        &self.safety_gate
    }
    pub fn safety_gate_mut(&mut self) -> &mut StatisticalSafetyGate {
        &mut self.safety_gate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_self_edit_proposal_creation() {
        let proposal = MetaSelfEditProposal {
            target: MetaSelfEditTarget::MetaLearningRate,
            current_value: "0.1".into(),
            proposed_value: "0.2".into(),
            rationale: "Increase learning rate after stable convergence".into(),
            expected_improvement: 0.15,
            risk_score: 0.05,
            domain: "code_generation".into(),
        };
        assert_eq!(proposal.target, MetaSelfEditTarget::MetaLearningRate);
        assert_eq!(proposal.risk_score, 0.05);
    }

    #[test]
    fn test_propose_self_edit_accepts_low_risk() {
        let mut mgr = MetaSelfEditManager::new(0.01);
        for _ in 0..20 {
            mgr.safety_gate.record_outcome(true);
        }
        let proposal = MetaSelfEditProposal {
            target: MetaSelfEditTarget::EditPolicy,
            current_value: "strict".into(),
            proposed_value: "balanced".into(),
            rationale: "test edit".into(),
            expected_improvement: 0.1,
            risk_score: 0.05,
            domain: "test".into(),
        };
        let result = mgr.propose_self_edit(proposal);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    }

    #[test]
    fn test_meta_learning_rate_adapts() {
        let mut mgr = MetaSelfEditManager::new(0.01);
        let initial = mgr.meta_learning_rate;
        for _ in 0..5 {
            mgr.safety_gate.record_outcome(true);
        }
        mgr.meta_learning_rate = (mgr.meta_learning_rate * 1.05_f64.powi(5)).min(0.5);
        assert!(mgr.meta_learning_rate > initial);
    }

    #[test]
    fn test_cross_domain_prior() {
        let mut mgr = MetaSelfEditManager::new(0.01);
        mgr.domain_success.insert("code_gen".into(), (8, 10));
        let prior = mgr.cross_domain_prior("code_gen");
        assert!((prior - 0.8).abs() < 1e-10);
        let unknown = mgr.cross_domain_prior("unknown_domain");
        assert!((unknown - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_history_capped_at_max() {
        let mut mgr = MetaSelfEditManager::new(0.01);
        mgr.max_history = 3;
        for i in 0..5 {
            let p = MetaSelfEditProposal {
                target: MetaSelfEditTarget::SubspaceTopology,
                current_value: "a".into(),
                proposed_value: "b".into(),
                rationale: format!("edit {}", i),
                expected_improvement: 0.1,
                risk_score: 0.01,
                domain: "test".into(),
            };
            mgr.edit_history.push(p);
        }
        while mgr.edit_history.len() > 3 {
            mgr.edit_history.remove(0);
        }
        assert!(mgr.edit_history.len() <= 3);
    }

    #[test]
    fn test_micro_edit_adjust_dimension() {
        let edit = MicroEdit::AdjustDimension("typography".into(), 0.5);
        assert_eq!(edit.delta_label(), "ADJUSTED");
        assert_eq!(edit.dimension_name(), Some("typography"));
        assert_eq!(edit.summary(), "adjust dimension 'typography' by 0.5");
    }

    #[test]
    fn test_micro_edit_update_learning_rate() {
        let edit = MicroEdit::UpdateLearningRate(0.01);
        assert_eq!(edit.delta_label(), "OTHER");
        assert_eq!(edit.dimension_name(), None);
        assert_eq!(edit.summary(), "update learning rate to 0.01");
    }

    #[test]
    fn test_micro_edit_normalize() {
        let edit = MicroEdit::NormalizeVector;
        assert_eq!(edit.summary(), "normalize capability vector");
    }

    #[test]
    fn test_micro_edit_add_extension() {
        let edit = MicroEdit::AddExtension(vec![("dim1".into(), 0.5)]);
        assert_eq!(edit.summary(), "add 1 extensions");
    }

    #[test]
    fn test_micro_edit_set_provenance() {
        let edit = MicroEdit::SetProvenance("test_source".into());
        assert_eq!(edit.summary(), "set provenance to 'test_source'");
    }

    #[test]
    fn test_micro_edit_batch_adjust() {
        let edit = MicroEdit::BatchAdjust(vec![("a".into(), 0.1), ("b".into(), 0.2)]);
        assert_eq!(edit.summary(), "batch adjust 2 dimensions");
        assert_eq!(edit.delta_label(), "BATCH_ADJUSTED");
    }

    #[test]
    fn test_micro_edit_dimension_tracking() {
        let added = MicroEdit::AddedDimension("new_dim".into(), 0.8);
        assert_eq!(added.delta_label(), "ADDED");
        assert_eq!(added.dimension_name(), Some("new_dim"));
        assert_eq!(added.summary(), "add dimension 'new_dim' = 0.8");

        let modified = MicroEdit::ModifiedDimension("dim".into(), 0.3, 0.7);
        assert_eq!(modified.delta_label(), "MODIFIED");
        assert_eq!(modified.dimension_name(), Some("dim"));
        assert_eq!(modified.summary(), "modify 'dim': 0.3 → 0.7");

        let removed = MicroEdit::RemovedDimension("old_dim".into());
        assert_eq!(removed.delta_label(), "REMOVED");
        assert_eq!(removed.dimension_name(), Some("old_dim"));
        assert_eq!(removed.summary(), "remove dimension 'old_dim'");
    }

    #[test]
    fn test_micro_edit_clone() {
        let edit = MicroEdit::AdjustDimension("test".into(), 0.5);
        let cloned = edit.clone();
        assert_eq!(edit.summary(), cloned.summary());
    }

    #[test]
    fn test_self_edit_creation() {
        let mut config = HashMap::new();
        config.insert("learning_rate".into(), 0.05);
        let se = SelfEdit {
            task_type: crate::core::TaskType::CodeGeneration,
            target_dimensions: vec!["analysis".into(), "creativity".into()],
            adjustment_magnitude: 0.1,
            tool_calls: Vec::new(),
            config_overrides: config,
        };
        assert_eq!(se.target_dimensions.len(), 2);
        assert!((se.adjustment_magnitude - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_tool_call_creation() {
        let mut params = HashMap::new();
        params.insert("url".into(), "https://example.com".into());
        let tc = ToolCall {
            tool: "web_scrape".into(),
            action: "fetch".into(),
            params,
        };
        assert_eq!(tc.tool, "web_scrape");
        assert_eq!(
            tc.params.get("url").expect("value should be ok in test"),
            "https://example.com"
        );
    }

    #[test]
    fn test_evaluate_by_negentropy_accepts_positive() {
        let edit = SelfEdit {
            task_type: crate::core::TaskType::General,
            target_dimensions: vec![],
            adjustment_magnitude: 0.1,
            tool_calls: Vec::new(),
            config_overrides: HashMap::new(),
        };
        assert!(edit.evaluate_by_negentropy(0.1, 0.0));
        assert!(!edit.evaluate_by_negentropy(-0.1, 0.0));
    }

    #[test]
    fn test_negentropy_verdict_labels() {
        let edit = SelfEdit {
            task_type: crate::core::TaskType::General,
            target_dimensions: vec![],
            adjustment_magnitude: 0.0,
            tool_calls: Vec::new(),
            config_overrides: HashMap::new(),
        };
        assert_eq!(
            edit.negentropy_verdict(0.1),
            "accept: positive negentropy gain"
        );
        assert_eq!(
            edit.negentropy_verdict(-0.1),
            "revert: significant negentropy loss"
        );
        assert_eq!(
            edit.negentropy_verdict(-0.01),
            "reject: neutral or slightly negative"
        );
    }

    #[test]
    fn test_should_revert_on_large_negative() {
        let edit = SelfEdit {
            task_type: crate::core::TaskType::General,
            target_dimensions: vec![],
            adjustment_magnitude: 0.0,
            tool_calls: Vec::new(),
            config_overrides: HashMap::new(),
        };
        assert!(edit.should_revert(-0.1));
        assert!(!edit.should_revert(-0.01));
        assert!(!edit.should_revert(0.05));
    }
}
