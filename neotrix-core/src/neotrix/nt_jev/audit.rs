//! JEV Audit — replayable decision records.
//!
//! Every automated decision must answer "why did we automate this case?"
//! (TypeSafe confidence-gated routing). Confidence alone is never proof, so
//! the audit trail stores the state hash, question version, policy version,
//! and model version alongside the outcome. If all you retain is `0.82`, you
//! cannot tell high-relevance/low-risk from low-relevance promoted by
//! weighting — preserve the raw record.
//!
//! Model versions must be pinned (e.g. `jev-1.13`): a `latest` alias can move
//! and silently invalidate calibrated thresholds.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use super::primitives::{ExitCode, JevResultSet};

/// Question-schema version for `nt_jev` gates/presets.
pub const JEV_QUESTION_VERSION: &str = "nt_jev/v1";
/// Policy/threshold version — bump whenever thresholds or weights change.
pub const JEV_POLICY_VERSION: &str = "nt_jev/policy/v1";

/// Hash a state snapshot for audit linkage (std `DefaultHasher`, u64).
pub fn state_hash_of(state: &str) -> u64 {
    let mut h = DefaultHasher::new();
    state.hash(&mut h);
    h.finish()
}

/// Current Unix timestamp in seconds (audit clock).
pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Replayable audit record for one JEV evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JevAudit {
    /// Hash of the exact state snapshot evaluated.
    pub state_hash: u64,
    /// Question schema version.
    pub question_version: String,
    /// Policy/threshold version.
    pub policy_version: String,
    /// Pinned model version (`jev-1.13`, `modernbert-large-local`, `simulation`).
    pub model_version: String,
    /// Question IDs evaluated, sorted.
    pub question_ids: Vec<String>,
    /// Overall exit code derived from the decisions.
    pub exit_code: ExitCode,
    /// Whether any answer needs human review.
    pub needs_review: bool,
    /// Unix timestamp (seconds) of evaluation.
    pub evaluated_at_secs: u64,
}

impl JevAudit {
    /// Build an audit record from an evaluation.
    pub fn new(
        state: &str,
        decisions: &JevResultSet,
        model_version: impl Into<String>,
    ) -> Self {
        let mut question_ids: Vec<String> = decisions.keys().cloned().collect();
        question_ids.sort();
        let needs_review = decisions.values().any(|d| d.needs_review());
        Self {
            state_hash: state_hash_of(state),
            question_version: JEV_QUESTION_VERSION.to_string(),
            policy_version: JEV_POLICY_VERSION.to_string(),
            model_version: model_version.into(),
            question_ids,
            exit_code: ExitCode::from_decisions(decisions),
            needs_review,
            evaluated_at_secs: now_secs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_jev::primitives::{DecisionStatus, JevDecision, NoulAnswer};

    fn decisions(review: bool) -> JevResultSet {
        let mut m = JevResultSet::new();
        m.insert(
            "q1".to_string(),
            JevDecision::Noul(NoulAnswer {
                noul: if review { 0.5 } else { 0.9 },
                needs_review: review,
                reason: None,
                status: if review {
                    DecisionStatus::Review
                } else {
                    DecisionStatus::Selected
                },
            }),
        );
        m
    }

    #[test]
    fn test_state_hash_stable() {
        assert_eq!(state_hash_of("abc"), state_hash_of("abc"));
        assert_ne!(state_hash_of("abc"), state_hash_of("abd"));
    }

    #[test]
    fn test_audit_clean() {
        let a = JevAudit::new("state", &decisions(false), "jev-1.13");
        assert_eq!(a.exit_code, ExitCode::Success);
        assert!(!a.needs_review);
        assert_eq!(a.question_ids, vec!["q1".to_string()]);
        assert_eq!(a.model_version, "jev-1.13");
    }

    #[test]
    fn test_audit_review() {
        let a = JevAudit::new("state", &decisions(true), "simulation");
        assert_eq!(a.exit_code, ExitCode::NeedsReview);
        assert!(a.needs_review);
    }
}
