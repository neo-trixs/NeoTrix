//! Memory Proficiency (AutoMem-inspired, arXiv 2607.01224).
//!
//! Memory management as a trainable cognitive skill with two-loop optimization:
//!   Outer loop: revises memory structure (schema, indices, clustering)
//!   Inner loop: trains memory action proficiency (search, store, link, consolidate)
//!
//! Key insight (AutoMem):
//!   Memory operations are first-class skills that can be independently learned.
//!   A dedicated memory loop (~2x-4x improvement) optimizes KB usage without
//!   changing the underlying LLM policy.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Memory actions that form the action vocabulary for the proficiency loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryAction {
    SearchFts,
    SearchGraph,
    SearchEmbed,
    StoreNode,
    LinkNodes,
    ConsolidateCluster,
    PruneStale,
    RefreshIndex,
    SummarizeSubgraph,
    HierarchicalRetrieval,
}

impl MemoryAction {
    pub fn all() -> Vec<Self> {
        vec![
            Self::SearchFts, Self::SearchGraph, Self::SearchEmbed,
            Self::StoreNode, Self::LinkNodes, Self::ConsolidateCluster,
            Self::PruneStale, Self::RefreshIndex, Self::SummarizeSubgraph,
            Self::HierarchicalRetrieval,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            Self::SearchFts => "search_fts",
            Self::SearchGraph => "search_graph",
            Self::SearchEmbed => "search_embed",
            Self::StoreNode => "store_node",
            Self::LinkNodes => "link_nodes",
            Self::ConsolidateCluster => "consolidate_cluster",
            Self::PruneStale => "prune_stale",
            Self::RefreshIndex => "refresh_index",
            Self::SummarizeSubgraph => "summarize_subgraph",
            Self::HierarchicalRetrieval => "hierarchical_retrieval",
        }
    }
}

/// A single memory action proficiency record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryActionRecord {
    pub action: MemoryAction,
    pub success: bool,
    pub duration_ms: u64,
    pub context_entropy: f64,
    pub outcome_score: f64,
}

/// Memory proficiency state: tracks per-action success rates and learns
/// optimal action selection for given KB contexts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProficiency {
    /// Per-action statistics: (total_attempts, successes, avg_duration_ms)
    pub action_stats: HashMap<MemoryAction, (u64, u64, f64)>,
    /// Recent action history for pattern detection
    pub history: VecDeque<MemoryActionRecord>,
    pub max_history: usize,
    /// Learned action preferences per context type
    pub context_preferences: HashMap<String, Vec<(MemoryAction, f64)>>,
    /// Outer-loop revision counter
    pub revision_count: u64,
    /// Inner-loop training steps
    pub training_steps: u64,
}

impl Default for MemoryProficiency {
    fn default() -> Self {
        Self {
            action_stats: HashMap::new(),
            history: VecDeque::new(),
            max_history: 1000,
            context_preferences: HashMap::new(),
            revision_count: 0,
            training_steps: 0,
        }
    }
}

impl MemoryProficiency {
    pub fn new() -> Self { Self::default() }

