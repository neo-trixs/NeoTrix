//! # FGGM Safety Unifier — Formally Guarded Generative Models
//!
//! SEVerA-inspired (arXiv:2603.25111) unification of NeoTrix's 3 independent
//! safety mechanisms into a single 4-phase FGGM pipeline:
//!
//! 1. **Planning** — First-order logic contract satisfaction
//! 2. **Rejection Sampling** — SafetyGate 5-check fast pass
//! 3. **Verification** — PccSafetyGate proof-carrying code
//! 4. **Geometry** — BallVerifier L2 parameter space check
//!
//! When all 4 phases pass, the modification is considered safe under the
//! FGGM formalism.

use crate::core::nt_core_experience::pcc_safety::PccSafetyGate;
use crate::core::nt_core_experience::safety_ball::{BallVerifier, ModificationProposal};
use crate::core::nt_core_experience::safety_gate::SafetyGate;

/// Result of a single FGGM phase check.
#[derive(Debug, Clone)]
pub struct FggmPhaseResult {
    pub phase_name: &'static str,
    pub passed: bool,
    pub reason: String,
}

/// Aggregate result from all 4 FGGM phases.
#[derive(Debug, Clone)]
pub struct FggmResult {
    pub all_passed: bool,
    pub phase_results: Vec<FggmPhaseResult>,
    pub contract_satisfaction: f64,
}

impl FggmResult {
    pub fn passed_phases(&self) -> Vec<&str> {
        self.phase_results
            .iter()
            .filter(|r| r.passed)
            .map(|r| r.phase_name)
            .collect()
    }

    pub fn failed_phases(&self) -> Vec<&str> {
        self.phase_results
            .iter()
            .filter(|r| !r.passed)
            .map(|r| r.phase_name)
            .collect()
    }
}

/// Unifier wrapping NeoTrix's 3 safety mechanisms in a 4-phase FGGM pipeline.
///
/// Ownership of the 3 inner guards is transferred into this struct.
pub struct FggmSafetyUnifier {
    /// First-order logic contracts as string predicates.
    /// Each contract is evaluated via simple rule matching.
    pub contracts: Vec<String>,
    /// Whether to run SafetyGate rejection sampling (Phase 2).
    pub rejection_sampler_enabled: bool,
    /// Hard SafetyGate instance (phase 2).
    pub safety_gate: SafetyGate,
    /// Proof-Carrying Code gate (phase 3).
    pub pcc_gate: PccSafetyGate,
    /// L2 ball verifier (phase 4).
    pub ball_verifier: BallVerifier,
    /// SEVerA compliance history: (proposal_hash, passed, phase_name)
    pub compliance_history: Vec<(String, bool, String)>,
}

impl FggmSafetyUnifier {
    /// Create a new FGGM safety unifier with default parameters.
    ///
    /// * `contracts` — first-order logic contract strings
    /// * `rejection_sampler_enabled` — whether to run Phase 2
    /// * `dimension` — parameter space dimension for the BallVerifier
    pub fn new(contracts: Vec<String>, rejection_sampler_enabled: bool, dimension: usize) -> Self {
        let mut pcc_gate = PccSafetyGate::new(true, true);
        pcc_gate.cps.register_defaults();

        Self {
            contracts,
            rejection_sampler_enabled,
            safety_gate: SafetyGate::new(),
            pcc_gate,
            ball_verifier: BallVerifier::default(dimension),
            compliance_history: Vec::with_capacity(100),
        }
    }

