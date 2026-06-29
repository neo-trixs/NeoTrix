use std::collections::{HashMap, VecDeque};

/// Which lever of evolution is being pulled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LeverType {
    /// Harness: scaffold, code structure, prompts, pipelines.
    Harness,
    /// Weight: numeric parameters, thresholds, calibration curves.
    Weight,
}

/// A single lever with its current state.
#[derive(Debug, Clone)]
pub struct LeverState {
    pub lever_type: LeverType,
    pub domain: String,
    pub current_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub last_modified_cycle: u64,
    pub modification_count: u64,
    pub success_rate: f64,
    pub history: VecDeque<(u64, f64, f64)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalOrigin {
    Reflection,
    Calibration,
    Evolution,
    Exploration,
}

/// A proposal to modify a specific lever.
#[derive(Debug, Clone)]
pub struct LeverProposal {
    pub id: u64,
    pub lever_type: LeverType,
    pub target_domain: String,
    pub current_value: f64,
    pub proposed_value: f64,
    pub expected_gain: f64,
    pub confidence: f64,
    pub reasoning: String,
    pub origin: ProposalOrigin,
}

/// A record of a completed lever modification.
#[derive(Debug, Clone)]
pub struct LeverModification {
    pub cycle: u64,
    pub lever_type: LeverType,
    pub domain: String,
    pub value_before: f64,
    pub value_after: f64,
    pub actual_gain: f64,
    pub predicted_gain: f64,
    pub success: bool,
}

/// Aggregate statistics for the dual-lever orchestrator.
#[derive(Debug, Clone)]
pub struct DualLeverStats {
    pub total_harness_levers: usize,
    pub total_weight_levers: usize,
    pub total_modifications: usize,
    pub overall_success_rate: f64,
    pub harness_success_rate: f64,
    pub weight_success_rate: f64,
    pub current_weight_lr: f64,
    pub stalled_domains: Vec<String>,
}

/// SIA-style Dual-Lever Orchestrator.
pub struct DualLeverOrchestrator {
    harness_levers: HashMap<String, LeverState>,
    weight_levers: HashMap<String, LeverState>,
    modification_history: VecDeque<LeverModification>,
    max_history: usize,
    next_proposal_id: u64,
    pub weight_lr: f64,
    pub success_threshold: f64,
}

impl DualLeverOrchestrator {
    /// Creates an empty orchestrator.
    pub fn new(max_history: usize) -> Self {
        Self {
            harness_levers: HashMap::new(),
            weight_levers: HashMap::new(),
            modification_history: VecDeque::new(),
            max_history,
            next_proposal_id: 1,
            weight_lr: 0.1,
            success_threshold: 0.0,
        }
    }

    /// Register a harness lever for a given domain.
    pub fn register_harness_lever(&mut self, domain: &str, initial_value: f64) {
        self.harness_levers.insert(
            domain.to_string(),
            LeverState {
                lever_type: LeverType::Harness,
                domain: domain.to_string(),
                current_value: initial_value,
                min_value: 0.0,
                max_value: f64::MAX,
                last_modified_cycle: 0,
                modification_count: 0,
                success_rate: 0.5,
                history: VecDeque::new(),
            },
        );
    }

    /// Register a weight lever with explicit bounds.
    pub fn register_weight_lever(&mut self, domain: &str, initial_value: f64, min: f64, max: f64) {
        let clamped = initial_value.clamp(min, max);
        self.weight_levers.insert(
            domain.to_string(),
            LeverState {
                lever_type: LeverType::Weight,
                domain: domain.to_string(),
                current_value: clamped,
                min_value: min,
                max_value: max,
                last_modified_cycle: 0,
                modification_count: 0,
                success_rate: 0.5,
                history: VecDeque::new(),
            },
        );
    }

    /// Propose a harness lever change.
    pub fn propose_harness_change(
        &mut self,
        domain: &str,
        proposed_value: f64,
        reasoning: &str,
        confidence: f64,
        origin: ProposalOrigin,
    ) -> Option<LeverProposal> {
        let state = self.harness_levers.get(domain)?;
        let id = self.next_proposal_id;
        self.next_proposal_id += 1;
        Some(LeverProposal {
            id,
            lever_type: LeverType::Harness,
            target_domain: domain.to_string(),
            current_value: state.current_value,
            proposed_value,
            expected_gain: proposed_value - state.current_value,
            confidence,
            reasoning: reasoning.to_string(),
            origin,
        })
    }

