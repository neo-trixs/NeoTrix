use super::guard::{GateResult, SelfModifyGuard};
use super::sandbox::SandboxValidator;
use std::collections::VecDeque;

/// What aspect of the agent to modify.
#[derive(Debug, Clone, PartialEq)]
pub enum ModifyTarget {
    Handler { name: String },
    Parameter { path: String },
    Primitive { name: String },
    PipelineStage { phase: String },
    SafetyGate { gate: String },
}

/// A full self-modification proposal with source code and rationale.
#[derive(Debug, Clone)]
pub struct SelfModifyProposal {
    pub id: u64,
    pub target: ModifyTarget,
    pub source_code: String,
    pub rationale: String,
    pub expected_impact: f64,
}

/// Safety level for self-modification operations.
/// Determines which safety gates are mandatory.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelfModifySafety {
    /// Only parameter tweaks — lowest risk
    ParamOnly,
    /// Allow handler rewriting — medium risk
    HandlerRewrite,
    /// Allow full self-modify including pipeline stages and safety gates — highest risk
    FullSelfModify,
}

/// The core self-modification agent.
///
/// Owns a queue of pending proposals, a safety guard stack for evaluation,
/// and a sandbox validator for compile-time verification.
pub struct SelfModifyAgent {
    pub proposals: VecDeque<SelfModifyProposal>,
    pub max_proposals: usize,
    pub safety_level: SelfModifySafety,
    pub guard: Option<SelfModifyGuard>,
    pub sandbox: Option<SandboxValidator>,
    next_id: u64,
}

impl SelfModifyAgent {
    pub fn new() -> Self {
        Self {
            proposals: VecDeque::with_capacity(50),
            max_proposals: 50,
            safety_level: SelfModifySafety::HandlerRewrite,
            guard: None,
            sandbox: None,
            next_id: 1,
        }
    }

    pub fn with_safety(mut self, level: SelfModifySafety) -> Self {
        self.safety_level = level;
        self
    }

    pub fn with_guard(mut self, guard: SelfModifyGuard) -> Self {
        self.guard = Some(guard);
        self
    }

    pub fn with_sandbox(mut self, sandbox: SandboxValidator) -> Self {
        self.sandbox = Some(sandbox);
        self
    }

    /// Enqueue a self-modification proposal with bounded history.
    pub fn enqueue(
        &mut self,
        target: ModifyTarget,
        source_code: String,
        rationale: String,
        expected_impact: f64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.proposals.push_back(SelfModifyProposal {
            id,
            target,
            source_code,
            rationale,
            expected_impact,
        });
        // Bounded: drain oldest 20% when over capacity
        if self.proposals.len() > self.max_proposals {
            let drain_count = self.max_proposals / 5;
            for _ in 0..drain_count {
                self.proposals.pop_front();
            }
        }
        id
    }

    /// Evaluate a proposal through all active safety gates.
    pub fn evaluate(&self, proposal: &SelfModifyProposal) -> GateResult {
        match self.guard {
            Some(ref guard) => guard.evaluate(proposal),
            None => GateResult::Approved,
        }
    }

    /// Attempt sandbox compilation of the proposal source code.
    pub fn validate_in_sandbox(
        &self,
        proposal: &SelfModifyProposal,
    ) -> Option<super::ValidationResult> {
        match self.sandbox {
            Some(ref sb) => {
                let test_code = format!(
                    "fn test_{}() {{ /* auto-generated validation for {}",
                    proposal.id, proposal.rationale
                );
                Some(sb.validate_source(&proposal.source_code, &test_code))
            }
            None => None,
        }
    }

    /// Check if the target is within the current safety level.
    pub fn is_target_allowed(&self, target: &ModifyTarget) -> bool {
        match self.safety_level {
            SelfModifySafety::ParamOnly => matches!(target, ModifyTarget::Parameter { .. }),
            SelfModifySafety::HandlerRewrite => matches!(
                target,
                ModifyTarget::Parameter { .. }
                    | ModifyTarget::Handler { .. }
                    | ModifyTarget::Primitive { .. }
            ),
            SelfModifySafety::FullSelfModify => true,
        }
    }

