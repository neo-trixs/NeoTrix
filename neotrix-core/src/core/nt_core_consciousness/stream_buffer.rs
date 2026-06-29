use std::collections::VecDeque;

use super::vsa_tag::VsaTagged;
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

pub const DEFAULT_STREAM_CAPACITY: usize = 1024;
pub const ATTENTION_SPAN_SAMPLES: usize = 64;

#[derive(Debug, Clone)]
pub struct ConsciousnessStream {
    buffer: VecDeque<VsaTagged>,
    capacity: usize,
    total_pushed: u64,
}

impl Default for ConsciousnessStream {
    fn default() -> Self {
        Self::new(DEFAULT_STREAM_CAPACITY)
    }
}

impl ConsciousnessStream {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
            total_pushed: 0,
        }
    }

    pub fn push(&mut self, tagged: VsaTagged) {
        self.total_pushed += 1;
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(tagged);
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn total_pushed(&self) -> u64 {
        self.total_pushed
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn recent(&self, n: usize) -> Vec<&VsaTagged> {
        let n = n.min(self.buffer.len());
        self.buffer.iter().skip(self.buffer.len() - n).collect()
    }

    pub fn current(&self) -> Option<&VsaTagged> {
        self.buffer.back()
    }

    pub fn at(&self, index: usize) -> Option<&VsaTagged> {
        self.buffer.get(index)
    }

    pub fn at_mut(&mut self, index: usize) -> Option<&mut VsaTagged> {
        self.buffer.get_mut(index)
    }

    pub fn entries_mut(&mut self) -> &mut VecDeque<VsaTagged> {
        &mut self.buffer
    }

    pub fn last_n_vectors(&self, n: usize) -> Vec<&[u8]> {
        self.recent(n).iter().map(|t| t.vector.as_slice()).collect()
    }

    pub fn bundled_self(&self, n: usize) -> Option<Vec<u8>> {
        let self_vecs: Vec<&[u8]> = self
            .buffer
            .iter()
            .rev()
            .take(n)
            .filter(|t: &&VsaTagged| t.is_self())
            .map(|t| t.vector.as_slice())
            .collect();
        if self_vecs.is_empty() {
            return None;
        }
        Some(QuantizedVSA::bundle(&self_vecs))
    }

    pub fn bundled_world(&self, n: usize) -> Option<Vec<u8>> {
        let world_vecs: Vec<&[u8]> = self
            .buffer
            .iter()
            .rev()
            .take(n)
            .filter(|t: &&VsaTagged| t.is_world())
            .map(|t| t.vector.as_slice())
            .collect();
        if world_vecs.is_empty() {
            return None;
        }
        Some(QuantizedVSA::bundle(&world_vecs))
    }

    pub fn self_world_coherence(&self) -> f64 {
        let self_bundle = match self.bundled_self(ATTENTION_SPAN_SAMPLES) {
            Some(v) => v,
            None => return 0.0,
        };
        let world_bundle = match self.bundled_world(ATTENTION_SPAN_SAMPLES) {
            Some(v) => v,
            None => return 0.0,
        };
        QuantizedVSA::similarity(&self_bundle, &world_bundle)
    }

    pub fn novelty(&self, vector: &[u8], lookback: usize) -> f64 {
        let lookback = lookback.min(self.buffer.len());
        if lookback == 0 {
            return 1.0;
        }
        let max_sim: f64 = self
            .buffer
            .iter()
            .rev()
            .take(lookback)
            .map(|t| QuantizedVSA::similarity(&t.vector, vector))
            .fold(0.0, f64::max);
        1.0 - max_sim
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &VsaTagged> {
        self.buffer.iter()
    }

    pub fn into_inner(self) -> VecDeque<VsaTagged> {
        self.buffer
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&VsaTagged) -> bool,
    {
        self.buffer.retain(f);
    }

    pub fn retention_rate(&self) -> f64 {
        if self.total_pushed == 0 {
            return 1.0;
        }
        self.buffer.len() as f64 / self.total_pushed.min(self.capacity as u64) as f64
    }

    pub fn compaction_needed(&self) -> Option<&'static str> {
        let ratio = self.buffer.len() as f64 / self.capacity as f64;
        if ratio > 0.75 {
            Some("hard")
        } else if ratio > 0.50 {
            Some("soft")
        } else {
            None
        }
    }

    pub fn compact(&mut self, _level: &str) {
        // Halve the buffer by keeping every other entry
        let keep: VecDeque<VsaTagged> = self.buffer.iter().step_by(2).cloned().collect();
        self.buffer = keep;
    }

    /// Remove VSA vectors with no VsaTag (orphaned data).
    pub fn remove_orphan_vsa(&mut self) -> usize {
        let before = self.buffer.len();
        self.buffer.retain(|entry| {
            // Every VsaTagged should have a valid tag; if vector is empty, treat as orphan
            !entry.vector.is_empty()
        });
        before - self.buffer.len()
    }

    /// Repair damaged VsaTag markers — ensure vector length matches VSA_DIM.
    pub fn repair_damaged_tags(&mut self) -> usize {
        let mut repaired = 0;
        for entry in self.buffer.iter_mut() {
            if entry.vector.len() != 4096 && !entry.vector.is_empty() {
                // Pad or truncate to 4096
                if entry.vector.len() < 4096 {
                    entry.vector.resize(4096, 0);
                } else {
                    entry.vector.truncate(4096);
                }
                repaired += 1;
            }
            if entry.confidence < 0.0 || entry.confidence > 1.0 {
                entry.confidence = entry.confidence.clamp(0.0, 1.0);
                repaired += 1;
            }
        }
        repaired
    }

    /// Fold duplicate consecutive entries (same vector, same tag).
    pub fn deduplicate(&mut self) -> usize {
        let before = self.buffer.len();
        let mut deduped: VecDeque<VsaTagged> = VecDeque::with_capacity(self.buffer.len());
        for entry in self.buffer.drain(..) {
            if let Some(last) = deduped.back() {
                if last.vector == entry.vector && last.tag == entry.tag {
                    continue;
                }
            }
            deduped.push_back(entry);
        }
        self.buffer = deduped;
        before - self.buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::vsa_tag::{
        VsaOrigin, VsaSelfCategory, VsaWorldCategory,
    };
    use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

    fn self_tagged(v: Vec<u8>) -> VsaTagged {
        VsaTagged::new(v, VsaOrigin::Self_(VsaSelfCategory::Thought))
    }

    fn world_tagged(v: Vec<u8>) -> VsaTagged {
        VsaTagged::new(v, VsaOrigin::World(VsaWorldCategory::UserInput))
    }

    fn filled_stream() -> ConsciousnessStream {
        let mut s = ConsciousnessStream::new(10);
        for _ in 0..5 {
            s.push(self_tagged(QuantizedVSA::random_binary()));
        }
        for _ in 0..5 {
            s.push(world_tagged(QuantizedVSA::random_binary()));
        }
        s
    }

    #[test]
    fn test_new_stream_empty() {
        let s = ConsciousnessStream::new(100);
        assert!(s.is_empty());
        assert_eq!(s.capacity(), 100);
    }

    #[test]
    fn test_push_and_len() {
        let mut s = ConsciousnessStream::new(100);
        s.push(self_tagged(vec![1; 100]));
        assert_eq!(s.len(), 1);
        assert_eq!(s.total_pushed(), 1);
    }

    #[test]
    fn test_capacity_enforced() {
        let mut s = ConsciousnessStream::new(3);
        for _ in 0..10 {
            s.push(self_tagged(QuantizedVSA::random_binary()));
        }
        assert_eq!(s.len(), 3);
        assert_eq!(s.total_pushed(), 10);
    }

    #[test]
    fn test_recent_returns_last_n() {
        let mut s = ConsciousnessStream::new(100);
        for i in 0..10 {
            let v = vec![i as u8; 10];
            s.push(self_tagged(v));
        }
        let recent = s.recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].vector[0], 7);
        assert_eq!(recent[2].vector[0], 9);
    }

    #[test]
    fn test_current_returns_last() {
        let mut s = ConsciousnessStream::new(100);
        s.push(self_tagged(vec![42; 10]));
        assert_eq!(s.current().unwrap().vector[0], 42);
    }

    #[test]
    fn test_bundled_self_returns_only_self() {
        let s = filled_stream();
        let bundle = s.bundled_self(100);
        assert!(bundle.is_some());
    }

    #[test]
    fn test_bundled_world_returns_only_world() {
        let s = filled_stream();
        let bundle = s.bundled_world(100);
        assert!(bundle.is_some());
    }

    #[test]
    fn test_self_world_coherence() {
        let s = filled_stream();
        let coherence = s.self_world_coherence();
        assert!(coherence >= 0.0 && coherence <= 1.0);
    }

    #[test]
    fn test_novelty_high_for_new_vector() {
        let mut s = ConsciousnessStream::new(100);
        for _ in 0..5 {
            s.push(self_tagged(vec![1; 100]));
        }
        let novel = s.novelty(&vec![0; 100], 10);
        assert!(novel > 0.5);
    }

    #[test]
    fn test_novelty_low_for_repeated_vector() {
        let mut s = ConsciousnessStream::new(100);
        for _ in 0..5 {
            s.push(self_tagged(vec![1; 100]));
        }
        let novel = s.novelty(&vec![1; 100], 10);
        assert!(novel < 0.5);
    }

    #[test]
    fn test_clear() {
        let mut s = filled_stream();
        s.clear();
        assert!(s.is_empty());
    }

    #[test]
    fn test_empty_novelty_returns_one() {
        let s = ConsciousnessStream::new(100);
        assert!((s.novelty(&vec![1; 10], 5) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_last_n_vectors() {
        let mut s = ConsciousnessStream::new(100);
        s.push(self_tagged(vec![1; 10]));
        s.push(self_tagged(vec![2; 10]));
        let vecs = s.last_n_vectors(2);
        assert_eq!(vecs.len(), 2);
        assert_eq!(vecs[0][0], 1);
        assert_eq!(vecs[1][0], 2);
    }

    #[test]
    fn test_retention_rate() {
        let mut s = ConsciousnessStream::new(5);
        for _ in 0..10 {
            s.push(self_tagged(vec![1; 10]));
        }
        assert!((s.retention_rate() - 1.0).abs() < 1e-9);
    }
}

/// Alias for backward compatibility. consciousness_cycle.rs references `StreamBuffer`.
pub type StreamBuffer = ConsciousnessStream;
