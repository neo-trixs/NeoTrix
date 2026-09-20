//! Gate judgment — claim verification with policy enforcement.

use super::verdict::{GateResult, JudgmentOutput};

/// Policy gate that wraps any judgment primitive
pub struct PolicyGate {
    /// Minimum confidence to pass
    pub threshold: f64,
    /// Whether to require human review for low confidence
    pub require_review: bool,
}

impl PolicyGate {
    pub fn new(threshold: f64, require_review: bool) -> Self {
        Self {
            threshold,
            require_review,
        }
    }

    pub fn evaluate(&self, output: &JudgmentOutput) -> GateResult {
        if output.confidence >= self.threshold {
            GateResult::Pass
        } else if self.require_review && output.confidence >= self.threshold * 0.5 {
            GateResult::Review
        } else {
            GateResult::Block
        }
    }
}
