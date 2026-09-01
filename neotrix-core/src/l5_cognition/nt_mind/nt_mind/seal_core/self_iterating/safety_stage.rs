//! SafetyCheckStage — learned safety alignment for SEAL pipeline
//!
//! Replaces the hardcoded `SafetyCheckResult::Passed` stub with:
//! 1. Safety feature extraction from edit proposals
//! 2. Threat classification (destabilizing, privacy-violating, capability-impairing)
//! 3. Safety score computation with configurable thresholds
//! 4. Rewards adjusted by safety score
//!
//! This is NOT a regex scanner like SecretScanner — it evaluates edit-level
//! safety based on capability delta analysis and edit proposal semantics.

use serde::{Serialize, Deserialize};
use super::hypercore::SafetyCheckResult;
use super::pipeline::StageResult;

/// Threat categories for self-modification proposals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatCategory {
    /// Edit would destabilize core capabilities
    Destabilizing,
    /// Edit accesses/modifies sensitive data paths
    PrivacyViolation,
    /// Edit would impair existing reasoning capability
    CapabilityImpairment,
    /// Edit introduces unverified dependencies
    DependencyRisk,
    /// No threat detected
    None,
}

/// Result of a safety check on an edit proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyCheckDetail {
    pub threat: ThreatCategory,
    pub severity: f64,
    pub description: String,
}

/// Safety Check Stage — evaluates edit proposals for risks.
#[derive(Debug, Clone)]
pub struct SafetyCheckStage {
    pub safety_threshold: f64,
    pub checks_performed: u64,
    pub blocked_proposals: u64,
    pub history: Vec<SafetyCheckDetail>,
}

impl Default for SafetyCheckStage {
    fn default() -> Self {
        Self::new()
    }
}

impl SafetyCheckStage {
    pub fn new() -> Self {
        Self {
            safety_threshold: 0.5,
            checks_performed: 0,
            blocked_proposals: 0,
            history: Vec::new(),
        }
    }

    /// Evaluate the safety of an edit proposal given capability deltas.
    ///
    /// Returns (StageResult, SafetyCheckResult, adjusted_reward).
    pub fn evaluate(
        &mut self,
        deltas: &[(String, f64)],
        current_reward: f64,
    ) -> (StageResult, SafetyCheckResult, f64) {
        self.checks_performed += 1;
        let result = StageResult::new("safety_check");

        let details = self.analyze_threats(deltas);
        let max_severity = details.iter().map(|d| d.severity).fold(0.0, f64::max);
        let has_critical = details.iter().any(|d| d.severity >= self.safety_threshold);

        // Store findings
        for detail in &details {
            if detail.severity > 0.3 {
                self.history.push(detail.clone());
            }
        }

        let safety_result = if has_critical {
            self.blocked_proposals += 1;
            SafetyCheckResult::Failed {
                reason: format!("critical threat (severity={:.2})", max_severity),
            }
        } else if max_severity > 0.3 {
            SafetyCheckResult::NeedsHumanReview {
                concern: format!("moderate risk (severity={:.2})", max_severity),
            }
        } else {
            SafetyCheckResult::Passed
        };

        // Adjust reward: safety penalty reduces reward proportionally to severity
        let adjusted_reward = current_reward * (1.0 - max_severity * 0.5).max(0.0);

        (result, safety_result, adjusted_reward)
    }

