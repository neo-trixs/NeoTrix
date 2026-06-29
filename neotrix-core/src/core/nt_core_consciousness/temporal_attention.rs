use std::collections::HashSet;

/// Maximum number of seen items to track before pruning (prevents unbounded growth)
const MAX_SEEN_ITEMS: usize = 10_000;

/// Configuration for temporal attention bias
#[derive(Debug, Clone)]
pub struct TemporalAttentionConfig {
    pub lambda: f64,
    pub novelty_gain: f64,
    pub expectation_gain: f64,
}

impl Default for TemporalAttentionConfig {
    fn default() -> Self {
        Self {
            lambda: 0.1,
            novelty_gain: 0.3,
            expectation_gain: 0.2,
        }
    }
}

/// A single attention item with temporal metadata
#[derive(Debug, Clone)]
pub struct AttentionItem {
    pub id: String,
    pub content_hash: u64,
    pub timestamp: u64,
    pub base_importance: f64,
    pub seen_count: u64,
}

/// Temporal attention bias engine
pub struct TemporalAttentionBias {
    config: TemporalAttentionConfig,
    seen_items: HashSet<u64>,
    seen_timestamps: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct AttentionStats {
    pub total_seen: usize,
    pub novelty_ratio: f64,
    pub mean_decay: f64,
}

impl TemporalAttentionBias {
    pub fn new(config: TemporalAttentionConfig) -> Self {
        Self {
            config,
            seen_items: HashSet::new(),
            seen_timestamps: Vec::new(),
        }
    }

    pub fn attend(&mut self, items: &[AttentionItem], expected_items: &[String]) -> Vec<f64> {
        if items.is_empty() {
            return Vec::new();
        }

        let now = items.iter().map(|i| i.timestamp).max().unwrap_or(0);

        let mut weights: Vec<f64> = items
            .iter()
            .map(|item| {
                let decay = self.time_decay_factor(item, now);
                let novelty = self.novelty_bonus(item);
                let expectation = if expected_items.contains(&item.id) {
                    self.config.expectation_gain
                } else {
                    0.0
                };
                item.base_importance * decay + novelty + expectation
            })
            .collect();

        // Track seen items
        for item in items {
            self.seen_items.insert(item.content_hash);
            self.seen_timestamps.push(item.timestamp);
        }

        // Prevent unbounded growth: discard oldest when watermark reached
        if self.seen_items.len() > MAX_SEEN_ITEMS {
            let threshold = self.seen_timestamps.len().saturating_sub(MAX_SEEN_ITEMS);
            self.seen_timestamps.drain(0..threshold);
            self.seen_items.clear();
            for item in items {
                self.seen_items.insert(item.content_hash);
            }
        }

        let sum: f64 = weights.iter().sum();
        if sum > 0.0 {
            for w in &mut weights {
                *w /= sum;
            }
        }

        weights
    }

    pub fn time_decay_factor(&self, item: &AttentionItem, now: u64) -> f64 {
        (-self.config.lambda * (now.saturating_sub(item.timestamp)) as f64).exp()
    }

    pub fn novelty_bonus(&self, item: &AttentionItem) -> f64 {
        if !self.seen_items.contains(&item.content_hash) {
            self.config.novelty_gain
        } else if item.seen_count == 0 {
            self.config.novelty_gain
        } else {
            self.config.novelty_gain / (1.0 + item.seen_count as f64)
        }
    }

    pub fn reset(&mut self) {
        self.seen_items.clear();
        self.seen_timestamps.clear();
    }

    pub fn stats(&self) -> AttentionStats {
        let total_seen = self.seen_items.len();
        // mean_decay requires items input; report 0.0 when no stats available
        AttentionStats {
            total_seen,
            novelty_ratio: 0.0,
            mean_decay: 0.0,
        }
    }

