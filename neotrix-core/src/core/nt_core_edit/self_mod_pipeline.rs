// REVIVED Evo 3 — dead_code removed

use crate::core::nt_core_consciousness::proof_search::{ModificationProposal, SafetyLevel};
use crate::core::nt_core_edit::pco::PcoChain;

/// Phase in the self-modification pipeline
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PipelinePhase {
    Propose,
    Sandbox,
    Simulate,
    Approve,
    HotReload,
}

/// Result of a pipeline stage
#[derive(Debug, Clone)]
pub struct StageResult {
    pub phase: PipelinePhase,
    pub passed: bool,
    pub details: String,
    pub safety_level: SafetyLevel,
}

/// Full pipeline result
#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub results: Vec<StageResult>,
    pub all_passed: bool,
    pub proposal_id: u64,
    pub final_safety: SafetyLevel,
}

/// Self-modification safety pipeline
pub struct SelfModPipeline {
    pub pco_chain: PcoChain,
    pub max_sandbox_ticks: u64,
    pub require_will_approval: bool,
    pub hot_reload_enabled: bool,
    proposal_counter: u64,
}

impl SelfModPipeline {
    pub fn new(hmac_key: [u8; 32]) -> Self {
        SelfModPipeline {
            pco_chain: PcoChain::new(hmac_key),
            max_sandbox_ticks: 100,
            require_will_approval: true,
            hot_reload_enabled: true,
            proposal_counter: 0,
        }
    }

    pub fn run_pipeline(&mut self, proposal: &ModificationProposal) -> PipelineResult {
        let mut results = Vec::new();
        let id = self.proposal_counter;
        self.proposal_counter += 1;

        let r1 = self.stage_propose(proposal);
        results.push(r1.clone());
        if !r1.passed {
            return PipelineResult {
                results,
                all_passed: false,
                proposal_id: id,
                final_safety: SafetyLevel::Unsafe,
            };
        }

        let r2 = self.stage_sandbox(proposal);
        results.push(r2.clone());
        if !r2.passed {
            return PipelineResult {
                results,
                all_passed: false,
                proposal_id: id,
                final_safety: SafetyLevel::Unsafe,
            };
        }

        let r3 = self.stage_simulate(proposal);
        results.push(r3.clone());
        if !r3.passed {
            return PipelineResult {
                results,
                all_passed: false,
                proposal_id: id,
                final_safety: SafetyLevel::Unsafe,
            };
        }

        let r4 = self.stage_approve(proposal);
        results.push(r4.clone());
        if !r4.passed {
            return PipelineResult {
                results,
                all_passed: false,
                proposal_id: id,
                final_safety: SafetyLevel::Unsafe,
            };
        }

        let r5 = self.stage_hot_reload(proposal);
        results.push(r5.clone());

        let safety = if r5.passed {
            SafetyLevel::Safe
        } else {
            SafetyLevel::Questionable
        };
        PipelineResult {
            results,
            all_passed: r5.passed,
            proposal_id: id,
            final_safety: safety,
        }
    }

    fn stage_propose(&self, proposal: &ModificationProposal) -> StageResult {
        let preconditions_met = proposal.preconditions.iter().all(|p| !p.is_empty());
        StageResult {
            phase: PipelinePhase::Propose,
            passed: preconditions_met,
            details: format!(
                "proposal {}: preconditions {}",
                proposal.id,
                if preconditions_met { "met" } else { "missing" }
            ),
            safety_level: if preconditions_met {
                SafetyLevel::Safe
            } else {
                SafetyLevel::Unsafe
            },
        }
    }

    fn stage_sandbox(&self, proposal: &ModificationProposal) -> StageResult {
        let has_target = !proposal.target.is_empty();
        let has_change = !proposal.change_description.is_empty();
        let ok = has_target && has_change;
        StageResult {
            phase: PipelinePhase::Sandbox,
            passed: ok,
            details: format!("sandbox: target={} change={}", has_target, has_change),
            safety_level: if ok {
                SafetyLevel::Safe
            } else {
                SafetyLevel::Unsafe
            },
        }
    }

    fn stage_simulate(&self, proposal: &ModificationProposal) -> StageResult {
        let expected_impact_ok = proposal.expected_impact >= 0.0 && proposal.expected_impact <= 1.0;
        StageResult {
            phase: PipelinePhase::Simulate,
            passed: expected_impact_ok,
            details: format!(
                "simulate: impact={:.2} range_ok={}",
                proposal.expected_impact, expected_impact_ok
            ),
            safety_level: if expected_impact_ok {
                SafetyLevel::Safe
            } else {
                SafetyLevel::Questionable
            },
        }
    }

    fn stage_approve(&self, proposal: &ModificationProposal) -> StageResult {
        let approved = !self.require_will_approval || proposal.expected_impact < 0.8;
        StageResult {
            phase: PipelinePhase::Approve,
            passed: approved,
            details: format!(
                "approve: require_will={} decision={}",
                self.require_will_approval,
                if approved { "approved" } else { "rejected" }
            ),
            safety_level: if approved {
                SafetyLevel::Safe
            } else {
                SafetyLevel::Unsafe
            },
        }
    }