    /// Analyze capability deltas for safety threats.
    fn analyze_threats(&self, deltas: &[(String, f64)]) -> Vec<SafetyCheckDetail> {
        let mut details = Vec::new();

        if deltas.is_empty() {
            return details;
        }

        // Assess destabilization: large negative deltas to core capabilities
        let core_caps = ["planning", "reasoning", "learning", "memory", "decision"];
        for (name, val) in deltas {
            if *val < -0.3 && core_caps.iter().any(|c| name.contains(c)) {
                details.push(SafetyCheckDetail {
                    threat: ThreatCategory::Destabilizing,
                    severity: val.abs().clamp(0.0, 1.0),
                    description: format!("{} would be reduced by {:.2}", name, val.abs()),
                });
            }
        }

        // Assess privacy violations: deltas to permission/sensitive paths
        let sensitive = ["secret", "key", "token", "password", "credential", "permission"];
        for (name, val) in deltas {
            if *val > 0.3 && sensitive.iter().any(|s| name.contains(s)) {
                details.push(SafetyCheckDetail {
                    threat: ThreatCategory::PrivacyViolation,
                    severity: val.clamp(0.0, 1.0),
                    description: format!("{} increase of {:.2} may expose sensitive data", name, val),
                });
            }
        }

        // Assess capability impairment
        for (name, val) in deltas {
            if *val < -0.5 {
                details.push(SafetyCheckDetail {
                    threat: ThreatCategory::CapabilityImpairment,
                    severity: val.abs().clamp(0.0, 1.0),
                    description: format!("{} severely reduced by {:.2}", name, val.abs()),
                });
            }
        }

        // Assess dependency risk
        for (name, _) in deltas {
            if name.contains("dependency") || name.contains("external") {
                details.push(SafetyCheckDetail {
                    threat: ThreatCategory::DependencyRisk,
                    severity: 0.4,
                    description: format!("external dependency change in {}", name),
                });
            }
        }

        details
    }

    /// Reset the safety check stage.
    pub fn reset(&mut self) {
        self.checks_performed = 0;
        self.blocked_proposals = 0;
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_deltas_pass() {
        let mut stage = SafetyCheckStage::new();
        let deltas = vec![("planning_ahead".into(), 0.05)];
        let (_, check, reward) = stage.evaluate(&deltas, 1.0);
        assert!(matches!(check, SafetyCheckResult::Passed));
        assert!((reward - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_destabilizing_delta_fails() {
        let mut stage = SafetyCheckStage::new();
        let deltas = vec![("reasoning_core".into(), -0.8)];
        let (_, check, _) = stage.evaluate(&deltas, 1.0);
        assert!(matches!(check, SafetyCheckResult::Failed { .. }));
    }

    #[test]
    fn test_privacy_violation_detected() {
        let stage = SafetyCheckStage::new();
        let deltas = vec![("secret_access".into(), 0.5)];
        let details = stage.analyze_threats(&deltas);
        assert!(details.iter().any(|d| d.threat == ThreatCategory::PrivacyViolation));
    }

    #[test]
    fn test_empty_deltas_always_pass() {
        let mut stage = SafetyCheckStage::new();
        let (_, check, reward) = stage.evaluate(&[], 1.0);
        assert!(matches!(check, SafetyCheckResult::Passed));
        assert!((reward - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_moderate_threat_needs_review() {
        let mut stage = SafetyCheckStage::new();
        let deltas = vec![("external_lib".into(), 0.35)];
        let (_, check, _) = stage.evaluate(&deltas, 1.0);
        assert!(matches!(check, SafetyCheckResult::NeedsHumanReview { .. }));
    }

    #[test]
    fn test_blocked_proposals_tracked() {
        let mut stage = SafetyCheckStage::new();
        assert_eq!(stage.blocked_proposals, 0);
        stage.evaluate(&[("reasoning_core".into(), -0.9)], 1.0);
        assert_eq!(stage.blocked_proposals, 1);
    }

    #[test]
    fn test_checks_performed_increments() {
        let mut stage = SafetyCheckStage::new();
        stage.evaluate(&[], 1.0);
        stage.evaluate(&[], 0.8);
        assert_eq!(stage.checks_performed, 2);
    }

    #[test]
    fn test_history_records_findings() {
        let mut stage = SafetyCheckStage::new();
        stage.evaluate(&[("reasoning_core".into(), -0.8)], 1.0);
        assert!(!stage.history.is_empty());
    }

    #[test]
    fn test_reset_clears_state() {
        let mut stage = SafetyCheckStage::new();
        stage.evaluate(&[("reasoning_core".into(), -0.8)], 1.0);
        stage.reset();
        assert_eq!(stage.checks_performed, 0);
        assert!(stage.history.is_empty());
    }
}
