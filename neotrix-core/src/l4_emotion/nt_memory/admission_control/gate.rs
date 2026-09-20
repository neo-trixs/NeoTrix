#![forbid(unsafe_code)]

/// Decision outcome for a write admission check.
#[derive(Debug, Clone, PartialEq)]
pub enum AdmissionDecision {
    Accept,
    Reject,
    FlagForReview,
}

/// Admission gate — compares a score against a threshold and records decisions.
pub struct AdmissionGate {
    pub min_score: f32,
    pub log: Vec<String>,
}

impl AdmissionGate {
    /// Create a new gate with the given minimum score threshold.
    pub fn new(min_score: f32) -> Self {
        Self {
            min_score,
            log: Vec::new(),
        }
    }

    /// Decide whether to accept, reject, or flag for review.
    ///
    /// - `score >= min_score * 1.5` → FlagForReview (unusually high, worth inspecting)
    /// - `score >= min_score` → Accept
    /// - otherwise → Reject
    pub fn decide(&mut self, score: f32) -> AdmissionDecision {
        let decision = if score >= self.min_score * 1.5 {
            AdmissionDecision::FlagForReview
        } else if score >= self.min_score {
            AdmissionDecision::Accept
        } else {
            AdmissionDecision::Reject
        };

        let msg = format!(
            "score={:.4} threshold={:.4} → {:?}",
            score, self.min_score, decision
        );
        self.log.push(msg);

        decision
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_above_threshold() {
        let mut gate = AdmissionGate::new(0.5);
        assert_eq!(gate.decide(0.6), AdmissionDecision::Accept);
    }

    #[test]
    fn reject_below_threshold() {
        let mut gate = AdmissionGate::new(0.5);
        assert_eq!(gate.decide(0.3), AdmissionDecision::Reject);
    }

    #[test]
    fn flag_for_review_when_very_high() {
        let mut gate = AdmissionGate::new(0.5);
        // 0.5 * 1.5 = 0.75
        assert_eq!(gate.decide(0.8), AdmissionDecision::FlagForReview);
    }

    #[test]
    fn accept_at_exact_threshold() {
        let mut gate = AdmissionGate::new(0.5);
        assert_eq!(gate.decide(0.5), AdmissionDecision::Accept);
    }

    #[test]
    fn log_records_all_decisions() {
        let mut gate = AdmissionGate::new(0.5);
        gate.decide(0.6);
        gate.decide(0.3);
        gate.decide(0.9);
        assert_eq!(gate.log.len(), 3);
        assert!(gate.log[0].contains("Accept"));
        assert!(gate.log[1].contains("Reject"));
        assert!(gate.log[2].contains("FlagForReview"));
    }

    #[test]
    fn zero_threshold_accepts_everything() {
        let mut gate = AdmissionGate::new(0.0);
        assert_eq!(gate.decide(0.0), AdmissionDecision::Accept);
        assert_eq!(gate.decide(1.0), AdmissionDecision::FlagForReview);
    }

    #[test]
    fn negative_score_rejected() {
        let mut gate = AdmissionGate::new(0.5);
        assert_eq!(gate.decide(-0.1), AdmissionDecision::Reject);
    }

    #[test]
    fn exact_flag_threshold() {
        let mut gate = AdmissionGate::new(0.5);
        assert_eq!(gate.decide(0.75), AdmissionDecision::FlagForReview);
    }
}
