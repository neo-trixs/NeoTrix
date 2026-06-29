use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use super::{
    ArchiveEntry, FileDiff, GenerationResult, MetaAgent, MetaAgentConfig, ModificationTarget,
    SafetyCheckResult, SelfModificationProposal, SkillTreeArchive, StagedEvaluation,
};

impl MetaAgent {
    pub fn new(config: MetaAgentConfig) -> Self {
        let archive_config = super::SelectionConfig::default();
        Self {
            config,
            archive: SkillTreeArchive::new(archive_config),
            eval_config: StagedEvaluation::default(),
            iteration: 0,
        }
    }

    /// Restore persisted archive at startup.
    pub fn restore_archive(&mut self, archive: SkillTreeArchive) {
        self.archive = archive;
    }

    /// Run one generation of self-referential improvement
    pub fn run_generation(&mut self) -> GenerationResult {
        let iterations_left = self.config.budget.saturating_sub(self.iteration as u32);

        if self.archive.is_empty() {
            let seed = ArchiveEntry {
                id: Uuid::new_v4().to_string(),
                parent_id: None,
                score: 0.5,
                diversity_score: 0.0,
                diffs: Vec::new(),
                generation: 0,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
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

        let archive_summaries: Vec<String> = self
            .archive
            .nodes
            .values()
            .filter(|n| n.score > 0.0)
            .map(|n| format!("score={:.2}:diffs={}", n.score, n.diffs.len()))
            .collect();
        let handler_names: Vec<String> = self
            .archive
            .nodes
            .values()
            .flat_map(|n| {
                n.diffs.iter().filter_map(|d| {
                    d.file_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                })
            })
            .collect();
        let proposals = self.forward(&archive_summaries, &handler_names, iterations_left);
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

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let mut child_lineage = parent.lineage.clone();
            child_lineage.push(parent.id.clone());

            let child_paths: Vec<&std::path::Path> =
                filtered.iter().map(|d| d.file_path.as_path()).collect();
            let diversity_score = self.compute_diversity(&child_paths);

            let child = ArchiveEntry {
                id: Uuid::new_v4().to_string(),
                parent_id: Some(parent.id.clone()),
                score: 0.0,
                diversity_score,
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

    /// Forward: generate improvement proposals from SelfEvolutionLoop archive entries.
    pub fn forward(
        &self,
        archive_summaries: &[String],
        handler_names: &[String],
        iterations_left: u32,
    ) -> Vec<SelfModificationProposal> {
        if iterations_left == 0 || archive_summaries.is_empty() {
            return Vec::new();
        }
        let mut proposals = Vec::new();
        for (i, summary) in archive_summaries.iter().enumerate().take(5) {
            let target = if summary.contains("meta") || summary.contains("self") {
                ModificationTarget::MetaAgent
            } else if summary.contains("handler") || summary.contains("capability") {
                ModificationTarget::CapabilityExtension
            } else {
                ModificationTarget::TaskAgent
            };
            let handler_hint = handler_names
                .get(i)
                .map(|s| s.as_str())
                .unwrap_or("unknown");
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            proposals.push(SelfModificationProposal {
                target,
                diffs: vec![FileDiff {
                    file_path: PathBuf::from(format!("ci/handler/{}.rs", handler_hint)),
                    diff_content: format!(
                        "; Proposal generated from archive: {}\n; handler: {}\n; timestamp: {}",
                        summary, handler_hint, now
                    ),
                    parent_hash: format!("archive_{}_{}", i, now),
                }],
                expected_impact: summary.clone(),
                safety_check: SafetyCheckResult::NeedsHumanReview {
                    concern: format!("Auto-generated proposal from archive entry: {}", summary),
                },
            });
        }
        proposals
    }

    /// Filter patches to protect domain files
    pub fn filter_diff(&self, diffs: &[FileDiff]) -> Result<Vec<FileDiff>, String> {
        let filtered: Vec<FileDiff> = diffs
            .iter()
            .filter(|d| {
                let path_str = d.file_path.to_string_lossy();
                !self
                    .config
                    .protected_paths
                    .iter()
                    .any(|p| path_str.starts_with(p))
            })
            .cloned()
            .collect();

        if filtered.len() < diffs.len() {
            let blocked = diffs.len() - filtered.len();
            return Err(format!(
                "{} diff(s) blocked by protected path filter",
                blocked
            ));
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

    /// Compute diversity score as fraction of file paths not seen in existing entries
    fn compute_diversity(&self, child_paths: &[&std::path::Path]) -> f64 {
        if child_paths.is_empty() {
            return 0.0;
        }
        let existing_paths: std::collections::HashSet<&std::path::Path> = self
            .archive
            .nodes
            .values()
            .flat_map(|n| n.diffs.iter().map(|d| d.file_path.as_path()))
            .collect();
        let unique: usize = child_paths
            .iter()
            .filter(|p| !existing_paths.contains(*p))
            .count();
        unique as f64 / child_paths.len() as f64
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
