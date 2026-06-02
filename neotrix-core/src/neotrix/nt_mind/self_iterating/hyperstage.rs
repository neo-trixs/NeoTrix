use crate::neotrix::nt_core_error::NeoTrixError;
use super::{BrainStage, StageDecision, SelfIteratingBrain};
use super::HyperMetaAgent;
use super::HyperAgentArchive;
use super::DGMMetaAgent;
use super::GenerativeReplay;
use super::SelfReferentialCheck;
use super::LatentEdit;
use super::SelfModificationProposal;
use super::SafetyCheckResult;


/// A BrainPipeline stage that uses the DGMMetaAgent for generative evolution.
pub struct DGMMetaEvolveStage {
    pub dgm_agent: DGMMetaAgent,
    pub archive: HyperAgentArchive,
    pub replay: GenerativeReplay,
    pub self_ref_check: SelfReferentialCheck,
}

impl DGMMetaEvolveStage {
    pub fn new(
        dgm_agent: DGMMetaAgent,
        archive: HyperAgentArchive,
        replay: GenerativeReplay,
        self_ref_check: SelfReferentialCheck,
    ) -> Self {
        Self { dgm_agent, archive, replay, self_ref_check }
    }

    pub fn evolve(&self) -> (LatentEdit, SelfModificationProposal) {
        let edit = self.dgm_agent.generate_edit(&self.archive);
        let proposal = self.dgm_agent.proposal_from_edit(&edit, &self.archive);
        (edit, proposal)
    }
}

impl BrainStage for DGMMetaEvolveStage {
    fn name(&self) -> &str {
        "dgm_meta_evolve"
    }

    fn process(
        &self,
        _brain: &mut SelfIteratingBrain,
    ) -> Result<StageDecision, NeoTrixError> {
        let (_edit, proposal) = self.evolve();
        match &proposal.safety_check {
            SafetyCheckResult::Passed => Ok(StageDecision::Continue),
            SafetyCheckResult::Failed { reason } => Ok(StageDecision::Skip(format!(
                "DGMMetaEvolve safety failed: {}",
                reason
            ))),
            SafetyCheckResult::NeedsHumanReview { concern } => {
                Ok(StageDecision::Rollback(format!(
                    "DGMMetaEvolve needs human review: {}",
                    concern
                )))
            }
        }
    }
}

/// A BrainPipeline stage that invokes the HyperMetaAgent and applies resulting diffs.
pub struct MetaEvolveStage {
    pub meta_agent: HyperMetaAgent,
    pub archive: HyperAgentArchive,
}

impl MetaEvolveStage {
    pub fn new(meta_agent: HyperMetaAgent, archive: HyperAgentArchive) -> Self {
        Self {
            meta_agent,
            archive,
        }
    }
}

impl BrainStage for MetaEvolveStage {
    fn name(&self) -> &str {
        "meta_evolve"
    }

    fn process(
        &self,
        _brain: &mut SelfIteratingBrain,
    ) -> Result<StageDecision, NeoTrixError> {
        let proposal = self.meta_agent.forward(&self.archive);
        match &proposal.safety_check {
            SafetyCheckResult::Passed => Ok(StageDecision::Continue),
            SafetyCheckResult::Failed { reason } => Ok(StageDecision::Skip(format!(
                "MetaEvolve safety failed: {}",
                reason
            ))),
            SafetyCheckResult::NeedsHumanReview { concern } => {
                Ok(StageDecision::Rollback(format!(
                    "MetaEvolve needs human review: {}",
                    concern
                )))
            }
        }
    }
}
