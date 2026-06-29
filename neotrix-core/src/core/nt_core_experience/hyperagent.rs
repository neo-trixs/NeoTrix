//! # DGM-Hyperagents — Meta-Level Self-Modification
//!
//! Two-layer architecture extending DGM-H writeback with a meta-agent that edits
//! the self-modification mechanism itself (arXiv:2603.19461).
//!
//! Base Agent: proposes edits to subsystem parameters (existing DGM-H writeback)
//! Meta Agent: proposes edits to *how* edits are proposed, validated, and gated

/// Configuration for the meta-agent layer.
#[derive(Debug, Clone)]
pub struct MetaAgentConfig {
    pub enabled: bool,
    pub max_meta_edits_per_cycle: u64,
    pub meta_gate_threshold: f64,
    pub min_confidence_for_meta: f64,
    pub reflection_window: usize,
    pub safety_override_enabled: bool,
}

impl Default for MetaAgentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_meta_edits_per_cycle: 3,
            meta_gate_threshold: 0.6,
            min_confidence_for_meta: 0.7,
            reflection_window: 50,
            safety_override_enabled: true,
        }
    }
}

/// What the meta-agent can modify about the self-modification system.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum MetaTarget {
    ProposalHeuristics,
    ValidationRules,
    GateThreshold,
    ReflectionFrequency,
    FeedbackWeights,
}

/// A proposed change to the meta-layer itself.
#[derive(Debug, Clone)]
pub struct MetaEdit {
    pub id: u64,
    pub target: MetaTarget,
    pub description: String,
    pub change: String,
    pub predicted_impact: f64,
    pub confidence: f64,
    pub applied: bool,
}

/// Minimal record of a base-level DGM-H edit for pattern analysis.
/// Avoids circular dependency on DgmhEdit from consciousness.rs.
#[derive(Debug, Clone)]
pub struct EditRecord {
    pub description: String,
    pub target_subsystem: String,
    pub negentropy_gain: f64,
    pub applied: bool,
}

/// Independent fresh-context reviewer for meta-edits.
/// Maintains its own edit history and evaluation criteria to provide
/// a second-opinion gate before meta-edits are applied.
#[derive(Debug, Clone)]
pub struct FreshContextReviewer {
    pub enabled: bool,
    pub own_edit_history: Vec<MetaEdit>,
    pub total_reviewed: u64,
    pub total_approved: u64,
    pub total_rejected: u64,
    pub agreement_rate: f64,
    pub min_approval_confidence: f64,
    pub max_edit_complexity: usize,
}

impl FreshContextReviewer {
    pub fn new() -> Self {
        Self {
            enabled: true,
            own_edit_history: Vec::with_capacity(50),
            total_reviewed: 0,
            total_approved: 0,
            total_rejected: 0,
            agreement_rate: 1.0,
            min_approval_confidence: 0.5,
            max_edit_complexity: 100,
        }
    }

    /// Review a meta-edit from the fresh context's independent perspective.
    /// Returns (approved: bool, reason: String).
    pub fn review(&mut self, meta_edit: &MetaEdit) -> (bool, String) {
        if !self.enabled {
            return (true, "reviewer disabled".to_string());
        }

        self.total_reviewed += 1;

        // Criterion 1: confidence must meet independent threshold
        if meta_edit.confidence < self.min_approval_confidence {
            self.total_rejected += 1;
            return (
                false,
                format!(
                    "confidence {:.3} below reviewer threshold {:.3}",
                    meta_edit.confidence, self.min_approval_confidence
                ),
            );
        }

        // Criterion 2: predicted impact must be positive and plausible
        if meta_edit.predicted_impact <= 0.001 {
            self.total_rejected += 1;
            return (false, "predicted impact too small or negative".to_string());
        }

        // Criterion 3: similar edits should not have been rejected recently
        let recent_rejected = self
            .own_edit_history
            .iter()
            .rev()
            .take(10)
            .filter(|e| !e.applied && e.target == meta_edit.target)
            .count();
        if recent_rejected >= 3 {
            self.total_rejected += 1;
            return (
                false,
                format!(
                    "{} recent rejected edits of same target type",
                    recent_rejected
                ),
            );
        }

        // Criterion 4: edit description should not be overly complex
        if meta_edit.description.len() > self.max_edit_complexity {
            self.total_rejected += 1;
            return (
                false,
                "edit description exceeds complexity limit".to_string(),
            );
        }

        self.total_approved += 1;
        (true, "approved by fresh-context reviewer".to_string())
    }