    /// Record a memory action outcome (inner-loop training step).
    pub fn record_action(&mut self, record: MemoryActionRecord) {
        let stats = self.action_stats.entry(record.action).or_insert((0, 0, 0.0));
        stats.0 += 1;
        if record.success {
            stats.1 += 1;
        }
        let total = stats.0 as f64;
        stats.2 = (stats.2 * (total - 1.0) + record.duration_ms as f64) / total;

        self.history.push_back(record);
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }
        self.training_steps += 1;
    }

    /// Get success rate for a specific action.
    pub fn success_rate(&self, action: &MemoryAction) -> f64 {
        self.action_stats.get(action)
            .map(|(total, successes, _)| {
                if *total > 0 { *successes as f64 / *total as f64 } else { 0.0 }
            })
            .unwrap_or(0.0)
    }

    /// Recommend best action for a given KB context.
    /// Returns (action, confidence).
    pub fn recommend_action(&self, context_key: &str) -> (MemoryAction, f64) {
        if let Some(prefs) = self.context_preferences.get(context_key) {
            if let Some((action, score)) = prefs.first() {
                return (*action, *score);
            }
        }
        // Fall back to global best action by success rate
        let best = MemoryAction::all().into_iter()
            .max_by(|a, b| {
                self.success_rate(a)
                    .partial_cmp(&self.success_rate(b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(MemoryAction::SearchFts);
        (best, self.success_rate(&best))
    }

    /// Outer-loop revision: analyze history and update context preferences.
    ///
    /// 1. Cluster recent history by context patterns
    /// 2. Identify high-success action sequences
    /// 3. Update context_preferences with improved strategies
    ///
    /// Returns number of preferences updated.
    pub fn outer_loop_revision(&mut self) -> usize {
        if self.history.len() < 10 {
            return 0;
        }

        let mut updates = 0;

        // Group actions by context entropy buckets
        let mut by_context: HashMap<&str, Vec<&MemoryActionRecord>> = HashMap::new();
        for record in &self.history {
            let key = if record.context_entropy < 0.3 {
                "low_entropy"
            } else if record.context_entropy < 0.7 {
                "medium_entropy"
            } else {
                "high_entropy"
            };
            by_context.entry(key).or_default().push(record);
        }

        for (context_key, records) in &by_context {
            let mut action_scores: HashMap<MemoryAction, (f64, usize)> = HashMap::new();
            for r in records {
                let entry = action_scores.entry(r.action).or_default();
                entry.0 += r.outcome_score;
                entry.1 += 1;
            }

            let mut ranked: Vec<(MemoryAction, f64)> = action_scores.into_iter()
                .map(|(action, (total, count))| {
                    (action, total / count.max(1) as f64)
                })
                .collect();
            ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            ranked.truncate(3);

            if !ranked.is_empty() {
                self.context_preferences.insert(context_key.to_string(), ranked);
                updates += 1;
            }
        }

        self.revision_count += 1;
        updates
    }

    /// Report current proficiency metrics.
    pub fn report(&self) -> MemoryProficiencyReport {
        let mut action_breakdown = Vec::new();
        for (action, (total, successes, avg_duration)) in &self.action_stats {
            action_breakdown.push(MemoryActionStats {
                action: *action,
                total_attempts: *total,
                successes: *successes,
                avg_duration_ms: *avg_duration,
                success_rate: if *total > 0 { *successes as f64 / *total as f64 } else { 0.0 },
            });
        }
        action_breakdown.sort_by(|a, b| b.success_rate.partial_cmp(&a.success_rate).unwrap_or(std::cmp::Ordering::Equal));

        MemoryProficiencyReport {
            total_actions: self.training_steps,
            revision_count: self.revision_count,
            action_breakdown,
            context_preferences: self.context_preferences.iter()
                .map(|(k, v)| (k.clone(), v.iter().map(|(a, s)| (a.name().to_string(), *s)).collect()))
                .collect(),
            overall_efficiency: self.action_stats.values()
                .map(|(total, successes, _)| {
                    if *total > 0 { *successes as f64 / *total as f64 } else { 0.0 }
                })
                .sum::<f64>() / self.action_stats.len().max(1) as f64,
        }
    }
}

/// Stats for a single memory action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryActionStats {
    pub action: MemoryAction,
    pub total_attempts: u64,
    pub successes: u64,
    pub avg_duration_ms: f64,
    pub success_rate: f64,
}

/// Proficiency report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProficiencyReport {
    pub total_actions: u64,
    pub revision_count: u64,
    pub action_breakdown: Vec<MemoryActionStats>,
    pub context_preferences: HashMap<String, Vec<(String, f64)>>,
    pub overall_efficiency: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_proficiency_default() {
        let prof = MemoryProficiency::default();
        assert_eq!(prof.training_steps, 0);
        assert!(prof.action_stats.is_empty());
    }

    #[test]
    fn test_record_action_tracks_stats() {
        let mut prof = MemoryProficiency::new();
        prof.record_action(MemoryActionRecord {
            action: MemoryAction::SearchFts,
            success: true,
            duration_ms: 10,
            context_entropy: 0.3,
            outcome_score: 0.9,
        });
        assert_eq!(prof.training_steps, 1);
        assert!((prof.success_rate(&MemoryAction::SearchFts) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_recommend_action_falls_back_globally() {
        let prof = MemoryProficiency::new();
        let (action, _) = prof.recommend_action("unknown");
        // Should fall back to a valid action
        assert!(MemoryAction::all().contains(&action));
    }

    #[test]
    fn test_outer_loop_revision_updates_preferences() {
        let mut prof = MemoryProficiency::new();
        // Add diverse history
        for _ in 0..20 {
            prof.record_action(MemoryActionRecord {
                action: MemoryAction::SearchFts,
                success: true,
                duration_ms: 5,
                context_entropy: 0.2,
                outcome_score: 0.9,
            });
        }
        prof.record_action(MemoryActionRecord {
            action: MemoryAction::SearchGraph,
            success: false,
            duration_ms: 100,
            context_entropy: 0.8,
            outcome_score: 0.1,
        });
        let updates = prof.outer_loop_revision();
        assert!(updates > 0);
        assert!(prof.revision_count >= 1);
    }

    #[test]
    fn test_proficiency_report() {
        let mut prof = MemoryProficiency::new();
        prof.record_action(MemoryActionRecord {
            action: MemoryAction::StoreNode,
            success: true,
            duration_ms: 20,
            context_entropy: 0.5,
            outcome_score: 0.8,
        });
        let report = prof.report();
        assert_eq!(report.total_actions, 1);
        assert!(!report.action_breakdown.is_empty());
        assert!(report.overall_efficiency > 0.0);
    }

    #[test]
    fn test_all_actions_have_unique_names() {
        let actions = MemoryAction::all();
        let mut names: Vec<&str> = actions.iter().map(|a| a.name()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), actions.len());
    }
}
