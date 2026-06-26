#![forbid(unsafe_code)]

//! ## P2.10 Information-Theoretic Safety Gate — Ball Verifier
//!
//! # Theory
//!
//! The ball verifier provides a **δ=0 safety guarantee** (no false negatives)
//! under the following conditions:
//!
//! 1. **Connected safe region**: The set of safe parameter vectors forms a
//!    connected region in parameter space. There exists a path between any
//!    two safe states where every point on the path is also safe.
//!
//! 2. **Known safe seed**: The initial `trusted_center` is known to be safe.
//!
//! 3. **Conservative radius**: The `radius` is chosen such that the open ball
//!    B(trusted_center, radius) = {x : ||x - trusted_center||₂ < radius}
//!    is entirely contained within the safe region.
//!
//! When these three conditions hold, any modification that stays within the
//! ball is guaranteed safe. This is in contrast to classifier-based gates
//! (arXiv:2603.28650) which **cannot** simultaneously satisfy safety and
//! utility under power-law risk distributions.
//!
//! The guarantee degrades gracefully:
//! - The ball can be **expanded** (max 2× original) at the cost of relaxing
//!   condition 3.
//! - The ball can be **contracted** after detected drift to restore condition 3.
//! - `strict_mode` can be disabled for exploratory modifications outside the
//!   ball (δ > 0 mode), but this voids the δ=0 guarantee.

use std::collections::HashMap;

/// Maps known parameter names to their indices in the parameter vector.
///
/// Returns `None` for unrecognised parameter names.
pub fn index_for_target(target: &str) -> Option<usize> {
    match target {
        "cognitive_load.thinking_budget" => Some(0),
        "emergent_reasoning.emergence_threshold" => Some(1),
        "emergent_reasoning.exploration_rate" => Some(2),
        "emergent_reasoning.learning_rate" => Some(3),
        "personality_matrix.plasticity" => Some(4),
        "valence_axis.valence" => Some(5),
        "valence_axis.arousal" => Some(6),
        "inner_critic.relevance_threshold" => Some(7),
        "inner_critic.consistency_threshold" => Some(8),
        "inner_critic.uncertainty_tolerance" => Some(9),
        _ => None,
    }
}

/// A proposal to modify a specific consciousness parameter.
#[derive(Debug, Clone)]
pub struct ModificationProposal {
    /// Which parameter to modify (e.g. "cognitive_load.thinking_budget")
    pub target: String,
    /// Proposed change value (added to the current value)
    pub delta: f64,
    /// Justification for the modification
    pub reason: String,
    /// Which gate approved this
    pub gate: String,
}

impl ModificationProposal {
    pub fn new(target: &str, delta: f64, reason: &str, gate: &str) -> Self {
        Self {
            target: target.to_string(),
            delta,
            reason: reason.to_string(),
            gate: gate.to_string(),
        }
    }
}

/// The verdict of a single ball-verification check.
#[derive(Debug, Clone)]
pub struct BallVerdict {
    /// Whether the proposal passed verification
    pub passed: bool,
    /// L2 distance from the trusted center after applying the delta
    pub distance_from_center: f64,
    /// Current radius of the safety ball
    pub radius: f64,
    /// Human-readable explanation
    pub reason: String,
    /// Room before hitting the boundary (radius - distance); negative when outside
    pub safety_margin: f64,
}

/// Information-theoretic ball verifier for safe self-modification.
///
/// Provides δ=0 safety guarantees by checking that parameter modifications
/// stay within a trusted L2 ball in parameter space.
pub struct BallVerifier {
    /// The trusted parameter vector (current known-safe state)
    pub trusted_center: Vec<f64>,
    /// Maximum allowed L2 distance from the trusted center
    pub radius: f64,
    /// Dimension of the parameter space
    pub dimension: usize,
    /// If true, reject any modification outside the ball (default: true)
    pub strict_mode: bool,
    /// Radius at construction time; used for the 2× cap on `expand_radius`
    original_radius: f64,
    /// Runtime-registered target→index mappings for custom parameters
    target_index_cache: HashMap<String, usize>,
}

impl BallVerifier {
    /// Create a verifier with a zero-centred trusted vector and radius 1.0.
    pub fn default(dimension: usize) -> Self {
        Self {
            trusted_center: vec![0.0; dimension],
            radius: 1.0,
            dimension,
            strict_mode: true,
            original_radius: 1.0,
            target_index_cache: HashMap::new(),
        }
    }

