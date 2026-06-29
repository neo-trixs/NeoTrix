#[derive(Debug, Clone)]
pub struct TemporalAttentionConfig {
    pub time_decay_rate: f64,
    pub novelty_boost: f64,
    pub expectation_bias: f64,
    pub max_history: usize,
}

impl Default for TemporalAttentionConfig {
    fn default() -> Self {
        Self {
            time_decay_rate: 0.95,
            novelty_boost: 1.5,
            expectation_bias: 0.3,
            max_history: 100,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TemporalEntry {
    pub id: String,
    pub content_vsa: Vec<u8>,
    pub created_cycle: u64,
    pub last_accessed_cycle: u64,
    pub access_count: u64,
    pub base_weight: f64,
    pub metadata: String,
}

#[derive(Debug, Clone)]
pub struct TemporalAttentionState {
    pub entries: Vec<TemporalEntry>,
    pub cycle: u64,
}

#[derive(Debug, Clone)]
pub struct TemporalAttentionBias {
    pub config: TemporalAttentionConfig,
    pub state: TemporalAttentionState,
}

impl TemporalAttentionBias {
    pub fn new(config: TemporalAttentionConfig) -> Self {
        Self {
            config,
            state: TemporalAttentionState {
                entries: Vec::new(),
                cycle: 0,
            },
        }
    }

    pub fn register(&mut self, id: String, content_vsa: Vec<u8>, weight: f64, metadata: String) {
        let is_expected = metadata.contains("expected");
        let effective_weight = if is_expected {
            (weight + self.config.expectation_bias).min(1.0)
        } else {
            weight
        };
        self.state.entries.push(TemporalEntry {
            id,
            content_vsa,
            created_cycle: self.state.cycle,
            last_accessed_cycle: self.state.cycle,
            access_count: 0,
            base_weight: effective_weight.max(0.0).min(1.0),
            metadata,
        });
    }

    pub fn access(&mut self, id: &str) {
        if let Some(entry) = self.state.entries.iter_mut().find(|e| e.id == id) {
            entry.last_accessed_cycle = self.state.cycle;
            entry.access_count += 1;
        }
    }

    pub fn attention_weight(&self, id: &str) -> f64 {
        if let Some(entry) = self.state.entries.iter().find(|e| e.id == id) {
            let age = self.state.cycle.saturating_sub(entry.last_accessed_cycle);
            let decay = self.decay_factor(age);
            let nb = self.novelty_bonus(entry);
            let eb = if entry.metadata.contains("expected") {
                self.config.expectation_bias
            } else {
                0.0
            };
            entry.base_weight * decay + nb + eb
        } else {
            0.0
        }
    }

    pub fn decay_factor(&self, age: u64) -> f64 {
        self.config.time_decay_rate.powi(age as i32)
    }

    pub fn novelty_bonus(&self, entry: &TemporalEntry) -> f64 {
        if entry.access_count == 0 {
            self.config.novelty_boost
        } else {
            0.0
        }
    }

    pub fn prune(&mut self, max_entries: usize) {
        if self.state.entries.len() <= max_entries {
            return;
        }
        let to_remove = self.state.entries.len() - max_entries;
        self.state.entries.sort_by(|a, b| {
            a.last_accessed_cycle
                .cmp(&b.last_accessed_cycle)
                .then_with(|| {
                    a.base_weight
                        .partial_cmp(&b.base_weight)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        self.state.entries.drain(0..to_remove);
    }

    pub fn attention_distribution(&self) -> Vec<(String, f64)> {
        let total: f64 = self
            .state
            .entries
            .iter()
            .map(|e| self.attention_weight(&e.id))
            .sum();
        if total <= 0.0 {
            return self
                .state
                .entries
                .iter()
                .map(|e| (e.id.clone(), 0.0))
                .collect();
        }
        self.state
            .entries
            .iter()
            .map(|e| {
                let raw = self.attention_weight(&e.id);
                (e.id.clone(), raw / total)
            })
            .collect()
    }

    pub fn tick(&mut self) {
        self.state.cycle += 1;
        if self.state.entries.len() > self.config.max_history {
            self.prune(self.config.max_history);
        }
    }

    pub fn clear(&mut self) {
        self.state.entries.clear();
        self.state.cycle = 0;
    }

    pub fn entry_count(&self) -> usize {
        self.state.entries.len()
    }

    pub fn current_cycle(&self) -> u64 {
        self.state.cycle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TemporalAttentionConfig::default();
        assert!((config.time_decay_rate - 0.95).abs() < 1e-6);
        assert!((config.novelty_boost - 1.5).abs() < 1e-6);
        assert!((config.expectation_bias - 0.3).abs() < 1e-6);
        assert_eq!(config.max_history, 100);
    }

    #[test]
    fn test_new_tab_initial_state() {
        let tab = TemporalAttentionBias::new(TemporalAttentionConfig::default());
        assert_eq!(tab.state.entries.len(), 0);
        assert_eq!(tab.state.cycle, 0);
    }

    #[test]
    fn test_registration_adds_entry() {
        let mut tab = TemporalAttentionBias::new(TemporalAttentionConfig::default());
        tab.register("a".to_string(), vec![0u8; 64], 0.5, "".to_string());
        assert_eq!(tab.entry_count(), 1);
        assert_eq!(tab.state.entries[0].id, "a");
        assert_eq!(tab.state.entries[0].access_count, 0);
        assert_eq!(tab.state.entries[0].created_cycle, 0);
    }

    #[test]
    fn test_register_novelty_boost() {
        let config = TemporalAttentionConfig {
            time_decay_rate: 0.95,
            novelty_boost: 2.0,
            expectation_bias: 0.0,
            max_history: 100,
        };
        let mut tab = TemporalAttentionBias::new(config);
        tab.register("a".to_string(), vec![0u8; 64], 0.5, "".to_string());
        // base_weight * decay(0) + novelty_bonus + expectation_bias
        // = 0.5 * 1.0 + 2.0 + 0.0 = 2.5
        let w = tab.attention_weight("a");
        assert!((w - 2.5).abs() < 1e-6, "got {}", w);
    }

    #[test]
    fn test_old_entry_lower_weight() {
        let config = TemporalAttentionConfig {
            time_decay_rate: 0.5,
            novelty_boost: 0.0,
            expectation_bias: 0.0,
            max_history: 100,
        };
        let mut tab = TemporalAttentionBias::new(config);
        tab.register("old".to_string(), vec![0u8; 64], 1.0, "".to_string());
        tab.tick();
        tab.tick();
        tab.tick();
        tab.register("new".to_string(), vec![0u8; 64], 1.0, "".to_string());
        let old_w = tab.attention_weight("old");
        let new_w = tab.attention_weight("new");
        assert!(
            old_w < new_w,
            "old weight {} should be less than new weight {}",
            old_w,
            new_w
        );
    }

    #[test]
    fn test_access_increases_weight() {
        let config = TemporalAttentionConfig {
            time_decay_rate: 0.5,
            novelty_boost: 0.0,
            expectation_bias: 0.0,
            max_history: 100,
        };
        let mut tab = TemporalAttentionBias::new(config);
        tab.register("x".to_string(), vec![0u8; 64], 1.0, "".to_string());
        // entry at cycle 0, last_accessed=0, age=0 => weight = 1.0 * 1.0 = 1.0
        let initial = tab.attention_weight("x");
        assert!((initial - 1.0).abs() < 1e-6);

        tab.tick();
        tab.tick();
        tab.tick();
        // age = 3, decay = 0.5^3 = 0.125, weight = 1.0 * 0.125 = 0.125
        let before = tab.attention_weight("x");
        assert!((before - 0.125).abs() < 1e-6, "got {}", before);

        tab.access("x");
        // last_accessed = 3, age = 0, weight = 1.0 * 1.0 = 1.0
        let after = tab.attention_weight("x");
        assert!(
            after > before,
            "weight after access {} should exceed weight before {}",
            after,
            before
        );
    }

    #[test]
    fn test_prune_removes_oldest() {
        let mut tab = TemporalAttentionBias::new(TemporalAttentionConfig::default());
        tab.register("a".to_string(), vec![0u8; 64], 0.5, "".to_string());
        tab.tick();
        tab.register("b".to_string(), vec![0u8; 64], 0.5, "".to_string());
        tab.tick();
        tab.register("c".to_string(), vec![0u8; 64], 0.5, "".to_string());
        assert_eq!(tab.entry_count(), 3);
        tab.prune(2);
        assert_eq!(tab.entry_count(), 2);
        // 'a' was created earliest and never accessed -> should be removed
        assert!(tab.state.entries.iter().any(|e| e.id == "b"));
        assert!(tab.state.entries.iter().any(|e| e.id == "c"));
    }

    #[test]
    fn test_prune_noop_under_capacity() {
        let mut tab = TemporalAttentionBias::new(TemporalAttentionConfig::default());
        tab.register("a".to_string(), vec![0u8; 64], 0.5, "".to_string());
        tab.register("b".to_string(), vec![0u8; 64], 0.5, "".to_string());
        tab.prune(10);
        assert_eq!(tab.entry_count(), 2);
    }

    #[test]
    fn test_attention_distribution_sums_to_one() {
        let config = TemporalAttentionConfig {
            time_decay_rate: 0.9,
            novelty_boost: 0.0,
            expectation_bias: 0.0,
            max_history: 100,
        };
        let mut tab = TemporalAttentionBias::new(config);
        tab.register("a".to_string(), vec![0u8; 64], 0.8, "".to_string());
        tab.tick();
        tab.register("b".to_string(), vec![0u8; 64], 0.5, "".to_string());
        tab.tick();
        tab.register("c".to_string(), vec![0u8; 64], 0.3, "".to_string());
        let dist = tab.attention_distribution();
        let sum: f64 = dist.iter().map(|(_, w)| w).sum();
        assert!((sum - 1.0).abs() < 1e-6, "sum = {}", sum);
        assert_eq!(dist.len(), 3);
    }

    #[test]
    fn test_tick_advances_cycle() {
        let mut tab = TemporalAttentionBias::new(TemporalAttentionConfig::default());
        assert_eq!(tab.current_cycle(), 0);
        tab.tick();
        assert_eq!(tab.current_cycle(), 1);
        tab.tick();
        tab.tick();
        assert_eq!(tab.current_cycle(), 3);
    }

    #[test]
    fn test_clear_resets_state() {
        let mut tab = TemporalAttentionBias::new(TemporalAttentionConfig::default());
        tab.register("a".to_string(), vec![0u8; 64], 0.5, "".to_string());
        tab.register("b".to_string(), vec![0u8; 64], 0.5, "".to_string());
        tab.tick();
        tab.tick();
        tab.clear();
        assert_eq!(tab.entry_count(), 0);
        assert_eq!(tab.current_cycle(), 0);
    }

    #[test]
    fn test_configurable_decay_rate() {
        let fast = TemporalAttentionConfig {
            time_decay_rate: 0.1,
            novelty_boost: 0.0,
            expectation_bias: 0.0,
            max_history: 100,
        };
        let slow = TemporalAttentionConfig {
            time_decay_rate: 0.99,
            novelty_boost: 0.0,
            expectation_bias: 0.0,
            max_history: 100,
        };
        let mut tab_fast = TemporalAttentionBias::new(fast);
        let mut tab_slow = TemporalAttentionBias::new(slow);
        tab_fast.register("x".to_string(), vec![0u8; 64], 1.0, "".to_string());
        tab_slow.register("x".to_string(), vec![0u8; 64], 1.0, "".to_string());
        tab_fast.tick();
        tab_slow.tick();
        tab_fast.tick();
        tab_slow.tick();
        let wf = tab_fast.attention_weight("x");
        let ws = tab_slow.attention_weight("x");
        assert!(
            wf < ws,
            "fast decay weight {} should be less than slow decay weight {}",
            wf,
            ws
        );
    }

    #[test]
    fn test_multiple_entries_different_ages() {
        let config = TemporalAttentionConfig {
            time_decay_rate: 0.8,
            novelty_boost: 0.0,
            expectation_bias: 0.0,
            max_history: 100,
        };
        let mut tab = TemporalAttentionBias::new(config);
        tab.register("a".to_string(), vec![0u8; 64], 1.0, "".to_string());
        tab.tick();
        tab.register("b".to_string(), vec![0u8; 64], 1.0, "".to_string());
        tab.tick();
        tab.register("c".to_string(), vec![0u8; 64], 1.0, "".to_string());
        let wa = tab.attention_weight("a");
        let wb = tab.attention_weight("b");
        let wc = tab.attention_weight("c");
        assert!(wa < wb, "oldest weight {} should be < mid {}", wa, wb);
        assert!(wb < wc, "mid weight {} should be < newest {}", wb, wc);
    }

    #[test]
    fn test_empty_state() {
        let tab = TemporalAttentionBias::new(TemporalAttentionConfig::default());
        assert_eq!(tab.attention_weight("nonexistent"), 0.0);
        let dist = tab.attention_distribution();
        assert!(dist.is_empty());
    }

    #[test]
    fn test_decay_factor_calculation() {
        let tab = TemporalAttentionBias::new(TemporalAttentionConfig {
            time_decay_rate: 0.5,
            novelty_boost: 0.0,
            expectation_bias: 0.0,
            max_history: 100,
        });
        assert!((tab.decay_factor(0) - 1.0).abs() < 1e-6);
        assert!((tab.decay_factor(1) - 0.5).abs() < 1e-6);
        assert!((tab.decay_factor(2) - 0.25).abs() < 1e-6);
        assert!((tab.decay_factor(3) - 0.125).abs() < 1e-6);
    }

    #[test]
    fn test_novelty_bonus_new_vs_old() {
        let config = TemporalAttentionConfig {
            time_decay_rate: 0.95,
            novelty_boost: 1.5,
            expectation_bias: 0.0,
            max_history: 100,
        };
        let mut tab = TemporalAttentionBias::new(config);
        tab.register("new".to_string(), vec![0u8; 64], 0.5, "".to_string());
        // access_count == 0 => novelty_bonus returns 1.5
        {
            let entry = tab.state.entries.iter().find(|e| e.id == "new").unwrap();
            assert!((tab.novelty_bonus(entry) - 1.5).abs() < 1e-6);
        }
        tab.access("new");
        {
            let entry = tab.state.entries.iter().find(|e| e.id == "new").unwrap();
            assert!((tab.novelty_bonus(entry) - 0.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_multiple_access_keeps_fresh() {
        let config = TemporalAttentionConfig {
            time_decay_rate: 0.5,
            novelty_boost: 0.0,
            expectation_bias: 0.0,
            max_history: 100,
        };
        let mut tab = TemporalAttentionBias::new(config);
        tab.register("x".to_string(), vec![0u8; 64], 0.8, "".to_string());
        for _ in 0..10 {
            tab.tick();
            tab.access("x");
        }
        let w = tab.attention_weight("x");
        assert!((w - 0.8).abs() < 1e-6, "weight should be 0.8, got {}", w);
    }

    #[test]
    fn test_prune_preserves_most_recent() {
        let mut tab = TemporalAttentionBias::new(TemporalAttentionConfig::default());
        tab.register("old1".to_string(), vec![0u8; 64], 0.3, "".to_string());
        tab.tick();
        tab.register("old2".to_string(), vec![0u8; 64], 0.3, "".to_string());
        tab.tick();
        tab.register("fresh".to_string(), vec![0u8; 64], 0.3, "".to_string());
        tab.prune(2);
        assert_eq!(tab.entry_count(), 2);
        assert!(
            tab.state.entries.iter().any(|e| e.id == "fresh"),
            "fresh should survive"
        );
    }

    #[test]
    fn test_single_entry_distribution() {
        let mut tab = TemporalAttentionBias::new(TemporalAttentionConfig::default());
        tab.register("only".to_string(), vec![0u8; 64], 0.7, "".to_string());
        let dist = tab.attention_distribution();
        assert_eq!(dist.len(), 1);
        assert!((dist[0].1 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_register_multiple_count() {
        let mut tab = TemporalAttentionBias::new(TemporalAttentionConfig::default());
        tab.register("a".to_string(), vec![0u8; 64], 0.5, "".to_string());
        tab.register("b".to_string(), vec![0u8; 64], 0.5, "".to_string());
        tab.register("c".to_string(), vec![0u8; 64], 0.5, "".to_string());
        assert_eq!(tab.entry_count(), 3);
    }

    #[test]
    fn test_attention_weight_unknown_id() {
        let tab = TemporalAttentionBias::new(TemporalAttentionConfig::default());
        assert_eq!(tab.attention_weight("nonexistent"), 0.0);
    }
}