    /// Propose a weight lever change. The proposed value is clamped to [min, max].
    pub fn propose_weight_change(
        &mut self,
        domain: &str,
        proposed_value: f64,
        reasoning: &str,
        confidence: f64,
        origin: ProposalOrigin,
    ) -> Option<LeverProposal> {
        let state = self.weight_levers.get(domain)?;
        let clamped = proposed_value.clamp(state.min_value, state.max_value);
        let id = self.next_proposal_id;
        self.next_proposal_id += 1;
        Some(LeverProposal {
            id,
            lever_type: LeverType::Weight,
            target_domain: domain.to_string(),
            current_value: state.current_value,
            proposed_value: clamped,
            expected_gain: clamped - state.current_value,
            confidence,
            reasoning: reasoning.to_string(),
            origin,
        })
    }

    /// Apply a proposal and record the modification.
    pub fn apply_modification(
        &mut self,
        proposal: &LeverProposal,
        actual_gain: f64,
    ) -> LeverModification {
        let success = actual_gain > self.success_threshold;
        let value_before = proposal.current_value;
        let value_after = proposal.proposed_value;
        let cycle = self.modification_history.len() as u64;

        let modification = LeverModification {
            cycle,
            lever_type: proposal.lever_type,
            domain: proposal.target_domain.clone(),
            value_before,
            value_after,
            actual_gain,
            predicted_gain: proposal.expected_gain,
            success,
        };

        let state = match proposal.lever_type {
            LeverType::Harness => self.harness_levers.get_mut(&proposal.target_domain),
            LeverType::Weight => self.weight_levers.get_mut(&proposal.target_domain),
        };

        if let Some(state) = state {
            state.current_value = value_after;
            state.last_modified_cycle = cycle;
            state.modification_count += 1;
            state.history.push_back((cycle, value_before, value_after));
            if state.history.len() > 20 {
                state.history.pop_front();
            }
            // Recompute per-lever success rate from last 20 modifications.
            let recent_mods: Vec<_> = self
                .modification_history
                .iter()
                .rev()
                .filter(|m| m.domain == proposal.target_domain)
                .take(20)
                .collect();
            if !recent_mods.is_empty() {
                let successes = recent_mods.iter().filter(|m| m.success).count();
                state.success_rate = successes as f64 / recent_mods.len() as f64;
            }
        }

        self.modification_history.push_back(modification.clone());
        while self.modification_history.len() > self.max_history {
            self.modification_history.pop_front();
        }

        modification
    }

    /// Get the current state of a lever by domain.
    pub fn lever_state(&self, domain: &str) -> Option<&LeverState> {
        self.harness_levers
            .get(domain)
            .or_else(|| self.weight_levers.get(domain))
    }

    pub fn harness_count(&self) -> usize {
        self.harness_levers.len()
    }

    pub fn weight_count(&self) -> usize {
        self.weight_levers.len()
    }

    pub fn modification_count(&self) -> usize {
        self.modification_history.len()
    }

    /// Returns the last N modifications.
    pub fn recent_modifications(&self, n: usize) -> Vec<&LeverModification> {
        self.modification_history.iter().rev().take(n).collect()
    }

    fn compute_success_rate_for_lever_type(&self, lever_type: Option<LeverType>) -> f64 {
        let iter: Box<dyn Iterator<Item = &LeverModification>> = match lever_type {
            Some(LeverType::Harness) => Box::new(
                self.modification_history
                    .iter()
                    .filter(|m| m.lever_type == LeverType::Harness),
            ),
            Some(LeverType::Weight) => Box::new(
                self.modification_history
                    .iter()
                    .filter(|m| m.lever_type == LeverType::Weight),
            ),
            None => Box::new(self.modification_history.iter()),
        };
        let collected: Vec<_> = iter.collect();
        if collected.is_empty() {
            return 0.0;
        }
        let successes = collected.iter().filter(|m| m.success).count();
        successes as f64 / collected.len() as f64
    }

    /// Overall success rate across all modifications.
    pub fn success_rate(&self) -> f64 {
        self.compute_success_rate_for_lever_type(None)
    }