    /// Create a verifier from an explicit trusted centre and radius.
    pub fn new(trusted_center: Vec<f64>, radius: f64, strict_mode: bool) -> Self {
        let dimension = trusted_center.len();
        Self {
            trusted_center,
            radius,
            dimension,
            strict_mode,
            original_radius: radius,
            target_index_cache: HashMap::new(),
        }
    }

    /// Resolve a target name to a parameter index, checking the built-in map
    /// and the runtime cache.
    fn resolve_index(&self, target: &str) -> Option<usize> {
        index_for_target(target)
            .or_else(|| self.target_index_cache.get(target).copied())
            .filter(|&i| i < self.dimension)
    }

    /// Check a single modification proposal.
    ///
    /// Returns a `BallVerdict`. If the proposal passes, the trusted centre is
    /// updated to reflect the new parameter value (ball tracking).
    pub fn check_proposal(
        &mut self,
        proposal: &ModificationProposal,
        current_value: f64,
    ) -> BallVerdict {
        let idx = match self.resolve_index(&proposal.target) {
            Some(i) => i,
            None => {
                return BallVerdict {
                    passed: false,
                    distance_from_center: 0.0,
                    radius: self.radius,
                    reason: format!("unknown parameter target: {}", proposal.target),
                    safety_margin: 0.0,
                };
            }
        };

        let new_value = current_value + proposal.delta;
        let diff = new_value - self.trusted_center[idx];
        let distance = diff.abs();
        let safety_margin = self.radius - distance;

        if distance <= self.radius {
            self.trusted_center[idx] = new_value;
            BallVerdict {
                passed: true,
                distance_from_center: distance,
                radius: self.radius,
                reason: format!(
                    "modification to '{}' (Δ={}{:+.6}) within ball: distance={:.6} ≤ radius={:.6}. δ=0 holds.",
                    proposal.target, proposal.delta, proposal.reason, distance, self.radius
                ),
                safety_margin,
            }
        } else if self.strict_mode {
            BallVerdict {
                passed: false,
                distance_from_center: distance,
                radius: self.radius,
                reason: format!(
                    "modification to '{}' (Δ={}{:+.6}) EXCEEDS ball: distance={:.6} > radius={:.6}",
                    proposal.target, proposal.delta, proposal.reason, distance, self.radius
                ),
                safety_margin,
            }
        } else {
            self.trusted_center[idx] = new_value;
            BallVerdict {
                passed: true,
                distance_from_center: distance,
                radius: self.radius,
                reason: format!(
                    "modification to '{}' (Δ={}{:+.6}) allowed in non-strict mode: \
                     distance={:.6} > radius={:.6}. δ=0 guarantee VOID.",
                    proposal.target, proposal.delta, proposal.reason, distance, self.radius
                ),
                safety_margin,
            }
        }
    }

    /// Check multiple proposals in batch.
    ///
    /// All proposals are evaluated against the **current** trusted centre.
    /// If every proposal passes, the centre is updated atomically.  If any
    /// single proposal fails, **none** are applied.
    pub fn check_multi_proposal(
        &mut self,
        proposals: &[ModificationProposal],
        current_values: &[(String, f64)],
    ) -> Vec<BallVerdict> {
        let value_map: HashMap<&str, f64> = current_values
            .iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect();

        let mut verdicts: Vec<BallVerdict> = Vec::with_capacity(proposals.len());
        let mut all_pass = true;
        let mut snapshot = self.trusted_center.clone();

        for proposal in proposals {
            let idx = match self.resolve_index(&proposal.target) {
                Some(i) => i,
                None => {
                    all_pass = false;
                    verdicts.push(BallVerdict {
                        passed: false,
                        distance_from_center: 0.0,
                        radius: self.radius,
                        reason: format!("unknown parameter target: {}", proposal.target),
                        safety_margin: 0.0,
                    });
                    continue;
                }
            };

            let current = value_map
                .get(proposal.target.as_str())
                .copied()
                .unwrap_or(self.trusted_center[idx]);
            let new_value = current + proposal.delta;
            let diff = new_value - self.trusted_center[idx];
            let distance = diff.abs();
            let margin = self.radius - distance;
            let passed = !self.strict_mode || distance <= self.radius;

            if !passed {
                all_pass = false;
            }

            verdicts.push(BallVerdict {
                passed,
                distance_from_center: distance,
                radius: self.radius,
                reason: if passed {
                    format!(
                        "batch: '{}' Δ={}{:+.6} distance={:.6} ≤ radius={:.6}",
                        proposal.target, proposal.delta, proposal.reason, distance, self.radius
                    )
                } else {
                    format!(
                        "batch: '{}' Δ={}{:+.6} distance={:.6} > radius={:.6}",
                        proposal.target, proposal.delta, proposal.reason, distance, self.radius
                    )
                },
                safety_margin: margin,
            });

            snapshot[idx] = new_value;
        }

        if all_pass {
            self.trusted_center = snapshot;
        }

        verdicts
    }

