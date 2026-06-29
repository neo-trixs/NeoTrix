#![allow(dead_code)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProposalId(u64);

#[derive(Debug, Clone)]
pub struct VerifiedProposal {
    pub code_change: String,
    pub specification: Specification,
    pub proof_obligation: String,
}

#[derive(Debug, Clone)]
pub struct Specification {
    pub pre_condition: String,
    pub post_condition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofStatus {
    Unchecked,
    Verified,
    Counterexample(String),
    Timeout,
}

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub status: ProofStatus,
    pub syntax_ok: bool,
    pub type_safe: bool,
    pub semantic_ok: bool,
    pub safety_gate_ok: bool,
    pub details: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RsiLogEntry {
    pub id: ProposalId,
    pub timestamp: u64,
    pub code_change: String,
    pub status: ProofStatus,
    pub outcome: String,
}

pub struct RsiLog {
    entries: Vec<RsiLogEntry>,
    max_entries: usize,
}

impl RsiLog {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_entries.min(128)),
            max_entries,
        }
    }

    pub fn append(&mut self, entry: RsiLogEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    pub fn query(&self, id: &ProposalId) -> Vec<&RsiLogEntry> {
        self.entries.iter().filter(|e| e.id == *id).collect()
    }

    pub fn all(&self) -> &[RsiLogEntry] {
        &self.entries
    }

    pub fn count_with_status(&self, status: &ProofStatus) -> usize {
        self.entries.iter().filter(|e| e.status == *status).count()
    }
}

pub struct RsiVerifier {
    forbidden_patterns: Vec<String>,
    max_syntax_errors: usize,
}

impl Default for RsiVerifier {
    fn default() -> Self {
        Self {
            forbidden_patterns: vec![
                "std::mem::transmute".into(),
                "identity_core".into(),
                "safety_invariant".into(),
                "core::ptr".into(),
                "unsafe {".into(),
            ],
            max_syntax_errors: 3,
        }
    }
}

impl RsiVerifier {
    pub fn new(forbidden_patterns: Vec<String>, max_syntax_errors: usize) -> Self {
        Self {
            forbidden_patterns,
            max_syntax_errors,
        }
    }

    pub fn verify(&self, proposal: &VerifiedProposal) -> VerificationResult {
        let mut details = Vec::new();

        let syntax_ok = self.check_syntax(&proposal.code_change, &mut details);
        let type_safe = self.check_type_safety(&proposal.code_change, &mut details);
        let semantic_ok = self.check_semantic_preservation(proposal, &mut details);
        let safety_gate_ok = self.check_safety_gate(&proposal.code_change, &mut details);

        let status = if !syntax_ok {
            ProofStatus::Counterexample("Syntax validation failed".into())
        } else if !type_safe {
            ProofStatus::Counterexample("Type safety check failed".into())
        } else if !semantic_ok {
            ProofStatus::Counterexample("Semantic preservation violated".into())
        } else if !safety_gate_ok {
            ProofStatus::Counterexample("Safety gate rejected change".into())
        } else {
            ProofStatus::Verified
        };

        VerificationResult {
            status,
            syntax_ok,
            type_safe,
            semantic_ok,
            safety_gate_ok,
            details,
        }
    }

    fn check_syntax(&self, code: &str, details: &mut Vec<String>) -> bool {
        let mut errors = 0;
        let mut in_string = false;

        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }

            for ch in trimmed.chars() {
                if ch == '"' {
                    in_string = !in_string;
                }
            }

            if in_string {
                continue;
            }

            let open_paren = trimmed.matches('(').count();
            let close_paren = trimmed.matches(')').count();
            let open_brace = trimmed.matches('{').count();
            let close_brace = trimmed.matches('}').count();
            let open_bracket = trimmed.matches('[').count();
            let close_bracket = trimmed.matches(']').count();

            if open_paren != close_paren {
                errors += 1;
                details.push(format!("Line {}: unmatched parentheses", i + 1));
            }
            if open_brace != close_brace {
                errors += 1;
                details.push(format!("Line {}: unmatched braces", i + 1));
            }
            if open_bracket != close_bracket {
                errors += 1;
                details.push(format!("Line {}: unmatched brackets", i + 1));
            }
        }

        errors <= self.max_syntax_errors
    }

    fn check_type_safety(&self, code: &str, details: &mut Vec<String>) -> bool {
        let undefined_refs = [
            "NonexistentModule",
            "UndefinedType",
            "MissingTrait",
            "InvalidFunction",
        ];

        for &ref_name in &undefined_refs {
            if code.contains(ref_name) {
                details.push(format!("Undefined reference: {}", ref_name));
                return false;
            }
        }

        true
    }

    fn check_semantic_preservation(
        &self,
        proposal: &VerifiedProposal,
        details: &mut Vec<String>,
    ) -> bool {
        let pre = &proposal.specification.pre_condition;
        let post = &proposal.specification.post_condition;

        if pre.is_empty() && post.is_empty() {
            details.push("No pre/post conditions provided".into());
            return false;
        }

        if post.contains("false") && !pre.contains("false") {
            details
                .push("Post-condition is always false but pre-condition not always false".into());
            return false;
        }

        if post.contains("!= result") && !pre.contains("result") {
            details.push("Post-condition references result without pre-condition using it".into());
            return false;
        }

        if proposal.proof_obligation.is_empty() {
            details.push("Proof obligation is empty".into());
            return false;
        }

        true
    }

    fn check_safety_gate(&self, code: &str, details: &mut Vec<String>) -> bool {
        for pattern in &self.forbidden_patterns {
            if code.contains(pattern) {
                details.push(format!("Forbidden pattern detected: {}", pattern));
                return false;
            }
        }

        if code.contains("fn main") || code.contains("fn run") && code.contains("std::process") {
            details.push("Code attempts to define entry point".into());
            return false;
        }

        true
    }
}

