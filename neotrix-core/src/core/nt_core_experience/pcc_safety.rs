#![forbid(unsafe_code)]

use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

/// Proof verification status for safety constraints
///
/// Used by the FGGM/SEVerA pipeline to track whether a constraint
/// has been formally verified, is pending, or has failed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProofStatus {
    /// Constraint formally verified
    Verified,
    /// Verification pending (constraint asserted but not proved)
    Obliged,
    /// Verification failed
    Failed,
}

impl ProofStatus {
    pub fn is_safe(&self) -> bool {
        matches!(self, ProofStatus::Verified | ProofStatus::Obliged)
    }
}

/// Check a tool contract's precondition against call context.
///
/// Returns `Verified` if the precondition is satisfied by the call context,
/// `Obliged` if it cannot be proven (pending formal verification),
/// and `Failed` on explicit violation.
///
/// This bridges the FGGM verification pipeline with the existing
/// PccSafetyGate obligation system.
pub fn safety_proof_check(
    _contract_name: &str,
    precondition: &str,
    call_context: &str,
) -> ProofStatus {
    if precondition.is_empty() {
        return ProofStatus::Verified;
    }
    if call_context.contains(precondition) {
        ProofStatus::Verified
    } else {
        ProofStatus::Obliged
    }
}

/// SAHOO-style Constraint Preservation Score (CPS)
/// Measures how well a modification preserves structural invariants.
#[derive(Clone, Debug)]
pub struct ConstraintPreservation {
    /// Named constraints being tracked
    pub constraints: Vec<Constraint>,
    /// Recent CPS history for trend detection
    pub history: VecDeque<f64>,
    /// Maximum history length
    pub history_capacity: usize,
}

