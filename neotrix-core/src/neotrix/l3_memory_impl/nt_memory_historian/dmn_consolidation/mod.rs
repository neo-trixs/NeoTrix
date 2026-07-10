#![forbid(unsafe_code)]

pub mod graph_executor;
pub mod retrieval;
pub mod three_tier;

use serde::{Deserialize, Serialize};

pub use self::graph_executor::LearnedGraphExecutor;
pub use self::retrieval::FamiliarityWeightedRetrieval;
pub use self::three_tier::{MemoryItem, MemoryTier, ThreeTierStore};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmnConfig {
    pub short_term_capacity: usize,
    pub medium_term_capacity: usize,
    pub consolidation_interval: u64,
    pub rehearsal_threshold: f64,
    pub graph_learning_rate: f64,
}

impl Default for DmnConfig {
    fn default() -> Self {
        Self {
            short_term_capacity: 10,
            medium_term_capacity: 50,
            consolidation_interval: 100,
            rehearsal_threshold: 0.6,
            graph_learning_rate: 0.1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationReport {
    pub short_term_before: usize,
    pub short_term_after: usize,
    pub medium_term_before: usize,
    pub medium_term_after: usize,
    pub long_term_before: usize,
    pub long_term_after: usize,
    pub items_consolidated: usize,
    pub items_rehearsed: usize,
    pub items_forgotten: usize,
    pub graph_edges_added: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmnStats {
    pub total_items: usize,
    pub avg_importance: f64,
    pub avg_consolidation_age: f64,
    pub tier_distribution: [usize; 3],
}

pub struct DMNConsolidation {
    pub short_term: ThreeTierStore,
    pub medium_term: ThreeTierStore,
    pub long_term: LearnedGraphExecutor,
    pub retrieval: FamiliarityWeightedRetrieval,
    pub config: DmnConfig,
    tick_count: u64,
}

impl DMNConsolidation {
    pub fn new(config: DmnConfig) -> Self {
        let graph_lr = config.graph_learning_rate;
        Self {
            short_term: ThreeTierStore::new(config.short_term_capacity, MemoryTier::ShortTerm),
            medium_term: ThreeTierStore::new(
                config.medium_term_capacity,
                MemoryTier::MediumTerm,
            ),
            long_term: LearnedGraphExecutor::new(graph_lr),
            retrieval: FamiliarityWeightedRetrieval::new(0.3, 0.2),
            config,
            tick_count: 0,
        }
    }

    pub fn encode(&mut self, content: String, importance: f64) {
        self.short_term.push(content, importance);
    }

    pub fn consolidate(&mut self) -> ConsolidationReport {
        let short_term_before = self.short_term.len();
        let medium_term_before = self.medium_term.len();
        let long_term_before = self.long_term.node_count();

        let short_candidates = self
            .short_term
            .consolidate_candidates(self.config.rehearsal_threshold);
        let short_ids: Vec<usize> = short_candidates.iter().map(|c| c.id).collect();
        for id in &short_ids {
            if let Some(item) = self.short_term.remove(*id) {
                let mut new_item = item.clone();
                new_item.tier = MemoryTier::MediumTerm;
                new_item.consolidation_age += 1;
                self.medium_term.push(new_item.content, new_item.importance);
            }
        }

        let medium_candidates = self.medium_term.consolidate_candidates(0.8);
        let medium_ids: Vec<usize> = medium_candidates.iter().map(|c| c.id).collect();
        for id in &medium_ids {
            if let Some(item) = self.medium_term.remove(*id) {
                let mut new_item = item.clone();
                new_item.tier = MemoryTier::LongTerm;
                new_item.consolidation_age += 1;
                self.long_term.add_node(new_item);
            }
        }

        let mut graph_edges_added = 0;
        let long_ids: Vec<usize> = self.long_term.nodes.keys().copied().collect();
        for i in 0..long_ids.len() {
            for j in (i + 1)..long_ids.len() {
                let a = long_ids[i];
                let b = long_ids[j];
                if let (Some(node_a), Some(node_b)) =
                    (self.long_term.nodes.get(&a), self.long_term.nodes.get(&b))
                {
                    let overlap = FamiliarityWeightedRetrieval::keyword_overlap(
                        &node_a.content,
                        &node_b.content,
                    );
                    if overlap > 0.0 {
                        let existing = self
                            .long_term
                            .graph
                            .get(&a)
                            .and_then(|edges| edges.iter().find(|(t, _)| *t == b));
                        if existing.is_none() {
                            self.long_term.add_edge(a, b, overlap);
                            graph_edges_added += 1;
                        }
                    }
                }
            }
        }

        self.retrieval.update_familiarity(
            &MemoryItem::new(0, "consolidation tick".to_string(), 0.0, MemoryTier::ShortTerm),
        );

        let short_term_after = self.short_term.len();
        let medium_term_after = self.medium_term.len();
        let long_term_after = self.long_term.node_count();
        let items_consolidated = short_ids.len() + medium_ids.len();

        ConsolidationReport {
            short_term_before,
            short_term_after,
            medium_term_before,
            medium_term_after,
            long_term_before,
            long_term_after,
            items_consolidated,
            items_rehearsed: 0,
            items_forgotten: 0,
            graph_edges_added,
        }
    }

    pub fn rehearse(&mut self, id: usize) {
        if self.short_term.get(id).is_some() {
            self.short_term.rehearse(id);
        } else if self.medium_term.get(id).is_some() {
            self.medium_term.rehearse(id);
        } else if self.long_term.nodes.contains_key(&id) {
            if let Some(node) = self.long_term.nodes.get_mut(&id) {
                node.access_count += 1;
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                node.last_accessed = now;
                node.importance = (node.importance + 0.05).max(0.0).min(1.0);
            }
        }
    }

    pub fn recall(&self, query: &str, max_results: usize) -> Vec<MemoryItem> {
        let mut all_items: Vec<MemoryItem> = Vec::new();
        for item in self.short_term.items_slice().iter() {
            all_items.push(item.clone());
        }
        for item in self.medium_term.items_slice().iter() {
            all_items.push(item.clone());
        }
        for item in self.long_term.nodes.values() {
            all_items.push(item.clone());
        }

        let indices = self.retrieval.search(&all_items, query, max_results);
        indices.into_iter().map(|(i, _)| all_items[i].clone()).collect()
    }

    pub fn forget(&mut self, threshold: f64) {
        let forget_ids = self.short_term.forget_candidates(threshold);
        for id in forget_ids {
            self.short_term.remove(id);
        }
        let forget_ids = self.medium_term.forget_candidates(threshold);
        for id in forget_ids {
            self.medium_term.remove(id);
        }
    }

    pub fn stats(&self) -> DmnStats {
        let mut all_items: Vec<&MemoryItem> = Vec::new();
        for item in self.short_term.items_slice().iter() {
            all_items.push(item);
        }
        for item in self.medium_term.items_slice().iter() {
            all_items.push(item);
        }
        for item in self.long_term.nodes.values() {
            all_items.push(item);
        }

        let total_items = all_items.len();
        let avg_importance = if total_items > 0 {
            all_items.iter().map(|i| i.importance).sum::<f64>() / total_items as f64
        } else {
            0.0
        };
        let avg_consolidation_age = if total_items > 0 {
            all_items.iter().map(|i| i.consolidation_age).sum::<u64>() as f64 / total_items as f64
        } else {
            0.0
        };
        let st = self.short_term.len();
        let mt = self.medium_term.len();
        let lt = self.long_term.node_count();

        DmnStats {
            total_items,
            avg_importance,
            avg_consolidation_age,
            tier_distribution: [st, mt, lt],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let config = DmnConfig::default();
        let mut dmn = DMNConsolidation::new(config);
        dmn.encode("test memory".to_string(), 0.7);
        assert_eq!(dmn.short_term.len(), 1);
    }

    #[test]
    fn test_consolidation_short_to_medium() {
        let config = DmnConfig {
            short_term_capacity: 10,
            medium_term_capacity: 10,
            rehearsal_threshold: 0.5,
            ..Default::default()
        };
        let mut dmn = DMNConsolidation::new(config);
        dmn.encode("important".to_string(), 0.9);
        dmn.encode("trivial".to_string(), 0.1);
        let report = dmn.consolidate();
        assert_eq!(report.items_consolidated, 1);
        assert_eq!(dmn.short_term.len(), 1);
        assert_eq!(dmn.medium_term.len(), 1);
    }

    #[test]
    fn test_consolidation_medium_to_long() {
        let config = DmnConfig {
            short_term_capacity: 10,
            medium_term_capacity: 10,
            rehearsal_threshold: 0.0,
            ..Default::default()
        };
        let mut dmn = DMNConsolidation::new(config);
        dmn.encode("very important".to_string(), 0.9);
        dmn.consolidate();
        dmn.medium_term.push("long term bound".to_string(), 0.9);
        let report = dmn.consolidate();
        assert!(report.items_consolidated >= 1);
    }

    #[test]
    fn test_recall_across_tiers() {
        let config = DmnConfig::default();
        let mut dmn = DMNConsolidation::new(config);
        dmn.encode("rust language".to_string(), 0.9);
        dmn.medium_term.push("systems programming".to_string(), 0.9);
        let results = dmn.recall("rust", 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_forget() {
        let config = DmnConfig::default();
        let mut dmn = DMNConsolidation::new(config);
        dmn.encode("keep me".to_string(), 0.9);
        dmn.encode("forget me".to_string(), 0.1);
        dmn.forget(0.5);
        assert_eq!(dmn.short_term.len(), 1);
        assert_eq!(dmn.short_term.get(1).unwrap().content, "keep me");
    }

    #[test]
    fn test_stats() {
        let config = DmnConfig::default();
        let mut dmn = DMNConsolidation::new(config);
        dmn.encode("first".to_string(), 0.8);
        dmn.encode("second".to_string(), 0.6);
        let stats = dmn.stats();
        assert_eq!(stats.total_items, 2);
        assert!((stats.avg_importance - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_rehearse_in_any_tier() {
        let mut dmn = DMNConsolidation::new(DmnConfig::default());
        let id = dmn.short_term.push("rehearsal test".to_string(), 0.5);
        let before = dmn.short_term.get(id).unwrap().importance;
        dmn.rehearse(id);
        let after = dmn.short_term.get(id).unwrap().importance;
        assert!((after - before - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_consolidation_report_counts() {
        let config = DmnConfig {
            short_term_capacity: 10,
            medium_term_capacity: 10,
            rehearsal_threshold: 0.0,
            ..Default::default()
        };
        let mut dmn = DMNConsolidation::new(config);
        dmn.encode("a".to_string(), 0.9);
        dmn.encode("b".to_string(), 0.9);
        let report = dmn.consolidate();
        assert!(report.short_term_before >= 2);
        assert_eq!(report.items_consolidated, 2);
    }

    #[test]
    fn test_empty_stats() {
        let dmn = DMNConsolidation::new(DmnConfig::default());
        let stats = dmn.stats();
        assert_eq!(stats.total_items, 0);
        assert!((stats.avg_importance - 0.0).abs() < 1e-6);
    }
}
