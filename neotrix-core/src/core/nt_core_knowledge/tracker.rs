use crate::core::nt_core_knowledge::KnowledgeSource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Records a single absorption event: which source, when, and the applied weight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorptionRecord {
    pub source: KnowledgeSource,
    pub timestamp: u64,
    pub weight: f64,
}

/// Tracks when a source was last accessed and how many times.
#[derive(Debug, Clone)]
pub struct SourceAccessRecord {
    pub last_accessed: Instant,
    pub access_count: usize,
}

/// Monitors access frequency for knowledge sources and prunes cold ones.
#[derive(Debug, Clone)]
pub struct SourceAccessTracker {
    records: HashMap<String, SourceAccessRecord>,
    decay_threshold: usize,
}

impl Default for SourceAccessTracker {
    fn default() -> Self {
        Self {
            records: HashMap::new(),
            decay_threshold: 3,
        }
    }
}

impl SourceAccessTracker {
    pub fn new(decay_threshold: usize) -> Self {
        Self {
            records: HashMap::new(),
            decay_threshold,
        }
    }

    /// Records one access to the given source, updating its timestamp and count.
    pub fn record_access(&mut self, source: &KnowledgeSource) {
        let key = format!("{:?}", source);
        let entry = self.records.entry(key).or_insert(SourceAccessRecord {
            last_accessed: Instant::now(),
            access_count: 0,
        });
        entry.last_accessed = Instant::now();
        entry.access_count += 1;
    }

    /// Returns true if the source has been accessed at least `decay_threshold` times.
    pub fn is_hot(&self, source: &KnowledgeSource) -> bool {
        let key = format!("{:?}", source);
        self.records
            .get(&key)
            .is_some_and(|r| r.access_count >= self.decay_threshold)
    }

    /// Returns sources that have been accessed fewer than `min_accesses` times.
    pub fn prune_cold(&self, min_accesses: usize) -> Vec<KnowledgeSource> {
        KnowledgeSource::all()
            .into_iter()
            .filter(|s| {
                let key = format!("{:?}", s);
                self.records
                    .get(&key)
                    .is_none_or(|r| r.access_count < min_accesses)
            })
            .collect()
    }

    /// Number of times a source has been accessed since tracking began.
    pub fn access_count(&self, source: &KnowledgeSource) -> usize {
        let key = format!("{:?}", source);
        self.records.get(&key).map_or(0, |r| r.access_count)
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use crate::core::nt_core_knowledge::*;

    #[test]
    fn test_record_access_increments_count() {
        let mut tracker = SourceAccessTracker::new(3);
        let source = KnowledgeSource::HeroUI;
        assert_eq!(tracker.access_count(&source), 0);
        tracker.record_access(&source);
        assert_eq!(tracker.access_count(&source), 1);
        tracker.record_access(&source);
        assert_eq!(tracker.access_count(&source), 2);
    }

    #[test]
    fn test_is_hot_after_threshold() {
        let mut tracker = SourceAccessTracker::new(2);
        let source = KnowledgeSource::MemOS;
        assert!(!tracker.is_hot(&source));
        tracker.record_access(&source);
        assert!(!tracker.is_hot(&source));
        tracker.record_access(&source);
        assert!(tracker.is_hot(&source));
        tracker.record_access(&source);
        assert!(tracker.is_hot(&source));
    }

    #[test]
    fn test_is_hot_false_for_never_accessed() {
        let tracker = SourceAccessTracker::new(1);
        let source = KnowledgeSource::BaseUI;
        assert!(!tracker.is_hot(&source));
    }

    #[test]
    fn test_prune_cold_returns_sources_below_threshold() {
        let mut tracker = SourceAccessTracker::new(5);
        let hot = KnowledgeSource::HeroUI;
        let cold = KnowledgeSource::BaseUI;
        for _ in 0..10 {
            tracker.record_access(&hot);
        }
        let cold_sources = tracker.prune_cold(5);
        assert!(cold_sources.contains(&cold));
        assert!(!cold_sources.contains(&hot));
    }

    #[test]
    fn test_sort_by_access_orders_correctly() {
        let mut tracker = SourceAccessTracker::new(1);
        let a = KnowledgeSource::HeroUI;
        let b = KnowledgeSource::BaseUI;
        let c = KnowledgeSource::ArcUI;
        tracker.record_access(&a);
        tracker.record_access(&a);
        tracker.record_access(&b);
        let sources = vec![b, a, c];
        let sorted = KnowledgeSource::sort_by_access(&sources, &tracker);
        assert_eq!(*sorted[0], a, "most accessed should be first");
        assert_eq!(*sorted[1], b, "second most accessed should be second");
        assert_eq!(*sorted[2], c, "never accessed should be last");
    }

    #[test]
    fn test_new_tracker_empty_records() {
        let tracker = SourceAccessTracker::new(10);
        assert_eq!(tracker.access_count(&KnowledgeSource::HeroUI), 0);
        assert!(!tracker.is_hot(&KnowledgeSource::HeroUI));
    }
}
