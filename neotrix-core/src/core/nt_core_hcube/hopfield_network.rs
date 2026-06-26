// REVIVED Task 2 — dead_code removed


#[derive(Debug, Clone)]
pub struct HopfieldNetwork {
    patterns: Vec<Vec<f64>>,
    dim: usize,
    beta: f64,
}

impl HopfieldNetwork {
    pub fn new(dim: usize, beta: f64) -> Self {
        Self {
            patterns: Vec::new(),
            dim,
            beta,
        }
    }

    pub fn store(&mut self, pattern: Vec<f64>) {
        assert_eq!(pattern.len(), self.dim);
        self.patterns.push(pattern);
    }

    pub fn energy(&self, state: &[f64]) -> f64 {
        let n = self.patterns.len();
        if n == 0 || state.len() != self.dim {
            return 0.0;
        }
        let mut e = 0.0;
        for p in &self.patterns {
            let dot: f64 = state.iter().zip(p.iter()).map(|(x, y)| x * y).sum();
            e += (-self.beta * dot).exp();
        }
        -e
    }

    pub fn retrieve(&self, probe: &[f64]) -> Vec<f64> {
        let mut out = probe.to_vec();
        for _ in 0..10 {
            let energy = self.energy(&out);
            for i in 0..self.dim {
                let _orig = out[i];
                out[i] = 1.0;
                let e1 = self.energy(&out);
                out[i] = -1.0;
                let e2 = self.energy(&out);
                out[i] = if e1 < e2 { 1.0 } else { -1.0 };
            }
            let new_energy = self.energy(&out);
            if (new_energy - energy).abs() < 1e-6 {
                break;
            }
        }
        out
    }

    pub fn retrieve_by_energy(&self, probe: &[f64]) -> Vec<(usize, f64)> {
        let mut scored: Vec<(usize, f64)> = self
            .patterns
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let dot: f64 = probe.iter().zip(p.iter()).map(|(x, y)| x * y).sum();
                let np: f64 = p.iter().map(|x| x * x).sum::<f64>().sqrt();
                let nq: f64 = probe.iter().map(|x| x * x).sum::<f64>().sqrt();
                let sim = if np == 0.0 || nq == 0.0 {
                    0.0
                } else {
                    dot / (np * nq)
                };
                (i, self.energy(p) + sim)
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
    }

    pub fn complete(&self, probe: &[f64], missing_mask: &[bool]) -> Vec<f64> {
        let filled = self.retrieve(probe);
        let mut result = probe.to_vec();
        for (i, &mask) in missing_mask.iter().enumerate() {
            if mask {
                result[i] = filled[i];
            }
        }
        result
    }

