#![forbid(unsafe_code)]

//! Temporal scoring with exponential decay (R-P121).
//!
//! More recent entries receive higher scores via exponential decay:
//!   score *= exp(-lambda * days_since_entry)

use std::f64::consts::E;

pub struct TemporalScorer {
    /// Decay rate (lambda). Higher = steeper decay.
    pub decay_rate: f64,
}

impl TemporalScorer {
    pub fn new(decay_rate: f64) -> Self {
        Self { decay_rate }
    }

    /// Apply temporal boost to a score based on entry age.
    ///
    /// - `base_score`: the original relevance score
    /// - `entry_timestamp`: Unix seconds when the entry was created
    /// - `query_time`: Unix seconds of the query
    pub fn temporal_boost(&self, base_score: f64, entry_timestamp: i64, query_time: i64) -> f64 {
        let age_days = ((query_time - entry_timestamp) as f64) / 86400.0;
        if age_days <= 0.0 {
            return base_score;
        }
        let decay = E.powf(-self.decay_rate * age_days);
        base_score * decay
    }

    /// Rank a list of (id, score, timestamp) entries by temporally-boosted score.
    pub fn rank(&self, entries: &[(String, f64, i64)], query_time: i64) -> Vec<(String, f64)> {
        let mut scored: Vec<(String, f64)> = entries
            .iter()
            .map(|(id, score, ts)| {
                let boosted = self.temporal_boost(*score, *ts, query_time);
                (id.clone(), boosted)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
    }
}

impl Default for TemporalScorer {
    /// Default decay rate of 0.01 — ~70-day half-life.
    fn default() -> Self {
        Self { decay_rate: 0.01 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_decay_for_same_time() {
        let scorer = TemporalScorer::default();
        let boosted = scorer.temporal_boost(1.0, 1_000_000, 1_000_000);
        assert!((boosted - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_recent_entry_higher() {
        let scorer = TemporalScorer::default();
        let recent = scorer.temporal_boost(1.0, 1_000_000, 1_001_000);
        let old = scorer.temporal_boost(1.0, 1_000_000, 1_100_000);
        assert!(recent > old);
    }

    #[test]
    fn test_zero_score() {
        let scorer = TemporalScorer::default();
        let boosted = scorer.temporal_boost(0.0, 1_000_000, 1_001_000);
        assert!(boosted.abs() < 1e-6);
    }

    #[test]
    fn test_rank_orders_by_recency() {
        let scorer = TemporalScorer::default();
        let entries = vec![
            ("a".into(), 1.0, 1_000_000),
            ("b".into(), 1.0, 1_000_100),
            ("c".into(), 1.0, 1_000_200),
        ];
        let ranked = scorer.rank(&entries, 1_000_300);
        assert_eq!(ranked[0].0, "c");
        assert_eq!(ranked[1].0, "b");
        assert_eq!(ranked[2].0, "a");
    }

    #[test]
    fn test_higher_decay_faster() {
        let fast = TemporalScorer::new(0.1);
        let slow = TemporalScorer::new(0.001);
        let ts = 1_000_000;
        let qt = 1_100_000;
        let f = fast.temporal_boost(1.0, ts, qt);
        let s = slow.temporal_boost(1.0, ts, qt);
        assert!(f < s);
    }

    #[test]
    fn test_temporal_boost_preserves_direction() {
        let scorer = TemporalScorer::default();
        let boosted = scorer.temporal_boost(0.5, 1_000_000, 1_001_000);
        assert!(boosted > 0.0 && boosted < 0.5);
    }

    #[test]
    fn test_rank_empty_entries() {
        let scorer = TemporalScorer::default();
        let ranked = scorer.rank(&[], 1_000_000);
        assert!(ranked.is_empty());
    }

    #[test]
    fn test_rank_equal_scores_sorted_by_recency() {
        let scorer = TemporalScorer::default();
        let entries = vec![
            ("old".into(), 1.0, 1_000_000),
            ("new".into(), 1.0, 1_000_200),
        ];
        let ranked = scorer.rank(&entries, 1_000_300);
        assert_eq!(ranked[0].0, "new");
        assert_eq!(ranked[1].0, "old");
    }

    #[test]
    fn test_default_decay_rate() {
        let scorer = TemporalScorer::default();
        assert!((scorer.decay_rate - 0.01).abs() < 1e-6);
    }
}