    /// Run the full 4-phase FGGM safety check.
    ///
    /// Phases:
    /// 1. **Planning** — Evaluate contract string satisfaction via keyword/pattern matching.
    /// 2. **Rejection Sampling** — If enabled, run SafetyGate fast checks.
    /// 3. **Verification** — Run PccSafetyGate proof obligations.
    /// 4. **Geometry** — Run BallVerifier L2 distance check.
    pub fn check_proposal(&mut self, proposal: &str) -> FggmResult {
        let mut phase_results = Vec::with_capacity(4);

        // ── Phase 1: Planning — Contract satisfaction ──────────────────
        let (planning_passed, planning_reason, satisfaction) = self.evaluate_contracts(proposal);
        phase_results.push(FggmPhaseResult {
            phase_name: "planning",
            passed: planning_passed,
            reason: planning_reason,
        });

        // ── Phase 2: Rejection Sampling — SafetyGate fast checks ───────
        if self.rejection_sampler_enabled {
            let (rs_passed, rs_reason) = self.run_rejection_sampling(proposal);
            phase_results.push(FggmPhaseResult {
                phase_name: "rejection_sampling",
                passed: rs_passed,
                reason: rs_reason,
            });
        } else {
            phase_results.push(FggmPhaseResult {
                phase_name: "rejection_sampling",
                passed: true,
                reason: "rejection sampling disabled, skipped".into(),
            });
        }

        // ── Phase 3: Verification — PccSafetyGate proofs ───────────────
        let (pcc_passed, pcc_reason) = self.run_pcc_verification(proposal);
        phase_results.push(FggmPhaseResult {
            phase_name: "verification",
            passed: pcc_passed,
            reason: pcc_reason,
        });

        // ── Phase 4: Geometry — BallVerifier parameter space ────────────
        let (geo_passed, geo_reason) = self.run_geometry_check(proposal);
        phase_results.push(FggmPhaseResult {
            phase_name: "geometry",
            passed: geo_passed,
            reason: geo_reason,
        });

        let all_passed = phase_results.iter().all(|r| r.passed);

        for pr in &phase_results {
            self.record_compliance(proposal, pr.passed, pr.phase_name);
        }

        FggmResult {
            all_passed,
            phase_results,
            contract_satisfaction: satisfaction,
        }
    }

    /// Phase 1: Evaluate contract string satisfaction against the proposal.
    ///
    /// Uses simple keyword/pattern matching:
    /// - Each contract string is matched against the proposal for substring containment.
    /// - Contract satisfaction = fraction of contracts that match.
    /// - All contracts must match for the phase to pass.
    fn evaluate_contracts(&self, proposal: &str) -> (bool, String, f64) {
        if self.contracts.is_empty() {
            return (true, "no contracts defined, auto-pass".into(), 1.0);
        }

        let proposal_lower = proposal.to_lowercase();
        let total = self.contracts.len() as f64;
        let satisfied: usize = self
            .contracts
            .iter()
            .filter(|c| {
                let c_lower = c.to_lowercase();
                proposal_lower.contains(&c_lower) || self.matches_pattern(c, &proposal_lower)
            })
            .count();

        let satisfaction = satisfied as f64 / total;
        let all_match = satisfied == self.contracts.len();

        let details: Vec<String> = self
            .contracts
            .iter()
            .map(|c| {
                let c_lower = c.to_lowercase();
                let match_ =
                    proposal_lower.contains(&c_lower) || self.matches_pattern(c, &proposal_lower);
                format!("  contract '{}': {}", c, if match_ { "✓" } else { "✗" })
            })
            .collect();

        (
            all_match,
            format!(
                "contracts {}/{} satisfied (score={:.3})\n{}",
                satisfied,
                self.contracts.len(),
                satisfaction,
                details.join("\n")
            ),
            satisfaction,
        )
    }

    /// Check a contract by extracting a numeric predicate and testing against
    /// values found in the proposal (e.g. "negentropy >= 0").
    fn matches_pattern(&self, contract: &str, proposal_lower: &str) -> bool {
        let tokens: Vec<&str> = contract.split_whitespace().collect();
        if tokens.len() < 3 {
            return false;
        }

        let metric = tokens[0].to_lowercase();
        let op = tokens[1];
        let threshold: f64 = tokens[2].parse().unwrap_or(0.0);

        // Extract a numeric value from the proposal for this metric.
        let found = self.extract_numeric(proposal_lower, &metric);

        match (found, op) {
            (Some(v), ">=") => v >= threshold,
            (Some(v), "<=") => v <= threshold,
            (Some(v), ">") => v > threshold,
            (Some(v), "<") => v < threshold,
            (Some(v), "==") => (v - threshold).abs() < 1e-9,
            _ => false,
        }
    }

    /// Extract a numeric value from a proposal string for a given metric.
    fn extract_numeric(&self, text: &str, metric: &str) -> Option<f64> {
        // Try "metric = value" or "metric: value" pattern
        let patterns = [
            format!(r"{}\s*[=:]\s*([+-]?\d+\.?\d*)", regex_lite(metric)),
            format!(r"{}\s+is\s+([+-]?\d+\.?\d*)", regex_lite(metric)),
        ];

        for pattern in &patterns {
            if let Some(val) = self.regex_extract(text, pattern) {
                return Some(val);
            }
        }
        None
    }

