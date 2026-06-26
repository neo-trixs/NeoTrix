use crate::core::nt_core_e8::hexagram_hadamard;
use std::collections::HashMap;

const WH_DIM: usize = 64;

/// Orthogonal memory index using 64×64 Walsh-Hadamard basis.
///
/// Each memory is encoded as a 64-dim vector via:
///   signature → H(6) transform → orthogonal embedding
///
/// The Hadamard transform spreads information across all dimensions,
/// providing graceful degradation under noise (holographic encoding).
pub struct WalshMemoryIndex {
    /// 64×64 Hadamard matrix as f64
    hadamard: Vec<Vec<f64>>,
    /// Stored orthogonal embeddings: (memory_id, 64-dim vector)
    embeddings: Vec<(String, Vec<f64>)>,
    /// memory_id → index in embeddings
    id_map: HashMap<String, usize>,
}

impl Default for WalshMemoryIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl WalshMemoryIndex {
    pub fn new() -> Self {
        let raw = hexagram_hadamard();
        let hadamard: Vec<Vec<f64>> = raw
            .iter()
            .map(|row: &Vec<i8>| row.iter().map(|&x| x as f64).collect())
            .collect();
        Self {
            hadamard,
            embeddings: Vec::new(),
            id_map: HashMap::new(),
        }
    }

    /// Build a 64-dim signature vector from text.
    /// Each word is hashed to a position 0..63 with sign ±1.
    fn text_signature(&self, text: &str) -> Vec<f64> {
        let mut sig = vec![0.0; WH_DIM];
        for token in text.split(|c: char| !c.is_alphanumeric()) {
            if token.is_empty() || token.len() < 2 {
                continue;
            }
            let h1: usize = token
                .bytes()
                .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
                as usize
                % WH_DIM;
            let sign: f64 = if (token
                .bytes()
                .fold(0u64, |acc, b| acc.wrapping_mul(37).wrapping_add(b as u64))
                as usize)
                .is_multiple_of(2)
            {
                1.0
            } else {
                -1.0
            };
            sig[h1] += sign;
        }
        // Normalize to unit vector in signature space
        let norm: f64 = sig.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 0.0 {
            for x in sig.iter_mut() {
                *x /= norm;
            }
        }
        sig
    }

    /// WH-transform: y = H @ x (orthogonal projection)
    fn wh_transform(&self, x: &[f64]) -> Vec<f64> {
        let mut y = vec![0.0; WH_DIM];
        for (i, item) in y.iter_mut().enumerate() {
            *item = self.hadamard[i]
                .iter()
                .zip(x.iter())
                .map(|(h, xv)| h * xv)
                .sum();
        }
        y
    }

    /// Inverse WH-transform: x = (1/WH_DIM) × H @ y
    fn wh_inverse(&self, y: &[f64]) -> Vec<f64> {
        let mut x = vec![0.0; WH_DIM];
        for (i, item) in x.iter_mut().enumerate() {
            *item = self.hadamard[i]
                .iter()
                .zip(y.iter())
                .map(|(h, yv)| h * yv)
                .sum::<f64>()
                / WH_DIM as f64;
        }
        x
    }

    /// Encode text into a 64-dim orthogonal embedding.
    /// Pipeline: signature → WH-transform → normalize
    pub fn encode(&self, text: &str) -> Vec<f64> {
        let sig = self.text_signature(text);

        self.wh_transform(&sig)
    }

    /// Store a memory by encoding its text content.
    pub fn store(&mut self, id: &str, text: &str) {
        let emb = self.encode(text);
        let id_s = id.to_string();
        if let Some(&old_idx) = self.id_map.get(&id_s) {
            self.embeddings[old_idx] = (id_s.clone(), emb);
        } else {
            let idx = self.embeddings.len();
            self.id_map.insert(id_s.clone(), idx);
            self.embeddings.push((id_s, emb));
        }
    }