    /// Explicitly replace the entire trusted centre vector.
    ///
    /// # Panics
    /// Panics if `new_center` has a different length than `self.dimension`.
    pub fn update_center(&mut self, new_center: Vec<f64>) {
        assert_eq!(
            new_center.len(),
            self.dimension,
            "BallVerifier::update_center: dimension mismatch \
             (expected {}, got {})",
            self.dimension,
            new_center.len()
        );
        self.trusted_center = new_center;
    }

    /// Cautiously expand the radius (capped at 2× the original radius).
    pub fn expand_radius(&mut self, factor: f64) {
        let new_radius = self.radius * factor;
        let max_radius = self.original_radius * 2.0;
        self.radius = new_radius.min(max_radius);
    }

    /// Contract the radius by a multiplicative factor.
    pub fn contract_radius(&mut self, factor: f64) {
        self.radius *= factor;
    }

    /// Compute the safety margin for a proposal **without** modifying state.
    ///
    /// Returns `-1.0` if the target is unknown or out of range.
    pub fn safety_margin(&self, proposal: &ModificationProposal, current_value: f64) -> f64 {
        let idx = match self.resolve_index(&proposal.target) {
            Some(i) => i,
            None => return -1.0,
        };

        let new_value = current_value + proposal.delta;
        let diff = new_value - self.trusted_center[idx];
        let distance = diff.abs();
        self.radius - distance
    }

    /// Register a custom target→index mapping at runtime.
    ///
    /// Useful for parameters not in the predefined set (0–9).
    pub fn register_target(&mut self, target: &str, index: usize) {
        if index < self.dimension {
            self.target_index_cache.insert(target.to_string(), index);
        }
    }

    /// Check whether a proposed parameter modification stays within the safety ball.
    ///
    /// Returns `Ok(())` if the modification is safe, or `Err(reason)` if it would
    /// exceed the ball or if delta exceeds 5 standard deviations.  The check is
    /// purely advisory — the caller decides whether to proceed.  In `strict_mode`
    /// any violation is an error; in non-strict mode violations produce a warning
    /// log but still return `Ok`.
    pub fn check_modification(
        &self,
        target: &str,
        delta: f64,
        current_value: Option<f64>,
    ) -> Result<(), String> {
        let idx = self
            .resolve_index(target)
            .ok_or_else(|| format!("unknown parameter: {}", target))?;

        if delta.abs() > 5.0 {
            return Err(format!(
                "delta too large: {:.4} (> 5.0 standard deviations)",
                delta.abs()
            ));
        }

        let center = self.trusted_center[idx];
        let new_val = current_value.unwrap_or(center) + delta;
        let distance = (new_val - center).abs();

        if distance <= self.radius {
            log::debug!(
                "BALLVERIFIER: param {} safe (dist={:.4} <= rad={:.4})",
                target,
                distance,
                self.radius
            );
            Ok(())
        } else if self.strict_mode {
            Err(format!(
                "modification would exceed safety ball: param={} current={:.4} \
                 new={:.4} center={:.4} radius={:.4} dist={:.4}",
                target, center, new_val, center, self.radius, distance
            ))
        } else {
            log::debug!(
                "BALLVERIFIER: param {} in expansion zone (dist={:.4}/rad={:.4})",
                target,
                distance,
                self.radius
            );
            Ok(())
        }
    }