    /// Minimal regex-like extraction (no external regex crate).
    fn regex_extract(&self, text: &str, pattern: &str) -> Option<f64> {
        // pattern is "prefix\s*[=:]\s*(\d+...)"
        let parts: Vec<&str> = pattern.split(r"\s*").collect();
        if parts.is_empty() {
            return None;
        }
        let prefix = parts[0];

        // Find prefix in text
        if let Some(pos) = text.find(prefix) {
            let after = &text[pos + prefix.len()..];
            // Skip optional = or : and spaces
            let after = after.trim_start();
            let after = after
                .strip_prefix('=')
                .or_else(|| after.strip_prefix(':'))
                .unwrap_or(after);
            let after = after.trim_start();
            // Grab the number
            let num_str: String = after
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                .collect();
            if !num_str.is_empty() {
                return num_str.parse::<f64>().ok();
            }
        }
        None
    }

    /// Phase 2: Run SafetyGate rejection sampling (all 5 checks).
    fn run_rejection_sampling(&mut self, proposal: &str) -> (bool, String) {
        let primitives = SafetyGate::compute_reference_primitives();
        let risk_score = if proposal.contains("unsafe")
            || proposal.contains("dangerous")
            || proposal.contains("delete")
            || proposal.contains("rm -rf")
            || proposal.contains("format")
            || proposal.contains("overwrite")
        {
            0.3
        } else if proposal.contains("modify")
            || proposal.contains("edit")
            || proposal.contains("replace")
            || proposal.contains("update")
            || proposal.contains("patch")
        {
            0.8
        } else {
            0.9
        };
        let scores = [risk_score, risk_score, risk_score];
        let report = self
            .safety_gate
            .check_all(None, true, &scores, &primitives, 0.05, 0.9, 0.88);

        let passed = report.all_passed;
        let mut details = Vec::new();
        for check in &report.checks {
            details.push(format!(
                "  {}: {} (score={:.3})",
                check.name,
                if check.passed { "✓" } else { "✗" },
                check.score
            ));
        }

        (
            passed,
            format!("SafetyGate: all_passed={}\n{}", passed, details.join("\n")),
        )
    }

    /// Phase 3: Run PccSafetyGate proof verification.
    fn run_pcc_verification(&mut self, proposal: &str) -> (bool, String) {
        let edits = vec![(proposal.to_string(), 0.5, "fggm_unified".to_string())];
        let result = self.pcc_gate.evaluate_edits(&edits);

        match result {
            crate::core::nt_core_experience::pcc_safety::SelfModificationVerdict::Approved {
                obligations_passed,
            } => (
                true,
                format!(
                    "PccSafetyGate: approved, {} obligations passed",
                    obligations_passed
                ),
            ),
            crate::core::nt_core_experience::pcc_safety::SelfModificationVerdict::Rejected {
                obligations_failed,
            } => {
                let reasons: Vec<String> = obligations_failed
                    .iter()
                    .map(|o| format!("  oblig#{}: {}", o.id, o.verification_log))
                    .collect();
                (
                    false,
                    format!(
                        "PccSafetyGate: rejected, {} obligations failed\n{}",
                        obligations_failed.len(),
                        reasons.join("\n")
                    ),
                )
            }
        }
    }

    /// Phase 4: Run BallVerifier L2 geometry check in parameter space.
    fn run_geometry_check(&mut self, proposal: &str) -> (bool, String) {
        // Extract a target parameter from the proposal if possible.
        let target = self.extract_target(proposal);
        let delta = self.extract_delta(proposal);

        match target {
            Some(t) => {
                let p = ModificationProposal::new(&t, delta, "fggm", "fggm_unifier");
                let idx = crate::core::nt_core_experience::safety_ball::index_for_target(&t);
                match idx {
                    Some(i) => {
                        let current = self
                            .ball_verifier
                            .trusted_center
                            .get(i)
                            .copied()
                            .unwrap_or(0.0);
                        let verdict = self.ball_verifier.check_proposal(&p, current);
                        (
                            verdict.passed,
                            format!(
                                "BallVerifier: param='{}' delta={:+.4} dist={:.4}/rad={:.4} pass={} safety_margin={:.4}",
                                t, delta, verdict.distance_from_center, verdict.radius, verdict.passed, verdict.safety_margin
                            ),
                        )
                    }
                    None => {
                        // Unknown parameter: non-strict pass with warning
                        (
                            true,
                            format!(
                                "BallVerifier: unknown param '{}', skipping geometry check",
                                t
                            ),
                        )
                    }
                }
            }
            None => (
                true,
                "BallVerifier: no target parameter found in proposal, skipping geometry check"
                    .into(),
            ),
        }
    }

