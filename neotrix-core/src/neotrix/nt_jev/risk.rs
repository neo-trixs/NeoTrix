//! JEV Risk tiers — confidence thresholds are policy, not physics.
//!
//! Ported from TypeSafe's confidence-gated routing pattern: the same confidence
//! number means different things depending on action risk.
//! - Reversible / read-only: tolerate lower confidence, but still record the
//!   input, question version, and result.
//! - Recoverable external: require higher confidence plus schema, permission,
//!   and idempotency checks in code.
//! - Irreversible / sensitive: confidence is necessary at most, never
//!   sufficient — human approval (or a stricter policy gate) is mandatory.
//!
//! Thresholds below are starting points. Calibrate per team with a golden set
//! and a risk model — never copy example numbers blindly into production.

/// Action risk tier for a JEV-gated decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskTier {
    /// Reversible or read-only: routing, retrieval, internal drafts.
    Reversible,
    /// Recoverable external action: review-queued draft, idempotent write.
    Recoverable,
    /// Irreversible or sensitive: send, delete, publish, spend, safety gate.
    Irreversible,
}

impl RiskTier {
    /// Starting-point confidence threshold for this tier.
    pub fn recommended_threshold(self) -> f64 {
        match self {
            Self::Reversible => 0.5,
            Self::Recoverable => 0.7,
            Self::Irreversible => 0.85,
        }
    }

    /// Whether human approval is mandatory regardless of confidence.
    pub fn requires_human(self) -> bool {
        matches!(self, Self::Irreversible)
    }

    /// Policy outcome for one decision: automate or route to human.
    ///
    /// `confidence` is the entropy-based confidence (see
    /// `confidence_from_probs`); `needs_review` is the model's own flag.
    /// Either signal alone forces review — agreement is what a trustworthy
    /// finish looks like.
    pub fn decide(self, confidence: f64, needs_review: bool) -> RiskDecision {
        if needs_review || confidence < self.recommended_threshold() {
            return RiskDecision::HumanReview;
        }
        if self.requires_human() {
            return RiskDecision::HumanReview;
        }
        RiskDecision::Automate
    }
}

/// Policy outcome for one decision under a risk tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskDecision {
    /// Proceed automatically; record the audit trail.
    Automate,
    /// Route to the human/escalation path.
    HumanReview,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thresholds_ordered() {
        assert!(
            RiskTier::Reversible.recommended_threshold()
                < RiskTier::Recoverable.recommended_threshold()
        );
        assert!(
            RiskTier::Recoverable.recommended_threshold()
                < RiskTier::Irreversible.recommended_threshold()
        );
    }

    #[test]
    fn test_irreversible_always_human() {
        assert!(RiskTier::Irreversible.requires_human());
        assert!(!RiskTier::Reversible.requires_human());
        // Even perfect confidence is never sufficient alone.
        assert_eq!(
            RiskTier::Irreversible.decide(1.0, false),
            RiskDecision::HumanReview
        );
    }

    #[test]
    fn test_reversible_automates_when_confident() {
        assert_eq!(
            RiskTier::Reversible.decide(0.9, false),
            RiskDecision::Automate
        );
        assert_eq!(
            RiskTier::Reversible.decide(0.2, false),
            RiskDecision::HumanReview
        );
    }

    #[test]
    fn test_model_flag_forces_review() {
        assert_eq!(
            RiskTier::Reversible.decide(0.99, true),
            RiskDecision::HumanReview
        );
    }
}
