//! JEV Conversion — convert existing NeoTrix decision types to JEV primitives
//!
//! This module provides `From` implementations that map the 63+ ad-hoc decision
//! enums to unified JEV primitives, enabling gradual migration.

use std::collections::HashMap;
use super::primitives::*;

// ═══════════════════════════════════════════════════════════════════
// L3 Shield conversions
// ═══════════════════════════════════════════════════════════════════

/// SafetyDecision → JevDecision (Noul)
///
/// Maps: Allowed → noul=0.9, Denied → noul=0.1, RequiresApproval → noul=0.5 + needs_review
impl From<&str> for JevDecision {
    fn from(s: &str) -> Self {
        match s {
            "allow" | "allowed" | "pass" => JevDecision::Noul(NoulAnswer {
                noul: 0.9,
                needs_review: false,
                reason: Some("explicit allow".into()),
                status: DecisionStatus::Selected,
            }),
            "deny" | "denied" | "block" => JevDecision::Noul(NoulAnswer {
                noul: 0.1,
                needs_review: false,
                reason: Some("explicit deny".into()),
                status: DecisionStatus::Selected,
            }),
            "review" | "pending" | "ask" => JevDecision::Noul(NoulAnswer {
                noul: 0.5,
                needs_review: true,
                reason: Some("requires review".into()),
                status: DecisionStatus::Review,
            }),
            _ => JevDecision::Noul(NoulAnswer {
                noul: 0.5,
                needs_review: true,
                reason: Some(format!("unknown decision: {}", s)),
                status: DecisionStatus::Error,
            }),
        }
    }
}

/// GuardVerdict → JevDecision (Noul)
pub fn guard_verdict_to_jev(verdict: &str) -> JevDecision {
    match verdict {
        "allow" => JevDecision::Noul(NoulAnswer {
            noul: 0.95,
            needs_review: false,
            reason: Some("guard allowed".into()),
            status: DecisionStatus::Selected,
        }),
        "ask" => JevDecision::Noul(NoulAnswer {
            noul: 0.5,
            needs_review: true,
            reason: Some("guard asks for confirmation".into()),
            status: DecisionStatus::Review,
        }),
        "deny" => JevDecision::Noul(NoulAnswer {
            noul: 0.05,
            needs_review: false,
            reason: Some("guard denied".into()),
            status: DecisionStatus::Selected,
        }),
        _ => JevDecision::Noul(NoulAnswer {
            noul: 0.5,
            needs_review: true,
            reason: Some(format!("unknown guard verdict: {}", verdict)),
            status: DecisionStatus::Error,
        }),
    }
}

/// Safety kernel decision → JevDecision (Noul)
pub fn safety_decision_to_jev(decision: &str, reason: &str) -> JevDecision {
    match decision {
        "allowed" => JevDecision::Noul(NoulAnswer {
            noul: 0.95,
            needs_review: false,
            reason: Some(reason.to_string()),
            status: DecisionStatus::Selected,
        }),
        "denied" => JevDecision::Noul(NoulAnswer {
            noul: 0.05,
            needs_review: false,
            reason: Some(reason.to_string()),
            status: DecisionStatus::Selected,
        }),
        "requires_approval" => JevDecision::Noul(NoulAnswer {
            noul: 0.5,
            needs_review: true,
            reason: Some(reason.to_string()),
            status: DecisionStatus::Review,
        }),
        _ => JevDecision::Noul(NoulAnswer {
            noul: 0.5,
            needs_review: true,
            reason: Some(format!("unknown safety decision: {} — {}", decision, reason)),
            status: DecisionStatus::Error,
        }),
    }
}

// ═══════════════════════════════════════════════════════════════════
// L4 Memory conversions
// ═══════════════════════════════════════════════════════════════════