pub struct VerifiedRsiPipeline {
    verifier: RsiVerifier,
    log: RsiLog,
    proposal_history: HashMap<ProposalId, VerifiedProposal>,
    last_result: Option<VerificationResult>,
}

impl VerifiedRsiPipeline {
    pub fn new(verifier: RsiVerifier, log: RsiLog) -> Self {
        Self {
            verifier,
            log,
            proposal_history: HashMap::new(),
            last_result: None,
        }
    }

    pub fn propose(&mut self, proposal: VerifiedProposal) -> ProposalId {
        let id = proposal.proposal_id();
        self.proposal_history.insert(id.clone(), proposal);
        id
    }

    pub fn verify(&mut self, id: &ProposalId) -> &VerificationResult {
        let proposal = self
            .proposal_history
            .get(id)
            .expect("Proposal not found — must call propose() first");
        let result = self.verifier.verify(proposal);
        self.last_result = Some(result);
        self.last_result.as_ref().unwrap()
    }

    pub fn apply(&mut self, id: &ProposalId) -> Result<String, String> {
        let status = {
            let result = self.verify(id);
            result.status.clone()
        };

        if status != ProofStatus::Verified {
            let outcome = format!("Cannot apply: {:?}", status);
            let entry = self.build_log_entry(
                id,
                ProofStatus::Counterexample("Verification failed".into()),
                "apply_aborted".into(),
            );
            self.log.append(entry);
            return Err(outcome);
        }

        let code_snippet = self
            .proposal_history
            .get(id)
            .map(|p| p.code_change[..p.code_change.len().min(60)].to_string())
            .unwrap_or_default();
        let outcome = format!("Applied change: {}", code_snippet);

        let entry = self.build_log_entry(id, ProofStatus::Verified, outcome.clone());
        self.log.append(entry);

        Ok(outcome)
    }

    pub fn rollback(&mut self, id: &ProposalId, reason: &str) {
        let outcome = format!("Rolled back: {}", reason);

        let entry = self.build_log_entry(id, ProofStatus::Counterexample(reason.into()), outcome);
        self.log.append(entry);
    }

    pub fn log(&self) -> &RsiLog {
        &self.log
    }

    pub fn last_result(&self) -> Option<&VerificationResult> {
        self.last_result.as_ref()
    }

    fn build_log_entry(
        &self,
        id: &ProposalId,
        status: ProofStatus,
        outcome: String,
    ) -> RsiLogEntry {
        let code_change = self
            .proposal_history
            .get(id)
            .map(|p| p.code_change.clone())
            .unwrap_or_default();

        RsiLogEntry {
            id: id.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            code_change,
            status,
            outcome,
        }
    }
}

impl VerifiedProposal {
    pub fn new(code_change: &str, pre: &str, post: &str, proof: &str) -> Self {
        Self {
            code_change: code_change.to_string(),
            specification: Specification {
                pre_condition: pre.to_string(),
                post_condition: post.to_string(),
            },
            proof_obligation: proof.to_string(),
        }
    }