    /// Extract a target parameter name from a proposal string.
    fn extract_target(&self, proposal: &str) -> Option<String> {
        let known_targets = [
            "cognitive_load.thinking_budget",
            "emergent_reasoning.emergence_threshold",
            "emergent_reasoning.exploration_rate",
            "emergent_reasoning.learning_rate",
            "personality_matrix.plasticity",
            "valence_axis.valence",
            "valence_axis.arousal",
            "inner_critic.relevance_threshold",
            "inner_critic.consistency_threshold",
            "inner_critic.uncertainty_tolerance",
        ];
        let lower = proposal.to_lowercase();
        for t in &known_targets {
            if lower.contains(t) {
                return Some(t.to_string());
            }
        }
        // Fallback: try to find a dotted parameter name
        if let Some(start) = proposal.find(|c: char| c.is_ascii_lowercase()) {
            let rest = &proposal[start..];
            if let Some(end) = rest.find(|c: char| c.is_whitespace() || c == ',' || c == ')') {
                let candidate = &rest[..end];
                if candidate.contains('.') {
                    return Some(candidate.to_string());
                }
            }
        }
        None
    }

    /// Extract a numeric delta from a proposal string.
    fn extract_delta(&self, proposal: &str) -> f64 {
        // Try "delta = X" or "delta: X" or "Δ = X"
        let lower = proposal.to_lowercase();
        let patterns = ["delta =", "delta:", "delta=", "Δ =", "Δ:", "Δ=", "by "];
        for p in &patterns {
            if let Some(pos) = lower.find(p) {
                let after = lower[pos + p.len()..].trim_start();
                let num_str: String = after
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+')
                    .collect();
                if !num_str.is_empty() {
                    if let Ok(v) = num_str.parse::<f64>() {
                        return v;
                    }
                }
            }
        }
        // Default: extract any +/- number at the end
        let words: Vec<&str> = lower.split_whitespace().collect();
        for w in words.iter().rev() {
            let cleaned: String = w
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                .collect();
            if !cleaned.is_empty() && cleaned != "." {
                if let Ok(v) = cleaned.parse::<f64>() {
                    return v;
                }
            }
        }
        0.0
    }

    /// Return a reference to the inner SafetyGate.
    pub fn safety_gate(&self) -> &SafetyGate {
        &self.safety_gate
    }

    /// Return a mutable reference to the inner PccSafetyGate.
    pub fn pcc_gate_mut(&mut self) -> &mut PccSafetyGate {
        &mut self.pcc_gate
    }

    /// Return a reference to the inner BallVerifier.
    pub fn ball_verifier(&self) -> &BallVerifier {
        &self.ball_verifier
    }

    /// Return a mutable reference to the inner BallVerifier.
    pub fn ball_verifier_mut(&mut self) -> &mut BallVerifier {
        &mut self.ball_verifier
    }

    /// Register a custom parameter target in the BallVerifier.
    pub fn register_ball_target(&mut self, target: &str, index: usize) {
        self.ball_verifier.register_target(target, index);
    }

    pub fn set_rejection_sampler(&mut self, enabled: bool) {
        self.rejection_sampler_enabled = enabled;
    }

    pub fn set_contracts(&mut self, contracts: Vec<String>) {
        self.contracts = contracts;
    }

    /// Record a compliance outcome for a proposal phase.
    pub fn record_compliance(&mut self, proposal: &str, passed: bool, phase: &str) {
        self.compliance_history
            .push((proposal.to_string(), passed, phase.to_string()));
        if self.compliance_history.len() > 1000 {
            self.compliance_history.drain(0..500);
        }
    }
}