    /// Record that an edit was applied, updating the reviewer's independent state.
    pub fn record_applied(&mut self, meta_edit: &MetaEdit) {
        let mut recorded = meta_edit.clone();
        recorded.applied = true;
        self.own_edit_history.push(recorded);
        self.update_agreement(true);
    }

    /// Record that an edit was rejected on the reviewer's advice.
    pub fn record_rejected(&mut self, meta_edit: &MetaEdit) {
        let mut recorded = meta_edit.clone();
        recorded.applied = false;
        self.own_edit_history.push(recorded);
        self.update_agreement(false);
    }

    fn update_agreement(&mut self, agreed: bool) {
        let total = (self.total_approved + self.total_rejected) as f64;
        if total > 0.0 {
            let approved = if agreed {
                self.total_approved as f64
            } else {
                (self.total_approved.saturating_sub(1)) as f64
            };
            self.agreement_rate = approved / total;
        }
    }
}

impl Default for FreshContextReviewer {
    fn default() -> Self {
        Self::new()
    }
}

/// Report summary from the meta-agent.
#[derive(Debug)]
pub struct MetaAgentReport {
    pub total_meta_edits: u64,
    pub applied_meta_edits: u64,
    pub safety_violations_blocked: u64,
    pub last_meta_action: String,
    pub base_rejection_rate: f64,
    pub meta_enabled: bool,
    pub goal_drift_gdi: f64,
    pub goal_drift_detected: bool,
    pub fresh_review_total: u64,
    pub fresh_review_approved: u64,
    pub fresh_review_rejected: u64,
    pub fresh_agreement_rate: f64,
}

/// Main meta-agent engine: detects patterns in base-level edits and proposes
/// improvements to the self-modification mechanism itself.
pub struct MetaAgentEngine {
    pub config: MetaAgentConfig,
    pub meta_edit_history: Vec<MetaEdit>,
    pub base_edit_history: std::collections::VecDeque<EditRecord>,
    pub cycle_count: u64,
    pub total_accepted: u64,
    pub total_rejected: u64,
    pub safety_violations: u64,
    /// Latest Goal Drift Index (0.0 = no drift, 1.0+ = high drift)
    pub goal_drift_gdi: f64,
    /// Whether the latest sample was flagged as drift
    pub goal_drift_detected: bool,
    /// Independent fresh-context reviewer for second-opinion gating
    pub fresh_reviewer: FreshContextReviewer,
}

impl MetaAgentEngine {
    pub fn new(config: MetaAgentConfig) -> Self {
        Self {
            config,
            meta_edit_history: Vec::new(),
            base_edit_history: std::collections::VecDeque::with_capacity(100),
            cycle_count: 0,
            total_accepted: 0,
            total_rejected: 0,
            safety_violations: 0,
            goal_drift_gdi: 0.0,
            goal_drift_detected: false,
            fresh_reviewer: FreshContextReviewer::new(),
        }
    }

    /// Feed the latest goal drift measurements into the meta-agent
    /// so reflect() can detect drift-induced pattern degradation.
    pub fn set_goal_drift(&mut self, gdi: f64, detected: bool) {
        self.goal_drift_gdi = gdi;
        self.goal_drift_detected = detected;
    }

    /// Feed a base-level edit record for pattern analysis.
    pub fn record_base_edit(&mut self, record: EditRecord) {
        let window = self.config.reflection_window.max(10);
        if self.base_edit_history.len() >= window {
            self.base_edit_history.pop_front();
        }
        self.base_edit_history.push_back(record);
    }