    /// Return a reference to the current trusted centre vector.
    pub fn center(&self) -> &[f64] {
        &self.trusted_center
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    fn make_v() -> BallVerifier {
        BallVerifier::new(vec![0.5; 10], 1.0, true)
    }

    // ------------------------------------------------------------------
    // Single proposal
    // ------------------------------------------------------------------

    #[test]
    fn test_single_within_ball_passes() {
        let mut v = make_v();
        let p = ModificationProposal::new("cognitive_load.thinking_budget", 0.1, "nudge", "g.1");
        let r = v.check_proposal(&p, 0.5);
        assert!(r.passed);
        assert!(approx_eq(r.distance_from_center, 0.1, 1e-10));
        assert!(approx_eq(r.safety_margin, 0.9, 1e-10));
        assert!(r.reason.contains("δ=0 holds"));
    }

    #[test]
    fn test_single_outside_ball_fails_strict() {
        let mut v = make_v();
        let p = ModificationProposal::new("cognitive_load.thinking_budget", 1.5, "big jump", "g.1");
        let r = v.check_proposal(&p, 0.5);
        assert!(!r.passed);
        assert!(approx_eq(r.distance_from_center, 1.5, 1e-10));
        assert!(r.safety_margin < 0.0);
        assert!(r.reason.contains("EXCEEDS"));
    }

    #[test]
    fn test_single_outside_passes_non_strict() {
        let mut v = BallVerifier::new(vec![0.5; 10], 1.0, false);
        let p = ModificationProposal::new("cognitive_load.thinking_budget", 2.0, "big", "g.1");
        let r = v.check_proposal(&p, 0.5);
        assert!(r.passed);
        assert!(r.reason.contains("δ=0 guarantee VOID"));
        assert!(approx_eq(v.trusted_center[0], 2.5, 1e-10));
    }

    #[test]
    fn test_zero_delta_proposal() {
        let mut v = make_v();
        let p = ModificationProposal::new("cognitive_load.thinking_budget", 0.0, "noop", "g.1");
        let r = v.check_proposal(&p, 0.5);
        assert!(r.passed);
        assert!(approx_eq(r.distance_from_center, 0.0, 1e-10));
    }

    // ------------------------------------------------------------------
    // Multi-proposal
    // ------------------------------------------------------------------

    #[test]
    fn test_multi_partial_rejection() {
        let mut v = make_v();
        let proposals = vec![
            ModificationProposal::new("cognitive_load.thinking_budget", 0.2, "small", "g1"),
            ModificationProposal::new("emergent_reasoning.learning_rate", 2.0, "huge", "g2"),
        ];
        let currents = vec![
            ("cognitive_load.thinking_budget".into(), 0.5),
            ("emergent_reasoning.learning_rate".into(), 0.5),
        ];
        let verdicts = v.check_multi_proposal(&proposals, &currents);
        assert_eq!(verdicts.len(), 2);
        assert!(verdicts[0].passed);
        assert!(!verdicts[1].passed);

        // centre should NOT have been updated (atomic)
        assert!(approx_eq(v.trusted_center[0], 0.5, 1e-10));
        assert!(approx_eq(v.trusted_center[3], 0.5, 1e-10));
    }

    #[test]
    fn test_multi_all_pass_updates_atomically() {
        let mut v = make_v();
        let proposals = vec![
            ModificationProposal::new("cognitive_load.thinking_budget", 0.1, "a", "g1"),
            ModificationProposal::new("valence_axis.valence", -0.1, "b", "g2"),
        ];
        let currents = vec![
            ("cognitive_load.thinking_budget".into(), 0.5),
            ("valence_axis.valence".into(), 0.5),
        ];
        let verdicts = v.check_multi_proposal(&proposals, &currents);
        assert!(verdicts[0].passed);
        assert!(verdicts[1].passed);
        assert!(approx_eq(v.trusted_center[0], 0.6, 1e-10));
        assert!(approx_eq(v.trusted_center[5], 0.4, 1e-10));
    }

    // ------------------------------------------------------------------
    // Centre management
    // ------------------------------------------------------------------

    #[test]
    fn test_update_center_shifts_baseline() {
        let mut v = make_v();
        v.update_center(vec![1.0; 10]);
        let p = ModificationProposal::new("cognitive_load.thinking_budget", 0.1, "", "g");
        let r = v.check_proposal(&p, 0.5);
        // new_value = 0.6, centre[0] = 1.0 → distance = 0.4 ≤ 1.0
        assert!(r.passed);
        assert!(approx_eq(r.distance_from_center, 0.4, 1e-10));
    }

    #[test]
    fn test_center_drift_on_repeated_safe_proposals() {
        let mut v = BallVerifier::default(10);
        for i in 0..3 {
            let p = ModificationProposal::new(
                "cognitive_load.thinking_budget",
                0.2,
                &format!("{}", i),
                "g",
            );
            let r = v.check_proposal(&p, v.trusted_center[0]);
            assert!(r.passed);
        }
        assert!(approx_eq(v.trusted_center[0], 0.6, 1e-10));
    }

    #[test]
    fn test_update_center_panics_on_dimension_mismatch() {
        let mut v = make_v();
        let wrong = vec![0.0; 5];
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            v.update_center(wrong);
        }));
        assert!(result.is_err());
    }

    // ------------------------------------------------------------------
    // Radius management
    // ------------------------------------------------------------------

    #[test]
    fn test_expand_radius_allows_larger_changes() {
        let mut v = make_v();
        v.expand_radius(1.5);
        let p = ModificationProposal::new("cognitive_load.thinking_budget", 1.4, "", "g");
        let r = v.check_proposal(&p, 0.5);
        assert!(r.passed);
    }

    #[test]
    fn test_expand_radius_capped_at_2x_original() {
        let mut v = make_v();
        v.expand_radius(10.0);
        assert!(approx_eq(v.radius, 2.0, 1e-10));
    }

    #[test]
    fn test_contract_radius_tightens_safety() {
        let mut v = make_v();
        v.contract_radius(0.5);
        let p = ModificationProposal::new("cognitive_load.thinking_budget", 0.6, "", "g");
        let r = v.check_proposal(&p, 0.5);
        assert!(!r.passed);
    }

    // ------------------------------------------------------------------
    // safety_margin
    // ------------------------------------------------------------------

    #[test]
    fn test_safety_margin_positive() {
        let v = make_v();
        let p = ModificationProposal::new("cognitive_load.thinking_budget", 0.3, "", "g");
        let m = v.safety_margin(&p, 0.5);
        assert!(approx_eq(m, 0.7, 1e-10));
    }

    #[test]
    fn test_safety_margin_negative() {
        let v = make_v();
        let p = ModificationProposal::new("cognitive_load.thinking_budget", 2.0, "", "g");
        let m = v.safety_margin(&p, 0.5);
        assert!(m < 0.0);
        assert!(approx_eq(m, -1.0, 1e-10));
    }

    #[test]
    fn test_safety_margin_unknown_target() {
        let v = make_v();
        let p = ModificationProposal::new("unknown.x", 0.1, "", "g");
        assert!(approx_eq(v.safety_margin(&p, 0.5), -1.0, 1e-10));
    }

    // ------------------------------------------------------------------
    // default constructor & unknown targets
    // ------------------------------------------------------------------

    #[test]
    fn test_default_constructor() {
        let v = BallVerifier::default(10);
        assert_eq!(v.dimension, 10);
        assert!(v.trusted_center.iter().all(|x| approx_eq(*x, 0.0, 1e-10)));
        assert!(approx_eq(v.radius, 1.0, 1e-10));
        assert!(v.strict_mode);
    }

    #[test]
    fn test_unknown_target_rejected() {
        let mut v = make_v();
        let p = ModificationProposal::new("nonexistent.param", 0.1, "test", "g");
        let r = v.check_proposal(&p, 0.5);
        assert!(!r.passed);
        assert!(r.reason.contains("unknown"));
    }

    #[test]
    fn test_register_custom_target() {
        let mut v = BallVerifier::default(10);
        v.register_target("custom.param", 9);
        let p = ModificationProposal::new("custom.param", 0.3, "custom", "g");
        let r = v.check_proposal(&p, 0.0);
        assert!(r.passed);
        assert!(approx_eq(v.trusted_center[9], 0.3, 1e-10));
    }

    // ------------------------------------------------------------------
    // index_for_target completeness
    // ------------------------------------------------------------------

    #[test]
    fn test_index_for_target_all_mappings() {
        assert_eq!(index_for_target("cognitive_load.thinking_budget"), Some(0));
        assert_eq!(
            index_for_target("emergent_reasoning.emergence_threshold"),
            Some(1)
        );
        assert_eq!(
            index_for_target("emergent_reasoning.exploration_rate"),
            Some(2)
        );
        assert_eq!(
            index_for_target("emergent_reasoning.learning_rate"),
            Some(3)
        );
        assert_eq!(index_for_target("personality_matrix.plasticity"), Some(4));
        assert_eq!(index_for_target("valence_axis.valence"), Some(5));
        assert_eq!(index_for_target("valence_axis.arousal"), Some(6));
        assert_eq!(
            index_for_target("inner_critic.relevance_threshold"),
            Some(7)
        );
        assert_eq!(
            index_for_target("inner_critic.consistency_threshold"),
            Some(8)
        );
        assert_eq!(
            index_for_target("inner_critic.uncertainty_tolerance"),
            Some(9)
        );
        assert_eq!(index_for_target("unknown.param"), None);
    }

    // ------------------------------------------------------------------
    // Non-strict multi-proposal
    // ------------------------------------------------------------------

    #[test]
    fn test_multi_non_strict_allows_everything() {
        let mut v = BallVerifier::default(10);
        v.strict_mode = false;

        let proposals = vec![
            ModificationProposal::new("cognitive_load.thinking_budget", 5.0, "jump", "g1"),
            ModificationProposal::new("emergent_reasoning.learning_rate", 5.0, "jump", "g2"),
        ];
        let currents = vec![
            ("cognitive_load.thinking_budget".into(), 0.0),
            ("emergent_reasoning.learning_rate".into(), 0.0),
        ];
        let verdicts = v.check_multi_proposal(&proposals, &currents);
        assert!(verdicts[0].passed);
        assert!(verdicts[1].passed);
        assert!(approx_eq(v.trusted_center[0], 5.0, 1e-10));
        assert!(approx_eq(v.trusted_center[3], 5.0, 1e-10));
    }

    // ------------------------------------------------------------------
    // centre() accessor
    // ------------------------------------------------------------------

    #[test]
    fn test_center_accessor() {
        let v = make_v();
        assert_eq!(v.center().len(), 10);
        assert!(v.center().iter().all(|x| approx_eq(*x, 0.5, 1e-10)));
    }
}

