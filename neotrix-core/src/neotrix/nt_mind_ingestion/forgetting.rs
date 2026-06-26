/// Forgetting strategy: LRU + importance weighting + decay curves + auto-archival

/// Forgetting configuration
#[derive(Debug, Clone)]
pub struct ForgettingConfig {
    pub max_entries: usize,
    pub soft_threshold: usize,     // when to start decaying
    pub decay_rate: f64,           // 0.0..1.0 per cycle
    pub importance_threshold: f64, // entries below this are candidates
}

impl Default for ForgettingConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            soft_threshold: 8000,
            decay_rate: 0.05,
            importance_threshold: 0.3,
        }
    }
}

/// A knowledge entry with metadata for forgetting decisions
#[derive(Debug, Clone)]
pub struct KnowledgeEntry {
    pub id: String,
    pub content: String,
    pub importance: f64, // 0.0..1.0
    pub access_count: u64,
    pub last_accessed: u64, // iteration number
    pub age: u64,           // iterations since creation
    pub vsa: Option<crate::core::nt_core_hcube::VsaVector<4096>>,
}

impl KnowledgeEntry {
    pub fn new(id: &str, content: &str, importance: f64, current_iteration: u64) -> Self {
        Self {
            id: id.to_string(),
            content: content.to_string(),
            importance: importance.clamp(0.0, 1.0),
            access_count: 1,
            last_accessed: current_iteration,
            age: 0,
            vsa: None,
        }
    }

    pub fn record_access(&mut self, current_iteration: u64) {
        self.access_count += 1;
        self.last_accessed = current_iteration;
    }
}

/// Compute decay factor based on LRU, age, and importance
pub fn compute_decay_factor(entry: &KnowledgeEntry, config: &ForgettingConfig) -> f64 {
    let importance_factor = 1.0 - entry.importance;
    let access_factor = 1.0 / (1.0 + (entry.access_count as f64).ln());
    let age_factor = (entry.age as f64 * config.decay_rate).min(1.0);
    (importance_factor * 0.5 + access_factor * 0.3 + age_factor * 0.2).clamp(0.0, 1.0)
}

/// Select candidates for forgetting (entries with high decay factor)
pub fn select_forget_candidates(
    entries: &[KnowledgeEntry],
    config: &ForgettingConfig,
) -> Vec<usize> {
    entries
        .iter()
        .enumerate()
        .filter(|(_, e)| compute_decay_factor(e, config) > config.importance_threshold)
        .map(|(i, _)| i)
        .collect()
}

/// Determine if archival is needed based on total entry count
pub fn archival_needed(entry_count: usize, config: &ForgettingConfig) -> Option<usize> {
    if entry_count > config.max_entries {
        Some(entry_count - config.soft_threshold)
    } else if entry_count > config.soft_threshold {
        Some((entry_count - config.soft_threshold) / 2)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(id: &str, importance: f64, access_count: u64, age: u64) -> KnowledgeEntry {
        KnowledgeEntry {
            id: id.to_string(),
            content: format!("content_{}", id),
            importance,
            access_count,
            last_accessed: 100,
            age,
            vsa: None,
        }
    }

    #[test]
    fn test_new_entry_defaults() {
        let e = KnowledgeEntry::new("test", "some content", 0.8, 42);
        assert_eq!(e.id, "test");
        assert_eq!(e.access_count, 1);
        assert_eq!(e.last_accessed, 42);
    }

    #[test]
    fn test_record_access_increments() {
        let mut e = KnowledgeEntry::new("t", "c", 0.5, 10);
        e.record_access(20);
        assert_eq!(e.access_count, 2);
        assert_eq!(e.last_accessed, 20);
    }

    #[test]
    fn test_decay_factor_high_importance_low() {
        let important = sample_entry("imp", 0.9, 100, 0);
        let trivial = sample_entry("triv", 0.1, 1, 100);
        let config = ForgettingConfig::default();
        let imp_decay = compute_decay_factor(&important, &config);
        let triv_decay = compute_decay_factor(&trivial, &config);
        assert!(
            imp_decay < triv_decay,
            "important entries should decay slower"
        );
    }

    #[test]
    fn test_decay_factor_bounds() {
        let e = sample_entry("t", 0.5, 5, 10);
        let config = ForgettingConfig::default();
        let factor = compute_decay_factor(&e, &config);
        assert!(factor >= 0.0);
        assert!(factor <= 1.0);
    }

    #[test]
    fn test_select_candidates() {
        let entries = vec![
            sample_entry("keep", 0.9, 100, 0),
            sample_entry("forget", 0.1, 1, 100),
        ];
        let config = ForgettingConfig::default();
        let candidates = select_forget_candidates(&entries, &config);
        assert_eq!(candidates, vec![1]); // only the low-importance one
    }

    #[test]
    fn test_archival_needed_below_threshold() {
        let config = ForgettingConfig::default();
        assert_eq!(archival_needed(100, &config), None);
    }

    #[test]
    fn test_archival_needed_above_soft() {
        let config = ForgettingConfig::default();
        let to_archive = archival_needed(9000, &config);
        assert!(to_archive.is_some());
        assert_eq!(to_archive.unwrap(), 500);
    }

    #[test]
    fn test_archival_needed_above_max() {
        let config = ForgettingConfig::default();
        let to_archive = archival_needed(12000, &config);
        assert!(to_archive.is_some());
        assert_eq!(to_archive.unwrap(), 4000);
    }
}
