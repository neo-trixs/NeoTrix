use serde::{Deserialize, Serialize};

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveState {
    pub self_vector: Vec<u8>,
    pub attention_focus: Vec<u8>,
    pub working_memory_content: Vec<u8>,
    pub emotional_valence: f64,
    pub current_domain: String,
    pub cycle_count: u64,
    pub recent_coherence: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct IngestionConfig {
    pub max_state_history: usize,
    pub bundle_dim: usize,
    pub state_ingestion_interval: u64,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            max_state_history: 100,
            bundle_dim: 4096,
            state_ingestion_interval: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DimSnapshot {
    pub state: CognitiveState,
    pub phase: String,
    pub subsystems_active: Vec<String>,
}

impl DimSnapshot {
    /// Compute SHA-256 hash over the cognitive state dimensions.
    pub fn compute_hash(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&self.state.self_vector);
        hasher.update(&self.state.attention_focus);
        hasher.update(&self.state.working_memory_content);
        hasher.update(&self.state.emotional_valence.to_le_bytes());
        hasher.update(self.state.current_domain.as_bytes());
        hasher.update(&self.state.cycle_count.to_le_bytes());
        hasher.update(&self.state.recent_coherence.to_le_bytes());
        hasher.update(&self.state.timestamp.to_le_bytes());
        hasher.update(self.phase.as_bytes());
        for sub in &self.subsystems_active {
            hasher.update(sub.as_bytes());
        }
        let result = hasher.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&result);
        arr
    }
}

pub struct CognitiveStateIngestion {
    current_state: CognitiveState,
    state_history: Vec<CognitiveState>,
    config: IngestionConfig,
    step: u64,
}

impl CognitiveStateIngestion {
    pub fn new(config: IngestionConfig) -> Self {
        let dim = config.bundle_dim;
        Self {
            current_state: CognitiveState {
                self_vector: vec![0u8; dim],
                attention_focus: vec![0u8; dim],
                working_memory_content: vec![0u8; dim],
                emotional_valence: 0.0,
                current_domain: String::new(),
                cycle_count: 0,
                recent_coherence: 0.0,
                timestamp: 0,
            },
            state_history: Vec::with_capacity(config.max_state_history),
            config,
            step: 0,
        }
    }

    pub fn ingest(
        &mut self,
        self_vsa: &[u8],
        wm_vsa: &[u8],
        attention_vsa: &[u8],
        valence: f64,
        domain: &str,
    ) {
        let coherence = if self.state_history.is_empty() {
            valence.clamp(-1.0, 1.0)
        } else {
            let prev = &self.state_history[self.state_history.len() - 1];
            let sim = QuantizedVSA::similarity(self_vsa, &prev.self_vector);
            let mut coh = sim;
            if !self.state_history.is_empty() {
                let window = self.state_history.len().min(5);
                let avg_sim: f64 = self
                    .state_history
                    .iter()
                    .rev()
                    .take(window)
                    .map(|s| QuantizedVSA::similarity(self_vsa, &s.self_vector))
                    .sum::<f64>()
                    / window as f64;
                coh = (sim + avg_sim) / 2.0;
            }
            coh
        };

        let state = CognitiveState {
            self_vector: self_vsa.to_vec(),
            attention_focus: attention_vsa.to_vec(),
            working_memory_content: wm_vsa.to_vec(),
            emotional_valence: valence.clamp(-1.0, 1.0),
            current_domain: domain.to_string(),
            cycle_count: self.step,
            recent_coherence: coherence,
            timestamp: self.step,
        };

        self.current_state = state.clone();
        self.state_history.push(state);
        if self.state_history.len() > self.config.max_state_history {
            self.state_history.remove(0);
        }
        self.step += 1;
    }

    pub fn current_state(&self) -> &CognitiveState {
        &self.current_state
    }

    pub fn cognitive_state_vector(&self) -> Vec<u8> {
        let dim = self.config.bundle_dim;
        let domain_vsa = self._domain_to_vsa(&self.current_state.current_domain, dim);
        QuantizedVSA::majority_bundle(&[
            &self.current_state.self_vector,
            &self.current_state.attention_focus,
            &self.current_state.working_memory_content,
            &domain_vsa,
        ])
    }

    fn _domain_to_vsa(&self, domain: &str, dim: usize) -> Vec<u8> {
        let _rng = rand::thread_rng();
        let mut v = vec![0u8; dim];
        let seed: u64 = domain
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        let mut state = seed;
        for i in 0..dim {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            v[i] = (((state >> ((i % 8) * 8)) & 0xFF) % 2) as u8;
        }
        v
    }