/// AdmissionDecision → JevDecision (Noul)
pub fn admission_decision_to_jev(decision: &str, score: f64) -> JevDecision {
    match decision {
        "accept" => JevDecision::Noul(NoulAnswer {
            noul: 0.9,
            needs_review: false,
            reason: Some(format!("admission accepted (score={:.4})", score)),
            status: DecisionStatus::Selected,
        }),
        "reject" => JevDecision::Noul(NoulAnswer {
            noul: 0.1,
            needs_review: false,
            reason: Some(format!("admission rejected (score={:.4})", score)),
            status: DecisionStatus::Selected,
        }),
        "flag_for_review" => JevDecision::Noul(NoulAnswer {
            noul: 0.5,
            needs_review: true,
            reason: Some(format!("admission flagged (score={:.4})", score)),
            status: DecisionStatus::Review,
        }),
        _ => JevDecision::Noul(NoulAnswer {
            noul: 0.5,
            needs_review: true,
            reason: Some(format!("unknown admission decision: {}", decision)),
            status: DecisionStatus::Error,
        }),
    }
}

// ═══════════════════════════════════════════════════════════════════
// L5 Cognition conversions
// ═══════════════════════════════════════════════════════════════════

/// Gate Verdict → JevDecision (Noul)
pub fn gate_verdict_to_jev(verdict: &str, context: Option<&str>) -> JevDecision {
    match verdict {
        "pass" => JevDecision::Noul(NoulAnswer {
            noul: 0.9,
            needs_review: false,
            reason: context.map(|s| s.to_string()),
            status: DecisionStatus::Selected,
        }),
        "review" => JevDecision::Noul(NoulAnswer {
            noul: 0.5,
            needs_review: true,
            reason: context.map(|s| s.to_string()),
            status: DecisionStatus::Review,
        }),
        "block" => JevDecision::Noul(NoulAnswer {
            noul: 0.1,
            needs_review: false,
            reason: context.map(|s| s.to_string()),
            status: DecisionStatus::Selected,
        }),
        _ => JevDecision::Noul(NoulAnswer {
            noul: 0.5,
            needs_review: true,
            reason: Some(format!("unknown verdict: {}", verdict)),
            status: DecisionStatus::Error,
        }),
    }
}

/// SupervisorDecision → JevDecision (Choice)
pub fn supervisor_decision_to_jev(decision: &str, reason: &str) -> JevDecision {
    let mut probs = HashMap::new();
    probs.insert(decision.to_string(), 0.9);

    let choices = vec![
        "continue".to_string(),
        "redirect".to_string(),
        "abort".to_string(),
        "record_failure".to_string(),
    ];

    // Set low probability for non-selected choices
    for c in &choices {
        if c != decision {
            probs.entry(c.clone()).or_insert(0.1 / (choices.len() - 1) as f64);
        }
    }

    JevDecision::Choice(ChoiceAnswer {
        choice: decision.to_string(),
        probabilities: probs,
        confidence: 0.8,
        margin: 0.8,
        needs_review: decision == "redirect" || decision == "abort",
        reason: Some(reason.to_string()),
        status: if decision == "continue" {
            DecisionStatus::Selected
        } else {
            DecisionStatus::Review
        },
    })
}

/// Circuit breaker verdict → JevDecision (Noul)
pub fn breaker_verdict_to_jev(verdict: &str, failure_count: u32) -> JevDecision {
    match verdict {
        "closed" => JevDecision::Noul(NoulAnswer {
            noul: 0.9,
            needs_review: false,
            reason: Some(format!("breaker closed (failures={})", failure_count)),
            status: DecisionStatus::Selected,
        }),
        "open" => JevDecision::Noul(NoulAnswer {
            noul: 0.1,
            needs_review: false,
            reason: Some(format!("breaker open (failures={})", failure_count)),
            status: DecisionStatus::Selected,
        }),
        "half_open" => JevDecision::Noul(NoulAnswer {
            noul: 0.5,
            needs_review: true,
            reason: Some(format!("breaker half-open (failures={})", failure_count)),
            status: DecisionStatus::Review,
        }),
        _ => JevDecision::Noul(NoulAnswer {
            noul: 0.5,
            needs_review: true,
            reason: Some(format!("unknown breaker verdict: {}", verdict)),
            status: DecisionStatus::Error,
        }),
    }
}