    fn stage_hot_reload(&mut self, proposal: &ModificationProposal) -> StageResult {
        if !self.hot_reload_enabled {
            return StageResult {
                phase: PipelinePhase::HotReload,
                passed: false,
                details: "hot reload disabled".into(),
                safety_level: SafetyLevel::Safe,
            };
        }
        let proof_data = format!(
            "self-mod:{}:impact={}",
            proposal.id, proposal.expected_impact
        );
        self.pco_chain
            .commit(proposal.change_vector.as_slice(), proof_data.as_bytes());
        StageResult {
            phase: PipelinePhase::HotReload,
            passed: true,
            details: format!("hot-reload complete, pco_len={}", self.pco_chain.len()),
            safety_level: SafetyLevel::Safe,
        }
    }

    pub fn pipeline_report(&self) -> String {
        format!(
            "SelfModPipeline: {} proposals, pco_chain_len={}, hot_reload={}, will_approval={}",
            self.proposal_counter,
            self.pco_chain.len(),
            self.hot_reload_enabled,
            self.require_will_approval,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_proposal(id: u64) -> ModificationProposal {
        ModificationProposal {
            id,
            target: "test_module".into(),
            change_description: "add field".into(),
            change_vector: vec![1, 0, 1],
            expected_impact: 0.3,
            preconditions: vec!["compiles".into()],
        }
    }

    fn make_bad_proposal(id: u64) -> ModificationProposal {
        ModificationProposal {
            id,
            target: "".into(),
            change_description: "".into(),
            change_vector: vec![],
            expected_impact: -0.5,
            preconditions: vec![],
        }
    }

    #[test]
    fn test_pipeline_good_proposal_passes() {
        let mut p = SelfModPipeline::new([42u8; 32]);
        let proposal = make_proposal(1);
        let result = p.run_pipeline(&proposal);
        assert!(result.all_passed);
        assert_eq!(result.results.len(), 5);
    }

    #[test]
    fn test_pipeline_bad_proposal_fails() {
        let mut p = SelfModPipeline::new([42u8; 32]);
        let proposal = make_bad_proposal(1);
        let result = p.run_pipeline(&proposal);
        assert!(!result.all_passed);
    }

    #[test]
    fn test_pipeline_early_exit_on_sandbox_fail() {
        let mut p = SelfModPipeline::new([42u8; 32]);
        let proposal = ModificationProposal {
            id: 1,
            target: "".into(),
            change_description: "".into(),
            change_vector: vec![],
            expected_impact: 0.3,
            preconditions: vec!["ok".into()],
        };
        let result = p.run_pipeline(&proposal);
        assert!(!result.all_passed);
        assert_eq!(result.results.len(), 2);
    }

    #[test]
    fn test_pipeline_early_exit_on_precondition_fail() {
        let mut p = SelfModPipeline::new([42u8; 32]);
        let proposal = ModificationProposal {
            id: 1,
            target: "t".into(),
            change_description: "c".into(),
            change_vector: vec![],
            expected_impact: 0.3,
            preconditions: vec!["".into()],
        };
        let result = p.run_pipeline(&proposal);
        assert!(!result.all_passed);
        assert_eq!(result.results.len(), 1);
    }

    #[test]
    fn test_hot_reload_disabled_fails() {
        let mut p = SelfModPipeline::new([42u8; 32]);
        p.hot_reload_enabled = false;
        let proposal = make_proposal(1);
        let result = p.run_pipeline(&proposal);
        assert!(!result.all_passed);
    }

    #[test]
    fn test_pipeline_report() {
        let p = SelfModPipeline::new([42u8; 32]);
        let report = p.pipeline_report();
        assert!(report.contains("SelfModPipeline"));
    }

    #[test]
    fn test_pco_chain_grows_on_success() {
        let mut p = SelfModPipeline::new([99u8; 32]);
        assert_eq!(p.pco_chain.len(), 0);
        p.run_pipeline(&make_proposal(1));
        assert_eq!(p.pco_chain.len(), 1);
    }

    #[test]
    fn test_stage_approve_rejects_high_impact() {
        let p = SelfModPipeline::new([42u8; 32]);
        let proposal = ModificationProposal {
            id: 1,
            target: "t".into(),
            change_description: "c".into(),
            change_vector: vec![],
            expected_impact: 0.95,
            preconditions: vec!["ok".into()],
        };
        let r = p.stage_approve(&proposal);
        assert!(!r.passed);
    }

    #[test]
    fn test_multiple_proposals_increment_counter() {
        let mut p = SelfModPipeline::new([42u8; 32]);
        p.run_pipeline(&make_proposal(0));
        p.run_pipeline(&make_proposal(1));
        assert_eq!(p.proposal_counter, 2);
        assert_eq!(p.pco_chain.len(), 2);
    }
}