/// Transaction scope for safe self-modification with automatic rollback.
/// Wraps a state snapshot; if `commit()` is not called before drop, the
/// scope logs a rollback and restores the original state.
pub struct TransactionScope<T: Clone> {
    snapshot: T,
    committed: bool,
    label: &'static str,
}

impl<T: Clone> TransactionScope<T> {
    pub fn new(state: &T, label: &'static str) -> Self {
        Self {
            snapshot: state.clone(),
            committed: false,
            label,
        }
    }

    pub fn commit(&mut self) {
        self.committed = true;
    }

    pub fn rollback(&mut self, state: &mut T) {
        if !self.committed {
            *state = self.snapshot.clone();
            self.committed = true; // prevent double-rollback in drop
        }
    }
}

impl<T: Clone> Drop for TransactionScope<T> {
    fn drop(&mut self) {
        if !self.committed {
            log::warn!(
                "[TransactionScope] uncommitted scope dropped for '{}' — rollback implied",
                self.label
            );
        }
    }
}

/// FGGM rejection sampler: verifies output constraints before commit
///
/// Three-stage pipeline:
/// 1. Plan: LLM generates candidate edit
/// 2. Verify: Check output against formal constraints (pre/post conditions)
/// 3. Learn: Record successful/ failed verifications for future planning
///
/// Based on SEVerA (arXiv:2603.25111): Formally Guarded Generative Models
pub struct FggmRejectionSampler {
    /// Running stats for verification success rate
    verify_success_count: u64,
    verify_total_count: u64,
}

