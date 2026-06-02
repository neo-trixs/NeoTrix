use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use super::{
    MetaAgent, MetaAgentConfig, EvolutionArchive, ArchiveEntry,
    FileDiff, SelfModificationProposal, ModificationTarget,
    SafetyCheckResult, GenerationResult, StagedEvaluation,
};

impl MetaAgent {
    pub fn new(config: MetaAgentConfig) -> Self {
        let archive_config = super::SelectionConfig::default();
        Self {
            config,
            archive: EvolutionArchive::new(archive_config),
            eval_config: StagedEvaluation::default(),
            iteration: 0,
        }
    }

    /// Run one generation of self-referential improvement
    pub fn run_generation(&mut self) -> GenerationResult {
        let iterations_left = self.config.budget.saturating_sub(self.iteration as u32);

        if self.archive.is_empty() {
            let seed = ArchiveEntry {
                id: Uuid::new_v4().to_string(),
                parent_id: None,
                score: 0.5,
                diffs: Vec::new(),
                generation: 0,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                lineage: Vec::new(),
                metadata: HashMap::new(),
            };
            self.archive.add(seed);
        }

        let parent_clone = self.archive.select_parent().cloned();
        let parent = match parent_clone {
            Some(p) => p,
            None => {
                return GenerationResult {
                    generation: self.iteration,
                    proposals_generated: 0,
                    proposals_accepted: 0,
                    best_score: self.archive.best().map(|e| e.score).unwrap_or(0.0),
                    archive_size: self.archive.len(),
                    rollbacks: 0,
                };
            }
        };

        let proposals = self.forward("repo", "eval", iterations_left);
        let proposals_generated = proposals.len();
        let mut accepted = 0usize;
        let mut rollbacks = 0usize;
        let parent_score = parent.score;

        for proposal in proposals {
            let safety = &proposal.safety_check;
            match safety {
                SafetyCheckResult::Passed => {}
                _ => continue,
            }

            let filtered = match self.filter_diff(&proposal.diffs) {
                Ok(d) => d,
                Err(_) => continue,
            };

            if filtered.is_empty() {
                continue;
            }

            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
            let mut child_lineage = parent.lineage.clone();
            child_lineage.push(parent.id.clone());

            let child = ArchiveEntry {
                id: Uuid::new_v4().to_string(),
                parent_id: Some(parent.id.clone()),
                score: 0.0,
                diffs: filtered,
                generation: self.iteration,
                timestamp: now,
                lineage: child_lineage,
                metadata: HashMap::new(),
            };

            let score = self.staged_eval(&child);
            let mut child = child;
            child.score = score;

            if self.should_rollback(score, parent_score) {
                rollbacks += 1;
            } else {
                self.archive.add(child);
                accepted += 1;
            }
        }

        self.iteration += 1;

        if self.archive.len() > self.archive.config.archive_capacity {
            let _pruned = self.archive.prune(0.01);
        }

        GenerationResult {
            generation: self.iteration,
            proposals_generated,
            proposals_accepted: accepted,
            best_score: self.archive.best().map(|e| e.score).unwrap_or(0.0),
            archive_size: self.archive.len(),
            rollbacks,
        }
    }

    /// Forward: generate improvement proposals given context.
    /// In real implementation, this calls LLM. Here, returns a placeholder proposal.
    pub fn forward(&self, _repo_path: &str, _eval_path: &str, iterations_left: u32) -> Vec<SelfModificationProposal> {
        if iterations_left == 0 {
            return Vec::new();
        }
        vec![SelfModificationProposal {
            target: ModificationTarget::CapabilityExtension,
            diffs: vec![FileDiff {
                file_path: PathBuf::from("src/agent.rs"),
                diff_content: "--- a/src/agent.rs\n+++ b/src/agent.rs\n@@ -1 +1 @@\n-// old\n+// new".to_string(),
                parent_hash: "abc123".to_string(),
            }],
            expected_impact: "Minor capability extension".to_string(),
            safety_check: SafetyCheckResult::Passed,
        }]
    }

    /// Filter patches to protect domain files
    pub fn filter_diff(&self, diffs: &[FileDiff]) -> Result<Vec<FileDiff>, String> {
        let filtered: Vec<FileDiff> = diffs
            .iter()
            .filter(|d| {
                let path_str = d.file_path.to_string_lossy();
                !self.config.protected_paths.iter().any(|p| path_str.starts_with(p))
            })
            .cloned()
            .collect();

        if filtered.len() < diffs.len() {
            let blocked = diffs.len() - filtered.len();
            return Err(format!("{} diff(s) blocked by protected path filter", blocked));
        }

        Ok(filtered)
    }

    /// Check if a modification is safe
    pub fn safety_check(&self, proposal: &SelfModificationProposal) -> SafetyCheckResult {
        if proposal.target == ModificationTarget::MetaAgent && !self.config.self_referential {
            return SafetyCheckResult::Failed {
                reason: "Self-referential modification disabled in config".to_string(),
            };
        }

        for diff in &proposal.diffs {
            let path_str = diff.file_path.to_string_lossy();
            for protected in &self.config.protected_paths {
                if path_str.starts_with(protected) {
                    return SafetyCheckResult::Failed {
                        reason: format!("Diff targets protected path: {}", path_str),
                    };
                }
            }
        }

        SafetyCheckResult::Passed
    }

    /// Stage evaluation: run subset, check score, if above threshold run full
    pub fn staged_eval(&self, _entry: &super::ArchiveEntry) -> f64 {
        let subset_score = 0.5;

        if subset_score >= self.eval_config.subset_threshold {
            subset_score
        } else {
            0.0
        }
    }
}
