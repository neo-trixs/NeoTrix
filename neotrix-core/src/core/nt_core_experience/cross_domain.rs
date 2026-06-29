use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::core::nt_core_experience::meta_evolution::TransferValidator;
use crate::core::nt_core_experience::self_evolution_loop::types::*;

/// Snapshot of a domain's evolution archive for cross-domain transfer.
/// Captures the top-K mutations that showed significant improvement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainArchiveSnapshot {
    /// The domain name (e.g. "cognitive", "perception", "action", "meta")
    pub domain: String,
    /// Top-K successful mutations from this domain's archive, with their score_after
    pub top_mutations: Vec<(MutationOp, f64)>,
    /// Number of steps in the source domain when snapshot was taken
    pub step_count: usize,
}

/// Orchestrates archive-driven cross-domain capability transfer.
///
/// Bridges to the existing `TransferValidator` for accuracy tracking,
/// while adding mutation-level transfer between domain archives.
/// The transfer mechanism works by:
/// 1. Snapshotting each domain's top-performing mutations
/// 2. Finding compatible candidates from other domains
/// 3. Attempting them in the target domain via the SEAL evolution loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainTransfer {
    /// Maps domain name → archive snapshot for transfer
    pub domain_archives: HashMap<String, DomainArchiveSnapshot>,
    /// Per-domain accuracy tracker (bridges to existing TransferValidator).
    /// Skipped from serde because TransferValidator contains `Instant` fields.
    #[serde(skip)]
    pub validator: TransferValidator,
    /// Success threshold for accepting a transferred mutation.
    /// A mutation is considered valuable when score_after - score_before >= threshold.
    pub transfer_acceptance_threshold: f64,
    /// Maximum mutations to transfer per cycle
    pub max_transfer_per_cycle: usize,
}

impl CrossDomainTransfer {
    pub fn new() -> Self {
        Self {
            domain_archives: HashMap::new(),
            validator: TransferValidator::new(),
            transfer_acceptance_threshold: 0.05,
            max_transfer_per_cycle: 3,
        }
    }

    /// Take a snapshot of the current archive for a given domain.
    /// Extracts the top-10 most successful mutations (score_after - score_before > threshold).
    pub fn snapshot_domain(&mut self, domain: &str, archive: &SelfEvolutionArchive) {
        let mut top: Vec<(MutationOp, f64)> = archive
            .steps
            .iter()
            .filter(|s| {
                s.score_after
                    .map(|after| after - s.score_before >= self.transfer_acceptance_threshold)
                    .unwrap_or(false)
            })
            .map(|s| (s.mutation.clone(), s.score_after.unwrap()))
            .collect();

        top.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        top.truncate(10);

        self.domain_archives.insert(
            domain.to_string(),
            DomainArchiveSnapshot {
                domain: domain.to_string(),
                top_mutations: top,
                step_count: archive.steps.len(),
            },
        );

        self.validator.record_accuracy(domain, archive.best_score);
    }

    /// Find the best transfer candidates from `source_domain` to `target_domain`.
    /// Returns mutations that:
    /// 1. Had high score improvement in source domain
    /// 2. Are compatible with target domain (same MutationOp variant type)
    /// 3. Haven't been tried in target domain before
    pub fn find_transfer_candidates(
        &self,
        source_domain: &str,
        target_domain: &str,
    ) -> Vec<(MutationOp, f64)> {
        let source = match self.domain_archives.get(source_domain) {
            Some(s) => s,
            None => return vec![],
        };

        let mut candidates: Vec<(MutationOp, f64)> = source
            .top_mutations
            .iter()
            // Only generic mutation types that can work across domains
            .filter(|(op, _)| {
                matches!(
                    op,
                    MutationOp::TuneParam { .. } | MutationOp::RewritePrimitive { .. }
                )
            })
            // Exclude mutations already tried in the target domain
            .filter(|(op, _)| {
                if let Some(target_snap) = self.domain_archives.get(target_domain) {
                    !target_snap
                        .top_mutations
                        .iter()
                        .any(|(existing, _)| match (op, existing) {
                            (
                                MutationOp::TuneParam { target: t1, .. },
                                MutationOp::TuneParam { target: t2, .. },
                            ) => t1 == t2,
                            (
                                MutationOp::RewritePrimitive { name: n1, .. },
                                MutationOp::RewritePrimitive { name: n2, .. },
                            ) => n1 == n2,
                            _ => false,
                        })
                } else {
                    true
                }
            })
            .map(|(op, score)| (op.clone(), *score))
            .collect();

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        candidates
    }

    /// Record a transfer attempt outcome to update validator stats.
    /// `score_delta` is the change in target domain score after applying the mutation.
    pub fn record_transfer(
        &mut self,
        source: &str,
        target: &str,
        mutation: &MutationOp,
        score_delta: f64,
    ) {
        self.validator.record_accuracy(target, score_delta);
        self.validator.check_transfer_success(source, target);
        log::info!(
            "CROSS_DOMAIN: transfer [{}] from {} → {} (delta={:.4}, success_rate={:.2}%)",
            mutation.summary(),
            source,
            target,
            score_delta,
            self.transfer_success_rate() * 100.0,
        );
    }

    /// Get the overall transfer success rate from the validator.
    pub fn transfer_success_rate(&self) -> f64 {
        self.validator.transfer_rate()
    }

    /// Generate a summary report of cross-domain transfer activity.
    pub fn report(&self) -> String {
        let mut lines = vec!["=== Cross-Domain Transfer Report ===".to_string()];
        lines.push(format!(
            "Success rate: {:.2}%",
            self.transfer_success_rate() * 100.0
        ));
        lines.push(format!(
            "Acceptance threshold: {:.3}",
            self.transfer_acceptance_threshold
        ));
        lines.push(format!(
            "Max transfers/cycle: {}",
            self.max_transfer_per_cycle
        ));
        lines.push(format!("Tracked domains: {}", self.domain_archives.len()));
        for (name, snap) in &self.domain_archives {
            lines.push(format!(
                "  {}: {} mutations, {} steps in snapshot",
                name,
                snap.top_mutations.len(),
                snap.step_count
            ));
            for (i, (op, score)) in snap.top_mutations.iter().enumerate().take(3) {
                lines.push(format!(
                    "    {}. [{}] score={:.4}",
                    i + 1,
                    op.summary(),
                    score
                ));
            }
        }
        lines.push(self.validator.report());
        lines.join("\n")
    }
}

impl Default for CrossDomainTransfer {
    fn default() -> Self {
        Self::new()
    }
}