    /// Success rate for harness modifications only.
    pub fn harness_success_rate(&self) -> f64 {
        self.compute_success_rate_for_lever_type(Some(LeverType::Harness))
    }

    /// Success rate for weight modifications only.
    pub fn weight_success_rate(&self) -> f64 {
        self.compute_success_rate_for_lever_type(Some(LeverType::Weight))
    }

    /// Auto-adjust the weight learning rate based on recent weight success rate.
    pub fn auto_adjust_weight_learning_rate(&mut self) {
        let recent_weight: Vec<_> = self
            .modification_history
            .iter()
            .rev()
            .filter(|m| m.lever_type == LeverType::Weight)
            .take(20)
            .collect();
        if recent_weight.len() < 5 {
            return;
        }
        let successes = recent_weight.iter().filter(|m| m.success).count();
        let rate = successes as f64 / recent_weight.len() as f64;
        if rate > 0.7 {
            self.weight_lr = (self.weight_lr * 1.2).min(0.5);
        } else if rate < 0.3 {
            self.weight_lr = (self.weight_lr * 0.8).max(0.01);
        }
    }

    /// Domains that have not been modified for more than `stall_threshold` modifications.
    pub fn stalled_domains(&self, stall_threshold: u64) -> Vec<String> {
        let current_cycle = self.modification_history.len() as u64;
        let mut stalled = Vec::new();
        for (domain, state) in self.harness_levers.iter() {
            if current_cycle.saturating_sub(state.last_modified_cycle) > stall_threshold {
                stalled.push(domain.clone());
            }
        }
        for (domain, state) in self.weight_levers.iter() {
            if current_cycle.saturating_sub(state.last_modified_cycle) > stall_threshold {
                stalled.push(domain.clone());
            }
        }
        stalled
    }

    /// Pending proposals — those where the lever's current value still matches the proposal's
    /// current_value (i.e., the proposal hasn't been applied yet).
    pub fn active_proposals(&self, _lever_type: Option<LeverType>) -> Vec<&LeverProposal> {
        // This orchestrator does not store a pending-proposal queue; proposals are consumed
        // immediately by apply_modification. This method is a stub for future use.
        Vec::new()
    }

