use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HippocampalTrace {
    pub trace_vector: Vec<u8>,
    pub separated_vector: Vec<u8>,
    pub binding_key: Vec<u8>,
    pub strength: f64,
    pub last_reinforced: u64,
    pub context_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HippocampalMemory {
    traces: Vec<HippocampalTrace>,
    max_traces: usize,
    dimension: usize,
}

impl HippocampalMemory {
    pub fn new(max_traces: usize, dimension: usize) -> Self {
        Self {
            traces: Vec::new(),
            max_traces,
            dimension,
        }
    }

    pub fn store(
        &mut self,
        trace_vector: Vec<u8>,
        binding_key: Vec<u8>,
        strength: f64,
        context_tags: Vec<String>,
    ) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let separated_vector = Self::pattern_separate(&trace_vector, &binding_key);
        self.traces.push(HippocampalTrace {
            trace_vector,
            separated_vector,
            binding_key,
            strength: strength.clamp(0.0, 1.0),
            last_reinforced: now,
            context_tags,
        });
        while self.traces.len() > self.max_traces {
            let idx = self
                .traces
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    a.strength
                        .partial_cmp(&b.strength)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i);
            if let Some(i) = idx {
                self.traces.swap_remove(i);
            } else {
                break;
            }
        }
    }

    pub fn complete(&self, cue: &[u8], threshold: f64) -> Option<&HippocampalTrace> {
        let mut best: Option<&HippocampalTrace> = None;
        let mut best_sim = threshold;
        for trace in &self.traces {
            let sim = QuantizedVSA::similarity(cue, &trace.trace_vector);
            if sim > best_sim {
                best_sim = sim;
                best = Some(trace);
            }
        }
        best
    }

    pub fn pattern_separate(vector: &[u8], key: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(key);
        let hash = hasher.finalize();
        let n = vector.len();
        let mut separated = Vec::with_capacity(n);
        for i in 0..n {
            let mask_byte = hash[i % hash.len()];
            let perm = (mask_byte as usize).wrapping_mul(i + 1) % 256;
            let xor_val = (perm as u8) ^ mask_byte;
            separated.push(vector[i] ^ xor_val);
        }
        separated
    }

    pub fn pattern_complete(cue: &[u8], stored: &[u8], key: &[u8]) -> Vec<u8> {
        let sim = QuantizedVSA::similarity(cue, stored);
        if sim < 0.3 {
            return cue.to_vec();
        }
        let mut hasher = Sha256::new();
        hasher.update(key);
        let hash = hasher.finalize();
        let n = cue.len().min(stored.len());
        let mut result = Vec::with_capacity(n);
        let alpha = sim;
        for i in 0..n {
            let mask_byte = hash[i % hash.len()];
            let blend = if i % 2 == 0 { stored[i] } else { cue[i] };
            let mix = (blend as f64 * alpha + cue[i] as f64 * (1.0 - alpha)) as u8;
            let perm = (mask_byte as usize).wrapping_mul(i + 1) % 256;
            let xor_val = (perm as u8) ^ mask_byte;
            result.push(mix ^ xor_val);
        }
        result
    }

    pub fn reinforce(&mut self, idx: usize, strength_delta: f64) {
        if let Some(trace) = self.traces.get_mut(idx) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            trace.strength = (trace.strength + strength_delta).clamp(0.0, 1.0);
            trace.last_reinforced = now;
        }
    }

    pub fn decay_all(&mut self, half_life_days: f64, elapsed_days: f64) {
        let decay_factor = (-std::f64::consts::LN_2 * elapsed_days / half_life_days).exp();
        for trace in &mut self.traces {
            trace.strength *= decay_factor;
        }
    }

    pub fn collect_for_consolidation(&self, threshold: f64) -> Vec<&HippocampalTrace> {
        self.traces
            .iter()
            .filter(|t| t.strength >= threshold)
            .collect()
    }

    pub fn trace_count(&self) -> usize {
        self.traces.len()
    }

    pub fn traces(&self) -> &[HippocampalTrace] {
        &self.traces
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory() -> HippocampalMemory {
        HippocampalMemory::new(100, 4096)
    }

    #[test]
    fn test_store_and_trace_count() {
        let mut hm = memory();
        hm.store(vec![1u8; 4096], vec![0u8; 4], 0.9, vec!["test".into()]);
        assert_eq!(hm.trace_count(), 1);
    }

    #[test]
    fn test_pattern_separation_orthogonalizes() {
        let v1 = vec![1u8; 4096];
        let key = b"key1";
        let sep1 = HippocampalMemory::pattern_separate(&v1, key);
        let sep2 = HippocampalMemory::pattern_separate(&v1, key);
        assert_eq!(sep1, sep2, "deterministic same key");
        let key2 = b"key2";
        let sep3 = HippocampalMemory::pattern_separate(&v1, key2);
        assert_ne!(sep1, sep3, "different keys produce different separations");
    }

    #[test]
    fn test_complete_finds_best_match() {
        let mut hm = memory();
        let target = vec![1u8; 4096];
        hm.store(target.clone(), vec![0u8; 4], 0.9, vec!["target".into()]);
        hm.store(vec![0u8; 4096], vec![0u8; 4], 0.5, vec!["noise".into()]);
        let result = hm.complete(&target, 0.5);
        assert!(result.is_some());
        assert_eq!(result.unwrap().context_tags[0], "target");
    }

    #[test]
    fn test_complete_below_threshold_returns_none() {
        let mut hm = memory();
        hm.store(vec![1u8; 4096], vec![0u8; 4], 0.9, vec!["target".into()]);
        let query = vec![0u8; 4096];
        let result = hm.complete(&query, 0.9);
        assert!(result.is_none());
    }

    #[test]
    fn test_reinforce_increases_strength() {
        let mut hm = memory();
        hm.store(vec![1u8; 4096], vec![0u8; 4], 0.5, vec![]);
        hm.reinforce(0, 0.3);
        assert!((hm.traces()[0].strength - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_decay_all() {
        let mut hm = memory();
        hm.store(vec![1u8; 4096], vec![0u8; 4], 1.0, vec![]);
        hm.decay_all(30.0, 30.0);
        assert!(hm.traces()[0].strength < 1.0);
        assert!(hm.traces()[0].strength > 0.4);
    }

    #[test]
    fn test_collect_for_consolidation() {
        let mut hm = memory();
        hm.store(vec![1u8; 4096], vec![0u8; 4], 0.9, vec!["strong".into()]);
        hm.store(vec![0u8; 4096], vec![0u8; 4], 0.3, vec!["weak".into()]);
        let candidates = hm.collect_for_consolidation(0.5);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].context_tags[0], "strong");
    }

    #[test]
    fn test_max_traces_enforced() {
        let mut hm = HippocampalMemory::new(2, 4096);
        for _ in 0..5 {
            hm.store(vec![1u8; 4096], vec![0u8; 4], 0.5, vec![]);
        }
        assert!(hm.trace_count() <= 2);
    }

    #[test]
    fn test_pattern_complete_low_similarity() {
        let cue = vec![0u8; 4096];
        let stored = vec![1u8; 4096];
        let result = HippocampalMemory::pattern_complete(&cue, &stored, b"key");
        assert_eq!(result.len(), cue.len());
    }
}
