use super::agent::SelfModifyProposal;

/// The result of evaluating a proposal through a safety gate.
#[derive(Debug, Clone, PartialEq)]
pub enum GateResult {
    Approved,
    Rejected { reason: String, gate: String },
}

/// Four-layer safety guard for self-modification operations.
///
/// Each layer is an optional `Box<dyn Fn>` — when `None`, the gate passes
/// in dev mode. When `Some`, the function must return true for approval.
///
/// Layers evaluated in order: Shield → Swords → LLM Validator → Ball Verifier.
/// If any layer rejects, the proposal is denied with the gate name and reason.
pub struct SelfModifyGuard {
    /// Shield bus: blocks unsafe handler/primitive targets
    pub shield_bus: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
    /// Swords check: validates that source code contains no dangerous patterns
    pub swords_check: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
    /// LLM validator: scores proposal quality on [0,1]; threshold 0.3
    pub llm_validator: Option<Box<dyn Fn(&str) -> f64 + Send + Sync>>,
    /// Ball verifier: constraint satisfaction check on the source
    pub ball_verifier: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
}

impl SelfModifyGuard {
    pub fn new() -> Self {
        Self {
            shield_bus: None,
            swords_check: None,
            llm_validator: None,
            ball_verifier: None,
        }
    }

    pub fn with_shield(mut self, f: Box<dyn Fn(&str) -> bool + Send + Sync>) -> Self {
        self.shield_bus = Some(f);
        self
    }

    pub fn with_swords(mut self, f: Box<dyn Fn(&str) -> bool + Send + Sync>) -> Self {
        self.swords_check = Some(f);
        self
    }

    pub fn with_llm_validator(mut self, f: Box<dyn Fn(&str) -> f64 + Send + Sync>) -> Self {
        self.llm_validator = Some(f);
        self
    }

    pub fn with_ball_verifier(mut self, f: Box<dyn Fn(&str) -> bool + Send + Sync>) -> Self {
        self.ball_verifier = Some(f);
        self
    }

    /// Evaluate a proposal through all layers sequentially.
    ///
    /// Layer 1 — Shield Bus: checks the target string for blocked patterns.
    /// Layer 2 — Swords Check: scans source code for dangerous constructs.
    /// Layer 3 — LLM Validator: quality score must exceed 0.3.
    /// Layer 4 — Ball Verifier: constraint satisfaction gate.
    ///
    /// Returns `Approved` only when all active layers pass.
    pub fn evaluate(&self, proposal: &SelfModifyProposal) -> GateResult {
        // Layer 1: Shield bus
        if let Some(ref shield) = self.shield_bus {
            let target_str = format!("{:?}", proposal.target);
            if !shield(&target_str) {
                return GateResult::Rejected {
                    reason: format!("shield_bus blocked target: {}", target_str),
                    gate: "shield_bus".into(),
                };
            }
        }

        // Layer 2: Swords check
        if let Some(ref swords) = self.swords_check {
            if !swords(&proposal.source_code) {
                return GateResult::Rejected {
                    reason: "swords_check detected dangerous pattern in source_code".into(),
                    gate: "swords_check".into(),
                };
            }
        }

        // Layer 3: LLM validator
        if let Some(ref validator) = self.llm_validator {
            let score = validator(&proposal.source_code);
            if score < 0.3 {
                return GateResult::Rejected {
                    reason: format!("llm_validator score {:.3} below threshold 0.3", score),
                    gate: "llm_validator".into(),
                };
            }
        }

        // Layer 4: Ball verifier
        if let Some(ref verifier) = self.ball_verifier {
            if !verifier(&proposal.source_code) {
                return GateResult::Rejected {
                    reason: "ball_verifier constraint not satisfied".into(),
                    gate: "ball_verifier".into(),
                };
            }
        }

        GateResult::Approved
    }

    /// Check if all four layers are active (production mode).
    pub fn is_fully_armed(&self) -> bool {
        self.shield_bus.is_some()
            && self.swords_check.is_some()
            && self.llm_validator.is_some()
            && self.ball_verifier.is_some()
    }
}

impl Default for SelfModifyGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_proposal() -> SelfModifyProposal {
        SelfModifyProposal {
            id: 1,
            target: super::super::agent::ModifyTarget::Parameter { path: "x".into() },
            source_code: "let x = 1;".into(),
            rationale: "test".into(),
            expected_impact: 0.5,
        }
    }

    #[test]
    fn test_new_guard_approves_without_layers() {
        let guard = SelfModifyGuard::new();
        assert_eq!(guard.evaluate(&test_proposal()), GateResult::Approved);
        assert!(!guard.is_fully_armed());
    }

    #[test]
    fn test_shield_bus_rejects() {
        let guard = SelfModifyGuard::new().with_shield(Box::new(|s| s != "bad_target"));
        let mut proposal = test_proposal();
        proposal.target = super::super::agent::ModifyTarget::Handler {
            name: "safe".into(),
        };
        assert_eq!(guard.evaluate(&proposal), GateResult::Approved);

        // Simulate shield blocking by using a closure that always returns false
        let guard = SelfModifyGuard::new().with_shield(Box::new(|_| false));
        assert!(
            matches!(guard.evaluate(&test_proposal()), GateResult::Rejected { gate, .. } if gate == "shield_bus")
        );
    }

    #[test]
    fn test_swords_check_rejects() {
        let guard = SelfModifyGuard::new().with_swords(Box::new(|code| !code.contains("unsafe")));
        let proposal = SelfModifyProposal {
            id: 2,
            target: super::super::agent::ModifyTarget::Primitive {
                name: "test".into(),
            },
            source_code: "unsafe { std::ptr::read(0) }".into(),
            rationale: "dangerous".into(),
            expected_impact: 0.1,
        };
        assert!(
            matches!(guard.evaluate(&proposal), GateResult::Rejected { gate, .. } if gate == "swords_check")
        );
    }

    #[test]
    fn test_llm_validator_below_threshold() {
        let guard = SelfModifyGuard::new().with_llm_validator(Box::new(|_| 0.1));
        assert!(
            matches!(guard.evaluate(&test_proposal()), GateResult::Rejected { gate, .. } if gate == "llm_validator")
        );
    }

    #[test]
    fn test_llm_validator_above_threshold() {
        let guard = SelfModifyGuard::new().with_llm_validator(Box::new(|_| 0.9));
        assert_eq!(guard.evaluate(&test_proposal()), GateResult::Approved);
    }

    #[test]
    fn test_ball_verifier_rejects() {
        let guard = SelfModifyGuard::new().with_ball_verifier(Box::new(|_| false));
        assert!(
            matches!(guard.evaluate(&test_proposal()), GateResult::Rejected { gate, .. } if gate == "ball_verifier")
        );
    }

    #[test]
    fn test_fully_armed_detection() {
        let guard = SelfModifyGuard::new()
            .with_shield(Box::new(|_| true))
            .with_swords(Box::new(|_| true))
            .with_llm_validator(Box::new(|_| 0.5))
            .with_ball_verifier(Box::new(|_| true));
        assert!(guard.is_fully_armed());
    }

    #[test]
    fn test_all_layers_pass() {
        let guard = SelfModifyGuard::new()
            .with_shield(Box::new(|_| true))
            .with_swords(Box::new(|_| true))
            .with_llm_validator(Box::new(|_| 0.5))
            .with_ball_verifier(Box::new(|_| true));
        assert_eq!(guard.evaluate(&test_proposal()), GateResult::Approved);
    }
}
