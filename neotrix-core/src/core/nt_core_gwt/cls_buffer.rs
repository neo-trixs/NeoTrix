/// Complementary Learning Systems (CLS) Buffer.
///
/// Implements the dual-memory architecture from MIRROR (AAAI 2026):
/// - **Fast buffer** (hippocampus): ring buffer of recent episodic experiences
/// - **Slow buffer** (neocortex): existing HyperCube VSA for semantic knowledge
///
/// Hybrid retrieval: fast → recency-weighted candidates → slow semantic rerank.
use std::collections::VecDeque;

/// An episodic experience record.
#[derive(Debug, Clone)]
pub struct Experience {
    /// Unique episode identifier.
    pub id: u64,
    /// E8 state hexagram at the time.
    pub e8_state: u8,
    /// GWT specialist activation vector snapshot.
    pub activation_snapshot: Vec<f64>,
    /// The content/outcome of the experience.
    pub description: String,
    /// Reward or utility signal (0.0–1.0).
    pub reward: f64,
    /// Timestamp (monotonic tick).
    pub tick: u64,
}

/// Complementary Learning Systems buffer.
#[derive(Debug, Clone)]
pub struct CLSBuffer {
    /// Fast buffer: ring buffer of recent experiences.
    fast_buffer: VecDeque<Experience>,
    /// Maximum capacity of the fast buffer.
    pub max_fast: usize,
    /// Next episode ID.
    next_id: u64,
    /// Current tick counter.
    tick: u64,
    /// Consolidation threshold: experiences with reward above this
    /// are candidates for slow (HyperCube) consolidation.
    pub consolidation_threshold: f64,
}

impl Default for CLSBuffer {
    fn default() -> Self {
        Self {
            fast_buffer: VecDeque::with_capacity(100),
            max_fast: 100,
            next_id: 1,
            tick: 0,
            consolidation_threshold: 0.7,
        }
    }
}

impl CLSBuffer {
    pub fn new(max_fast: usize) -> Self {
        Self {
            fast_buffer: VecDeque::with_capacity(max_fast),
            max_fast,
            ..Default::default()
        }
    }

    /// Record a new experience into the fast buffer.
    pub fn record(
        &mut self,
        e8_state: u8,
        activation_snapshot: Vec<f64>,
        description: String,
        reward: f64,
    ) -> u64 {
        self.tick += 1;
        let id = self.next_id;
        self.next_id += 1;

        if self.fast_buffer.len() >= self.max_fast {
            self.fast_buffer.pop_front();
        }

        self.fast_buffer.push_back(Experience {
            id,
            e8_state,
            activation_snapshot,
            description,
            reward: reward.max(0.0).min(1.0),
            tick: self.tick,
        });

        id
    }

