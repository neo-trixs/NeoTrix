use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_NOISE_THRESHOLD: f64 = 0.3;
const DEFAULT_CONSOLIDATION_THRESHOLD: f64 = 0.65;
const DEFAULT_ALPHA: f64 = 0.3;
const DEFAULT_BUFFER_CAPACITY: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum UtilitySignal {
    Importance(f64),
    Surprise(f64),
    Emotion(f64),
    Novelty(f64),
    Curiosity(f64),
}

#[derive(Debug, Clone)]
pub struct GatedItem {
    pub id: u64,
    pub vsa_vector: Vec<u8>,
    pub utility: f64,
    pub signals: Vec<UtilitySignal>,
    pub timestamp_ns: u64,
    pub goal_relevance: f64,
    pub access_count: u32,
    pub consolidated: bool,
}

impl GatedItem {
    pub fn new(id: u64, vsa_vector: Vec<u8>, utility: f64, goal_relevance: f64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self {
            id,
            vsa_vector,
            utility,
            signals: Vec::new(),
            timestamp_ns: now,
            goal_relevance,
            access_count: 0,
            consolidated: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AttentionGate {
    noise_threshold: f64,
    consolidation_threshold: f64,
    alpha: f64,
    buffer_capacity: usize,
    buffer: VecDeque<GatedItem>,
    consolidated_items: Vec<GatedItem>,
    goal_vector: Vec<u8>,
    total_gated: u64,
    total_discarded: u64,
    total_consolidated: u64,
}

impl AttentionGate {
    pub fn new(goal_vector: Vec<u8>) -> Self {
        Self {
            noise_threshold: DEFAULT_NOISE_THRESHOLD,
            consolidation_threshold: DEFAULT_CONSOLIDATION_THRESHOLD,
            alpha: DEFAULT_ALPHA,
            buffer_capacity: DEFAULT_BUFFER_CAPACITY,
            buffer: VecDeque::with_capacity(DEFAULT_BUFFER_CAPACITY),
            consolidated_items: Vec::new(),
            goal_vector,
            total_gated: 0,
            total_discarded: 0,
            total_consolidated: 0,
        }
    }

    pub fn with_noise_threshold(mut self, t: f64) -> Self {
        self.noise_threshold = t;
        self
    }
    pub fn with_consolidation_threshold(mut self, t: f64) -> Self {
        self.consolidation_threshold = t;
        self
    }
    pub fn with_alpha(mut self, a: f64) -> Self {
        self.alpha = a;
        self
    }
    pub fn with_capacity(mut self, c: usize) -> Self {
        self.buffer_capacity = c;
        self.buffer.reserve(c);
        self
    }
    pub fn with_goal(mut self, g: Vec<u8>) -> Self {
        self.goal_vector = g;
        self
    }

    pub fn set_goal(&mut self, goal: Vec<u8>) {
        self.goal_vector = goal;
    }

    pub fn gate(
        &mut self,
        id: u64,
        item_vsa: &[u8],
        signals: Vec<UtilitySignal>,
    ) -> Option<GatedItem> {
        let goal_relevance =
            if self.goal_vector.len() == item_vsa.len() && self.goal_vector.len() >= 64 {
                self.compute_vsa_similarity(&self.goal_vector, item_vsa)
            } else {
                0.5
            };

        if goal_relevance < self.noise_threshold {
            self.total_discarded += 1;
            return None;
        }

        let utility = self.compute_utility(&signals);

        let mut item = GatedItem::new(id, item_vsa.to_vec(), utility, goal_relevance);
        item.signals = signals;

        if self.buffer.len() >= self.buffer_capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(item.clone());
        self.total_gated += 1;

        if utility >= self.consolidation_threshold {
            item.consolidated = true;
            self.consolidated_items.push(item.clone());
            self.total_consolidated += 1;
        }

        Some(item)
    }

    pub fn replay_score(&self, item: &GatedItem) -> f64 {
        let freq_bonus = 1.0 + self.alpha * (item.access_count as f64).ln_1p();
        item.utility * freq_bonus
    }

    pub fn top_replay(&self, n: usize) -> Vec<&GatedItem> {
        let mut items: Vec<&GatedItem> = self.buffer.iter().collect();
        items.sort_by(|a, b| {
            self.replay_score(b)
                .partial_cmp(&self.replay_score(a))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        items.into_iter().take(n).collect()
    }

    pub fn record_access(&mut self, id: u64) {
        for item in self.buffer.iter_mut() {
            if item.id == id {
                item.access_count += 1;
                return;
            }
        }
    }

    pub fn should_consolidate(&self, utility: f64) -> bool {
        utility >= self.consolidation_threshold
    }

    fn compute_utility(&self, signals: &[UtilitySignal]) -> f64 {
        if signals.is_empty() {
            return 0.5;
        }
        let sum: f64 = signals
            .iter()
            .map(|s| match s {
                UtilitySignal::Importance(v) => *v,
                UtilitySignal::Surprise(v) => *v,
                UtilitySignal::Emotion(v) => *v,
                UtilitySignal::Novelty(v) => *v,
                UtilitySignal::Curiosity(v) => *v,
            })
            .sum();
        sum / signals.len() as f64
    }

    fn compute_vsa_similarity(&self, a: &[u8], b: &[u8]) -> f64 {
        if a.len() != b.len() {
            return 0.0;
        }
        let matching = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (x ^ y).count_ones())
            .sum::<u32>() as f64;
        let total = (a.len() * 8) as f64;
        1.0 - (matching / total)
    }

    pub fn drain_consolidated(&mut self) -> Vec<GatedItem> {
        let mut drained = Vec::new();
        self.buffer.retain(|item| {
            if item.consolidated {
                drained.push(item.clone());
                false
            } else {
                true
            }
        });
        drained
    }

    pub fn stats(&self) -> GateStats {
        GateStats {
            buffer_size: self.buffer.len(),
            buffer_capacity: self.buffer_capacity,
            consolidated_count: self.consolidated_items.len(),
            total_gated: self.total_gated,
            total_discarded: self.total_discarded,
            total_consolidated: self.total_consolidated,
            discard_rate: if self.total_gated + self.total_discarded > 0 {
                self.total_discarded as f64 / (self.total_gated + self.total_discarded) as f64
            } else {
                0.0
            },
            noise_threshold: self.noise_threshold,
            consolidation_threshold: self.consolidation_threshold,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GateStats {
    pub buffer_size: usize,
    pub buffer_capacity: usize,
    pub consolidated_count: usize,
    pub total_gated: u64,
    pub total_discarded: u64,
    pub total_consolidated: u64,
    pub discard_rate: f64,
    pub noise_threshold: f64,
    pub consolidation_threshold: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vec(val: u8, len: usize) -> Vec<u8> {
        vec![val; len]
    }

    #[test]
    fn noise_gate_discards_low_relevance() {
        let mut gate = AttentionGate::new(make_vec(0x00, 512));
        let low = make_vec(0xFF, 512); // ~0 Hamming similarity → below 0.3 threshold
        let result = gate.gate(1, &low, vec![UtilitySignal::Importance(0.9)]);
        assert!(result.is_none());
        assert_eq!(gate.stats().total_discarded, 1);
    }

    #[test]
    fn high_relevance_passes_gate() {
        let mut gate = AttentionGate::new(make_vec(0x00, 512));
        let high = make_vec(0x00, 512); // identical → similarity 1.0
        let result = gate.gate(2, &high, vec![UtilitySignal::Importance(0.9)]);
        assert!(result.is_some());
        assert_eq!(gate.stats().total_gated, 1);
    }

    #[test]
    fn utility_from_multiple_signals() {
        let gate = AttentionGate::new(make_vec(0x00, 512));
        let signals = vec![
            UtilitySignal::Importance(0.8),
            UtilitySignal::Surprise(0.6),
            UtilitySignal::Emotion(0.4),
        ];
        let utility = gate.compute_utility(&signals);
        assert!((utility - 0.6).abs() < 1e-10);
    }

    #[test]
    fn empty_signals_returns_default() {
        let gate = AttentionGate::new(make_vec(0x00, 512));
        assert!((gate.compute_utility(&[]) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn fifo_eviction_when_full() {
        let mut gate = AttentionGate::new(make_vec(0x00, 512))
            .with_capacity(3)
            .with_noise_threshold(0.0);
        let v = make_vec(0x00, 512);
        for i in 0..5 {
            gate.gate(i, &v, vec![UtilitySignal::Importance(0.5)]);
        }
        assert_eq!(gate.stats().buffer_size, 3);
        let ids: Vec<u64> = gate.buffer.iter().map(|item| item.id).collect();
        assert_eq!(ids, vec![2, 3, 4]);
    }

    #[test]
    fn replay_score_includes_frequency_bonus() {
        let gate = AttentionGate::new(make_vec(0x00, 512)).with_alpha(0.5);
        let mut item = GatedItem::new(1, make_vec(0x00, 512), 0.5, 1.0);
        item.access_count = 3;
        let score = gate.replay_score(&item);
        let expected = 0.5 * (1.0 + 0.5 * (3.0_f64).ln_1p());
        assert!((score - expected).abs() < 1e-10);
    }

    #[test]
    fn consolidation_threshold_gating() {
        let mut gate = AttentionGate::new(make_vec(0x00, 512))
            .with_consolidation_threshold(0.7)
            .with_noise_threshold(0.0);
        let v = make_vec(0x00, 512);
        gate.gate(10, &v, vec![UtilitySignal::Importance(0.9)]);
        gate.gate(11, &v, vec![UtilitySignal::Importance(0.3)]);
        assert_eq!(gate.stats().total_consolidated, 1);
        assert_eq!(gate.consolidated_items.len(), 1);
        assert_eq!(gate.consolidated_items[0].id, 10);
    }

    #[test]
    fn drain_consolidated_removes_them_from_buffer() {
        let mut gate = AttentionGate::new(make_vec(0x00, 512))
            .with_noise_threshold(0.0)
            .with_consolidation_threshold(0.5);
        let v = make_vec(0x00, 512);
        gate.gate(20, &v, vec![UtilitySignal::Importance(0.9)]);
        gate.gate(21, &v, vec![UtilitySignal::Importance(0.1)]);
        let drained = gate.drain_consolidated();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].id, 20);
        assert_eq!(gate.stats().buffer_size, 1);
        assert_eq!(gate.buffer[0].id, 21);
    }

    #[test]
    fn vsa_similarity_identical_vectors() {
        let gate = AttentionGate::new(make_vec(0x00, 512));
        let a = make_vec(0xAB, 512);
        let sim = gate.compute_vsa_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn vsa_similarity_opposite_vectors() {
        let gate = AttentionGate::new(make_vec(0x00, 512));
        let a = make_vec(0xFF, 512);
        let b = make_vec(0x00, 512);
        let sim = gate.compute_vsa_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-10);
    }

    #[test]
    fn stats_tracking() {
        let mut gate = AttentionGate::new(make_vec(0x00, 512)).with_noise_threshold(0.0);
        let v = make_vec(0x00, 512);
        for i in 0..5 {
            gate.gate(i, &v, vec![UtilitySignal::Importance(0.6)]);
        }
        gate.gate(
            99,
            &make_vec(0xFF, 512),
            vec![UtilitySignal::Importance(0.1)],
        );
        let s = gate.stats();
        assert_eq!(s.total_gated, 5);
        assert_eq!(s.total_discarded, 1);
        assert_eq!(s.buffer_size, 5);
        assert!((s.discard_rate - 1.0 / 6.0).abs() < 1e-10);
        assert_eq!(s.noise_threshold, DEFAULT_NOISE_THRESHOLD);
        assert_eq!(s.consolidation_threshold, DEFAULT_CONSOLIDATION_THRESHOLD);
    }

    #[test]
    fn top_replay_returns_highest_scores() {
        let mut gate = AttentionGate::new(make_vec(0x00, 512)).with_noise_threshold(0.0);
        let v = make_vec(0x00, 512);
        gate.gate(1, &v, vec![UtilitySignal::Importance(0.3)]);
        gate.gate(2, &v, vec![UtilitySignal::Importance(0.9)]);
        gate.gate(3, &v, vec![UtilitySignal::Importance(0.6)]);
        let top = gate.top_replay(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].id, 2);
        assert_eq!(top[1].id, 3);
    }

    #[test]
    fn record_access_increments_counter() {
        let mut gate = AttentionGate::new(make_vec(0x00, 512)).with_noise_threshold(0.0);
        let v = make_vec(0x00, 512);
        gate.gate(1, &v, vec![UtilitySignal::Importance(0.5)]);
        gate.record_access(1);
        gate.record_access(1);
        assert_eq!(gate.buffer[0].access_count, 2);
    }
}
