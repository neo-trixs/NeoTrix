use std::collections::VecDeque;

/// Quality dimensions for a single absorption event.
#[derive(Debug, Clone)]
pub struct AbsorptionRecord {
    /// Source label (paper title / repo name / domain)
    pub source: String,
    /// Cycle when absorption occurred
    pub cycle: u64,
    /// Understanding depth (1-5): 1=skim, 2=read, 3=understand, 4=absorb, 5=master
    pub depth: u8,
    /// Number of upstream/downstream mappings generated
    pub mapping_count: u32,
    /// Number of defects or gaps identified
    pub defect_count: u32,
    /// Implementation completion rate (0.0-1.0)
    pub completion_rate: f64,
    /// Cross-session re-access count
    pub reaccess_count: u32,
    /// Last cycle this record was accessed
    pub last_accessed_cycle: u64,
}

/// Aggregate quality metrics across all absorption events.
#[derive(Debug, Clone)]
pub struct AbsorptionQualityReport {
    /// Total absorption events recorded
    pub total_events: usize,
    /// Average depth across all events
    pub avg_depth: f64,
    /// Average mapping count
    pub avg_mappings: f64,
    /// Average completion rate
    pub avg_completion_rate: f64,
    /// Retention rate (events with reaccess_count > 0 / total)
    pub retention_rate: f64,
    /// Most frequent source domain
    pub top_domain: String,
}

/// Meta-level tracker for external knowledge absorption quality.
///
/// Addresses CXV.5 (absorption meta-defect): provides a quantitative
/// quality metric and cross-session retention tracking.
pub struct AbsorptionQualityTracker {
    records: VecDeque<AbsorptionRecord>,
    max_records: usize,
}

impl Default for AbsorptionQualityTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl AbsorptionQualityTracker {
    pub fn new() -> Self {
        Self {
            records: VecDeque::with_capacity(128),
            max_records: 500,
        }
    }

    pub fn with_max(max: usize) -> Self {
        Self {
            records: VecDeque::with_capacity(max.min(5000)),
            max_records: max.min(5000),
        }
    }

    /// Record a new absorption event with quality dimensions.
    pub fn record(
        &mut self,
        source: &str,
        cycle: u64,
        depth: u8,
        mapping_count: u32,
        defect_count: u32,
        completion_rate: f64,
    ) {
        let depth = depth.clamp(1, 5);
        let completion_rate = completion_rate.clamp(0.0, 1.0);
        if self.records.len() >= self.max_records {
            self.records.pop_front();
        }
        self.records.push_back(AbsorptionRecord {
            source: source.to_string(),
            cycle,
            depth,
            mapping_count,
            defect_count,
            completion_rate,
            reaccess_count: 0,
            last_accessed_cycle: cycle,
        });
    }

    /// Mark an existing source as re-accessed (tracking cross-session retention).
    pub fn mark_reaccessed(&mut self, source: &str, cycle: u64) -> bool {
        if let Some(rec) = self.records.iter_mut().rev().find(|r| r.source == source) {
            rec.reaccess_count += 1;
            rec.last_accessed_cycle = cycle;
            true
        } else {
            false
        }
    }

    /// Query whether a source was absorbed before.
    pub fn has_source(&self, source: &str) -> bool {
        self.records.iter().any(|r| r.source == source)
    }

    /// Generate aggregate report for meta-cognitive insight.
    pub fn report(&self) -> AbsorptionQualityReport {
        let total = self.records.len();
        if total == 0 {
            return AbsorptionQualityReport {
                total_events: 0,
                avg_depth: 0.0,
                avg_mappings: 0.0,
                avg_completion_rate: 0.0,
                retention_rate: 0.0,
                top_domain: String::new(),
            };
        }

        let avg_depth = self.records.iter().map(|r| r.depth as f64).sum::<f64>() / total as f64;
        let avg_mappings = self
            .records
            .iter()
            .map(|r| r.mapping_count as f64)
            .sum::<f64>()
            / total as f64;
        let avg_completion =
            self.records.iter().map(|r| r.completion_rate).sum::<f64>() / total as f64;
        let retained = self.records.iter().filter(|r| r.reaccess_count > 0).count();
        let retention_rate = retained as f64 / total as f64;

        // Most common source prefix as pseudo-top-domain
        let mut domain_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for rec in &self.records {
            let domain = rec
                .source
                .split('/')
                .next()
                .unwrap_or(&rec.source)
                .to_string();
            *domain_counts.entry(domain).or_insert(0) += 1;
        }
        let top_domain = domain_counts
            .into_iter()
            .max_by_key(|(_, c)| *c)
            .map(|(d, _)| d)
            .unwrap_or_default();

        AbsorptionQualityReport {
            total_events: total,
            avg_depth,
            avg_mappings,
            avg_completion_rate: avg_completion,
            retention_rate,
            top_domain,
        }
    }

    /// Number of stored records.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tracker_empty() {
        let t = AbsorptionQualityTracker::new();
        assert!(t.is_empty());
        assert_eq!(t.len(), 0);
    }

    #[test]
    fn test_record_absorption() {
        let mut t = AbsorptionQualityTracker::new();
        t.record("paper:attention-as-binding", 100, 4, 3, 2, 0.8);
        assert_eq!(t.len(), 1);
        assert!(t.has_source("paper:attention-as-binding"));
    }

    #[test]
    fn test_reaccess_tracking() {
        let mut t = AbsorptionQualityTracker::new();
        t.record("repo:mem0", 100, 3, 5, 1, 0.6);
        assert!(t.mark_reaccessed("repo:mem0", 200));
        let r = t.report();
        assert!(r.retention_rate > 0.0);
    }

    #[test]
    fn test_report_empty() {
        let t = AbsorptionQualityTracker::new();
        let r = t.report();
        assert_eq!(r.total_events, 0);
        assert!(r.avg_completion_rate.abs() < 1e-10);
    }

    #[test]
    fn test_report_aggregates() {
        let mut t = AbsorptionQualityTracker::new();
        t.record("alpha:paper1", 100, 4, 3, 2, 0.8);
        t.record("alpha:paper2", 101, 3, 1, 0, 0.5);
        t.record("github:repo1", 102, 5, 7, 3, 0.9);
        let r = t.report();
        assert_eq!(r.total_events, 3);
        assert!((r.avg_depth - 4.0).abs() < 0.1);
        assert!((r.avg_mappings - 3.666).abs() < 0.1);
    }

    #[test]
    fn test_depth_clamped() {
        let mut t = AbsorptionQualityTracker::new();
        t.record("test", 1, 10, 0, 0, 1.0);
        let r = t.report();
        assert!(
            (r.avg_depth - 5.0).abs() < 0.01,
            "depth must be clamped to 5"
        );
    }

    #[test]
    fn test_completion_clamped() {
        let mut t = AbsorptionQualityTracker::new();
        t.record("test", 1, 3, 0, 0, 1.5);
        let r = t.report();
        assert!((r.avg_completion_rate - 1.0).abs() < 0.01);
    }
}