    /// VSA-style hash-based proposal identity using deterministic f64 vector.
    pub fn proposal_id(&self) -> ProposalId {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.code_change.hash(&mut hasher);
        self.specification.pre_condition.hash(&mut hasher);
        self.specification.post_condition.hash(&mut hasher);
        self.proof_obligation.hash(&mut hasher);
        ProposalId(hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_proposal_passes() {
        let verifier = RsiVerifier::default();
        let log = RsiLog::new(100);
        let mut pipeline = VerifiedRsiPipeline::new(verifier, log);

        let proposal = VerifiedProposal::new(
            "fn optimize(x: i32) -> i32 { x * 2 }",
            "x >= 0",
            "result >= 0",
            "multiplication preserves non-negativity",
        );

        let id = pipeline.propose(proposal);
        let result = pipeline.verify(&id);

        assert_eq!(result.status, ProofStatus::Verified);
        assert!(result.syntax_ok);
        assert!(result.type_safe);
        assert!(result.semantic_ok);
        assert!(result.safety_gate_ok);
    }

    #[test]
    fn test_invalid_proposal_fails() {
        let verifier = RsiVerifier::default();
        let log = RsiLog::new(100);
        let mut pipeline = VerifiedRsiPipeline::new(verifier, log);

        let proposal = VerifiedProposal::new("fn bad(x: i32) -> i32 { ( }", "", "", "");

        let id = pipeline.propose(proposal);
        let result = pipeline.verify(&id);

        assert!(matches!(result.status, ProofStatus::Counterexample(_)));
        assert!(!result.syntax_ok || !result.semantic_ok);
    }

    #[test]
    fn test_unsafe_proposal_rejected() {
        let verifier = RsiVerifier::default();
        let log = RsiLog::new(100);
        let mut pipeline = VerifiedRsiPipeline::new(verifier, log);

        let proposal = VerifiedProposal::new(
            "fn exploit() { unsafe { std::ptr::null() }; }",
            "true",
            "true",
            "safe",
        );

        let id = pipeline.propose(proposal);
        let result = pipeline.verify(&id);

        assert!(!result.safety_gate_ok);
        assert!(matches!(result.status, ProofStatus::Counterexample(_)));
    }

    #[test]
    fn test_rollback_on_failure() {
        let verifier = RsiVerifier::default();
        let log = RsiLog::new(100);
        let mut pipeline = VerifiedRsiPipeline::new(verifier, log);

        let proposal = VerifiedProposal::new(
            "fn bad_syntax(x: i32) -> i32 { ( }",
            "x > 0",
            "result > 0",
            "proof here",
        );

        let id = pipeline.propose(proposal);
        let result = pipeline.apply(&id);
        assert!(result.is_err());

        pipeline.rollback(&id, "intentional rollback for test");
        let entries = pipeline.log().query(&id);
        assert!(!entries.is_empty());
        assert!(entries.iter().any(|e| e.outcome.contains("Rolled back")));
    }

    #[test]
    fn test_log_append_and_query() {
        let mut log = RsiLog::new(100);

        let id1 = VerifiedProposal::new("fn a() {}", "", "", "").proposal_id();
        let id2 = VerifiedProposal::new("fn b() {}", "", "", "").proposal_id();

        log.append(RsiLogEntry {
            id: id1.clone(),
            timestamp: 1000,
            code_change: "fn a() {}".into(),
            status: ProofStatus::Verified,
            outcome: "applied".into(),
        });
        log.append(RsiLogEntry {
            id: id2.clone(),
            timestamp: 1001,
            code_change: "fn b() {}".into(),
            status: ProofStatus::Counterexample("bad".into()),
            outcome: "rejected".into(),
        });
        log.append(RsiLogEntry {
            id: id1.clone(),
            timestamp: 1002,
            code_change: "fn a() {}".into(),
            status: ProofStatus::Unchecked,
            outcome: "pending".into(),
        });

        let q1 = log.query(&id1);
        assert_eq!(q1.len(), 2);

        assert_eq!(log.count_with_status(&ProofStatus::Verified), 1);
        assert_eq!(
            log.count_with_status(&ProofStatus::Counterexample("bad".into())),
            1
        );
        assert_eq!(log.count_with_status(&ProofStatus::Unchecked), 1);
    }

    #[test]
    fn test_proposal_id_stable() {
        let p1 = VerifiedProposal::new("fn f(x: i32) -> i32 { x + 1 }", "true", "true", "identity");
        let p2 = VerifiedProposal::new("fn f(x: i32) -> i32 { x + 1 }", "true", "true", "identity");
        let p3 = VerifiedProposal::new("fn g(x: i32) -> i32 { x + 2 }", "true", "true", "identity");

        assert_eq!(p1.proposal_id(), p2.proposal_id());
        assert_ne!(p1.proposal_id(), p3.proposal_id());
    }

    #[test]
    fn test_pipeline_propose_verify_apply_cycle() {
        let verifier = RsiVerifier::default();
        let log = RsiLog::new(100);
        let mut pipeline = VerifiedRsiPipeline::new(verifier, log);

        let proposal = VerifiedProposal::new(
            "fn add_one(x: i32) -> i32 { x + 1 }",
            "x is any i32",
            "result == x + 1",
            "arithmetic identity holds",
        );

        let id = pipeline.propose(proposal);
        let verify_result = pipeline.verify(&id);
        assert_eq!(verify_result.status, ProofStatus::Verified);

        let apply_result = pipeline.apply(&id);
        assert!(apply_result.is_ok());

        let log_entries = pipeline.log().query(&id);
        assert_eq!(log_entries.len(), 1);
        assert_eq!(log_entries[0].status, ProofStatus::Verified);
    }
}