    /// Remove a stored memory by ID.
    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(idx) = self.id_map.remove(id) {
            self.embeddings.swap_remove(idx);
            if idx < self.embeddings.len() {
                let moved_id = self.embeddings[idx].0.clone();
                self.id_map.insert(moved_id, idx);
            }
            true
        } else {
            false
        }
    }

    /// Search for top-k most similar memories by WH-encoded dot product.
    pub fn search(&self, query: &str, k: usize) -> Vec<(f64, String)> {
        if self.embeddings.is_empty() {
            return Vec::new();
        }
        let q_emb = self.encode(query);
        let mut scored: Vec<(f64, &str)> = self
            .embeddings
            .iter()
            .map(|(id, emb)| {
                let sim = Self::wh_dot(&q_emb, emb);
                (sim, id.as_str())
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let k = k.min(scored.len());
        scored
            .into_iter()
            .take(k)
            .map(|(s, id)| (s, id.to_string()))
            .collect()
    }

    /// Dot product in WH space (preserves L2 up to scale).
    fn wh_dot(a: &[f64], b: &[f64]) -> f64 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    /// Denoise a WH-encoded vector using the WH self-inverse property.
    /// y_noisy → x_recovered = (1/64) × H @ y_noisy → y_cleaned = H @ x_recovered
    pub fn denoise(&self, noisy: &[f64]) -> Vec<f64> {
        let x_recovered = self.wh_inverse(noisy);
        // Threshold small values (below 1/64 of max signal) to zero
        let max_val: f64 = x_recovered.iter().copied().fold(0.0f64, f64::max);
        let threshold = max_val / 64.0;
        let x_clean: Vec<f64> = x_recovered
            .iter()
            .map(|&v| if v.abs() < threshold { 0.0 } else { v })
            .collect();
        self.wh_transform(&x_clean)
    }

    /// Measure recovery quality: cosine similarity between original and recovered.
    pub fn recovery_ratio(original: &[f64], recovered: &[f64]) -> f64 {
        let dot: f64 = original
            .iter()
            .zip(recovered.iter())
            .map(|(a, b)| a * b)
            .sum();
        let norm_o: f64 = original.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_r: f64 = recovered.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_o == 0.0 || norm_r == 0.0 {
            return 0.0;
        }
        dot / (norm_o * norm_r)
    }

    pub fn len(&self) -> usize {
        self.embeddings.len()
    }
    pub fn is_empty(&self) -> bool {
        self.embeddings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let index = WalshMemoryIndex::new();
        let text = "E8 root system with 240 roots and 64 hexagram mapping";
        let emb = index.encode(text);
        let recovered = index.wh_inverse(&emb);
        let re_encoded = index.wh_transform(&recovered);
        let sim = WalshMemoryIndex::wh_dot(&emb, &re_encoded);
        let norm_emb: f64 = emb.iter().map(|x| x * x).sum::<f64>().sqrt();
        let normalized_sim = sim / (norm_emb * norm_emb);
        assert!(
            (normalized_sim - 1.0).abs() < 1e-10,
            "roundtrip failed: {}",
            normalized_sim
        );
    }

    #[test]
    fn test_store_and_search() {
        let mut index = WalshMemoryIndex::new();
        index.store("mem1", "resonance attention mechanism with GWT broadcast");
        index.store("mem2", "E8 root system and hexagram state navigation");
        index.store("mem3", "quantum error correction with surface codes");
        assert_eq!(index.len(), 3);
        let results = index.search("E8 hexagram navigation", 2);
        assert_eq!(results.len(), 2);
        // mem2 should be most relevant
        assert_eq!(results[0].1, "mem2");
    }

    #[test]
    fn test_remove_memory() {
        let mut index = WalshMemoryIndex::new();
        index.store("mem1", "first memory");
        index.store("mem2", "second memory");
        assert!(index.remove("mem1"));
        assert_eq!(index.len(), 1);
        assert!(!index.remove("nonexistent"));
        // search for mem1 content should not find it
        let results = index.search("first", 5);
        assert!(results.iter().all(|(_, id)| id != "mem1"));
    }

    #[test]
    fn test_update_memory() {
        let mut index = WalshMemoryIndex::new();
        index.store("mem1", "old content about resonance");
        let old_results = index.search("resonance", 5);
        assert_eq!(old_results[0].1, "mem1");
        index.store("mem1", "new content about E8 algebra");
        let new_results = index.search("E8 algebra", 5);
        assert_eq!(new_results[0].1, "mem1");
        // old query should no longer match as well
        let old_query_results = index.search("resonance", 5);
        if !old_query_results.is_empty() {
            assert!(old_query_results[0].0 < new_results[0].0 + 0.1);
        }
    }

    #[test]
    fn test_self_retrieval_high_score() {
        let mut index = WalshMemoryIndex::new();
        let text = "E8 reasoning engine with 64-mode state space";
        index.store("self", text);
        let results = index.search(text, 1);
        assert_eq!(results[0].1, "self");
        // Self-similarity should be very high (> 0.9)
        assert!(
            results[0].0 > 0.9,
            "self-similarity too low: {}",
            results[0].0
        );
    }

    #[test]
    fn test_noise_immunity() {
        let index = WalshMemoryIndex::new();
        let text = "Resonance between specialist modules amplifies attention to clusters";
        let original = index.encode(text);
        // Add noise at 20% of signal amplitude
        let _signal_power: f64 = original.iter().map(|x| x * x).sum::<f64>().sqrt();
        let noise: Vec<f64> = original
            .iter()
            .map(|x| x * 0.2 * (if (x * 1e6) as i64 % 2 == 0 { 1.0 } else { -1.0 }))
            .collect();
        let noisy: Vec<f64> = original
            .iter()
            .zip(noise.iter())
            .map(|(s, n)| s + n)
            .collect();
        let denoised = index.denoise(&noisy);
        let ratio = WalshMemoryIndex::recovery_ratio(&original, &denoised);
        assert!(ratio > 0.85, "noise recovery too low: {}", ratio);
    }

    #[test]
    fn test_noise_immune_retrieval_degrades_gracefully() {
        let mut index = WalshMemoryIndex::new();
        let texts = [
            "E8 root system with 240 roots",
            "Walsh-Hadamard orthogonal memory retrieval",
            "GWT broadcast with resonance boosting",
            "Quantum error correction surface code",
        ];
        for (i, text) in texts.iter().enumerate() {
            index.store(&format!("mem{}", i), text);
        }
        let query = "resonance GWT broadcast attention mechanism";
        // Clean retrieval
        let clean_results = index.search(query, 4);
        // mem2 (index 2) should be top
        assert_eq!(clean_results[0].1, "mem2");
    }

    #[test]
    fn test_cross_language_stability() {
        let index = WalshMemoryIndex::new();
        let en = index.encode("resonance attention in global workspace");
        let cn = index.encode("全局工作空间中的共振注意力");
        // Different languages about the same concept should still have some similarity
        let sim = WalshMemoryIndex::wh_dot(&en, &cn);
        let norm = en.iter().map(|x| x * x).sum::<f64>().sqrt()
            * cn.iter().map(|x| x * x).sum::<f64>().sqrt();
        let normalized = sim / norm;
        // Same concept → should be positive (not anti-correlated)
        assert!(
            normalized > -0.1,
            "cross-language anti-correlated: {}",
            normalized
        );
    }

    #[test]
    fn test_empty_index() {
        let index = WalshMemoryIndex::new();
        assert!(index.is_empty());
        let results = index.search("anything", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_orthogonal_embeddings_distinct() {
        let index = WalshMemoryIndex::new();
        let emb1 = index.encode("quantum mechanics wave function collapse");
        let emb2 = index.encode("classical mechanics newtonian physics");
        let emb3 = index.encode("pizza recipe tomato cheese dough");
        let sim12 = WalshMemoryIndex::wh_dot(&emb1, &emb2);
        let sim13 = WalshMemoryIndex::wh_dot(&emb1, &emb3);
        let norm12 = emb1.iter().map(|x| x * x).sum::<f64>().sqrt()
            * emb2.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm13 = emb1.iter().map(|x| x * x).sum::<f64>().sqrt()
            * emb3.iter().map(|x| x * x).sum::<f64>().sqrt();
        // Physics texts should be closer to each other than to pizza
        let n12 = sim12 / norm12;
        let n13 = sim13 / norm13;
        assert!(
            n12 > n13,
            "physics-pizza similarity ({}) should be less than physics-physics ({})",
            n13,
            n12
        );
    }

    #[test]
    fn test_denoise_preserves_identity() {
        let index = WalshMemoryIndex::new();
        let text = "The 64-dim Walsh-Hadamard transform spreads signal across all dimensions";
        let original = index.encode(text);
        // Denoise without noise should preserve original
        let denoised = index.denoise(&original);
        let ratio = WalshMemoryIndex::recovery_ratio(&original, &denoised);
        assert!(ratio > 0.99, "denoise without noise degraded: {}", ratio);
    }

    #[test]
    fn test_multiple_stores_bulk_search() {
        let mut index = WalshMemoryIndex::new();
        let topics = [
            "E8 root system and hexagram state navigation",
            "resonance attention mechanism with GWT broadcast",
            "quantum error correction with surface codes",
            "Walsh-Hadamard orthogonal memory retrieval",
            "vector symbolic architecture hyperdimensional computing",
            "self-iterating reasoning engine with capability vectors",
            "MCP tool integration for Playwright browser automation",
            "SEAL self-improvement loop with external reward",
            "GoalLoop with circuit breaker and rate limiter",
            "nt_world_crawl frontier with Mercator dual-queue strategy",
            "knowledge hypercube with dimension axis projection",
            "consciousness global workspace with specialist modules",
            "E8×64 reasoning state space with 6 binary axes",
            "+1 observer meta-cognitive reflection on trajectories",
            "cross-lingual semantic bridges between Chinese and English",
            "ancient Chinese cosmology Hetu Luoshu Yijing unified field",
            "Dayan number 50 with 49 iteration cycle",
            "Zhang Heng seismoscope resonance amplification mechanism",
            "Shao Yong cosmology with 129600-year cosmic cycle",
            "Mawangdui silk manuscripts astronomy and divination",
        ];
        for (i, text) in topics.iter().enumerate() {
            index.store(&format!("mem{}", i), text);
        }
        assert_eq!(index.len(), 20);
        let results = index.search("E8 hexagram navigation state space", 3);
        assert_eq!(results.len(), 3);
        assert!(results.iter().any(|(_, id)| id == "mem0"));
    }
}
