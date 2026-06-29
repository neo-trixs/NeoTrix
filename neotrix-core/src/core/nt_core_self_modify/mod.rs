#![forbid(unsafe_code)]

pub mod agent;
pub mod guard;
pub mod sandbox;

pub use agent::{ModifyTarget, SelfModifyAgent, SelfModifyProposal, SelfModifySafety};
pub use guard::{GateResult, SelfModifyGuard};
pub use sandbox::{SandboxValidator, ValidationResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_modify_agent_creation() {
        let agent = SelfModifyAgent::new();
        assert_eq!(agent.safety_level, SelfModifySafety::HandlerRewrite);
        assert!(agent.proposals.is_empty());
        assert!(agent.guard.is_none());
        assert!(agent.sandbox.is_none());
    }

    #[test]
    fn test_enqueue_proposal_increments_id() {
        let mut agent = SelfModifyAgent::new();
        let id1 = agent.enqueue(
            ModifyTarget::Parameter { path: "x".into() },
            "let x = 1;".into(),
            "test".into(),
            0.5,
        );
        let id2 = agent.enqueue(
            ModifyTarget::Handler { name: "foo".into() },
            "fn foo() {}".into(),
            "add handler".into(),
            0.7,
        );
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(agent.proposals.len(), 2);
    }

    #[test]
    fn test_gate_result_variants() {
        match GateResult::Approved {
            GateResult::Approved => (),
            _ => panic!("expected Approved"),
        }
        match &(GateResult::Rejected {
            reason: String::new(),
            gate: String::new(),
        }) {
            GateResult::Rejected { reason, gate } => {
                assert!(gate.is_empty() || !gate.is_empty());
            }
            _ => panic!("expected Rejected"),
        }
    }

    #[test]
    fn test_sandbox_validator_dry_run() {
        let sv = SandboxValidator::new().with_dry_run(true);
        assert!(sv.dry_run);
        let result = sv.validate_source("fn main() {}", "assert!(true)");
        assert!(result.compiles);
        assert!(result.tests_pass);
    }

    #[test]
    fn test_modify_target_debug_clone() {
        let t1 = ModifyTarget::Parameter {
            path: "a.b.c".into(),
        };
        let t2 = t1.clone();
        assert_eq!(format!("{:?}", t2), "Parameter { path: \"a.b.c\" }");
    }
}