    /// Proposal count for status reports.
    pub fn pending_count(&self) -> usize {
        self.proposals.len()
    }

    /// Human-readable status.
    pub fn status(&self) -> String {
        format!(
            "SelfModifyAgent: {} pending, safety={:?}, guard={}, sandbox={}",
            self.proposals.len(),
            self.safety_level,
            if self.guard.is_some() {
                "active"
            } else {
                "none"
            },
            if self.sandbox.is_some() {
                "active"
            } else {
                "none"
            },
        )
    }
}

impl Default for SelfModifyAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_agent_defaults() {
        let agent = SelfModifyAgent::new();
        assert_eq!(agent.pending_count(), 0);
        assert_eq!(agent.max_proposals, 50);
        assert_eq!(agent.safety_level, SelfModifySafety::HandlerRewrite);
        assert!(agent.guard.is_none());
        assert!(agent.sandbox.is_none());
    }

    #[test]
    fn test_enqueue_proposal() {
        let mut agent = SelfModifyAgent::new();
        let id = agent.enqueue(
            ModifyTarget::Parameter {
                path: "thinking_budget".into(),
            },
            "let x = 1;".into(),
            "increase thinking budget".into(),
            0.3,
        );
        assert_eq!(id, 1);
        assert_eq!(agent.pending_count(), 1);
        let p = &agent.proposals[0];
        assert_eq!(p.id, 1);
        assert!(p.rationale.contains("thinking"));
    }

    #[test]
    fn test_enqueue_bounded() {
        let mut agent = SelfModifyAgent::new();
        agent.max_proposals = 10;
        for i in 0..20 {
            agent.enqueue(
                ModifyTarget::Parameter {
                    path: format!("param_{}", i),
                },
                "code".into(),
                format!("test {}", i),
                0.5,
            );
        }
        assert!(agent.proposals.len() <= 10);
    }

    #[test]
    fn test_is_target_allowed_param_only() {
        let agent = SelfModifyAgent::new().with_safety(SelfModifySafety::ParamOnly);
        assert!(agent.is_target_allowed(&ModifyTarget::Parameter { path: "x".into() }));
        assert!(!agent.is_target_allowed(&ModifyTarget::Handler { name: "h".into() }));
        assert!(!agent.is_target_allowed(&ModifyTarget::PipelineStage { phase: "p".into() }));
    }

    #[test]
    fn test_is_target_allowed_handler_rewrite() {
        let agent = SelfModifyAgent::new().with_safety(SelfModifySafety::HandlerRewrite);
        assert!(agent.is_target_allowed(&ModifyTarget::Parameter { path: "x".into() }));
        assert!(agent.is_target_allowed(&ModifyTarget::Handler { name: "h".into() }));
        assert!(!agent.is_target_allowed(&ModifyTarget::SafetyGate { gate: "g".into() }));
    }

    #[test]
    fn test_is_target_allowed_full() {
        let agent = SelfModifyAgent::new().with_safety(SelfModifySafety::FullSelfModify);
        assert!(agent.is_target_allowed(&ModifyTarget::SafetyGate { gate: "g".into() }));
        assert!(agent.is_target_allowed(&ModifyTarget::Primitive { name: "p".into() }));
    }

    #[test]
    fn test_evaluate_no_guard() {
        let agent = SelfModifyAgent::new();
        let p = SelfModifyProposal {
            id: 1,
            target: ModifyTarget::Parameter { path: "x".into() },
            source_code: "let x = 1;".into(),
            rationale: "test".into(),
            expected_impact: 0.5,
        };
        assert_eq!(agent.evaluate(&p), GateResult::Approved);
    }

    #[test]
    fn test_status_format() {
        let agent = SelfModifyAgent::new();
        let s = agent.status();
        assert!(s.contains("0 pending"));
        assert!(s.contains("HandlerRewrite"));
    }

    #[test]
    fn test_with_sandbox() {
        let sb = SandboxValidator::new().with_dry_run(true);
        let agent = SelfModifyAgent::new().with_sandbox(sb);
        assert!(agent.sandbox.is_some());
    }
}