    pub fn state_similarity(&self, other: &CognitiveState) -> f64 {
        QuantizedVSA::similarity(&self.current_state.self_vector, &other.self_vector)
    }

    pub fn state_trend(&self, window: usize) -> String {
        if self.state_history.len() < 2 {
            return "insufficient".to_string();
        }
        let window = window.min(self.state_history.len());
        let relevant: Vec<&CognitiveState> = self.state_history.iter().rev().take(window).collect();
        if relevant.len() < 2 {
            return "insufficient".to_string();
        }
        let mut increases = 0;
        let mut decreases = 0;
        for i in 1..relevant.len() {
            let prev = relevant[i].recent_coherence;
            let curr = relevant[i - 1].recent_coherence;
            if curr > prev + 0.01 {
                increases += 1;
            } else if curr < prev - 0.01 {
                decreases += 1;
            }
        }
        let total = relevant.len() - 1;
        if increases as f64 / total as f64 > 0.6 {
            "integrating".to_string()
        } else if decreases as f64 / total as f64 > 0.6 {
            "destabilizing".to_string()
        } else {
            "stable".to_string()
        }
    }

    pub fn state_entropy(&self) -> f64 {
        let n = self.state_history.len().min(10);
        if n < 2 {
            return 0.0;
        }
        let recent: Vec<Vec<u8>> = self
            .state_history
            .iter()
            .rev()
            .take(n)
            .map(|s| {
                let dv = self._domain_to_vsa(&s.current_domain, self.config.bundle_dim);
                QuantizedVSA::majority_bundle(&[
                    &s.self_vector,
                    &s.attention_focus,
                    &s.working_memory_content,
                    &dv,
                ])
            })
            .collect();
        let mut patterns: Vec<Vec<u8>> = Vec::new();
        let mut counts: Vec<usize> = Vec::new();
        for v in &recent {
            let mut found = false;
            for (j, p) in patterns.iter().enumerate() {
                if QuantizedVSA::similarity(v, p) > 0.9 {
                    counts[j] += 1;
                    found = true;
                    break;
                }
            }
            if !found {
                patterns.push(v.clone());
                counts.push(1);
            }
        }
        let total = n as f64;
        let entropy: f64 = counts
            .iter()
            .map(|c| {
                let p = *c as f64 / total;
                -p * p.log2()
            })
            .sum();
        let max_entropy = (n as f64).log2();
        if max_entropy > 0.0 {
            entropy / max_entropy
        } else {
            0.0
        }
    }

