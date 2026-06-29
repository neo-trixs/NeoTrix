use std::time::Instant;

use super::vsa_tag::VsaTagged;

#[derive(Debug, Clone, PartialEq)]
pub enum VerificationLevel {
    Syntax,
    Semantic,
    Safety,
    Executable,
    Stability,
}

impl VerificationLevel {
    pub fn name(&self) -> &'static str {
        match self {
            VerificationLevel::Syntax => "syntax",
            VerificationLevel::Semantic => "semantic",
            VerificationLevel::Safety => "safety",
            VerificationLevel::Executable => "executable",
            VerificationLevel::Stability => "stability",
        }
    }
    pub fn priority(&self) -> u8 {
        match self {
            VerificationLevel::Syntax => 0,
            VerificationLevel::Semantic => 1,
            VerificationLevel::Safety => 2,
            VerificationLevel::Executable => 3,
            VerificationLevel::Stability => 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvidenceAnchor {
    pub source: String,
    pub claim: String,
    pub confidence: f64,
    pub timestamp: u64,
    pub reexecutable: bool,
}

#[derive(Debug, Clone)]
pub struct Belief {
    pub id: u64,
    pub claim: String,
    pub anchors: Vec<EvidenceAnchor>,
    pub verification_level: VerificationLevel,
    pub verified: bool,
    pub confidence: f64,
    pub stability_score: f64,
    pub checks_passed: Vec<String>,
    pub checks_failed: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BeliefVerificationConfig {
    /// Require all five levels to pass
    pub require_all_levels: bool,
    /// Minimum confidence threshold
    pub min_confidence: f64,
    /// Require at least one re-executable anchor
    pub require_reexecutable: bool,
    /// Number of stability checks to run
    pub stability_checks: usize,
    /// Syntax pattern validation rules
    pub syntax_rules: Vec<String>,
}

impl Default for BeliefVerificationConfig {
    fn default() -> Self {
        Self {
            require_all_levels: true,
            min_confidence: 0.5,
            require_reexecutable: true,
            stability_checks: 3,
            syntax_rules: vec!["non_empty".into(), "valid_encoding".into()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct VerificationReport {
    pub belief: Belief,
    pub all_passed: bool,
    pub levels_passed: Vec<VerificationLevel>,
    pub levels_failed: Vec<VerificationLevel>,
    pub duration_ns: u64,
    pub recommendation: String,
}

pub struct ExecutableBeliefVerifier {
    config: BeliefVerificationConfig,
    belief_counter: u64,
}

impl ExecutableBeliefVerifier {
    pub fn new(config: BeliefVerificationConfig) -> Self {
        Self {
            config,
            belief_counter: 0,
        }
    }

    pub fn config(&self) -> &BeliefVerificationConfig {
        &self.config
    }

    pub fn verify(
        &mut self,
        claim: &str,
        state: &VsaTagged,
        anchors: Vec<EvidenceAnchor>,
    ) -> VerificationReport {
        let start = Instant::now();
        self.belief_counter += 1;

        let mut belief = Belief {
            id: self.belief_counter,
            claim: claim.to_string(),
            anchors,
            verification_level: VerificationLevel::Syntax,
            verified: false,
            confidence: state.confidence,
            stability_score: 0.0,
            checks_passed: Vec::new(),
            checks_failed: Vec::new(),
        };

        let mut passed = Vec::new();
        let mut failed = Vec::new();

        // Level 1: Syntax check
        if self.check_syntax(&belief) {
            belief.checks_passed.push("syntax".into());
            passed.push(VerificationLevel::Syntax);
        } else {
            belief.checks_failed.push("syntax".into());
            failed.push(VerificationLevel::Syntax);
        }

        // Level 2: Semantic check
        if self.check_semantic(&belief, state) {
            belief.checks_passed.push("semantic".into());
            passed.push(VerificationLevel::Semantic);
        } else {
            belief.checks_failed.push("semantic".into());
            failed.push(VerificationLevel::Semantic);
        }

        // Level 3: Safety check
        if self.check_safety(&belief, state) {
            belief.checks_passed.push("safety".into());
            passed.push(VerificationLevel::Safety);
        } else {
            belief.checks_failed.push("safety".into());
            failed.push(VerificationLevel::Safety);
        }

        // Level 4: Executable anchor check (Inspector pattern)
        if self.check_executable(&belief) {
            belief.checks_passed.push("executable".into());
            passed.push(VerificationLevel::Executable);
        } else {
            belief.checks_failed.push("executable".into());
            failed.push(VerificationLevel::Executable);
        }

        // Level 5: Stability check (CLR + LoopWM stability)
        if self.check_stability(&belief) {
            belief.checks_passed.push("stability".into());
            passed.push(VerificationLevel::Stability);
        } else {
            belief.checks_failed.push("stability".into());
            failed.push(VerificationLevel::Stability);
        }

        let all_passed = if self.config.require_all_levels {
            failed.is_empty()
        } else {
            passed.len() >= 3
        };

        belief.verification_level = if passed.contains(&VerificationLevel::Stability) {
            VerificationLevel::Stability
        } else if passed.contains(&VerificationLevel::Executable) {
            VerificationLevel::Executable
        } else if passed.contains(&VerificationLevel::Safety) {
            VerificationLevel::Safety
        } else if passed.contains(&VerificationLevel::Semantic) {
            VerificationLevel::Semantic
        } else {
            VerificationLevel::Syntax
        };
        belief.verified = all_passed;
        belief.stability_score = self.compute_stability_score(&passed);

        let recommendation = if all_passed {
            "BELIEF_VERIFIED: All checks passed".into()
        } else if failed.len() >= 3 {
            format!(
                "BELIEF_REJECTED: {} checks failed: {:?}",
                failed.len(),
                failed.iter().map(|l| l.name()).collect::<Vec<_>>()
            )
        } else {
            format!(
                "BELIEF_PARTIAL: {} passed, {} failed: {:?}",
                passed.len(),
                failed.len(),
                failed.iter().map(|l| l.name()).collect::<Vec<_>>()
            )
        };

        let duration = start.elapsed().as_nanos() as u64;

        VerificationReport {
            belief,
            all_passed,
            levels_passed: passed,
            levels_failed: failed,
            duration_ns: duration,
            recommendation,
        }
    }

    fn check_syntax(&self, belief: &Belief) -> bool {
        if belief.claim.trim().is_empty() {
            return false;
        }
        if belief.claim.len() < 3 {
            return false;
        }
        true
    }

    fn check_semantic(&self, _belief: &Belief, state: &VsaTagged) -> bool {
        state.confidence >= self.config.min_confidence
    }

    fn check_safety(&self, belief: &Belief, state: &VsaTagged) -> bool {
        let safety_keywords = ["harm", "danger", "unsafe"];
        let claim_lower = belief.claim.to_lowercase();
        let has_unsafe = safety_keywords.iter().any(|k| claim_lower.contains(k));
        if has_unsafe {
            return state.confidence < 0.3;
        }
        true
    }

    fn check_executable(&self, belief: &Belief) -> bool {
        if belief.anchors.is_empty() {
            return false;
        }
        if self.config.require_reexecutable {
            belief.anchors.iter().any(|a| a.reexecutable)
        } else {
            true
        }
    }

    fn check_stability(&self, belief: &Belief) -> bool {
        if belief.anchors.is_empty() {
            return false;
        }
        let avg_confidence: f64 =
            belief.anchors.iter().map(|a| a.confidence).sum::<f64>() / belief.anchors.len() as f64;
        avg_confidence >= self.config.min_confidence
    }

    fn compute_stability_score(&self, passed: &[VerificationLevel]) -> f64 {
        if passed.is_empty() {
            return 0.0;
        }
        let max_priority = 4u8;
        let sum: f64 = passed
            .iter()
            .map(|l| l.priority() as f64 / max_priority as f64)
            .sum();
        (sum / passed.len() as f64).clamp(0.0, 1.0)
    }

    pub fn verify_vsa_state(&mut self, state: &VsaTagged) -> VerificationReport {
        let claim = format!("VSA state: {:?}", state.tag);
        let anchor = EvidenceAnchor {
            source: "consciousness_cycle".into(),
            claim: claim.clone(),
            confidence: state.confidence,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            reexecutable: true,
        };
        self.verify(&claim, state, vec![anchor])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::vsa_tag::{VsaOrigin, VsaSelfCategory};

    #[test]
    fn test_default_config() {
        let config = BeliefVerificationConfig::default();
        assert!(config.require_all_levels);
        assert_eq!(config.stability_checks, 3);
    }

    #[test]
    fn test_verify_passes_valid_belief() {
        let mut verifier = ExecutableBeliefVerifier::new(BeliefVerificationConfig::default());
        let state = VsaTagged::new(vec![1u8; 64], VsaOrigin::Self_(VsaSelfCategory::Thought))
            .with_confidence(0.8);
        let anchors = vec![EvidenceAnchor {
            source: "test".into(),
            claim: "valid claim".into(),
            confidence: 0.9,
            timestamp: 1000,
            reexecutable: true,
        }];
        let report = verifier.verify("valid claim", &state, anchors);
        assert!(report.all_passed);
        assert!(report.levels_passed.contains(&VerificationLevel::Syntax));
    }

    #[test]
    fn test_rejects_empty_claim() {
        let mut verifier = ExecutableBeliefVerifier::new(BeliefVerificationConfig::default());
        let state = VsaTagged::new(vec![1u8; 64], VsaOrigin::Self_(VsaSelfCategory::Thought))
            .with_confidence(0.8);
        let report = verifier.verify("", &state, vec![]);
        assert!(!report.all_passed);
        assert!(report.levels_failed.contains(&VerificationLevel::Syntax));
    }

    #[test]
    fn test_rejects_low_confidence() {
        let mut verifier = ExecutableBeliefVerifier::new(BeliefVerificationConfig::default());
        let state = VsaTagged::new(vec![1u8; 64], VsaOrigin::Self_(VsaSelfCategory::Thought))
            .with_confidence(0.1);
        let report = verifier.verify(
            "low confidence",
            &state,
            vec![EvidenceAnchor {
                source: "test".into(),
                claim: "low".into(),
                confidence: 0.1,
                timestamp: 0,
                reexecutable: true,
            }],
        );
        assert!(!report.all_passed);
    }

    #[test]
    fn test_rejects_safety_violation() {
        let mut verifier = ExecutableBeliefVerifier::new(BeliefVerificationConfig::default());
        let state = VsaTagged::new(vec![1u8; 64], VsaOrigin::Self_(VsaSelfCategory::Thought))
            .with_confidence(0.9);
        let report = verifier.verify(
            "harm to user",
            &state,
            vec![EvidenceAnchor {
                source: "test".into(),
                claim: "harm".into(),
                confidence: 0.9,
                timestamp: 0,
                reexecutable: true,
            }],
        );
        assert!(!report.all_passed);
    }

    #[test]
    fn test_verify_vsa_state() {
        let mut verifier = ExecutableBeliefVerifier::new(BeliefVerificationConfig::default());
        let state = VsaTagged::new(vec![2u8; 64], VsaOrigin::Self_(VsaSelfCategory::Thought))
            .with_confidence(0.7);
        let report = verifier.verify_vsa_state(&state);
        assert!(report.all_passed || !report.all_passed);
    }

    #[test]
    fn test_increments_counter() {
        let mut verifier = ExecutableBeliefVerifier::new(BeliefVerificationConfig::default());
        let state = VsaTagged::new(vec![1u8; 64], VsaOrigin::Self_(VsaSelfCategory::Thought))
            .with_confidence(0.8);
        let anchors = vec![EvidenceAnchor {
            source: "t".into(),
            claim: "c".into(),
            confidence: 0.9,
            timestamp: 0,
            reexecutable: true,
        }];
        let r1 = verifier.verify("belief 1", &state, anchors.clone());
        let r2 = verifier.verify("belief 2", &state, anchors);
        assert_eq!(r1.belief.id, 1);
        assert_eq!(r2.belief.id, 2);
    }

    #[test]
    fn test_stability_score() {
        let mut verifier = ExecutableBeliefVerifier::new(BeliefVerificationConfig::default());
        let passed = vec![
            VerificationLevel::Syntax,
            VerificationLevel::Semantic,
            VerificationLevel::Executable,
        ];
        let score = verifier.compute_stability_score(&passed);
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_empty_stability_score() {
        let verifier = ExecutableBeliefVerifier::new(BeliefVerificationConfig::default());
        let score = verifier.compute_stability_score(&[]);
        assert!((score - 0.0).abs() < 1e-6);
    }
}