    pub fn capacity(&self) -> f64 {
        self.dim as f64 / (4.0 * (self.patterns.len() as f64).ln().max(1.0))
    }

    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
    pub fn dimension(&self) -> usize {
        self.dim
    }
    pub fn clear(&mut self) {
        self.patterns.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    fn bipolar(dim: usize, rng: &mut impl rand::Rng) -> Vec<f64> {
        (0..dim)
            .map(|_| if rng.gen_bool(0.5) { 1.0 } else { -1.0 })
            .collect()
    }

    #[test]
    fn test_store_and_retrieve() {
        let mut hop = HopfieldNetwork::new(64, 1.0);
        let mut rng = rand::thread_rng();
        let p = bipolar(64, &mut rng);
        hop.store(p.clone());
        let noisy: Vec<f64> = p
            .iter()
            .map(|x| if rng.gen_bool(0.2) { -x } else { *x })
            .collect();
        let retrieved = hop.retrieve(&noisy);
        let sim: f64 = p
            .iter()
            .zip(retrieved.iter())
            .filter(|(a, b)| a == b)
            .count() as f64
            / 64.0;
        assert!(sim > 0.7, "retrieval sim={}", sim);
    }

    #[test]
    fn test_multiple_patterns() {
        let mut hop = HopfieldNetwork::new(32, 1.0);
        let mut rng = rand::thread_rng();
        let patterns: Vec<Vec<f64>> = (0..5).map(|_| bipolar(32, &mut rng)).collect();
        for p in &patterns {
            hop.store(p.clone());
        }
        for p in &patterns {
            let retrieved = hop.retrieve(p);
            let sim: f64 = p
                .iter()
                .zip(retrieved.iter())
                .filter(|(a, b)| a == b)
                .count() as f64
                / 32.0;
            assert!(sim > 0.6, "multi-pattern retrieval sim={}", sim);
        }
    }

    #[test]
    fn test_energy_decreases() {
        let mut hop = HopfieldNetwork::new(32, 1.0);
        let mut rng = rand::thread_rng();
        let p = bipolar(32, &mut rng);
        hop.store(p.clone());
        let noisy: Vec<f64> = p
            .iter()
            .map(|x| if rng.gen_bool(0.3) { -x } else { *x })
            .collect();
        let e_before = hop.energy(&noisy);
        let retrieved = hop.retrieve(&noisy);
        let e_after = hop.energy(&retrieved);
        assert!(e_after <= e_before + 1e-6, "energy should decrease");
    }

    #[test]
    fn test_capacity_estimate() {
        let hop = HopfieldNetwork::new(128, 1.0);
        let cap = hop.capacity();
        assert!(cap.is_finite());
        assert!(cap > 0.0);
    }

    #[test]
    fn test_retrieve_by_energy_ranking() {
        let mut hop = HopfieldNetwork::new(32, 1.0);
        let mut rng = rand::thread_rng();
        let p1 = bipolar(32, &mut rng);
        let p2 = bipolar(32, &mut rng);
        hop.store(p1.clone());
        hop.store(p2.clone());
        let ranked = hop.retrieve_by_energy(&p1);
        assert_eq!(ranked[0].0, 0, "first pattern should rank highest");
    }

    #[test]
    fn test_complete_missing() {
        let mut hop = HopfieldNetwork::new(32, 1.0);
        let mut rng = rand::thread_rng();
        let p = bipolar(32, &mut rng);
        hop.store(p.clone());
        let mut probe = vec![0.0; 32];
        let mask: Vec<bool> = (0..32).map(|i| i < 16).collect();
        for i in 16..32 {
            probe[i] = p[i];
        }
        let completed = hop.complete(&probe, &mask);
        let sim: f64 = p
            .iter()
            .zip(completed.iter())
            .filter(|(a, b)| a == b)
            .count() as f64
            / 32.0;
        assert!(sim > 0.5, "pattern completion sim={}", sim);
    }

    #[test]
    fn test_clear() {
        let mut hop = HopfieldNetwork::new(32, 1.0);
        let mut rng = rand::thread_rng();
        hop.store(bipolar(32, &mut rng));
        hop.clear();
        assert_eq!(hop.pattern_count(), 0);
    }

    #[test]
    fn test_energy_of_stored_pattern() {
        let mut hop = HopfieldNetwork::new(32, 1.0);
        let mut rng = rand::thread_rng();
        let p = bipolar(32, &mut rng);
        hop.store(p.clone());
        let e = hop.energy(&p);
        assert!(e < 0.0, "energy of stored pattern should be negative");
    }

    #[test]
    fn test_noise_robustness() {
        let mut hop = HopfieldNetwork::new(64, 2.0);
        let mut rng = rand::thread_rng();
        let p = bipolar(64, &mut rng);
        hop.store(p.clone());
        for noise_level in &[0.1, 0.3, 0.5] {
            let noisy: Vec<f64> = p
                .iter()
                .map(|x| if rng.gen_bool(*noise_level) { -x } else { *x })
                .collect();
            let retrieved = hop.retrieve(&noisy);
            let sim: f64 = p
                .iter()
                .zip(retrieved.iter())
                .filter(|(a, b)| a == b)
                .count() as f64
                / 64.0;
            assert!(sim > 0.5, "noise={} sim={}", noise_level, sim);
        }
    }
}