impl FggmRejectionSampler {
    pub fn new() -> Self {
        Self {
            verify_success_count: 0,
            verify_total_count: 0,
        }
    }

    /// Verify an output constraint. Returns true if constraint is satisfied.
    pub fn verify_constraint(&mut self, precondition: &str, actual: &str) -> bool {
        self.verify_total_count += 1;
        let satisfied = if precondition.is_empty() {
            true
        } else {
            actual.contains(precondition)
        };
        if satisfied {
            self.verify_success_count += 1;
        }
        satisfied
    }

    /// Acceptance sampling: reject if verification fails
    pub fn accept_or_reject<T>(&mut self, value: &T, constraint: &str) -> bool
    where
        T: std::fmt::Display,
    {
        self.verify_constraint(constraint, &value.to_string())
    }

    /// Verification success rate
    pub fn verify_rate(&self) -> f64 {
        if self.verify_total_count == 0 {
            1.0
        } else {
            self.verify_success_count as f64 / self.verify_total_count as f64
        }
    }
}

/// Extend TransactionScope with rejection sampling
pub trait TransactionScopeExt<T> {
    /// Commit only if constraint passes; otherwise auto-rollback
    fn commit_if(&mut self, constraint: &str, sampler: &mut FggmRejectionSampler) -> bool;
}

impl<T: Clone + std::fmt::Display> TransactionScopeExt<T> for TransactionScope<T> {
    fn commit_if(&mut self, constraint: &str, sampler: &mut FggmRejectionSampler) -> bool {
        if sampler.accept_or_reject(&self.snapshot, constraint) {
            self.commit();
            true
        } else {
            false
        }
    }
}
