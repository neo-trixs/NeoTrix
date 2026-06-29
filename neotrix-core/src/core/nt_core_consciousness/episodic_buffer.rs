use super::vsa_tag::VsaTagged;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct EpisodicEntry {
    pub id: u64,
    pub state: VsaTagged,
    pub cycle_number: u64,
    pub timestamp: u64,
    pub label: String,
    pub significance: f64,
}

#[derive(Debug, Clone)]
pub struct BufferConfig {
    /// Maximum number of entries in the ring buffer
    pub capacity: usize,
    /// Minimum significance threshold for automatic retention
    pub auto_retain_threshold: f64,
    /// Whether to automatically prune old entries when full
    pub auto_prune: bool,
    /// If auto_prune, fraction of oldest entries to remove
    pub prune_fraction: f64,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            capacity: 500,
            auto_retain_threshold: 0.3,
            auto_prune: true,
            prune_fraction: 0.2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecallResult {
    pub entries: Vec<EpisodicEntry>,
    pub query_similarity: f64,
    pub count: usize,
}

pub struct EpisodicConsciousnessBuffer {
    config: BufferConfig,
    buffer: VecDeque<EpisodicEntry>,
    entry_counter: u64,
}

impl EpisodicConsciousnessBuffer {
    pub fn new(config: BufferConfig) -> Self {
        let capacity = config.capacity;
        Self {
            config,
            buffer: VecDeque::with_capacity(capacity),
            entry_counter: 0,
        }
    }