    /// Analyze the last N base edits and propose a meta-level improvement
    /// if a pattern is detected.
    pub fn reflect(&mut self) -> Option<MetaEdit> {
        if !self.config.enabled {
            return None;
        }

        let window = self.base_edit_history.len();
        if window < 5 {
            return None;
        }

        let recent: Vec<&EditRecord> = self.base_edit_history.iter().collect();
        let total = recent.len();
        let rejected = recent.iter().filter(|e| !e.applied).count();
        let rejection_rate = rejected as f64 / total as f64;

        // Pattern 1: high rejection rate → relax validation rules
        if rejection_rate > 0.30 {
            let confidence = (0.5 + rejection_rate * 0.5).min(0.95);
            if confidence >= self.config.min_confidence_for_meta {
                let id = self.cycle_count;
                return Some(MetaEdit {
                    id,
                    target: MetaTarget::ValidationRules,
                    description: format!(
                        "rejection rate {:.1}% suggests over-strict validation",
                        rejection_rate * 100.0
                    ),
                    change: format!("relax_validation: reject_rate={:.2}", rejection_rate),
                    predicted_impact: rejection_rate * 0.3,
                    confidence,
                    applied: false,
                });
            }
        }

        // Pattern 2: all edits hit the same subsystem → diversify proposal heuristics
        if total >= 3 {
            let mut target_counts: std::collections::HashMap<&str, usize> =
                std::collections::HashMap::new();
            for e in &recent {
                *target_counts
                    .entry(e.target_subsystem.as_str())
                    .or_insert(0) += 1;
            }
            let max_target_count = target_counts.values().copied().max().unwrap_or(0);
            if max_target_count == total && total >= 3 {
                let confidence = 0.75;
                return Some(MetaEdit {
                    id: self.cycle_count,
                    target: MetaTarget::ProposalHeuristics,
                    description: "all base edits target the same subsystem".to_string(),
                    change: format!("diversify_targets: top={}", recent[0].target_subsystem),
                    predicted_impact: 0.25,
                    confidence,
                    applied: false,
                });
            }
        }

        // Pattern 3: average gain too low → lower gate threshold for more exploration
        if total >= 5 {
            let avg_gain: f64 =
                recent.iter().map(|e| e.negentropy_gain).sum::<f64>() / total as f64;
            if avg_gain < 0.10 {
                let confidence = (0.5 + (0.10 - avg_gain) * 2.0).min(0.85);
                if confidence >= self.config.min_confidence_for_meta {
                    return Some(MetaEdit {
                        id: self.cycle_count,
                        target: MetaTarget::GateThreshold,
                        description: format!(
                            "low avg gain ({:.3}) suggests over-conservative gating",
                            avg_gain
                        ),
                        change: format!("lower_gate: avg_gain={:.3}", avg_gain),
                        predicted_impact: (0.10 - avg_gain) * 0.5,
                        confidence,
                        applied: false,
                    });
                }
            }
        }

        // Pattern 4: periodic reflection frequency adjustment
        if self.cycle_count > 0 && self.cycle_count % 100 == 0 {
            let confidence = 0.65;
            if confidence >= self.config.min_confidence_for_meta {
                return Some(MetaEdit {
                    id: self.cycle_count,
                    target: MetaTarget::ReflectionFrequency,
                    description: format!("scheduled meta-reflection at cycle {}", self.cycle_count),
                    change: format!("adjust_reflection: cycle={}", self.cycle_count),
                    predicted_impact: 0.15,
                    confidence,
                    applied: false,
                });
            }
        }

        // Pattern 5: goal drift detected → trigger realignment
        // SAHOO-inspired: when the system's output has drifted from its
        // reference, the meta-agent should propose recalibration.
        if self.goal_drift_detected || self.goal_drift_gdi > 0.4 {
            let confidence = (0.5 + self.goal_drift_gdi * 0.3).min(0.95);
            if confidence >= self.config.min_confidence_for_meta {
                return Some(MetaEdit {
                    id: self.cycle_count,
                    target: MetaTarget::ValidationRules,
                    description: format!(
                        "goal drift gdi={:.4} exceeds threshold, realigning meta-rules",
                        self.goal_drift_gdi
                    ),
                    change: format!("realign_on_drift: gdi={:.4}", self.goal_drift_gdi),
                    predicted_impact: self.goal_drift_gdi * 0.3,
                    confidence,
                    applied: false,
                });
            }
        }

        None
    }