    pub fn stats_for(&self, items: &[AttentionItem], now: u64) -> AttentionStats {
        let total_seen = self.seen_items.len();
        if items.is_empty() {
            return AttentionStats {
                total_seen,
                novelty_ratio: 0.0,
                mean_decay: 0.0,
            };
        }
        let novel_count = items
            .iter()
            .filter(|i| !self.seen_items.contains(&i.content_hash))
            .count();
        let novelty_ratio = novel_count as f64 / items.len() as f64;
        let mean_decay = items
            .iter()
            .map(|i| self.time_decay_factor(i, now))
            .sum::<f64>()
            / items.len() as f64;
        AttentionStats {
            total_seen,
            novelty_ratio,
            mean_decay,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(id: &str, hash: u64, timestamp: u64, importance: f64, seen: u64) -> AttentionItem {
        AttentionItem {
            id: id.to_string(),
            content_hash: hash,
            timestamp,
            base_importance: importance,
            seen_count: seen,
        }
    }

    #[test]
    fn test_default_config() {
        let config = TemporalAttentionConfig::default();
        assert!((config.lambda - 0.1).abs() < 1e-9);
        assert!((config.novelty_gain - 0.3).abs() < 1e-9);
        assert!((config.expectation_gain - 0.2).abs() < 1e-9);
    }

    #[test]
    fn test_single_item_normalized_to_one() {
        let config = TemporalAttentionConfig::default();
        let mut bias = TemporalAttentionBias::new(config);
        let items = vec![make_item("a", 1, 100, 0.8, 0)];
        let weights = bias.attend(&items, &[]);
        assert_eq!(weights.len(), 1);
        assert!((weights[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_two_identical_items_equal_weights() {
        let config = TemporalAttentionConfig::default();
        let mut bias = TemporalAttentionBias::new(config);
        let items = vec![
            make_item("a", 1, 100, 0.8, 0),
            make_item("b", 2, 100, 0.8, 0),
        ];
        let weights = bias.attend(&items, &[]);
        assert_eq!(weights.len(), 2);
        assert!((weights[0] - weights[1]).abs() < 1e-6);
    }

    #[test]
    fn test_newer_item_gets_higher_weight() {
        let config = TemporalAttentionConfig::default();
        let mut bias = TemporalAttentionBias::new(config);
        let items = vec![
            make_item("old", 1, 50, 1.0, 0),
            make_item("new", 2, 100, 1.0, 0),
        ];
        let weights = bias.attend(&items, &[]);
        assert!(
            weights[1] > weights[0],
            "newer item should have higher weight, got {:?}",
            weights
        );
    }

    #[test]
    fn test_novel_item_gets_bonus() {
        let config = TemporalAttentionConfig::default();
        let mut bias = TemporalAttentionBias::new(config);
        // First pass: both items are novel
        let items = vec![
            make_item("a", 1, 100, 0.5, 0),
            make_item("b", 2, 200, 0.5, 0),
        ];
        let _ = bias.attend(&items, &[]);
        // Second pass: 'a' has been seen (hash 1), 'c' is novel (hash 3)
        let items2 = vec![
            make_item("a", 1, 300, 0.5, 1),
            make_item("c", 3, 300, 0.5, 0),
        ];
        let weights = bias.attend(&items2, &[]);
        assert!(
            weights[1] > weights[0],
            "novel item should get higher weight than seen item"
        );
    }

    #[test]
    fn test_expected_item_gets_pre_activation_bonus() {
        let config = TemporalAttentionConfig::default();
        let mut bias = TemporalAttentionBias::new(config);
        let items = vec![
            make_item("a", 1, 100, 0.5, 0),
            make_item("b", 2, 100, 0.5, 0),
        ];
        let weights = bias.attend(&items, &["b".to_string()]);
        assert!(
            weights[1] > weights[0],
            "expected item should get pre-activation bonus"
        );
    }

    #[test]
    fn test_attend_updates_seen_items() {
        let config = TemporalAttentionConfig::default();
        let mut bias = TemporalAttentionBias::new(config);
        let items = vec![make_item("a", 42, 100, 0.5, 0)];
        let _ = bias.attend(&items, &[]);
        assert!(bias.seen_items.contains(&42));
    }

    #[test]
    fn test_reset_clears_all_state() {
        let config = TemporalAttentionConfig::default();
        let mut bias = TemporalAttentionBias::new(config);
        let items = vec![make_item("a", 1, 100, 0.5, 0)];
        let _ = bias.attend(&items, &[]);
        assert_eq!(bias.seen_items.len(), 1);
        bias.reset();
        assert!(bias.seen_items.is_empty());
        assert!(bias.seen_timestamps.is_empty());
    }

    #[test]
    fn test_stats_returns_correct_counts() {
        let config = TemporalAttentionConfig::default();
        let mut bias = TemporalAttentionBias::new(config);
        let items = vec![
            make_item("a", 1, 100, 0.5, 0),
            make_item("b", 2, 200, 0.5, 0),
        ];
        let now = 200;
        let stats = bias.stats_for(&items, now);
        // before attend: total_seen = 0
        assert_eq!(stats.total_seen, 0);
        let _ = bias.attend(&items, &[]);
        let stats2 = bias.stats_for(&items, now);
        assert_eq!(stats2.total_seen, 2);
    }

    #[test]
    fn test_very_old_item_weight_approaches_zero() {
        let config = TemporalAttentionConfig {
            lambda: 1.0,
            novelty_gain: 0.0,
            expectation_gain: 0.0,
        };
        let mut bias = TemporalAttentionBias::new(config);
        let items = vec![make_item("ancient", 1, 0, 1.0, 0)];
        let weights = bias.attend(&items, &[]);
        // Δt = 100, decay = exp(-1.0 * 100) ≈ 0
        assert!(weights[0] < 1e-30, "very old item weight should approach 0");
    }

    #[test]
    fn test_multiple_calls_accumulate_seen_items() {
        let config = TemporalAttentionConfig::default();
        let mut bias = TemporalAttentionBias::new(config);
        let items1 = vec![make_item("a", 1, 100, 0.5, 0)];
        let items2 = vec![make_item("b", 2, 200, 0.5, 0)];
        let items3 = vec![make_item("c", 3, 300, 0.5, 0)];
        let _ = bias.attend(&items1, &[]);
        let _ = bias.attend(&items2, &[]);
        let _ = bias.attend(&items3, &[]);
        assert_eq!(bias.seen_items.len(), 3);
    }

    #[test]
    fn test_config_changes_affect_weights() {
        let config = TemporalAttentionConfig {
            lambda: 0.5,
            novelty_gain: 0.0,
            expectation_gain: 0.0,
        };
        let mut bias = TemporalAttentionBias::new(config);
        let items = vec![
            make_item("old", 1, 0, 1.0, 0),
            make_item("new", 2, 100, 1.0, 0),
        ];
        let weights = bias.attend(&items, &[]);
        // With lambda=0.5, Δt=100 → decay=exp(-50) ≈ 1.9e-22, Δt=0 → decay=1.0
        assert!(
            (weights[1] - 1.0).abs() < 1e-10,
            "new item should dominate with high decay rate"
        );
        assert!(
            weights[0] < 1e-20,
            "old item should near-zero with high decay"
        );
    }

    #[test]
    fn test_seen_count_diminishing_novelty() {
        let config = TemporalAttentionConfig::default();
        let mut bias = TemporalAttentionBias::new(config.clone());
        // Register hash 1 as seen via first attend
        let items = vec![make_item("a", 1, 100, 0.0, 0)];
        let _ = bias.attend(&items, &[]);

        // Now same hash but with seen_count=5 — should get smaller bonus
        let item_seen_many = make_item("a", 1, 200, 0.0, 5);
        let bonus = bias.novelty_bonus(&item_seen_many);
        let expected = config.novelty_gain / (1.0 + 5.0);
        assert!((bonus - expected).abs() < 1e-9);
    }

    #[test]
    fn test_empty_items_returns_empty() {
        let config = TemporalAttentionConfig::default();
        let mut bias = TemporalAttentionBias::new(config.clone());
        let weights = bias.attend(&[], &[]);
        assert!(weights.is_empty());
    }

    #[test]
    fn test_time_decay_factor_directly() {
        let config = TemporalAttentionConfig {
            lambda: 0.5,
            novelty_gain: 0.0,
            expectation_gain: 0.0,
        };
        let bias = TemporalAttentionBias::new(config);
        let item = make_item("x", 1, 10, 0.5, 0);
        let decay = bias.time_decay_factor(&item, 20);
        let expected = (-0.5 * 10.0_f64).exp();
        assert!((decay - expected).abs() < 1e-9);
    }
}
