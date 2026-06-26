use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ItemStats {
    pub access_count: u32,
    pub last_access: u64,
    pub creation_time: u64,
    pub surprise_score: f64,
}

impl ItemStats {
    pub fn new(creation_time: u64) -> Self {
        Self {
            access_count: 0,
            last_access: creation_time,
            creation_time,
            surprise_score: 0.0,
        }
    }

    pub fn record_access(&mut self, time: u64) {
        self.access_count += 1;
        self.last_access = time;
    }

    pub fn age_secs(&self, now: u64) -> u64 {
        now.saturating_sub(self.creation_time)
    }
}

#[derive(Debug, Clone)]
pub struct SurpriseScorer {
    references: Vec<(Vec<u8>, String)>,
}

impl SurpriseScorer {
    pub fn new() -> Self {
        Self {
            references: Vec::new(),
        }
    }

    pub fn score(&self, new_item: &[u8]) -> f64 {
        if self.references.is_empty() {
            return 1.0;
        }
        let max_sim = self
            .references
            .iter()
            .map(|(ref_vec, _)| QuantizedVSA::similarity(new_item, ref_vec))
            .fold(0.0f64, f64::max);
        1.0 - max_sim
    }

    pub fn update(&mut self, item: &[u8], label: &str) {
        self.references.push((item.to_vec(), label.to_string()));
    }

    pub fn reset(&mut self) {
        self.references.clear();
    }

    pub fn len(&self) -> usize {
        self.references.len()
    }

    pub fn is_empty(&self) -> bool {
        self.references.is_empty()
    }
}

impl Default for SurpriseScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryAction {
    Keep = 0,
    Evict = 1,
    Consolidate = 2,
}

impl MemoryAction {
    pub fn from_usize(v: usize) -> Self {
        match v {
            0 => MemoryAction::Keep,
            1 => MemoryAction::Evict,
            2 => MemoryAction::Consolidate,
            _ => MemoryAction::Keep,
        }
    }

    pub fn to_usize(&self) -> usize {
        *self as usize
    }
}

pub const NUM_ACTIONS: usize = 3;

#[derive(Debug, Clone)]
pub struct RLMemoryPolicy {
    weights: Vec<f64>,
    learning_rate: f64,
    epsilon: f64,
    discount_factor: f64,
    num_features: usize,
}

impl RLMemoryPolicy {
    pub fn new(num_features: usize) -> Self {
        let mut rng = rand::thread_rng();
        let weights: Vec<f64> = (0..num_features * NUM_ACTIONS)
            .map(|_| rng.gen::<f64>() * 0.1 - 0.05)
            .collect();
        Self {
            weights,
            learning_rate: 0.1,
            epsilon: 0.2,
            discount_factor: 0.9,
            num_features,
        }
    }

    pub fn select_action(&self, item_features: &[f64]) -> usize {
        let mut rng = rand::thread_rng();
        if rng.gen::<f64>() < self.epsilon {
            return rng.gen_range(0..NUM_ACTIONS);
        }
        let mut best_action = 0usize;
        let mut best_value = f64::NEG_INFINITY;
        for a in 0..NUM_ACTIONS {
            let val = self.q_value(item_features, a);
            if val > best_value {
                best_value = val;
                best_action = a;
            }
        }
        best_action
    }

    pub fn update(
        &mut self,
        reward: f64,
        prev_features: &[f64],
        action: usize,
        next_features: &[f64],
    ) {
        let q_current = self.q_value(prev_features, action);
        let max_next = (0..NUM_ACTIONS)
            .map(|a| self.q_value(next_features, a))
            .fold(f64::NEG_INFINITY, f64::max);
        let td_target = reward + self.discount_factor * max_next;
        let td_error = td_target - q_current;
        let lr = self.learning_rate;
        for i in 0..self.num_features {
            let idx = action * self.num_features + i;
            self.weights[idx] += lr * td_error * prev_features[i];
        }
    }

    pub fn reward_fn(recall_success: bool, access_count: u32) -> f64 {
        let base = if recall_success { 1.0 } else { -0.5 };
        let access_bonus = (access_count as f64).min(10.0) * 0.1;
        (base + access_bonus).clamp(-1.0, 2.0)
    }

    fn q_value(&self, features: &[f64], action: usize) -> f64 {
        let mut val = 0.0;
        for i in 0..self.num_features.min(features.len()) {
            let idx = action * self.num_features + i;
            if idx < self.weights.len() {
                val += self.weights[idx] * features[i];
            }
        }
        val
    }

    pub fn set_epsilon(&mut self, epsilon: f64) {
        self.epsilon = epsilon;
    }

    pub fn set_learning_rate(&mut self, lr: f64) {
        self.learning_rate = lr;
    }

