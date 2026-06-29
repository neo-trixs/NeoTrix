use crate::core::nt_core_hcube::hlb_bind::fwht;

#[derive(Debug, Clone)]
pub struct KronekerCodebook {
    seeds: Vec<Vec<f64>>,
    dim: usize,
}

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub index: usize,
    pub seed_id: usize,
    pub similarity: f64,
}

impl KronekerCodebook {
    pub fn new(dim: usize) -> Self {
        let dim_pow2 = dim.next_power_of_two();
        KronekerCodebook {
            seeds: Vec::new(),
            dim: dim_pow2,
        }
    }

    pub fn add_seed(&mut self, seed: u64) {
        use rand::Rng;
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut vec: Vec<f64> = (0..self.dim)
            .map(|_| if rng.gen_bool(0.5) { 1.0 } else { -1.0 })
            .collect();
        fwht(&mut vec);
        self.seeds.push(vec);
    }

    pub fn seed_count(&self) -> usize {
        self.seeds.len()
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn capacity(&self) -> usize {
        self.seeds.len() * self.dim
    }

    pub fn cleanup(&self, query: &[u8], k: usize) -> Vec<MatchResult> {
        let len = query.len().min(self.dim);
        let mut qf: Vec<f64> = query
            .iter()
            .take(len)
            .map(|&x| if x == 0 { -1.0 } else { 1.0 })
            .collect();
        let original_len = qf.len();
        qf.resize(self.dim, -1.0);

        let mut results: Vec<MatchResult> = Vec::new();

        for (seed_id, seed_vec) in self.seeds.iter().enumerate() {
            let mut qc = qf.clone();
            for i in 0..self.dim {
                qc[i] *= seed_vec[i];
            }
            fwht(&mut qc);
            let dim_f = self.dim as f64;
            for i in 0..original_len {
                let sim = (qc[i] / dim_f).clamp(-1.0, 1.0);
                let idx = seed_id * self.dim + i;
                results.push(MatchResult {
                    index: idx,
                    seed_id,
                    similarity: sim,
                });
            }
        }

        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(k);
        results
    }

    pub fn cleanup_with_threshold(&self, query: &[u8], threshold: f64) -> Vec<MatchResult> {
        let k = self.capacity();
        let mut all = self.cleanup(query, k);
        all.retain(|m| m.similarity >= threshold);
        all
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

    #[test]
    fn test_new_codebook_empty() {
        let cb = KronekerCodebook::new(64);
        assert_eq!(cb.seed_count(), 0);
        assert_eq!(cb.capacity(), 0);
    }

    #[test]
    fn test_add_seed_increases_capacity() {
        let mut cb = KronekerCodebook::new(64);
        cb.add_seed(42);
        assert_eq!(cb.seed_count(), 1);
        assert_eq!(cb.capacity(), 64);
    }

    #[test]
    fn test_cleanup_returns_exact_k() {
        let mut cb = KronekerCodebook::new(64);
        cb.add_seed(0);
        cb.add_seed(1);
        let query = QuantizedVSA::random_binary();
        let results = cb.cleanup(&query, 5);
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_cleanup_sorted_descending() {
        let mut cb = KronekerCodebook::new(64);
        cb.add_seed(0);
        cb.add_seed(1);
        let query = QuantizedVSA::random_binary();
        let results = cb.cleanup(&query, 10);
        for w in results.windows(2) {
            assert!(w[0].similarity >= w[1].similarity);
        }
    }

    #[test]
    fn test_cleanup_similarity_in_range() {
        let mut cb = KronekerCodebook::new(64);
        cb.add_seed(0);
        let query = QuantizedVSA::random_binary();
        let results = cb.cleanup(&query, 5);
        for r in &results {
            assert!(r.similarity >= -1.0 && r.similarity <= 1.0);
        }
    }

    #[test]
    fn test_cleanup_multiple_seeds() {
        let mut cb = KronekerCodebook::new(32);
        cb.add_seed(10);
        cb.add_seed(20);
        cb.add_seed(30);
        assert_eq!(cb.capacity(), 96);
        let query = QuantizedVSA::random_binary();
        let results = cb.cleanup(&query, 50);
        assert!(results.len() <= 50);
    }

    #[test]
    fn test_cleanup_with_threshold() {
        let mut cb = KronekerCodebook::new(64);
        cb.add_seed(0);
        cb.add_seed(1);
        let query = QuantizedVSA::random_binary();
        let results = cb.cleanup_with_threshold(&query, 0.5);
        for r in &results {
            assert!(r.similarity >= 0.5);
        }
    }

    #[test]
    fn test_cleanup_different_pow2_dims() {
        for dim in [16, 32, 64] {
            let mut cb = KronekerCodebook::new(dim);
            cb.add_seed(0);
            let query: Vec<u8> = (0..dim).map(|i| if i % 3 == 0 { 1 } else { 0 }).collect();
            let results = cb.cleanup(&query, 3);
            assert_eq!(results.len(), 3, "failed for dim={}", dim);
        }
    }
}