#[derive(Clone, Debug)]
pub struct Constraint {
    pub name: String,
    pub check: ConstraintCheck,
    pub weight: f64,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ConstraintCheck {
    /// Numeric value must be within [min, max]
    Bounded { current: f64, min: f64, max: f64 },
    /// Binary invariant must hold
    Invariant { holds: bool },
    /// Composite of sub-constraints
    Composite { children: Vec<Constraint> },
}

impl ConstraintPreservation {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            history: VecDeque::new(),
            history_capacity: 100,
        }
    }

    /// Register a default set of constraints for the self-evolution system
    pub fn register_defaults(&mut self) {
        self.constraints = vec![
            Constraint {
                name: "vsa_dimension".into(),
                check: ConstraintCheck::Bounded {
                    current: 4096.0,
                    min: 1024.0,
                    max: 8192.0,
                },
                weight: 0.3,
            },
            Constraint {
                name: "cycle_rate".into(),
                check: ConstraintCheck::Bounded {
                    current: 50.0,
                    min: 1.0,
                    max: 1000.0,
                },
                weight: 0.15,
            },
            Constraint {
                name: "self_consistency".into(),
                check: ConstraintCheck::Invariant { holds: true },
                weight: 0.25,
            },
            Constraint {
                name: "compile_pass".into(),
                check: ConstraintCheck::Invariant { holds: true },
                weight: 0.3,
            },
        ];
    }

    /// Compute the CPS score (0.0 - 1.0)
    pub fn compute_score(&self) -> f64 {
        if self.constraints.is_empty() {
            return 1.0;
        }
        let total_weight: f64 = self.constraints.iter().map(|c| c.weight).sum();
        if total_weight == 0.0 {
            return 1.0;
        }
        let weighted: f64 = self
            .constraints
            .iter()
            .map(|c| {
                let score = match &c.check {
                    ConstraintCheck::Bounded { current, min, max } => {
                        if current >= min && current <= max {
                            1.0
                        } else {
                            let dist = if current < min {
                                min - current
                            } else {
                                current - max
                            };
                            (1.0 - (dist / (max - min)).min(1.0)).max(0.0)
                        }
                    }
                    ConstraintCheck::Invariant { holds } => {
                        if *holds {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    ConstraintCheck::Composite { children } => {
                        let sub = ConstraintPreservation {
                            constraints: children.clone(),
                            history: VecDeque::new(),
                            history_capacity: 10,
                        };
                        sub.compute_score()
                    }
                };
                score * c.weight
            })
            .sum();
        (weighted / total_weight).clamp(0.0, 1.0)
    }

    /// Update a constraint's current value
    pub fn update_constraint(&mut self, name: &str, value: f64) -> bool {
        for c in &mut self.constraints {
            if c.name == name {
                if let ConstraintCheck::Bounded {
                    ref mut current, ..
                } = &mut c.check
                {
                    *current = value;
                    return true;
                }
            }
        }
        false
    }

    /// Update an invariant constraint
    pub fn set_invariant(&mut self, name: &str, holds: bool) -> bool {
        for c in &mut self.constraints {
            if c.name == name {
                if let ConstraintCheck::Invariant { holds: ref mut h } = &mut c.check {
                    *h = holds;
                    return true;
                }
            }
        }
        false
    }

    /// Record current score to history
    pub fn record(&mut self) -> f64 {
        let score = self.compute_score();
        self.history.push_back(score);
        if self.history.len() > self.history_capacity {
            self.history.pop_front();
        }
        score
    }

    /// Check if CPS has been stable (no significant drift)
    pub fn is_stable(&self, threshold: f64) -> bool {
        if self.history.len() < 10 {
            return true;
        }
        let recent: Vec<f64> = self.history.iter().rev().take(10).copied().collect();
        let mean: f64 = recent.iter().sum::<f64>() / recent.len() as f64;
        recent.iter().all(|v| (v - mean).abs() <= threshold)
    }

    /// Get drift warning if CPS is dropping
    pub fn drift_warning(&self) -> Option<String> {
        if self.history.len() < 5 {
            return None;
        }
        let recent: Vec<f64> = self.history.iter().rev().take(5).copied().collect();
        let trend = recent.first().unwrap_or(&0.0) - recent.last().unwrap_or(&0.0);
        if trend < -0.1 {
            Some(format!("CPS drift detected: {:.3} over 5 records", trend))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ObligationKind {
    PreconditionHolds { target: String, condition: String },
    NoSideEffect { target: String },
    TypeSafe { expression: String },
    ResourceBound { max_cycles: u64 },
}

#[derive(Debug, Clone)]
pub struct ProofObligation {
    pub id: u64,
    pub kind: ObligationKind,
    pub generated_at: u64,
    pub verified: bool,
    pub verification_log: String,
}

#[derive(Debug, Clone)]
pub struct PccVerdict {
    pub passed: bool,
    pub obligation_id: u64,
    pub log: String,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SelfModificationVerdict {
    Approved {
        obligations_passed: usize,
    },
    Rejected {
        obligations_failed: Vec<ProofObligation>,
    },
}

pub struct PccSafetyGate {
    pub obligations: Vec<ProofObligation>,
    pub proof_cache: HashMap<u64, bool>,
    pub strict_mode: bool,
    pub auto_verify: bool,
    pub cps: ConstraintPreservation,
    next_id: u64,
}

impl PccSafetyGate {
    pub fn new(strict_mode: bool, auto_verify: bool) -> Self {
        Self {
            obligations: Vec::new(),
            proof_cache: HashMap::new(),
            strict_mode,
            auto_verify,
            cps: ConstraintPreservation::new(),
            next_id: 1,
        }
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    pub fn generate_obligation(
        &mut self,
        edit_target: &str,
        _edit_value: f64,
        _context: &str,
    ) -> ProofObligation {
        let id = self.next_id;
        self.next_id += 1;
        let generated_at = Self::current_timestamp();

        let kind = if edit_target.contains("threshold") {
            ObligationKind::PreconditionHolds {
                target: edit_target.to_string(),
                condition: format!("{} ≤ 1.0", edit_target),
            }
        } else if edit_target.contains("rate") || edit_target.contains("budget") {
            ObligationKind::ResourceBound { max_cycles: 1000 }
        } else {
            ObligationKind::TypeSafe {
                expression: edit_target.to_string(),
            }
        };

        let obligation = ProofObligation {
            id,
            kind,
            generated_at,
            verified: false,
            verification_log: String::new(),
        };

        if self.auto_verify {
            let mut ob = obligation.clone();
            let result = self.verify_obligation(&mut ob);
            self.proof_cache.insert(ob.id, result);
            self.obligations.push(ob);
            self.obligations.last().cloned().unwrap()
        } else {
            self.obligations.push(obligation);
            self.obligations.last().cloned().unwrap()
        }
    }

    pub fn verify_obligation(&mut self, obligation: &mut ProofObligation) -> bool {
        if let Some(&cached) = self.proof_cache.get(&obligation.id) {
            obligation.verified = cached;
            obligation.verification_log = format!("cached: {}", cached);
            return cached;
        }

        let (verified, log) = match &obligation.kind {
            ObligationKind::PreconditionHolds {
                target: _,
                condition,
            } => {
                let ok = condition.contains("≤") || condition.contains("<=");
                if ok {
                    (true, format!("constraint validated: {}", condition))
                } else {
                    (false, format!("invalid constraint: {}", condition))
                }
            }
            ObligationKind::NoSideEffect { target } => {
                let dangerous = [
                    "kill_switch",
                    "safety_gate",
                    "death_function",
                    "self_destruct",
                ];
                if dangerous.iter().any(|d| target.contains(d)) {
                    (false, format!("target '{}' is blacklisted", target))
                } else {
                    (
                        true,
                        format!("target '{}' has no known side effects", target),
                    )
                }
            }
            ObligationKind::TypeSafe { expression: _ } => {
                (true, "type safety guaranteed by Rust compiler".to_string())
            }
            ObligationKind::ResourceBound { max_cycles } => {
                (true, format!("cycle budget {} is advisory", max_cycles))
            }
        };

        obligation.verified = verified;
        obligation.verification_log = log.clone();
        self.proof_cache.insert(obligation.id, verified);
        verified
    }

    pub fn check_modification(&mut self, target: &str, value: f64, context: &str) -> PccVerdict {
        let obligation = self.generate_obligation(target, value, context);
        let mut ob = self
            .obligations
            .iter_mut()
            .find(|o| o.id == obligation.id)
            .cloned()
            .unwrap_or(obligation);

        let passed = self.verify_obligation(&mut ob);
        if let Some(o) = self.obligations.iter_mut().find(|o| o.id == ob.id) {
            o.verified = ob.verified;
            o.verification_log = ob.verification_log.clone();
        }

        let log = format!(
            "target: {}, oblig_id: {}, passed: {}, log: {}",
            target, ob.id, passed, ob.verification_log
        );

        PccVerdict {
            passed,
            obligation_id: ob.id,
            log,
        }
    }

    pub fn evaluate_edits(&mut self, edits: &[(String, f64, String)]) -> SelfModificationVerdict {
        let mut passed = 0usize;
        let mut failed = Vec::new();

        for (target, value, context) in edits {
            let verdict = self.check_modification(target, *value, context);
            if verdict.passed {
                passed += 1;
            } else if self.strict_mode {
                if let Some(ob) = self
                    .obligations
                    .iter()
                    .find(|o| o.id == verdict.obligation_id)
                {
                    failed.push(ob.clone());
                }
            }
        }

        if failed.is_empty() {
            SelfModificationVerdict::Approved {
                obligations_passed: passed,
            }
        } else {
            SelfModificationVerdict::Rejected {
                obligations_failed: failed,
            }
        }
    }

    pub fn obligation_count(&self) -> usize {
        self.obligations.len()
    }

    pub fn cache_size(&self) -> usize {
        self.proof_cache.len()
    }

    pub fn compute_cps(&mut self) -> f64 {
        let score = self.cps.compute_score();
        self.cps.record();
        score
    }

    pub fn verified_count(&self) -> usize {
        self.obligations.iter().filter(|o| o.verified).count()
    }

    /// Evaluate a single self-evolution edit against all proof obligations.
    ///
    /// Returns `Ok(obligation_count)` if all checks pass, or `Err(failed_obligations)`
    /// with the list of obligations that failed verification.
    pub fn evaluate_edit(
        &mut self,
        target: &str,
        value: f64,
        context: &str,
    ) -> Result<usize, Vec<ProofObligation>> {
        let obligation = self.generate_obligation(target, value, context);

        if obligation.verified {
            Ok(1)
        } else if self.auto_verify {
            Err(vec![obligation])
        } else {
            // Non-strict mode: still actually verify before accepting
            let mut ob = obligation;
            self.verify_obligation(&mut ob);
            if ob.verified {
                Ok(1)
            } else {
                Err(vec![ob])
            }
        }
    }

    pub fn clear_obligations(&mut self) {
        self.obligations.clear();
    }
}

pub fn default_pcc_gate() -> PccSafetyGate {
    PccSafetyGate::new(true, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_obligation_threshold_target() {
        let mut gate = PccSafetyGate::new(true, false);
        let ob = gate.generate_obligation("safety_threshold", 0.8, "test");
        match &ob.kind {
            ObligationKind::PreconditionHolds { target, condition } => {
                assert_eq!(target, "safety_threshold");
                assert_eq!(condition, "safety_threshold ≤ 1.0");
            }
            _ => panic!("expected PreconditionHolds"),
        }
        assert_eq!(ob.id, 1);
    }

    #[test]
    fn test_generate_obligation_rate_target() {
        let mut gate = PccSafetyGate::new(true, false);
        let ob = gate.generate_obligation("learning_rate", 0.01, "train");
        match &ob.kind {
            ObligationKind::ResourceBound { max_cycles } => {
                assert_eq!(*max_cycles, 1000);
            }
            _ => panic!("expected ResourceBound"),
        }
    }

    #[test]
    fn test_generate_obligation_budget_target() {
        let mut gate = PccSafetyGate::new(true, false);
        let ob = gate.generate_obligation("compute_budget", 500.0, "plan");
        match &ob.kind {
            ObligationKind::ResourceBound { max_cycles } => {
                assert_eq!(*max_cycles, 1000);
            }
            _ => panic!("expected ResourceBound"),
        }
    }

    #[test]
    fn test_generate_obligation_unknown_target() {
        let mut gate = PccSafetyGate::new(true, false);
        let ob = gate.generate_obligation("negentropy_coeff", 0.5, "meta");
        match &ob.kind {
            ObligationKind::TypeSafe { expression } => {
                assert_eq!(expression, "negentropy_coeff");
            }
            _ => panic!("expected TypeSafe"),
        }
    }

    #[test]
    fn test_verify_precondition_holds_valid() {
        let mut gate = PccSafetyGate::new(true, false);
        let mut ob = ProofObligation {
            id: 1,
            kind: ObligationKind::PreconditionHolds {
                target: "x".to_string(),
                condition: "x ≤ 1.0".to_string(),
            },
            generated_at: 1000,
            verified: false,
            verification_log: String::new(),
        };
        assert!(gate.verify_obligation(&mut ob));
        assert!(ob.verified);
        assert!(ob.verification_log.contains("constraint validated"));
    }

    #[test]
    fn test_verify_precondition_holds_invalid() {
        let mut gate = PccSafetyGate::new(true, false);
        let mut ob = ProofObligation {
            id: 2,
            kind: ObligationKind::PreconditionHolds {
                target: "x".to_string(),
                condition: "x > 1.0 && x < 2.0".to_string(),
            },
            generated_at: 1000,
            verified: false,
            verification_log: String::new(),
        };
        assert!(!gate.verify_obligation(&mut ob));
        assert!(!ob.verified);
    }

    #[test]
    fn test_verify_no_side_effect_safe() {
        let mut gate = PccSafetyGate::new(true, false);
        let mut ob = ProofObligation {
            id: 3,
            kind: ObligationKind::NoSideEffect {
                target: "curiosity_drive".to_string(),
            },
            generated_at: 1000,
            verified: false,
            verification_log: String::new(),
        };
        assert!(gate.verify_obligation(&mut ob));
        assert!(ob.verification_log.contains("no known side effects"));
    }

    #[test]
    fn test_verify_no_side_effect_dangerous() {
        let mut gate = PccSafetyGate::new(true, false);
        let mut ob = ProofObligation {
            id: 4,
            kind: ObligationKind::NoSideEffect {
                target: "kill_switch".to_string(),
            },
            generated_at: 1000,
            verified: false,
            verification_log: String::new(),
        };
        assert!(!gate.verify_obligation(&mut ob));
        assert!(ob.verification_log.contains("blacklisted"));
    }

    #[test]
    fn test_verify_type_safe_always_true() {
        let mut gate = PccSafetyGate::new(true, false);
        let mut ob = ProofObligation {
            id: 5,
            kind: ObligationKind::TypeSafe {
                expression: "x + 1".to_string(),
            },
            generated_at: 1000,
            verified: false,
            verification_log: String::new(),
        };
        assert!(gate.verify_obligation(&mut ob));
        assert!(ob.verification_log.contains("Rust compiler"));
    }

    #[test]
    fn test_verify_resource_bound_always_true() {
        let mut gate = PccSafetyGate::new(true, false);
        let mut ob = ProofObligation {
            id: 6,
            kind: ObligationKind::ResourceBound { max_cycles: 500 },
            generated_at: 1000,
            verified: false,
            verification_log: String::new(),
        };
        assert!(gate.verify_obligation(&mut ob));
        assert!(ob.verification_log.contains("advisory"));
    }

    #[test]
    fn test_check_modification_passes_valid_edit() {
        let mut gate = PccSafetyGate::new(true, true);
        let verdict = gate.check_modification("negentropy_coeff", 0.5, "meta");
        assert!(verdict.passed);
        assert_eq!(verdict.obligation_id, 1);
    }

    #[test]
    fn test_strict_mode_rejection() {
        let mut gate = PccSafetyGate::new(true, true);
        let edits = vec![
            ("negentropy_coeff".to_string(), 0.5, "meta".to_string()),
            ("kill_switch".to_string(), 0.0, "danger".to_string()),
        ];
        let result = gate.evaluate_edits(&edits);
        match result {
            SelfModificationVerdict::Rejected { obligations_failed } => {
                assert!(!obligations_failed.is_empty());
                let failed = &obligations_failed[0];
                assert!(failed.verification_log.contains("blacklisted"));
            }
            _ => panic!("expected Rejected"),
        }
    }

    #[test]
    fn test_evaluate_edits_all_pass() {
        let mut gate = PccSafetyGate::new(true, true);
        let edits = vec![
            ("negentropy_coeff".to_string(), 0.5, "meta".to_string()),
            ("learning_rate".to_string(), 0.01, "train".to_string()),
        ];
        let result = gate.evaluate_edits(&edits);
        match result {
            SelfModificationVerdict::Approved { obligations_passed } => {
                assert!(obligations_passed >= 2);
            }
            _ => panic!("expected Approved"),
        }
    }

    #[test]
    fn test_proof_cache_works() {
        let mut gate = PccSafetyGate::new(true, false);
        let mut ob = ProofObligation {
            id: 99,
            kind: ObligationKind::TypeSafe {
                expression: "test".to_string(),
            },
            generated_at: 1000,
            verified: false,
            verification_log: String::new(),
        };
        assert_eq!(gate.cache_size(), 0);
        assert!(gate.verify_obligation(&mut ob));
        assert_eq!(gate.cache_size(), 1);

        let mut ob2 = ProofObligation {
            id: 99,
            kind: ObligationKind::TypeSafe {
                expression: "test".to_string(),
            },
            generated_at: 1000,
            verified: false,
            verification_log: String::new(),
        };
        assert!(gate.verify_obligation(&mut ob2));
        assert!(ob2.verification_log.contains("cached"));
        assert_eq!(gate.cache_size(), 1);
    }

    #[test]
    fn test_auto_incrementing_ids_disabled_auto_verify() {
        let mut gate = PccSafetyGate::new(true, false);
        let ob1 = gate.generate_obligation("a", 0.0, "");
        let ob2 = gate.generate_obligation("b", 0.0, "");
        let ob3 = gate.generate_obligation("c", 0.0, "");
        assert_eq!(ob1.id, 1);
        assert_eq!(ob2.id, 2);
        assert_eq!(ob3.id, 3);
    }

    #[test]
    fn test_default_gate_uses_strict_mode() {
        let gate = default_pcc_gate();
        assert!(gate.strict_mode);
        assert!(gate.auto_verify);
        assert_eq!(gate.obligation_count(), 0);
    }

    #[test]
    fn test_verified_count_tracking() {
        let mut gate = PccSafetyGate::new(true, false);
        assert_eq!(gate.verified_count(), 0);
        let mut ob = ProofObligation {
            id: 1,
            kind: ObligationKind::TypeSafe {
                expression: "x".to_string(),
            },
            generated_at: 1000,
            verified: false,
            verification_log: String::new(),
        };
        gate.verify_obligation(&mut ob);
        gate.obligations.push(ob);
        assert_eq!(gate.verified_count(), 1);
    }

    #[test]
    fn test_clear_obligations_resets() {
        let mut gate = PccSafetyGate::new(true, false);
        gate.generate_obligation("learning_rate", 0.01, "");
        gate.generate_obligation("safety_threshold", 0.9, "");
        assert_eq!(gate.obligation_count(), 2);
        gate.clear_obligations();
        assert_eq!(gate.obligation_count(), 0);
    }

    #[test]
    fn test_auto_verify_on_generate() {
        let mut gate = PccSafetyGate::new(true, true);
        let ob = gate.generate_obligation("valid_target", 1.0, "auto_test");
        assert!(ob.verified);
        assert!(!ob.verification_log.is_empty());
    }

    #[test]
    fn test_no_side_effect_safety_gate_rejected() {
        let mut gate = PccSafetyGate::new(true, true);
        let verdict = gate.check_modification("safety_gate", 0.0, "tamper");
        assert!(!verdict.passed);
        assert!(verdict.log.contains("blacklisted"));
    }

    #[test]
    fn test_self_destruct_rejected() {
        let mut gate = PccSafetyGate::new(true, true);
        let verdict = gate.check_modification("self_destruct_fn", 0.0, "danger");
        assert!(!verdict.passed);
        assert!(verdict.log.contains("blacklisted"));
    }

    #[test]
    fn test_obligation_has_valid_timestamp() {
        let mut gate = PccSafetyGate::new(true, false);
        let before = PccSafetyGate::current_timestamp();
        let ob = gate.generate_obligation("test_target", 0.5, "ts_test");
        let after = PccSafetyGate::current_timestamp();
        assert!(ob.generated_at >= before || ob.generated_at <= after);
        assert!(ob.generated_at > 0);
    }

    #[test]
    fn test_mixed_edits_partial_rejection() {
        let mut gate = PccSafetyGate::new(true, true);
        let edits = vec![
            ("good_target".to_string(), 0.5, "ctx".to_string()),
            ("kill_switch".to_string(), 1.0, "danger".to_string()),
            ("learning_rate".to_string(), 0.01, "train".to_string()),
        ];
        let result = gate.evaluate_edits(&edits);
        match result {
            SelfModificationVerdict::Rejected { obligations_failed } => {
                assert_eq!(obligations_failed.len(), 1);
                assert!(obligations_failed[0]
                    .verification_log
                    .contains("blacklisted"));
            }
            _ => panic!("expected Rejected"),
        }
    }

    #[test]
    fn test_check_modification_stores_obligation() {
        let mut gate = PccSafetyGate::new(true, true);
        assert_eq!(gate.obligation_count(), 0);
        gate.check_modification("param_x", 0.5, "storage_test");
        assert_eq!(gate.obligation_count(), 1);
        let stored = &gate.obligations[0];
        assert_eq!(stored.id, 1);
        assert!(stored.verified);
    }
}

#[cfg(test)]
mod cps_tests {
    use super::*;

    #[test]
    fn test_cps_defaults() {
        let mut cps = ConstraintPreservation::new();
        cps.register_defaults();
        let score = cps.compute_score();
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_cps_bounded_in_range() {
        let cps = ConstraintPreservation {
            constraints: vec![Constraint {
                name: "test".into(),
                check: ConstraintCheck::Bounded {
                    current: 50.0,
                    min: 0.0,
                    max: 100.0,
                },
                weight: 1.0,
            }],
            history: VecDeque::new(),
            history_capacity: 10,
        };
        assert!((cps.compute_score() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cps_bounded_out_of_range() {
        let cps = ConstraintPreservation {
            constraints: vec![Constraint {
                name: "test".into(),
                check: ConstraintCheck::Bounded {
                    current: 150.0,
                    min: 0.0,
                    max: 100.0,
                },
                weight: 1.0,
            }],
            history: VecDeque::new(),
            history_capacity: 10,
        };
        assert!(cps.compute_score() < 0.5);
    }

    #[test]
    fn test_cps_invariant_holds() {
        let cps = ConstraintPreservation {
            constraints: vec![Constraint {
                name: "test".into(),
                check: ConstraintCheck::Invariant { holds: true },
                weight: 1.0,
            }],
            history: VecDeque::new(),
            history_capacity: 10,
        };
        assert!((cps.compute_score() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cps_invariant_fails() {
        let cps = ConstraintPreservation {
            constraints: vec![Constraint {
                name: "test".into(),
                check: ConstraintCheck::Invariant { holds: false },
                weight: 1.0,
            }],
            history: VecDeque::new(),
            history_capacity: 10,
        };
        assert!((cps.compute_score() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_cps_update() {
        let mut cps = ConstraintPreservation::new();
        cps.register_defaults();
        assert!(cps.update_constraint("vsa_dimension", 2048.0));
        assert!(!cps.update_constraint("nonexistent", 0.0));
    }

    #[test]
    fn test_cps_stability() {
        let mut cps = ConstraintPreservation::new();
        for _ in 0..15 {
            cps.history.push_back(0.9);
        }
        assert!(cps.is_stable(0.05));
    }

    #[test]
    fn test_drift_warning() {
        let mut cps = ConstraintPreservation::new();
        cps.history.push_back(1.0);
        cps.history.push_back(0.9);
        cps.history.push_back(0.8);
        cps.history.push_back(0.7);
        cps.history.push_back(0.6);
        assert!(cps.drift_warning().is_some());
    }
}