    /// Aggregate statistics.
    pub fn stats(&self) -> DualLeverStats {
        DualLeverStats {
            total_harness_levers: self.harness_count(),
            total_weight_levers: self.weight_count(),
            total_modifications: self.modification_count(),
            overall_success_rate: self.success_rate(),
            harness_success_rate: self.harness_success_rate(),
            weight_success_rate: self.weight_success_rate(),
            current_weight_lr: self.weight_lr,
            stalled_domains: self.stalled_domains(20),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_orchestrator_empty() {
        let o = DualLeverOrchestrator::new(100);
        assert_eq!(o.harness_count(), 0);
        assert_eq!(o.weight_count(), 0);
        assert_eq!(o.modification_count(), 0);
        assert_eq!(o.success_rate(), 0.0);
        assert_eq!(o.weight_lr, 0.1);
    }

    #[test]
    fn test_register_harness_lever() {
        let mut o = DualLeverOrchestrator::new(100);
        o.register_harness_lever("prompt_templates", 1.0);
        assert_eq!(o.harness_count(), 1);
        let state = o.lever_state("prompt_templates").unwrap();
        assert_eq!(state.lever_type, LeverType::Harness);
        assert_eq!(state.current_value, 1.0);
    }

    #[test]
    fn test_register_weight_lever_with_bounds() {
        let mut o = DualLeverOrchestrator::new(100);
        o.register_weight_lever("ece_threshold", 0.15, 0.01, 0.5);
        assert_eq!(o.weight_count(), 1);
        let state = o.lever_state("ece_threshold").unwrap();
        assert_eq!(state.lever_type, LeverType::Weight);
        assert_eq!(state.current_value, 0.15);
        assert_eq!(state.min_value, 0.01);
        assert_eq!(state.max_value, 0.5);
    }

    #[test]
    fn test_propose_harness_change() {
        let mut o = DualLeverOrchestrator::new(100);
        o.register_harness_lever("prompt_templates", 1.0);
        let proposal = o.propose_harness_change(
            "prompt_templates",
            2.0,
            "Add structured output",
            0.8,
            ProposalOrigin::Reflection,
        );
        assert!(proposal.is_some());
        let p = proposal.unwrap();
        assert_eq!(p.lever_type, LeverType::Harness);
        assert_eq!(p.current_value, 1.0);
        assert_eq!(p.proposed_value, 2.0);
        assert_eq!(p.expected_gain, 1.0);
    }

    #[test]
    fn test_propose_weight_change_clamps_to_bounds() {
        let mut o = DualLeverOrchestrator::new(100);
        o.register_weight_lever("ece_threshold", 0.15, 0.01, 0.5);
        // Propose a value above max — should clamp.
        let proposal = o.propose_weight_change(
            "ece_threshold",
            10.0,
            "Too aggressive",
            0.5,
            ProposalOrigin::Calibration,
        );
        assert!(proposal.is_some());
        let p = proposal.unwrap();
        assert_eq!(p.proposed_value, 0.5);
        assert_eq!(p.proposed_value, p.current_value + p.expected_gain);

        // Propose a value below min — should clamp.
        let proposal2 = o.propose_weight_change(
            "ece_threshold",
            -1.0,
            "Too low",
            0.5,
            ProposalOrigin::Calibration,
        );
        assert!(proposal2.is_some());
        let p2 = proposal2.unwrap();
        assert_eq!(p2.proposed_value, 0.01);
    }

    #[test]
    fn test_apply_modification_records_history() {
        let mut o = DualLeverOrchestrator::new(100);
        o.register_harness_lever("prompt_templates", 1.0);
        let proposal = o
            .propose_harness_change(
                "prompt_templates",
                2.0,
                "Improve",
                0.7,
                ProposalOrigin::Exploration,
            )
            .unwrap();
        let modification = o.apply_modification(&proposal, 0.5);
        assert!(modification.success);
        assert_eq!(modification.value_before, 1.0);
        assert_eq!(modification.value_after, 2.0);
        assert_eq!(modification.actual_gain, 0.5);
        assert_eq!(o.modification_count(), 1);
        let state = o.lever_state("prompt_templates").unwrap();
        assert_eq!(state.current_value, 2.0);
        assert_eq!(state.modification_count, 1);
    }

    #[test]
    fn test_success_rate_calculation() {
        let mut o = DualLeverOrchestrator::new(100);
        o.register_harness_lever("prompt_templates", 1.0);
        o.register_weight_lever("lr", 0.1, 0.01, 0.5);

        let p1 = o
            .propose_harness_change(
                "prompt_templates",
                2.0,
                "a",
                0.5,
                ProposalOrigin::Exploration,
            )
            .unwrap();
        o.apply_modification(&p1, 0.3);

        let p2 = o
            .propose_weight_change("lr", 0.2, "b", 0.5, ProposalOrigin::Evolution)
            .unwrap();
        o.apply_modification(&p2, -0.1); // failure

        let p3 = o
            .propose_harness_change(
                "prompt_templates",
                3.0,
                "c",
                0.5,
                ProposalOrigin::Reflection,
            )
            .unwrap();
        o.apply_modification(&p3, 0.1);

        // Overall: 2 successes out of 3
        assert!((o.success_rate() - 2.0 / 3.0).abs() < 1e-9);
        // Harness: 2 out of 2
        assert!((o.harness_success_rate() - 1.0).abs() < 1e-9);
        // Weight: 0 out of 1
        assert!((o.weight_success_rate() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_auto_adjust_learning_rate_high_success() {
        let mut o = DualLeverOrchestrator::new(100);
        o.register_weight_lever("lr", 0.1, 0.01, 0.5);
        o.weight_lr = 0.1;

        // Create 15 successful weight modifications.
        for i in 1..=15 {
            let prop = o
                .propose_weight_change(
                    "lr",
                    0.1 + i as f64 * 0.01,
                    "tune",
                    0.5,
                    ProposalOrigin::Calibration,
                )
                .unwrap();
            o.apply_modification(&prop, 0.05); // success
        }

        let prev_lr = o.weight_lr;
        o.auto_adjust_weight_learning_rate();
        assert!(
            o.weight_lr > prev_lr,
            "lr should increase after high success rate"
        );
    }

    #[test]
    fn test_stalled_domains_detection() {
        let mut o = DualLeverOrchestrator::new(100);
        o.register_harness_lever("active_lever", 1.0);
        o.register_weight_lever("stalled_lever", 0.5, 0.0, 1.0);

        // Apply 5 modifications to "active_lever" only.
        for i in 1..=5 {
            let prop = o
                .propose_harness_change(
                    "active_lever",
                    i as f64,
                    "tuning",
                    0.5,
                    ProposalOrigin::Exploration,
                )
                .unwrap();
            o.apply_modification(&prop, 0.1);
        }

        // With 5 total modifications, stalled_lever's last_modified_cycle is 0.
        // stall_threshold = 3 means anything untouched for > 3 mods is stalled.
        let stalled = o.stalled_domains(3);
        assert!(stalled.contains(&"stalled_lever".to_string()));
        assert!(!stalled.contains(&"active_lever".to_string()));
    }

    #[test]
    fn test_stats_after_modifications() {
        let mut o = DualLeverOrchestrator::new(100);
        o.register_harness_lever("prompt", 1.0);
        o.register_weight_lever("threshold", 0.3, 0.0, 1.0);

        let p1 = o
            .propose_harness_change("prompt", 2.0, "refine", 0.6, ProposalOrigin::Reflection)
            .unwrap();
        o.apply_modification(&p1, 0.4);

        let p2 = o
            .propose_weight_change("threshold", 0.5, "adjust", 0.6, ProposalOrigin::Calibration)
            .unwrap();
        o.apply_modification(&p2, -0.2);

        let stats = o.stats();
        assert_eq!(stats.total_harness_levers, 1);
        assert_eq!(stats.total_weight_levers, 1);
        assert_eq!(stats.total_modifications, 2);
        assert!((stats.overall_success_rate - 0.5).abs() < 1e-9);
        assert!((stats.harness_success_rate - 1.0).abs() < 1e-9);
        assert!((stats.weight_success_rate - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_recent_modifications() {
        let mut o = DualLeverOrchestrator::new(100);
        o.register_harness_lever("x", 0.0);
        for i in 0..5 {
            let prop = o
                .propose_harness_change("x", i as f64, "", 0.5, ProposalOrigin::Exploration)
                .unwrap();
            o.apply_modification(&prop, 0.1);
        }
        let recent = o.recent_modifications(3);
        assert_eq!(recent.len(), 3);
        // Most recent last
        assert_eq!(recent[0].value_after, 4.0);
        assert_eq!(recent[2].value_after, 2.0);
    }

    #[test]
    fn test_propose_unknown_domain_returns_none() {
        let mut o = DualLeverOrchestrator::new(100);
        let prop =
            o.propose_harness_change("nonexistent", 1.0, "", 0.5, ProposalOrigin::Exploration);
        assert!(prop.is_none());
        let prop =
            o.propose_weight_change("nonexistent", 1.0, "", 0.5, ProposalOrigin::Exploration);
        assert!(prop.is_none());
    }

    #[test]
    fn test_register_weight_clamps_initial_value() {
        let mut o = DualLeverOrchestrator::new(100);
        o.register_weight_lever("test", 100.0, 0.0, 1.0);
        let state = o.lever_state("test").unwrap();
        assert!((state.current_value - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_auto_adjust_learning_rate_low_success() {
        let mut o = DualLeverOrchestrator::new(100);
        o.register_weight_lever("lr", 0.1, 0.01, 0.5);
        o.weight_lr = 0.1;

        for i in 1..=15 {
            let prop = o
                .propose_weight_change(
                    "lr",
                    0.1 + i as f64 * 0.01,
                    "tune",
                    0.5,
                    ProposalOrigin::Calibration,
                )
                .unwrap();
            o.apply_modification(&prop, -0.05); // failure
        }

        let prev_lr = o.weight_lr;
        o.auto_adjust_weight_learning_rate();
        assert!(
            o.weight_lr < prev_lr,
            "lr should decrease after low success rate"
        );
    }

    #[test]
    fn test_lever_history_bounded() {
        let mut o = DualLeverOrchestrator::new(100);
        o.register_harness_lever("x", 0.0);
        for i in 0..30 {
            let prop = o
                .propose_harness_change("x", i as f64, "", 0.5, ProposalOrigin::Exploration)
                .unwrap();
            o.apply_modification(&prop, 0.1);
        }
        let state = o.lever_state("x").unwrap();
        // history should be bounded to 20 entries
        assert_eq!(state.history.len(), 20);
    }
}