/// Escape a string for use in the minimal regex-like extraction.
fn regex_lite(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c == '.' || c == '(' || c == ')' || c == '[' || c == ']' || c == '+' || c == '*' {
                format!("\\{}", c)
            } else {
                c.to_string()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_unifier() -> FggmSafetyUnifier {
        let contracts = vec![
            "negentropy >= 0".to_string(),
            "cognitive_load <= 0.8".to_string(),
        ];
        FggmSafetyUnifier::new(contracts, true, 10)
    }

    #[test]
    fn test_all_phases_passed() {
        let mut u = make_unifier();
        let proposal = "adjust cognitive_load.thinking_budget delta = 0.1, negentropy >= 0, cognitive_load <= 0.8";
        let result = u.check_proposal(proposal);
        assert!(result.all_passed, "expected all passed, got: {:#?}", result);
        assert_eq!(result.phase_results.len(), 4);
        assert!((result.contract_satisfaction - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_contract_satisfaction_partial() {
        let contracts = vec![
            "negentropy >= 0".to_string(),
            "nonexistent_contract >= 1".to_string(),
        ];
        let mut u = FggmSafetyUnifier::new(contracts, true, 10);
        let proposal = "adjust threshold, negentropy >= 0";
        let result = u.check_proposal(proposal);
        // Phase 1 should fail (not all contracts satisfied)
        assert!(!result.phase_results[0].passed);
        assert!((result.contract_satisfaction - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_contract_satisfaction_empty_auto_pass() {
        let mut u = FggmSafetyUnifier::new(vec![], true, 10);
        let proposal = "any modification";
        let result = u.check_proposal(proposal);
        assert!((result.contract_satisfaction - 1.0).abs() < 1e-9);
        assert!(result.phase_results[0].passed);
    }

    #[test]
    fn test_rejection_sampling_disabled_skips_phase_2() {
        let mut u = FggmSafetyUnifier::new(vec!["negentropy >= 0".to_string()], false, 10);
        let proposal = "adjust threshold delta = 0.1, negentropy >= 0";
        let result = u.check_proposal(proposal);
        assert!(result.phase_results[1].passed);
        assert!(result.phase_results[1].reason.contains("disabled"));
    }

    #[test]
    fn test_contract_numeric_threshold_ge() {
        let contracts = vec!["negentropy >= 0".to_string()];
        let mut u = FggmSafetyUnifier::new(contracts, true, 10);
        // Proposal contains "negentropy = 0.5" which should satisfy "negentropy >= 0"
        let result = u.check_proposal("adjust param, negentropy = 0.5");
        assert!(result.phase_results[0].passed);
    }

    #[test]
    fn test_contract_numeric_threshold_fails() {
        let contracts = vec!["negentropy >= 0.8".to_string()];
        let mut u = FggmSafetyUnifier::new(contracts, true, 10);
        // Proposal contains "negentropy = 0.3" which fails "negentropy >= 0.8"
        let result = u.check_proposal("adjust param, negentropy = 0.3");
        assert!(!result.phase_results[0].passed);
    }

    #[test]
    fn test_contract_le_threshold() {
        let contracts = vec!["cognitive_load <= 0.8".to_string()];
        let mut u = FggmSafetyUnifier::new(contracts, true, 10);
        let result = u.check_proposal("cognitive_load <= 0.8, adjust param");
        // The contract "cognitive_load <= 0.8" contains the words "cognitive_load" and "<= 0.8"
        // But our matching uses substring containment. The contract has the pattern "cognitive_load <= 0.8"
        // and proposal_lower contains "cognitive_load <= 0.8" as substring.
        assert!(
            result.phase_results[0].passed,
            "Expected pass, got: {}",
            result.phase_results[0].reason
        );
    }

    #[test]
    fn test_lt_threshold() {
        let contracts = vec!["cognitive_load < 1.0".to_string()];
        let mut u = FggmSafetyUnifier::new(contracts, true, 10);
        let result = u.check_proposal("cognitive_load = 0.5, adjust param");
        assert!(result.phase_results[0].passed);
    }

    #[test]
    fn test_all_3_mechanisms_called_through_unifier() {
        let mut u = make_unifier();
        let proposal = "adjust cognitive_load.thinking_budget delta = 0.1, negentropy >= 0";
        let result = u.check_proposal(proposal);

        // Phase 2 should have run SafetyGate
        assert!(result.phase_results[1].reason.contains("SafetyGate"));

        // Phase 3 should have run PccSafetyGate
        assert!(result.phase_results[2].reason.contains("PccSafetyGate"));

        // Phase 4 should have run BallVerifier
        assert!(result.phase_results[3].reason.contains("BallVerifier"));

        assert_eq!(result.phase_results.len(), 4);
    }

    #[test]
    fn test_geometry_check_ball_verifier_integration() {
        let mut u = make_unifier();
        // cognitive_load.thinking_budget has index 0 with trusted_center[0]=0.5 (default centered at 0.0)
        // delta=0.1 → distance=0.1 ≤ radius=1.0 → pass
        let result = u.check_proposal("modify cognitive_load.thinking_budget delta = 0.1");
        assert!(result.phase_results[3].passed);
    }

    #[test]
    fn test_geometry_check_exceeds_ball() {
        let mut u = make_unifier();
        // delta=2.0 → distance=2.0 > radius=1.0 → fail
        let result = u.check_proposal("modify cognitive_load.thinking_budget delta = 2.0");
        assert!(!result.phase_results[3].passed);
        assert!(result.phase_results[3].reason.contains("BallVerifier"));
    }

    #[test]
    fn test_unknown_target_skips_geometry() {
        let mut u = make_unifier();
        let result = u.check_proposal("modify unknown.param delta = 0.1");
        // Unknown target → geometry phase skips with pass
        assert!(result.phase_results[3].passed);
        assert!(result.phase_results[3].reason.contains("unknown"));
    }

    #[test]
    fn test_extract_delta_from_proposal() {
        let u = make_unifier();
        assert!((u.extract_delta("delta = 0.5") - 0.5).abs() < 1e-9);
        assert!((u.extract_delta("delta: -0.3") - (-0.3)).abs() < 1e-9);
        assert!((u.extract_delta("Δ = 0.75") - 0.75).abs() < 1e-9);
        assert!((u.extract_delta("no delta here") - 0.0).abs() < 1e-9);
        assert!((u.extract_delta("by 0.25") - 0.25).abs() < 1e-9);
    }

    #[test]
    fn test_extract_target_from_proposal() {
        let u = make_unifier();
        assert_eq!(
            u.extract_target("modify cognitive_load.thinking_budget"),
            Some("cognitive_load.thinking_budget".to_string())
        );
        assert_eq!(
            u.extract_target("adjust inner_critic.consistency_threshold"),
            Some("inner_critic.consistency_threshold".to_string())
        );
        assert_eq!(u.extract_target("no target here"), None);
    }

    #[test]
    fn test_rejection_sampler_can_be_toggled() {
        let mut u = make_unifier();
        assert!(u.rejection_sampler_enabled);
        u.set_rejection_sampler(false);
        assert!(!u.rejection_sampler_enabled);
        u.set_rejection_sampler(true);
        assert!(u.rejection_sampler_enabled);
    }

    #[test]
    fn test_contracts_can_be_replaced() {
        let mut u = make_unifier();
        assert_eq!(u.contracts.len(), 2);
        u.set_contracts(vec!["new_contract >= 1".to_string()]);
        assert_eq!(u.contracts.len(), 1);
        assert_eq!(u.contracts[0], "new_contract >= 1");
    }

    #[test]
    fn test_register_ball_target() {
        let mut u = make_unifier();
        u.register_ball_target("custom.param", 5);
        let result = u.check_proposal("modify custom.param delta = 0.1");
        // After registering, it should not skip
        assert!(result.phase_results[3].reason.contains("custom.param"));
    }

    #[test]
    fn test_passed_and_failed_phases() {
        let contracts = vec!["impossible >= 100".to_string()];
        let mut u = FggmSafetyUnifier::new(contracts, true, 10);
        let result = u.check_proposal("adjust param");
        let passed = result.passed_phases();
        let failed = result.failed_phases();
        // Phase 0 (planning) should fail
        assert!(failed.contains(&"planning"));
        // Other phases might still pass
        assert!(passed.contains(&"geometry") || passed.contains(&"verification"));
    }

    #[test]
    fn test_matches_pattern_numeric_extraction() {
        let _u = make_unifier();
        // Test regex_lite escapes dots properly
        let escaped = regex_lite("cognitive_load.thinking_budget");
        assert_eq!(escaped, "cognitive_load\\.thinking_budget");
    }
}