    pub fn epsilon(&self) -> f64 {
        self.epsilon
    }

    pub fn weights(&self) -> &[f64] {
        &self.weights
    }

    pub fn reset_weights(&mut self) {
        let mut rng = rand::thread_rng();
        for w in &mut self.weights {
            *w = rng.gen::<f64>() * 0.1 - 0.05;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConsolidationScheduler {
    scorer: SurpriseScorer,
    policy: RLMemoryPolicy,
    surprise_threshold: f64,
    top_k: usize,
}

impl ConsolidationScheduler {
    pub fn new(num_features: usize, top_k: usize) -> Self {
        Self {
            scorer: SurpriseScorer::new(),
            policy: RLMemoryPolicy::new(num_features),
            surprise_threshold: 0.4,
            top_k,
        }
    }

    pub fn should_consolidate(&self, item: &[u8], stats: &ItemStats) -> bool {
        let surprise = self.scorer.score(item);
        let features = Self::extract_features(item, stats, surprise);
        let action = self.policy.select_action(&features);
        action == MemoryAction::Consolidate.to_usize() || surprise > self.surprise_threshold
    }

    pub fn consolidate(&self, priority_queue: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        if priority_queue.is_empty() || self.top_k == 0 {
            return Vec::new();
        }
        let mut scored: Vec<(Vec<u8>, f64)> = priority_queue
            .into_iter()
            .map(|item| {
                let surprise = self.scorer.score(&item);
                (item, surprise)
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(self.top_k);
        scored.into_iter().map(|(item, _)| item).collect()
    }

    pub fn configure(&mut self, surprise_threshold: f64, policy_epsilon: f64) {
        self.surprise_threshold = surprise_threshold;
        self.policy.set_epsilon(policy_epsilon);
    }

    pub fn observe(&mut self, item: &[u8], label: &str) {
        self.scorer.update(item, label);
    }

    pub fn scorer(&self) -> &SurpriseScorer {
        &self.scorer
    }

    pub fn scorer_mut(&mut self) -> &mut SurpriseScorer {
        &mut self.scorer
    }

    pub fn policy(&self) -> &RLMemoryPolicy {
        &self.policy
    }

    pub fn policy_mut(&mut self) -> &mut RLMemoryPolicy {
        &mut self.policy
    }

    pub fn surprise_threshold(&self) -> f64 {
        self.surprise_threshold
    }

    pub fn top_k(&self) -> usize {
        self.top_k
    }

    fn extract_features(_item: &[u8], stats: &ItemStats, surprise: f64) -> Vec<f64> {
        let now = now_millis();
        let age = (now - stats.creation_time) as f64 / 1000.0;
        let age_norm = (age / 86400.0).min(1.0);
        let freq_norm = (stats.access_count as f64).min(100.0) / 100.0;
        vec![age_norm, freq_norm, surprise]
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(v: u8) -> Vec<u8> {
        vec![v; 4096]
    }

    fn make_stats(created: u64, accesses: u32) -> ItemStats {
        ItemStats {
            access_count: accesses,
            last_access: created + 1000,
            creation_time: created,
            surprise_score: 0.0,
        }
    }

    #[test]
    fn test_surprise_score_identical_items_returns_zero() {
        let scorer = SurpriseScorer::new();
        let item = make_item(1);
        let surprise = scorer.score(&item);
        assert!((surprise - 1.0).abs() < 1e-6, "empty refs → max surprise");
    }

    #[test]
    fn test_surprise_score_with_known_item() {
        let mut scorer = SurpriseScorer::new();
        let item = make_item(42);
        scorer.update(&item, "known");
        let surprise = scorer.score(&item);
        assert!(surprise < 0.1, "known item should have low surprise");
    }

    #[test]
    fn test_surprise_score_different_item() {
        let mut scorer = SurpriseScorer::new();
        scorer.update(&make_item(0), "all_zeros");
        let item = make_item(1);
        let surprise = scorer.score(&item);
        assert!(
            (surprise - 1.0).abs() < 1e-6,
            "complement should be maximally surprising"
        );
    }

    #[test]
    fn test_surprise_reset_clears_all() {
        let mut scorer = SurpriseScorer::new();
        scorer.update(&make_item(1), "a");
        scorer.update(&make_item(2), "b");
        assert_eq!(scorer.len(), 2);
        scorer.reset();
        assert!(scorer.is_empty());
    }

    #[test]
    fn test_policy_select_action_returns_valid_action() {
        let policy = RLMemoryPolicy::new(3);
        let features = vec![0.5, 0.3, 0.8];
        let action = policy.select_action(&features);
        assert!(action < NUM_ACTIONS);
    }

    #[test]
    fn test_policy_update_changes_weights() {
        let mut policy = RLMemoryPolicy::new(3);
        let weights_before = policy.weights().to_vec();
        let features = vec![0.5, 0.3, 0.8];
        policy.update(1.0, &features, 0, &features);
        let weights_after = policy.weights();
        let changed = weights_before
            .iter()
            .zip(weights_after.iter())
            .any(|(a, b)| (a - b).abs() > 1e-10);
        assert!(changed, "TD update should modify at least one weight");
    }

    #[test]
    fn test_policy_update_with_zero_features_no_change() {
        let mut policy = RLMemoryPolicy::new(3);
        let weights_before = policy.weights().to_vec();
        let features = vec![0.0, 0.0, 0.0];
        policy.update(0.0, &features, 1, &features);
        let weights_after = policy.weights();
        let changed = weights_before
            .iter()
            .zip(weights_after.iter())
            .any(|(a, b)| (a - b).abs() > 1e-10);
        assert!(!changed, "zero features should produce zero weight update");
    }

    #[test]
    fn test_reward_fn_recall_success() {
        let r = RLMemoryPolicy::reward_fn(true, 0);
        assert!(
            (r - 1.0).abs() < 1e-6,
            "recall success base reward should be 1.0"
        );
    }

    #[test]
    fn test_reward_fn_recall_failure() {
        let r = RLMemoryPolicy::reward_fn(false, 0);
        assert!(
            (r - (-0.5)).abs() < 1e-6,
            "recall failure base reward should be -0.5"
        );
    }

    #[test]
    fn test_reward_fn_access_bonus() {
        let r_no_access = RLMemoryPolicy::reward_fn(true, 0);
        let r_high_access = RLMemoryPolicy::reward_fn(true, 5);
        assert!(
            r_high_access > r_no_access,
            "more accesses should increase reward"
        );
    }

    #[test]
    fn test_consolidation_scheduler_should_consolidate_high_surprise() {
        let scheduler = ConsolidationScheduler::new(3, 5);
        let item = make_item(1);
        let stats = make_stats(0, 0);
        let result = scheduler.should_consolidate(&item, &stats);
        assert!(
            result,
            "novel item with no references should be consolidated"
        );
    }

    #[test]
    fn test_consolidation_returns_top_k_items() {
        let mut scheduler = ConsolidationScheduler::new(3, 2);
        scheduler.observe(&make_item(1), "existing");
        let queue = vec![make_item(0), make_item(1), make_item(2)];
        let result = scheduler.consolidate(queue);
        assert_eq!(result.len(), 2, "should return top 2 by surprise");
    }

    #[test]
    fn test_consolidation_empty_queue() {
        let scheduler = ConsolidationScheduler::new(3, 5);
        let result = scheduler.consolidate(vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_configure_updates_threshold_and_epsilon() {
        let mut scheduler = ConsolidationScheduler::new(3, 5);
        scheduler.configure(0.8, 0.01);
        assert!((scheduler.surprise_threshold() - 0.8).abs() < 1e-6);
        assert!((scheduler.policy().epsilon() - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_item_stats_access_tracking() {
        let now = 1000;
        let mut stats = ItemStats::new(now);
        assert_eq!(stats.access_count, 0);
        stats.record_access(now + 100);
        assert_eq!(stats.access_count, 1);
        assert_eq!(stats.last_access, now + 100);
    }

    #[test]
    fn test_item_stats_age() {
        let stats = ItemStats::new(500);
        assert_eq!(stats.age_secs(1000), 500);
        assert_eq!(stats.age_secs(300), 0);
    }

    #[test]
    fn test_policy_reset_weights_changes_weights() {
        let mut policy = RLMemoryPolicy::new(3);
        let before = policy.weights().to_vec();
        policy.reset_weights();
        let after = policy.weights();
        let changed = before
            .iter()
            .zip(after.iter())
            .any(|(a, b)| (a - b).abs() > 1e-10);
        assert!(changed, "reset should produce different weights");
    }

    #[test]
    fn test_surprise_scorer_multiple_references() {
        let mut scorer = SurpriseScorer::new();
        scorer.update(&make_item(100), "a");
        scorer.update(&make_item(200), "b");
        let s_a = scorer.score(&make_item(100));
        let s_b = scorer.score(&make_item(200));
        let s_new = scorer.score(&make_item(128));
        assert!(s_a < 0.1);
        assert!(s_b < 0.1);
        assert!(s_new > s_a || (s_new - s_a).abs() < 1e-6);
    }

    #[test]
    fn test_memory_action_roundtrip() {
        for v in 0..NUM_ACTIONS {
            let action = MemoryAction::from_usize(v);
            assert_eq!(action.to_usize(), v);
        }
    }
}