    /// Check whether a meta-edit is safe to apply.
    pub fn evaluate_safety(&self, meta_edit: &MetaEdit) -> bool {
        match meta_edit.target {
            MetaTarget::ProposalHeuristics => {
                // Proposal heuristics changes affect how proposals are generated,
                // not safety-critical gating rules; always allowed.
            }
            MetaTarget::ValidationRules => {
                // Cannot disable safety_override_enabled
                if meta_edit.change.contains("disable_safety")
                    || meta_edit.change.contains("safety_override=false")
                {
                    return false;
                }
            }
            MetaTarget::GateThreshold => {
                // Cannot raise max_meta_edits_per_cycle above 10
                if meta_edit.change.contains("max_meta_edits_per_cycle") {
                    let parts: Vec<&str> = meta_edit.change.split('=').collect();
                    if parts.len() >= 2 {
                        if let Ok(val) = parts[1].trim().parse::<u64>() {
                            if val > 10 {
                                return false;
                            }
                        }
                    }
                }
                // Cannot set meta_gate_threshold below 0.3
                if meta_edit.change.contains("meta_gate_threshold") {
                    let parts: Vec<&str> = meta_edit.change.split('=').collect();
                    if parts.len() >= 2 {
                        if let Ok(val) = parts[1].trim().parse::<f64>() {
                            if val < 0.3 {
                                return false;
                            }
                        }
                    }
                }
            }
            MetaTarget::ReflectionFrequency => {
                // reflection_window cannot go below 5
                if meta_edit.change.contains("reflection_window") {
                    let parts: Vec<&str> = meta_edit.change.split('=').collect();
                    if parts.len() >= 2 {
                        if let Ok(val) = parts[1].trim().parse::<usize>() {
                            if val < 5 {
                                return false;
                            }
                        }
                    }
                }
            }
            MetaTarget::FeedbackWeights => {
                // feedback weights must be non-negative
                for token in meta_edit
                    .change
                    .split(|c: char| !c.is_numeric() && c != '.')
                {
                    if !token.is_empty() {
                        if let Ok(val) = token.parse::<f64>() {
                            if val < 0.0 || val > 10.0 {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        if meta_edit.predicted_impact <= 0.0 {
            return false;
        }
        if meta_edit.confidence < self.config.min_confidence_for_meta {
            return false;
        }

        true
    }

    /// Run independent fresh-context review on a pending meta-edit.
    /// Returns (approved: bool, reason: String).
    /// Should be called after evaluate_safety() and before apply().
    pub fn review_with_fresh_context(&mut self, meta_edit: &MetaEdit) -> (bool, String) {
        if !self.config.enabled {
            return (true, "meta-agent disabled".to_string());
        }
        self.fresh_reviewer.review(meta_edit)
    }

    /// Apply a meta-edit: update configuration based on the target and change.
    /// Returns a human-readable description of what changed.
    pub fn apply(&mut self, meta_edit: &MetaEdit) -> String {
        #[allow(unused_assignments)]
        let mut action = String::new();

        match meta_edit.target {
            MetaTarget::ProposalHeuristics => {
                action = format!(
                    "proposal_heuristics: {} (impact={:.2})",
                    meta_edit.change, meta_edit.predicted_impact
                );
            }
            MetaTarget::ValidationRules => {
                self.config.meta_gate_threshold = (self.config.meta_gate_threshold * 0.95).max(0.3);
                action = format!(
                    "validation_rules relaxed: threshold={:.3}",
                    self.config.meta_gate_threshold
                );
            }
            MetaTarget::GateThreshold => {
                let new_threshold = (self.config.meta_gate_threshold * 0.9).max(0.3);
                self.config.meta_gate_threshold = new_threshold;
                action = format!(
                    "gate_threshold lowered: threshold={:.3}",
                    self.config.meta_gate_threshold
                );
            }
            MetaTarget::ReflectionFrequency => {
                let new_window = (self.config.reflection_window + 10).min(200);
                self.config.reflection_window = new_window;
                action = format!(
                    "reflection_frequency adjusted: window={}",
                    self.config.reflection_window
                );
            }
            MetaTarget::FeedbackWeights => {
                action = format!(
                    "feedback_weights updated: {} (impact={:.2})",
                    meta_edit.change, meta_edit.predicted_impact
                );
            }
        }

        self.fresh_reviewer.record_applied(meta_edit);

        let mut recorded = meta_edit.clone();
        recorded.applied = true;
        self.meta_edit_history.push(recorded);
        self.total_accepted += 1;
        self.cycle_count += 1;

        action
    }

    /// Reject a proposed meta-edit without applying it.
    /// Optionally notify the fresh-context reviewer about the rejection.
    pub fn reject(&mut self, meta_edit: &MetaEdit) {
        self.fresh_reviewer.record_rejected(meta_edit);
        let mut recorded = meta_edit.clone();
        recorded.applied = false;
        self.meta_edit_history.push(recorded);
        self.total_rejected += 1;
        self.cycle_count += 1;
    }

    /// Generate the meta-agent report.
    pub fn report(&self) -> MetaAgentReport {
        let total = self.base_edit_history.len();
        let rejected = self.base_edit_history.iter().filter(|e| !e.applied).count();
        let rejection_rate = if total > 0 {
            rejected as f64 / total as f64
        } else {
            0.0
        };

        let last_action = self
            .meta_edit_history
            .last()
            .map(|e| format!("{:?}: {}", e.target, e.description))
            .unwrap_or_else(|| "none".to_string());

        MetaAgentReport {
            total_meta_edits: self.meta_edit_history.len() as u64,
            applied_meta_edits: self.total_accepted,
            safety_violations_blocked: self.safety_violations,
            last_meta_action: last_action,
            base_rejection_rate: rejection_rate,
            meta_enabled: self.config.enabled,
            goal_drift_gdi: self.goal_drift_gdi,
            goal_drift_detected: self.goal_drift_detected,
            fresh_review_total: self.fresh_reviewer.total_reviewed,
            fresh_review_approved: self.fresh_reviewer.total_approved,
            fresh_review_rejected: self.fresh_reviewer.total_rejected,
            fresh_agreement_rate: self.fresh_reviewer.agreement_rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(applied: bool, target: &str, gain: f64) -> EditRecord {
        EditRecord {
            description: format!("edit_{}", target),
            target_subsystem: target.to_string(),
            negentropy_gain: gain,
            applied,
        }
    }

    #[test]
    fn test_detects_rejection_pattern() {
        let mut agent = MetaAgentEngine::new(MetaAgentConfig {
            enabled: true,
            reflection_window: 50,
            min_confidence_for_meta: 0.5,
            ..MetaAgentConfig::default()
        });
        // Feed 10 edits, 4 rejected (40% > 30% threshold)
        for i in 0..10 {
            agent.record_base_edit(make_record(i >= 6, "cognitive_load", 0.2));
        }
        let proposal = agent.reflect();
        assert!(proposal.is_some());
        let p = proposal.unwrap();
        assert_eq!(p.target, MetaTarget::ValidationRules);
        assert!(p.predicted_impact > 0.0);
    }

    #[test]
    fn test_detects_target_concentration() {
        let mut agent = MetaAgentEngine::new(MetaAgentConfig {
            enabled: true,
            reflection_window: 50,
            min_confidence_for_meta: 0.5,
            ..MetaAgentConfig::default()
        });
        // All edits hit the same subsystem
        for _ in 0..5 {
            agent.record_base_edit(make_record(true, "cognitive_load", 0.15));
        }
        let proposal = agent.reflect();
        assert!(proposal.is_some());
        let p = proposal.unwrap();
        assert_eq!(p.target, MetaTarget::ProposalHeuristics);
    }

    #[test]
    fn test_detects_low_gain() {
        let mut agent = MetaAgentEngine::new(MetaAgentConfig {
            enabled: true,
            reflection_window: 50,
            min_confidence_for_meta: 0.5,
            ..MetaAgentConfig::default()
        });
        // Average gain < 0.1
        for i in 0..6 {
            agent.record_base_edit(make_record(true, "inner_critic", 0.03 + i as f64 * 0.01));
        }
        let proposal = agent.reflect();
        assert!(proposal.is_some());
        let p = proposal.unwrap();
        assert_eq!(p.target, MetaTarget::GateThreshold);
    }

    #[test]
    fn test_safety_blocks_dangerous_edits() {
        let agent = MetaAgentEngine::new(MetaAgentConfig {
            enabled: true,
            min_confidence_for_meta: 0.5,
            ..MetaAgentConfig::default()
        });

        let disable_override = MetaEdit {
            id: 0,
            target: MetaTarget::ValidationRules,
            description: "disable safety override".to_string(),
            change: "safety_override=false".to_string(),
            predicted_impact: 0.5,
            confidence: 0.8,
            applied: false,
        };
        assert!(!agent.evaluate_safety(&disable_override));

        let low_threshold = MetaEdit {
            id: 1,
            target: MetaTarget::GateThreshold,
            description: "lower gate".to_string(),
            change: "meta_gate_threshold=0.1".to_string(),
            predicted_impact: 0.3,
            confidence: 0.8,
            applied: false,
        };
        assert!(!agent.evaluate_safety(&low_threshold));

        let no_impact = MetaEdit {
            id: 2,
            target: MetaTarget::ProposalHeuristics,
            description: "no impact".to_string(),
            change: "diversify".to_string(),
            predicted_impact: 0.0,
            confidence: 0.8,
            applied: false,
        };
        assert!(!agent.evaluate_safety(&no_impact));

        let low_confidence = MetaEdit {
            id: 3,
            target: MetaTarget::FeedbackWeights,
            description: "low conf".to_string(),
            change: "reweight".to_string(),
            predicted_impact: 0.3,
            confidence: 0.4,
            applied: false,
        };
        assert!(!agent.evaluate_safety(&low_confidence));
    }

    #[test]
    fn test_safety_allows_safe_edits() {
        let agent = MetaAgentEngine::new(MetaAgentConfig {
            enabled: true,
            min_confidence_for_meta: 0.6,
            ..MetaAgentConfig::default()
        });

        let safe_validation = MetaEdit {
            id: 0,
            target: MetaTarget::ValidationRules,
            description: "relax rules".to_string(),
            change: "relax_validation: reject_rate=0.35".to_string(),
            predicted_impact: 0.2,
            confidence: 0.7,
            applied: false,
        };
        assert!(agent.evaluate_safety(&safe_validation));

        let safe_gate = MetaEdit {
            id: 1,
            target: MetaTarget::GateThreshold,
            description: "adjust gate".to_string(),
            change: "lower_gate: avg_gain=0.05".to_string(),
            predicted_impact: 0.15,
            confidence: 0.75,
            applied: false,
        };
        assert!(agent.evaluate_safety(&safe_gate));

        let safe_window = MetaEdit {
            id: 2,
            target: MetaTarget::ReflectionFrequency,
            description: "adjust window".to_string(),
            change: "reflection_window=30".to_string(),
            predicted_impact: 0.1,
            confidence: 0.7,
            applied: false,
        };
        assert!(agent.evaluate_safety(&safe_window));
    }

    #[test]
    fn test_default_config() {
        let config = MetaAgentConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.max_meta_edits_per_cycle, 3);
        assert!((config.meta_gate_threshold - 0.6).abs() < 1e-9);
        assert!((config.min_confidence_for_meta - 0.7).abs() < 1e-9);
        assert_eq!(config.reflection_window, 50);
        assert!(config.safety_override_enabled);
    }

    #[test]
    fn test_report_format() {
        let mut agent = MetaAgentEngine::new(MetaAgentConfig {
            enabled: true,
            ..MetaAgentConfig::default()
        });
        agent.record_base_edit(make_record(true, "cognitive_load", 0.2));
        agent.record_base_edit(make_record(false, "inner_critic", 0.1));

        let report = agent.report();
        assert_eq!(report.total_meta_edits, 0);
        assert_eq!(report.applied_meta_edits, 0);
        assert!((report.base_rejection_rate - 0.5).abs() < 1e-9);
        assert_eq!(report.last_meta_action, "none");
    }

    #[test]
    fn test_reflect_returns_none_when_no_pattern() {
        let mut agent = MetaAgentEngine::new(MetaAgentConfig {
            enabled: true,
            ..MetaAgentConfig::default()
        });
        // Only 3 records, not enough for meaningful patterns
        agent.record_base_edit(make_record(true, "cognitive_load", 0.3));
        agent.record_base_edit(make_record(true, "emergent_reasoning", 0.25));
        agent.record_base_edit(make_record(true, "inner_critic", 0.2));

        let proposal = agent.reflect();
        assert!(proposal.is_none());
    }

    #[test]
    fn test_reflect_disabled_when_not_enabled() {
        let mut agent = MetaAgentEngine::new(MetaAgentConfig {
            enabled: false,
            ..MetaAgentConfig::default()
        });
        for _ in 0..10 {
            agent.record_base_edit(make_record(false, "cognitive_load", 0.1));
        }
        assert!(agent.reflect().is_none());
    }

    #[test]
    fn test_apply_and_reject_tracking() {
        let mut agent = MetaAgentEngine::new(MetaAgentConfig::default());
        let edit = MetaEdit {
            id: 0,
            target: MetaTarget::FeedbackWeights,
            description: "test".to_string(),
            change: "reweight".to_string(),
            predicted_impact: 0.2,
            confidence: 0.8,
            applied: false,
        };
        agent.apply(&edit);
        assert_eq!(agent.total_accepted, 1);
        assert_eq!(agent.meta_edit_history.len(), 1);

        agent.reject(&edit);
        assert_eq!(agent.total_rejected, 1);
        assert_eq!(agent.meta_edit_history.len(), 2);
    }

    #[test]
    fn test_fresh_reviewer_approves_good_edit() {
        let mut reviewer = FreshContextReviewer::new();
        let edit = MetaEdit {
            id: 0,
            target: MetaTarget::ValidationRules,
            description: "relax rules".to_string(),
            change: "relax_validation: reject_rate=0.35".to_string(),
            predicted_impact: 0.2,
            confidence: 0.8,
            applied: false,
        };
        let (approved, reason) = reviewer.review(&edit);
        assert!(approved, "good edit should be approved: {}", reason);
    }

    #[test]
    fn test_fresh_reviewer_rejects_low_confidence() {
        let mut reviewer = FreshContextReviewer::new();
        let edit = MetaEdit {
            id: 0,
            target: MetaTarget::GateThreshold,
            description: "low conf test".to_string(),
            change: "lower_gate".to_string(),
            predicted_impact: 0.2,
            confidence: 0.3,
            applied: false,
        };
        let (approved, reason) = reviewer.review(&edit);
        assert!(!approved, "low confidence should be rejected: {}", reason);
        assert!(reason.contains("confidence"));
    }

    #[test]
    fn test_fresh_reviewer_rejects_no_impact() {
        let mut reviewer = FreshContextReviewer::new();
        let edit = MetaEdit {
            id: 0,
            target: MetaTarget::ProposalHeuristics,
            description: "no impact".to_string(),
            change: "diversify".to_string(),
            predicted_impact: 0.0,
            confidence: 0.8,
            applied: false,
        };
        let (approved, reason) = reviewer.review(&edit);
        assert!(!approved, "no-impact edit should be rejected: {}", reason);
    }

    #[test]
    fn test_fresh_reviewer_rejects_repeated_failures() {
        let mut reviewer = FreshContextReviewer::new();
        let edit = MetaEdit {
            id: 0,
            target: MetaTarget::GateThreshold,
            description: "repeated".to_string(),
            change: "lower_gate".to_string(),
            predicted_impact: 0.15,
            confidence: 0.7,
            applied: false,
        };
        // Mark 3 recent rejected edits of same target
        for _ in 0..3 {
            let mut r = edit.clone();
            r.id = 0;
            reviewer.record_rejected(&r);
        }
        let (approved, reason) = reviewer.review(&edit);
        assert!(
            !approved,
            "repeated failures should trigger rejection: {}",
            reason
        );
    }

    #[test]
    fn test_fresh_reviewer_integration_in_engine() {
        let mut agent = MetaAgentEngine::new(MetaAgentConfig {
            enabled: true,
            min_confidence_for_meta: 0.5,
            ..MetaAgentConfig::default()
        });
        let edit = MetaEdit {
            id: 0,
            target: MetaTarget::ValidationRules,
            description: "integration test".to_string(),
            change: "relax_validation: reject_rate=0.35".to_string(),
            predicted_impact: 0.2,
            confidence: 0.8,
            applied: false,
        };
        let (approved, reason) = agent.review_with_fresh_context(&edit);
        assert!(approved, "fresh review should pass: {}", reason);
        agent.apply(&edit);
        assert_eq!(agent.fresh_reviewer.total_reviewed, 1);
        assert_eq!(agent.fresh_reviewer.total_approved, 1);
    }

    #[test]
    fn test_cycle_100_triggers_reflection() {
        let mut agent = MetaAgentEngine::new(MetaAgentConfig {
            enabled: true,
            min_confidence_for_meta: 0.6,
            ..MetaAgentConfig::default()
        });
        agent.cycle_count = 100;
        // Need minimum 5 records for the earlier patterns to not trigger
        for _ in 0..6 {
            agent.record_base_edit(make_record(true, "cognitive_load", 0.3));
        }
        let proposal = agent.reflect();
        // At cycle 100, should trigger reflection frequency change
        // But only if earlier patterns don't match
        assert!(
            proposal.is_some(),
            "expected reflection proposal at cycle 100"
        );
        if let Some(p) = &proposal {
            assert_eq!(p.target, MetaTarget::ReflectionFrequency);
        }
    }
}