    /// Query the fast buffer by recency-weighted similarity.
    /// Returns up to `top_k` experiences ordered by (recency_weight + reward).
    pub fn query_fast(&self, e8_state: u8, top_k: usize) -> Vec<&Experience> {
        if self.fast_buffer.is_empty() {
            return Vec::new();
        }
        let current_tick = self.tick;
        let max_tick = current_tick.max(1);

        let mut scored: Vec<(f64, &Experience)> = self.fast_buffer.iter()
            .map(|exp| {
                let state_sim = if exp.e8_state == e8_state { 1.0 } else { 0.0 };
                let recency = exp.tick as f64 / max_tick as f64;
                let score = state_sim * 0.5 + recency * 0.3 + exp.reward * 0.2;
                (score, exp)
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        scored.into_iter().take(top_k).map(|(_, exp)| exp).collect()
    }

    /// Query fast buffer by activation similarity (cosine).
    pub fn query_fast_by_activation(
        &self, activation: &[f64], top_k: usize,
    ) -> Vec<&Experience> {
        if self.fast_buffer.is_empty() {
            return Vec::new();
        }
        let mut scored: Vec<(f64, &Experience)> = self.fast_buffer.iter()
            .map(|exp| {
                let sim = cosine_similarity(activation, &exp.activation_snapshot);
                let recency = exp.tick as f64 / self.tick.max(1) as f64;
                (sim * 0.6 + recency * 0.2 + exp.reward * 0.2, exp)
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        scored.into_iter().take(top_k).map(|(_, exp)| exp).collect()
    }

    /// Get experiences above consolidation threshold.
    pub fn consolidation_candidates(&self) -> Vec<&Experience> {
        self.fast_buffer.iter()
            .filter(|e| e.reward >= self.consolidation_threshold)
            .collect()
    }

    /// Remove consolidated or low-value experiences from fast buffer.
    pub fn prune(&mut self, min_reward: f64) {
        self.fast_buffer.retain(|e| e.reward >= min_reward);
    }

    /// Number of experiences in fast buffer.
    pub fn len(&self) -> usize {
        self.fast_buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fast_buffer.is_empty()
    }

    /// Clear the fast buffer.
    pub fn clear(&mut self) {
        self.fast_buffer.clear();
    }
}

/// Cosine similarity between two slices.
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f64 = a.iter().map(|x| x * x).sum();
    let nb: f64 = b.iter().map(|x| x * x).sum();
    if na > 0.0 && nb > 0.0 {
        dot / (na.sqrt() * nb.sqrt())
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_activation() -> Vec<f64> {
        vec![0.5, 0.3, 0.8, 0.1, 0.6]
    }

    #[test]
    fn test_default() {
        let buf = CLSBuffer::default();
        assert!(buf.is_empty());
        assert_eq!(buf.max_fast, 100);
    }

    #[test]
    fn test_record_and_length() {
        let mut buf = CLSBuffer::new(10);
        let id = buf.record(42, sample_activation(), "test".into(), 0.8);
        assert_eq!(id, 1);
        assert_eq!(buf.len(), 1);
    }

    #[test]
    fn test_query_fast_returns_most_relevant() {
        let mut buf = CLSBuffer::new(10);
        buf.record(0, sample_activation(), "irrelevant".into(), 0.1);
        buf.record(42, sample_activation(), "match".into(), 0.9);
        let results = buf.query_fast(42, 5);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].description, "match");
    }

    #[test]
    fn test_query_fast_empty() {
        let buf = CLSBuffer::new(10);
        let results = buf.query_fast(0, 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_query_fast_top_k() {
        let mut buf = CLSBuffer::new(10);
        for i in 0..10 {
            buf.record(i as u8, sample_activation(), format!("exp_{}", i), 0.5);
        }
        let results = buf.query_fast(5, 3);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_query_by_activation() {
        let mut buf = CLSBuffer::new(10);
        buf.record(0, vec![1.0, 0.0], "match".into(), 0.5);
        buf.record(0, vec![0.0, 1.0], "mismatch".into(), 0.5);
        let results = buf.query_fast_by_activation(&[0.9, 0.1], 5);
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|e| e.description == "match"));
    }

    #[test]
    fn test_consolidation_candidates() {
        let mut buf = CLSBuffer::new(10);
        buf.record(0, sample_activation(), "low".into(), 0.3);
        buf.record(0, sample_activation(), "high".into(), 0.9);
        let candidates = buf.consolidation_candidates();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].description, "high");
    }

    #[test]
    fn test_prune_removes_low_value() {
        let mut buf = CLSBuffer::new(10);
        buf.record(0, sample_activation(), "low".into(), 0.2);
        buf.record(0, sample_activation(), "high".into(), 0.8);
        buf.prune(0.5);
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.fast_buffer[0].description, "high");
    }

    #[test]
    fn test_ring_buffer_eviction() {
        let mut buf = CLSBuffer::new(3);
        buf.record(0, sample_activation(), "a".into(), 0.5);
        buf.record(0, sample_activation(), "b".into(), 0.5);
        buf.record(0, sample_activation(), "c".into(), 0.5);
        buf.record(0, sample_activation(), "d".into(), 0.5);
        assert_eq!(buf.len(), 3);
        assert!(buf.fast_buffer.iter().all(|e| e.description != "a"));
    }

    #[test]
    fn test_clear() {
        let mut buf = CLSBuffer::new(10);
        buf.record(0, sample_activation(), "x".into(), 0.5);
        buf.clear();
        assert!(buf.is_empty());
    }
}