/// Disagreement gate → JevDecision (Score)
pub fn disagreement_gate_to_jev(level: &str, score: f64) -> JevDecision {
    let legend: Vec<String> = vec![
        "low_disagreement".into(),
        "moderate".into(),
        "high_disagreement".into(),
        "critical".into(),
    ];

    let score_idx = match level {
        "low" => 0.0,
        "moderate" => 1.0,
        "high" => 2.0,
        "critical" => 3.0,
        _ => 1.0,
    };

    let mut probs = HashMap::new();
    for (i, l) in legend.iter().enumerate() {
        probs.insert(l.clone(), if i as f64 == score_idx { 0.8 } else { 0.2 / 3.0 });
    }

    JevDecision::Score(ScoreAnswer {
        score: score_idx,
        probabilities: probs,
        confidence: 0.7,
        legend,
        needs_review: score_idx >= 2.0,
        reason: Some(format!("disagreement level: {} (raw_score={:.2})", level, score)),
        status: if score_idx >= 2.0 {
            DecisionStatus::Review
        } else {
            DecisionStatus::Scored
        },
    })
}

// ═══════════════════════════════════════════════════════════════════
// Opt-in migration trait (no layer inversion)
// ═══════════════════════════════════════════════════════════════════

/// Opt-in migration trait for concrete decision enums.
///
/// Implemented **by the owning module** (not here), so `nt_jev` never depends
/// on L1–L6 types and layering stays clean. This is the migration path for the
/// ~77 ad-hoc decision enums across the codebase — one `impl` at a time:
/// ```ignore
/// use crate::neotrix::nt_jev::{JevDecision, ToJev};
/// impl ToJev for SafetyDecision {
///     fn to_jev(&self) -> JevDecision {
///         // map variants → Noul/Choice/Score, then validate + audit
///         todo!()
///     }
/// }
/// ```
pub trait ToJev {
    /// Convert this decision into a unified JEV primitive.
    fn to_jev(&self) -> JevDecision;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guard_verdict_allow() {
        let d = guard_verdict_to_jev("allow");
        assert!(!d.needs_review());
        assert_eq!(d.status(), DecisionStatus::Selected);
    }

    #[test]
    fn test_guard_verdict_ask() {
        let d = guard_verdict_to_jev("ask");
        assert!(d.needs_review());
        assert_eq!(d.status(), DecisionStatus::Review);
    }

    #[test]
    fn test_safety_decision_allowed() {
        let d = safety_decision_to_jev("allowed", "policy permits");
        assert!(!d.needs_review());
    }

    #[test]
    fn test_safety_decision_requires_approval() {
        let d = safety_decision_to_jev("requires_approval", "irreversible action");
        assert!(d.needs_review());
    }

    #[test]
    fn test_gate_verdict() {
        let d = gate_verdict_to_jev("pass", Some("all checks passed"));
        assert!(!d.needs_review());
        assert_eq!(d.status(), DecisionStatus::Selected);

        let d = gate_verdict_to_jev("block", Some("schema validation failed"));
        assert!(!d.needs_review());
        assert_eq!(d.status(), DecisionStatus::Selected);
    }

    #[test]
    fn test_admission_decision() {
        let d = admission_decision_to_jev("accept", 0.75);
        assert!(!d.needs_review());

        let d = admission_decision_to_jev("flag_for_review", 1.2);
        assert!(d.needs_review());
    }

    #[test]
    fn test_supervisor_continue() {
        let d = supervisor_decision_to_jev("continue", "task progressing");
        if let JevDecision::Choice(c) = d {
            assert_eq!(c.choice, "continue");
            assert!(!c.needs_review);
        } else {
            panic!("Expected Choice");
        }
    }

    #[test]
    fn test_supervisor_abort() {
        let d = supervisor_decision_to_jev("abort", "max failures reached");
        assert!(d.needs_review());
    }

    #[test]
    fn test_breaker_closed() {
        let d = breaker_verdict_to_jev("closed", 0);
        assert!(!d.needs_review());
    }

    #[test]
    fn test_breaker_half_open() {
        let d = breaker_verdict_to_jev("half_open", 3);
        assert!(d.needs_review());
    }

    #[test]
    fn test_disagreement_low() {
        let d = disagreement_gate_to_jev("low", 0.2);
        assert!(!d.needs_review());
    }

    #[test]
    fn test_disagreement_critical() {
        let d = disagreement_gate_to_jev("critical", 0.9);
        assert!(d.needs_review());
    }
}
