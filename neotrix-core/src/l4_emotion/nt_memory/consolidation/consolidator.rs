#![forbid(unsafe_code)]

//! Memory consolidation — promotes short-term memories to long-term tiers
//! based on access frequency, age, and novelty metrics.
//!
//! Promotion rules (R-P117, R-MEM09):
//! - access_count > threshold_core → promote to Core
//! - age_turns > threshold_archive → demote to Archive
//! - novelty < threshold_prune → prune (discard)

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// A memory entry fed into the consolidator from short-term storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationMemoryEntry {
    pub id: String,
    pub content: String,
    pub access_count: u64,
    pub created_at_turn: u64,
    pub novelty: f64,
}

/// Classification of how a memory was consolidated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionType {
    /// Kept in short-term (no rule triggered).
    Retained,
    /// Promoted to core — frequently accessed.
    Core,
    /// Demoted to archive — too old.
    Archive,
    /// Discarded — insufficient novelty.
    Pruned,
}

/// The result of consolidating a single memory entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidatedMemory {
    pub original_id: String,
    pub promotion_type: PromotionType,
    pub consolidated_at: i64,
}

/// Thresholds governing consolidation behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationConfig {
    /// Minimum access count to promote to Core.
    pub access_count_core: u64,
    /// Age in turns after which a memory is archived.
    pub age_turns_archive: u64,
    /// Novelty below which a memory is pruned.
    pub novelty_prune: f64,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            access_count_core: 5,
            age_turns_archive: 100,
            novelty_prune: 0.2,
        }
    }
}

/// Memory consolidator — evaluates short-term entries and produces consolidation decisions.
#[derive(Debug)]
pub struct MemoryConsolidator {
    config: ConsolidationConfig,
}

impl MemoryConsolidator {
    pub fn new(config: ConsolidationConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(ConsolidationConfig::default())
    }

    /// Consolidate a batch of short-term memory entries.
    ///
    /// Each entry is evaluated against the three rules in priority order:
    /// 1. **Prune**: novelty < threshold → discard
    /// 2. **Core**: access_count > threshold → promote to core
    /// 3. **Archive**: age_turns > threshold → demote to archive
    /// 4. **Retain**: no rule triggered → keep in short-term
    pub fn consolidate(&self, short_term: Vec<ConsolidationMemoryEntry>) -> Vec<ConsolidatedMemory> {
        let now = Self::now_turn_ts();
        short_term
            .into_iter()
            .map(|entry| {
                let promotion = if entry.novelty < self.config.novelty_prune {
                    PromotionType::Pruned
                } else if entry.access_count > self.config.access_count_core {
                    PromotionType::Core
                } else if (now as u64).saturating_sub(entry.created_at_turn) > self.config.age_turns_archive {
                    PromotionType::Archive
                } else {
                    PromotionType::Retained
                };

                ConsolidatedMemory {
                    original_id: entry.id,
                    promotion_type: promotion,
                    consolidated_at: now,
                }
            })
            .collect()
    }

    /// Convenience: filter only the IDs that survived consolidation (not pruned).
    pub fn retained_ids<'a>(&self, results: &'a [ConsolidatedMemory]) -> Vec<&'a str> {
        results
            .iter()
            .filter(|c| c.promotion_type != PromotionType::Pruned)
            .map(|c| c.original_id.as_str())
            .collect()
    }

    fn now_turn_ts() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }
}

impl Default for MemoryConsolidator {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, access: u64, age: u64, novelty: f64) -> ConsolidationMemoryEntry {
        ConsolidationMemoryEntry {
            id: id.to_string(),
            content: format!("content-{id}"),
            access_count: access,
            created_at_turn: age,
            novelty,
        }
    }

    #[test]
    fn test_low_novelty_pruned() {
        let c = MemoryConsolidator::with_defaults();
        let results = c.consolidate(vec![entry("e1", 10, 1, 0.1)]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].promotion_type, PromotionType::Pruned);
    }

    #[test]
    fn test_high_access_promoted_to_core() {
        let c = MemoryConsolidator::with_defaults();
        let results = c.consolidate(vec![entry("e2", 6, 1, 0.5)]);
        assert_eq!(results[0].promotion_type, PromotionType::Core);
    }

    #[test]
    fn test_old_entry_archived() {
        let c = MemoryConsolidator::with_defaults();
        let now = MemoryConsolidator::now_turn_ts();
        let results = c.consolidate(vec![entry("e3", 2, now as u64 - 101, 0.5)]);
        assert_eq!(results[0].promotion_type, PromotionType::Archive);
    }

    #[test]
    fn test_retained_when_no_rule_matches() {
        let c = MemoryConsolidator::with_defaults();
        let now = MemoryConsolidator::now_turn_ts();
        let results = c.consolidate(vec![entry("e4", 3, now as u64 - 10, 0.5)]);
        assert_eq!(results[0].promotion_type, PromotionType::Retained);
    }

    #[test]
    fn test_prune_takes_priority_over_core() {
        let c = MemoryConsolidator::with_defaults();
        let results = c.consolidate(vec![entry("e5", 10, 1, 0.05)]);
        assert_eq!(results[0].promotion_type, PromotionType::Pruned);
    }

    #[test]
    fn test_retained_ids_excludes_pruned() {
        let c = MemoryConsolidator::with_defaults();
        let results = c.consolidate(vec![
            entry("a", 10, 1, 0.5),
            entry("b", 1, 1, 0.1),
            entry("c", 1, 1, 0.8),
        ]);
        let ids = c.retained_ids(&results);
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"c"));
        assert!(!ids.contains(&"b"));
    }

    #[test]
    fn test_custom_config() {
        let config = ConsolidationConfig {
            access_count_core: 2,
            age_turns_archive: 50,
            novelty_prune: 0.1,
        };
        let c = MemoryConsolidator::new(config);
        let results = c.consolidate(vec![entry("x", 3, 1, 0.15)]);
        assert_eq!(results[0].promotion_type, PromotionType::Core);
    }

    #[test]
    fn test_consolidated_at_timestamp_is_set() {
        let c = MemoryConsolidator::with_defaults();
        let results = c.consolidate(vec![entry("ts1", 1, 1, 0.5)]);
        assert!(results[0].consolidated_at > 0);
    }

    #[test]
    fn test_empty_consolidate_returns_empty() {
        let c = MemoryConsolidator::with_defaults();
        let results = c.consolidate(vec![]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_boundary_novelty_not_pruned() {
        let c = MemoryConsolidator::with_defaults();
        let results = c.consolidate(vec![entry("e1", 1, 1, 0.2)]);
        assert_ne!(results[0].promotion_type, PromotionType::Pruned);
    }

    #[test]
    fn test_boundary_access_exact_not_core() {
        let c = MemoryConsolidator::with_defaults();
        let results = c.consolidate(vec![entry("e1", 5, 1, 0.5)]);
        assert_ne!(results[0].promotion_type, PromotionType::Core);
    }
}
