#![forbid(unsafe_code)]

//! Episodic Store — compressed summaries, max 500 entries.
//!
//! Tier 4 of the five-tier cascade. Holds compressed episodic summaries
//! derived from short-term memories that share temporal or semantic proximity.

use std::collections::BTreeMap;
use std::time::Instant;

/// A compressed episodic memory summary.
#[derive(Debug, Clone)]
pub struct Episode {
    pub id: u64,
    pub summary: String,
    pub created_at: Instant,
    pub source_ids: Vec<u64>,
    pub importance: f64,
    pub tags: Vec<String>,
}

/// EpisodicStore: compressed summaries, max 500 entries.
#[derive(Debug)]
pub struct EpisodicStore {
    episodes: BTreeMap<u64, Episode>,
    max_capacity: usize,
    next_id: u64,
}

impl Default for EpisodicStore {
    fn default() -> Self {
        Self {
            episodes: BTreeMap::new(),
            max_capacity: 500,
            next_id: 1,
        }
    }
}

impl EpisodicStore {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            episodes: BTreeMap::new(),
            max_capacity,
            next_id: 1,
        }
    }

    /// Compress and store a new episode. Evicts lowest-importance if at capacity.
    pub fn store(
        &mut self,
        summary: String,
        source_ids: Vec<u64>,
        importance: f64,
        tags: Vec<String>,
    ) -> u64 {
        if self.episodes.len() >= self.max_capacity {
            self.evict_lowest_importance();
        }

        let id = self.next_id;
        self.next_id += 1;

        self.episodes.insert(
            id,
            Episode {
                id,
                summary,
                created_at: Instant::now(),
                source_ids,
                importance,
                tags,
            },
        );
        id
    }

    /// Recall an episode by id.
    pub fn recall(&self, id: u64) -> Option<&Episode> {
        self.episodes.get(&id)
    }

    /// Search episodes by tag.
    pub fn search_by_tag(&self, tag: &str) -> Vec<&Episode> {
        self.episodes
            .values()
            .filter(|e| e.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Return all episodes sorted by importance descending.
    pub fn all_by_importance(&self) -> Vec<&Episode> {
        let mut items: Vec<&Episode> = self.episodes.values().collect();
        items.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
        items
    }

    /// Find episodes whose summary contains the query string.
    pub fn search_by_content(&self, query: &str) -> Vec<&Episode> {
        self.episodes
            .values()
            .filter(|e| e.summary.contains(query))
            .collect()
    }

    /// Remove an episode by id.
    pub fn remove(&mut self, id: u64) -> Option<Episode> {
        self.episodes.remove(&id)
    }

    pub fn len(&self) -> usize {
        self.episodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.episodes.is_empty()
    }

    fn evict_lowest_importance(&mut self) {
        if let Some((min_id, _)) = self
            .episodes
            .iter()
            .min_by(|a, b| a.1.importance.partial_cmp(&b.1.importance).unwrap())
        {
            let min_id = *min_id;
            self.episodes.remove(&min_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_recall() {
        let mut es = EpisodicStore::default();
        let id = es.store("summary".into(), vec![1, 2], 0.8, vec!["tag".into()]);
        let ep = es.recall(id).unwrap();
        assert_eq!(ep.summary, "summary");
        assert_eq!(ep.source_ids, vec![1, 2]);
    }

    #[test]
    fn search_by_tag() {
        let mut es = EpisodicStore::default();
        es.store("a".into(), vec![], 0.5, vec!["rust".into()]);
        es.store("b".into(), vec![], 0.5, vec!["python".into()]);
        let results = es.search_by_tag("rust");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn eviction_of_lowest_importance() {
        let mut es = EpisodicStore::new(2);
        es.store("low".into(), vec![], 0.1, vec![]);
        es.store("mid".into(), vec![], 0.5, vec![]);
        es.store("high".into(), vec![], 0.9, vec![]);
        assert_eq!(es.len(), 2);
        assert!(es.search_by_content("low").is_empty());
        assert!(!es.search_by_content("high").is_empty());
    }
}
