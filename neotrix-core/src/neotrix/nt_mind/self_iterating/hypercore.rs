use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use super::HyperAgentArchive;
use super::{ParentSelection, HyperAgentRecord};

/// Represents a file-level diff produced by the meta-agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub file_path: PathBuf,
    pub diff_content: String,
    pub parent_hash: String,
}

/// What component a SelfModificationProposal targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModificationTarget {
    TaskAgent,
    MetaAgent,
    ImprovementMechanism,
    CapabilityExtension,
}

/// Outcome of a safety check performed on a proposed modification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SafetyCheckResult {
    Passed,
    Failed { reason: String },
    NeedsHumanReview { concern: String },
}

/// A proposal from the meta-agent that can modify task-agent code,
/// meta-agent code, or the improvement mechanism itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModificationProposal {
    pub target: ModificationTarget,
    pub diffs: Vec<FileDiff>,
    pub expected_impact: String,
    pub safety_check: SafetyCheckResult,
}

/// Self-referential agent that reads repo code and generates code diffs
/// improving both task agents and its own meta-agent logic.
pub struct HyperMetaAgent {
    pub budget: u32,
    pub self_referential: bool,
    pub protected_paths: Vec<String>,
}

impl HyperMetaAgent {
    pub fn new(budget: u32, self_referential: bool) -> Self {
        Self {
            budget,
            self_referential,
            protected_paths: Vec::new(),
        }
    }

    /// Generate a SelfModificationProposal from archive.
    /// Uses parent selection strategy + latent variation to propose diffs.
    pub fn forward(&self, archive: &HyperAgentArchive) -> SelfModificationProposal {
        let parent = archive.select_parent();
        match parent {
            Some(p) => self.mutate_from_parent(p, archive),
            None => self.seed_proposal(),
        }
    }

    /// Mutate a parent record to produce a child proposal
    fn mutate_from_parent(&self, parent: &HyperAgentRecord, archive: &HyperAgentArchive) -> SelfModificationProposal {
        let mut impact_parts = Vec::new();

        let target = if parent.generation < 5 {
            ModificationTarget::CapabilityExtension
        } else if parent.score.map_or(false, |s| s > 0.7) {
            ModificationTarget::MetaAgent
        } else if archive.config.strategy == ParentSelection::DiversityWeighted && parent.novelty_score > 0.5 {
            ModificationTarget::ImprovementMechanism
        } else {
            ModificationTarget::TaskAgent
        };

        impact_parts.push(format!("parent_score={:.3}", parent.score.unwrap_or(0.0)));
        impact_parts.push(format!("parent_novelty={:.3}", parent.novelty_score));
        impact_parts.push(format!("generation={}", parent.generation + 1));
        impact_parts.push(format!("archive_size={}", archive.records.len()));

        SelfModificationProposal {
            target,
            diffs: Vec::new(),
            expected_impact: impact_parts.join(" | "),
            safety_check: SafetyCheckResult::Passed,
        }
    }

    /// Seed the archive with an initial proposal (no parents available)
    fn seed_proposal(&self) -> SelfModificationProposal {
        SelfModificationProposal {
            target: ModificationTarget::CapabilityExtension,
            diffs: Vec::new(),
            expected_impact: "seed: initial archive entry".to_string(),
            safety_check: SafetyCheckResult::Passed,
        }
    }

    pub fn filter_protected_paths(&self, diffs: &[FileDiff]) -> Vec<FileDiff> {
        diffs
            .iter()
            .filter(|d| {
                !self
                    .protected_paths
                    .iter()
                    .any(|p| d.file_path.to_string_lossy().contains(p.as_str()))
            })
            .cloned()
            .collect()
    }

    pub fn evaluate_proposal(
        &self,
        proposal: &SelfModificationProposal,
        forecast_cumulative_fe: f64,
    ) -> f64 {
        let fe_score = (-forecast_cumulative_fe).exp().clamp(0.0, 1.0);
        match proposal.target {
            ModificationTarget::MetaAgent => fe_score * 0.9,
            ModificationTarget::ImprovementMechanism => fe_score * 1.1,
            ModificationTarget::CapabilityExtension => fe_score * 0.8,
            ModificationTarget::TaskAgent => fe_score * 0.7,
        }
    }
}