    pub fn snapshot(&self, phase: &str, subsystems: &[&str]) -> DimSnapshot {
        DimSnapshot {
            state: self.current_state.clone(),
            phase: phase.to_string(),
            subsystems_active: subsystems.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn reset(&mut self) {
        let dim = self.config.bundle_dim;
        self.current_state = CognitiveState {
            self_vector: vec![0u8; dim],
            attention_focus: vec![0u8; dim],
            working_memory_content: vec![0u8; dim],
            emotional_valence: 0.0,
            current_domain: String::new(),
            cycle_count: 0,
            recent_coherence: 0.0,
            timestamp: 0,
        };
        self.state_history.clear();
        self.step = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> IngestionConfig {
        IngestionConfig {
            max_state_history: 100,
            bundle_dim: 4096,
            state_ingestion_interval: 1,
        }
    }

    #[test]
    fn test_initial_state() {
        let csi = CognitiveStateIngestion::new(default_config());
        let s = csi.current_state();
        assert!(s.self_vector.iter().all(|&b| b == 0));
        assert!(s.attention_focus.iter().all(|&b| b == 0));
        assert!(s.working_memory_content.iter().all(|&b| b == 0));
        assert_eq!(s.emotional_valence, 0.0);
        assert!(s.current_domain.is_empty());
        assert_eq!(s.cycle_count, 0);
        assert_eq!(s.timestamp, 0);
    }

    #[test]
    fn test_ingest_updates_state() {
        let mut csi = CognitiveStateIngestion::new(default_config());
        let self_vsa = vec![1u8; 4096];
        let wm_vsa = vec![0u8; 4096];
        let att_vsa = vec![1u8; 4096];
        csi.ingest(&self_vsa, &wm_vsa, &att_vsa, 0.5, "reasoning");
        assert_eq!(csi.current_state.self_vector, vec![1u8; 4096]);
        assert_eq!(csi.current_state.current_domain, "reasoning");
        assert!((csi.current_state.emotional_valence - 0.5).abs() < 1e-6);
        assert_eq!(csi.current_state.timestamp, 0);
    }

    #[test]
    fn test_state_history_max() {
        let config = IngestionConfig {
            max_state_history: 5,
            bundle_dim: 4096,
            state_ingestion_interval: 1,
        };
        let mut csi = CognitiveStateIngestion::new(config);
        let att = vec![0u8; 4096];
        for i in 0..10 {
            let sv = vec![if i % 2 == 0 { 1u8 } else { 0u8 }; 4096];
            let wm = vec![0u8; 4096];
            csi.ingest(&sv, &wm, &att, 0.0, "test");
        }
        assert_eq!(csi.state_history.len(), 5);
        assert_eq!(csi.step, 10);
    }

    #[test]
    fn test_cognitive_state_vector() {
        let mut csi = CognitiveStateIngestion::new(default_config());
        let self_vsa = vec![1u8; 4096];
        let wm_vsa = vec![0u8; 4096];
        let att_vsa = vec![1u8; 4096];
        csi.ingest(&self_vsa, &wm_vsa, &att_vsa, 0.0, "test");
        let bundled = csi.cognitive_state_vector();
        assert_eq!(bundled.len(), 4096);
    }

    #[test]
    fn test_state_similarity() {
        let mut csi = CognitiveStateIngestion::new(default_config());
        let a = vec![1u8; 4096];
        let _b = vec![0u8; 4096];
        let wm = vec![0u8; 4096];
        let att = vec![0u8; 4096];
        csi.ingest(&a, &wm, &att, 0.0, "d1");
        let other = CognitiveState {
            self_vector: a.clone(),
            attention_focus: vec![0u8; 4096],
            working_memory_content: vec![0u8; 4096],
            emotional_valence: 0.0,
            current_domain: "d2".to_string(),
            cycle_count: 0,
            recent_coherence: 0.0,
            timestamp: 0,
        };
        let sim = csi.state_similarity(&other);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_state_trend_insufficient() {
        let csi = CognitiveStateIngestion::new(default_config());
        assert_eq!(csi.state_trend(5), "insufficient");
    }

    #[test]
    fn test_state_trend_tracking() {
        let mut csi = CognitiveStateIngestion::new(default_config());
        let att = vec![0u8; 4096];
        let wm = vec![0u8; 4096];
        for i in 0..10 {
            let sv = vec![if i % 2 == 0 { 1u8 } else { 0u8 }; 4096];
            csi.ingest(&sv, &wm, &att, 0.0, "trend");
        }
        let trend = csi.state_trend(10);
        assert!(trend == "stable" || trend == "integrating" || trend == "destabilizing");
    }

    #[test]
    fn test_snapshot_creation() {
        let mut csi = CognitiveStateIngestion::new(default_config());
        let self_vsa = vec![1u8; 4096];
        let wm_vsa = vec![0u8; 4096];
        let att_vsa = vec![0u8; 4096];
        csi.ingest(&self_vsa, &wm_vsa, &att_vsa, -0.3, "reflect");
        let snap = csi.snapshot("reflect", &["e8", "gwt", "hcube"]);
        assert_eq!(snap.phase, "reflect");
        assert_eq!(snap.subsystems_active, vec!["e8", "gwt", "hcube"]);
        assert_eq!(snap.state.current_domain, "reflect");
    }

    #[test]
    fn test_reset() {
        let mut csi = CognitiveStateIngestion::new(default_config());
        let att = vec![0u8; 4096];
        let wm = vec![0u8; 4096];
        for i in 0..5 {
            let sv = vec![1u8; 4096];
            csi.ingest(&sv, &wm, &att, 0.1 * i as f64, "pre");
        }
        assert_eq!(csi.state_history.len(), 5);
        assert_eq!(csi.step, 5);
        csi.reset();
        assert_eq!(csi.step, 0);
        assert_eq!(csi.state_history.len(), 0);
        assert!(csi.current_state().self_vector.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_state_entropy_bounds() {
        let mut csi = CognitiveStateIngestion::new(default_config());
        let att = vec![0u8; 4096];
        let wm = vec![0u8; 4096];
        for i in 0..10 {
            let sv = vec![(i as u8) % 2; 4096];
            csi.ingest(&sv, &wm, &att, 0.0, "entropy");
        }
        let e = csi.state_entropy();
        assert!((0.0..=1.0).contains(&e));
    }
}