    pub fn config(&self) -> &BufferConfig {
        &self.config
    }
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Push a new consciousness state into the episodic buffer.
    /// If the buffer is full, evicts the oldest low-significance entries.
    pub fn push(&mut self, state: VsaTagged, cycle_number: u64, label: String) -> u64 {
        self.entry_counter += 1;

        let significance = self.compute_significance(&state);

        let entry = EpisodicEntry {
            id: self.entry_counter,
            state,
            cycle_number,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            label,
            significance,
        };

        if self.buffer.len() >= self.config.capacity {
            if self.config.auto_prune {
                let remove_count =
                    (self.config.capacity as f64 * self.config.prune_fraction) as usize;
                let remove_count = remove_count.max(1);
                let mut candidates: Vec<usize> = (0..self.buffer.len()).collect();
                candidates.sort_by(|&a, &b| {
                    self.buffer[a]
                        .significance
                        .partial_cmp(&self.buffer[b].significance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                let to_remove: std::collections::HashSet<usize> =
                    candidates.iter().take(remove_count).copied().collect();
                let mut new_buffer: VecDeque<EpisodicEntry> =
                    VecDeque::with_capacity(self.config.capacity);
                for (i, e) in self.buffer.drain(..).enumerate() {
                    if !to_remove.contains(&i) {
                        new_buffer.push_back(e);
                    }
                }
                self.buffer = new_buffer;
            } else {
                self.buffer.pop_front();
            }
        }

        let retain = entry.significance >= self.config.auto_retain_threshold;
        self.buffer.push_back(entry);

        if retain {
            // Mark for long-term consolidation (stub: just counts)
        }

        self.entry_counter
    }

    /// Search for the most similar entries to a query state.
    pub fn recall_similar(&self, query: &VsaTagged, k: usize) -> RecallResult {
        let mut scored: Vec<(f64, &EpisodicEntry)> = self
            .buffer
            .iter()
            .map(|e| {
                let sim = self.vector_similarity(&query.vector, &e.state.vector);
                (sim, e)
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let best_sim = scored.first().map(|(s, _)| *s).unwrap_or(0.0);
        let matched: Vec<EpisodicEntry> =
            scored.into_iter().take(k).map(|(_, e)| e.clone()).collect();

        RecallResult {
            count: matched.len(),
            query_similarity: best_sim,
            entries: matched,
        }
    }

    /// Recall entries by cycle number range.
    pub fn recall_range(&self, start_cycle: u64, end_cycle: u64) -> Vec<&EpisodicEntry> {
        self.buffer
            .iter()
            .filter(|e| e.cycle_number >= start_cycle && e.cycle_number <= end_cycle)
            .collect()
    }

    /// Replay the last N entries (for consolidation / dream).
    pub fn replay_last(&self, n: usize) -> Vec<&EpisodicEntry> {
        let n = n.min(self.buffer.len());
        self.buffer.iter().rev().take(n).collect()
    }

    /// Get entries above a significance threshold.
    pub fn significant_entries(&self, min_significance: f64) -> Vec<&EpisodicEntry> {
        self.buffer
            .iter()
            .filter(|e| e.significance >= min_significance)
            .collect()
    }

    /// Clear the buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.entry_counter = 0;
    }

    fn compute_significance(&self, state: &VsaTagged) -> f64 {
        let confidence_factor = state.confidence;
        let entropy = self.compute_entropy(&state.vector);
        (confidence_factor * 0.5 + entropy * 0.5).clamp(0.0, 1.0)
    }

    fn compute_entropy(&self, vector: &[u8]) -> f64 {
        let n = vector.len().min(64);
        if n == 0 {
            return 0.0;
        }
        let sum: f64 = vector
            .iter()
            .take(n)
            .map(|&b| {
                let p = b as f64 / 255.0;
                if p < 0.01 || p > 0.99 {
                    0.0
                } else {
                    -p * p.log2() - (1.0 - p) * (1.0 - p).log2()
                }
            })
            .sum();
        (sum / n as f64) / 8.0
    }

    fn vector_similarity(&self, a: &[u8], b: &[u8]) -> f64 {
        let n = a.len().min(b.len());
        if n == 0 {
            return 0.0;
        }
        let same = a.iter().zip(b.iter()).filter(|(x, y)| x == y).count();
        same as f64 / n as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::{VsaOrigin, VsaSelfCategory};

    fn make_state(seed: u8, conf: f64) -> VsaTagged {
        VsaTagged::new(vec![seed; 64], VsaOrigin::Self_(VsaSelfCategory::Thought))
            .with_confidence(conf)
    }

    #[test]
    fn test_push_and_len() {
        let mut buf = EpisodicConsciousnessBuffer::new(BufferConfig::default());
        buf.push(make_state(1, 0.8), 1, "first".into());
        buf.push(make_state(2, 0.7), 2, "second".into());
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn test_recall_similar() {
        let mut buf = EpisodicConsciousnessBuffer::new(BufferConfig::default());
        buf.push(make_state(10, 0.9), 1, "a".into());
        buf.push(make_state(20, 0.8), 2, "b".into());
        let recall = buf.recall_similar(&make_state(10, 0.9), 1);
        assert_eq!(recall.count, 1);
    }

    #[test]
    fn test_replay_last() {
        let mut buf = EpisodicConsciousnessBuffer::new(BufferConfig::default());
        for i in 0..5 {
            buf.push(make_state(i, 0.5), i as u64, format!("e{}", i));
        }
        let replayed = buf.replay_last(3);
        assert_eq!(replayed.len(), 3);
        assert_eq!(replayed[0].cycle_number, 4);
    }

    #[test]
    fn test_recall_range() {
        let mut buf = EpisodicConsciousnessBuffer::new(BufferConfig::default());
        for i in 0..10 {
            buf.push(make_state(i, 0.5), i as u64, format!("e{}", i));
        }
        let range = buf.recall_range(3, 6);
        assert_eq!(range.len(), 4);
    }

    #[test]
    fn test_auto_prune() {
        let mut buf = EpisodicConsciousnessBuffer::new(BufferConfig {
            capacity: 10,
            auto_prune: true,
            prune_fraction: 0.3,
            ..Default::default()
        });
        for i in 0..15 {
            // Alternate significance by using different conf values
            buf.push(
                VsaTagged::new(vec![i; 64], VsaOrigin::Self_(VsaSelfCategory::Thought))
                    .with_confidence(if i < 5 { 0.1 } else { 0.8 }),
                i as u64,
                format!("e{}", i),
            );
        }
        assert!(buf.len() <= 10);
    }

    #[test]
    fn test_clear() {
        let mut buf = EpisodicConsciousnessBuffer::new(BufferConfig::default());
        buf.push(make_state(1, 0.5), 1, "x".into());
        buf.clear();
        assert!(buf.is_empty());
    }

    #[test]
    fn test_significant_entries() {
        let mut buf = EpisodicConsciousnessBuffer::new(BufferConfig::default());
        buf.push(make_state(1, 0.9), 1, "high".into());
        buf.push(make_state(2, 0.2), 2, "low".into());
        let sig = buf.significant_entries(0.5);
        assert_eq!(sig.len(), 1);
        assert_eq!(sig[0].label, "high");
    }
}
