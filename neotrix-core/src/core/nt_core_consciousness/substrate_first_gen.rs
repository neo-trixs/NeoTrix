// REVIVED Task 1 — dead_code removed 2026-06-24

use std::collections::HashMap;

/// Token generated from the ODE substrate before any LLM call.
#[derive(Debug, Clone)]
pub struct SubstrateToken {
    pub token_id: u64,
    pub ode_state: Vec<f64>,
    pub content: String,
    pub confidence: f64,
    pub timestamp: u64,
    pub generation_mode: GenerationMode,
}

/// Mode of generation from ODE substrate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GenerationMode {
    DirectReadout,
    VsaGuided,
    Hybrid,
}

/// Substrate-first token generator.
/// Ensures ODE readout happens BEFORE LLM call.
#[derive(Debug, Clone)]
pub struct SubstrateFirstGenerator {
    pub substrate_dim: usize,
    pub vocabulary: HashMap<u64, String>,
    pub vsa_vectors: HashMap<u64, Vec<f64>>,
    pub temperature: f64,
    pub top_k: usize,
    pub generated: Vec<SubstrateToken>,
    pub max_history: usize,
}

impl SubstrateFirstGenerator {
    pub fn new(substrate_dim: usize) -> Self {
        Self {
            substrate_dim,
            vocabulary: HashMap::new(),
            vsa_vectors: HashMap::new(),
            temperature: 1.0,
            top_k: 5,
            generated: Vec::new(),
            max_history: 100,
        }
    }

    pub fn register_concept(&mut self, id: u64, text: &str, vsa: Vec<f64>) {
        self.vocabulary.insert(id, text.to_string());
        self.vsa_vectors.insert(id, vsa);
    }

    pub fn generate_from_ode(
        &mut self,
        ode_state: &[f64],
        mode: GenerationMode,
        tick: u64,
    ) -> SubstrateToken {
        let token_id = self.generated.len() as u64;

        let (content, confidence) = match mode {
            GenerationMode::DirectReadout => {
                if let Some((_, text, sim)) = self.nearest_concept(ode_state) {
                    (text, sim)
                } else {
                    ("<no concept>".to_string(), 0.0)
                }
            }
            GenerationMode::VsaGuided => {
                let mut best_text = "<no concept>".to_string();
                let mut best_conf = 0.0;
                for (id, vsa_vec) in &self.vsa_vectors {
                    if vsa_vec.len() != ode_state.len() {
                        continue;
                    }
                    let bound: Vec<f64> = ode_state
                        .iter()
                        .zip(vsa_vec.iter())
                        .map(|(o, v)| o * v)
                        .collect();
                    let sim = self.cosine_similarity(&bound, vsa_vec);
                    if sim > best_conf {
                        best_conf = sim;
                        best_text = self.vocabulary.get(id).cloned().unwrap_or_default();
                    }
                }
                (best_text, best_conf)
            }
            GenerationMode::Hybrid => {
                let (d_text, d_conf) = if let Some((_, text, sim)) = self.nearest_concept(ode_state)
                {
                    (text, sim)
                } else {
                    ("<no concept>".to_string(), 0.0)
                };

                let (v_text, v_conf) = {
                    let mut best_text = "<no concept>".to_string();
                    let mut best_conf = 0.0;
                    for (id, vsa_vec) in &self.vsa_vectors {
                        if vsa_vec.len() != ode_state.len() {
                            continue;
                        }
                        let bound: Vec<f64> = ode_state
                            .iter()
                            .zip(vsa_vec.iter())
                            .map(|(o, v)| o * v)
                            .collect();
                        let sim = self.cosine_similarity(&bound, vsa_vec);
                        if sim > best_conf {
                            best_conf = sim;
                            best_text = self.vocabulary.get(id).cloned().unwrap_or_default();
                        }
                    }
                    (best_text, best_conf)
                };

                let avg_conf = (d_conf + v_conf) / 2.0;
                let text = if avg_conf > 0.0 {
                    if d_conf >= v_conf {
                        d_text
                    } else {
                        v_text
                    }
                } else {
                    "<no concept>".to_string()
                };
                (text, avg_conf)
            }
        };

        let token = SubstrateToken {
            token_id,
            ode_state: ode_state.to_vec(),
            content,
            confidence,
            timestamp: tick,
            generation_mode: mode,
        };

        self.generated.push(token.clone());
        if self.generated.len() > self.max_history {
            self.generated.remove(0);
        }

        token
    }

    pub fn cosine_similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
    }

    pub fn nearest_concept(&self, vector: &[f64]) -> Option<(u64, String, f64)> {
        let mut best: Option<(u64, f64)> = None;
        for (id, vsa_vec) in &self.vsa_vectors {
            let sim = self.cosine_similarity(vector, vsa_vec);
            if best.map_or(true, |(_, s)| sim > s) {
                best = Some((*id, sim));
            }
        }
        best.and_then(|(id, sim)| self.vocabulary.get(&id).map(|text| (id, text.clone(), sim)))
    }

    pub fn top_k_concepts(&self, vector: &[f64], k: usize) -> Vec<(u64, String, f64)> {
        let mut scored: Vec<(u64, f64)> = self
            .vsa_vectors
            .keys()
            .map(|id| (*id, self.cosine_similarity(vector, &self.vsa_vectors[id])))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
            .into_iter()
            .filter_map(|(id, sim)| self.vocabulary.get(&id).map(|text| (id, text.clone(), sim)))
            .collect()
    }

    pub fn readout_entropy(&self, ode_state: &[f64]) -> f64 {
        let n = self.vsa_vectors.len();
        if n == 0 {
            return 0.0;
        }
        let similarities: Vec<f64> = self
            .vsa_vectors
            .values()
            .map(|v| self.cosine_similarity(ode_state, v))
            .collect();

        let max_sim = similarities
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let exp_sum: f64 = similarities.iter().map(|s| (s - max_sim).exp()).sum();
        let probs: Vec<f64> = similarities
            .iter()
            .map(|s| (s - max_sim).exp() / exp_sum)
            .collect();

        let entropy: f64 = probs
            .iter()
            .filter(|p| **p > 0.0)
            .map(|p| -p * p.ln())
            .sum();

        let max_entropy = (n as f64).ln();
        if max_entropy == 0.0 {
            0.0
        } else {
            (entropy / max_entropy).clamp(0.0, 1.0)
        }
    }

    pub fn readout_history(&self, n: usize) -> Vec<&SubstrateToken> {
        let len = self.generated.len();
        let start = if len > n { len - n } else { 0 };
        self.generated[start..].iter().collect()
    }

    pub fn substrate_richness(&self) -> f64 {
        let len = self.generated.len();
        if len == 0 {
            return 0.0;
        }
        let mut seen = std::collections::HashSet::new();
        for token in &self.generated {
            seen.insert(token.content.clone());
        }
        seen.len() as f64 / len as f64
    }

    pub fn reset(&mut self) {
        self.generated.clear();
        self.vocabulary.clear();
        self.vsa_vectors.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_new() {
        let gen = SubstrateFirstGenerator::new(64);
        assert_eq!(gen.substrate_dim, 64);
        assert!(gen.vocabulary.is_empty());
        assert!(gen.vsa_vectors.is_empty());
        assert_eq!(gen.temperature, 1.0);
        assert_eq!(gen.top_k, 5);
        assert!(gen.generated.is_empty());
    }

    #[test]
    fn test_register_concept() {
        let mut gen = SubstrateFirstGenerator::new(64);
        let vsa = vec![0.1; 64];
        gen.register_concept(1, "hello", vsa.clone());
        assert_eq!(gen.vocabulary.len(), 1);
        assert_eq!(gen.vocabulary.get(&1).unwrap(), "hello");
        assert_eq!(gen.vsa_vectors.len(), 1);
    }

    #[test]
    fn test_direct_readout_basic() {
        let mut gen = SubstrateFirstGenerator::new(4);
        gen.register_concept(1, "concept_a", vec![1.0, 0.0, 0.0, 0.0]);
        gen.register_concept(2, "concept_b", vec![0.0, 1.0, 0.0, 0.0]);

        let token = gen.generate_from_ode(&[0.9, 0.1, 0.0, 0.0], GenerationMode::DirectReadout, 1);
        assert_eq!(token.content, "concept_a");
        assert!(token.confidence > 0.9);
        assert_eq!(token.timestamp, 1);
        assert_eq!(token.token_id, 0);
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let gen = SubstrateFirstGenerator::new(64);
        let v = vec![1.0, 2.0, 3.0, 4.0];
        let sim = gen.cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_nearest_concept() {
        let mut gen = SubstrateFirstGenerator::new(3);
        gen.register_concept(10, "x", vec![1.0, 0.0, 0.0]);
        gen.register_concept(20, "y", vec![0.0, 1.0, 0.0]);
        gen.register_concept(30, "z", vec![0.0, 0.0, 1.0]);

        let result = gen.nearest_concept(&[0.0, 0.9, 0.1]);
        assert!(result.is_some());
        let (id, text, sim) = result.unwrap();
        assert_eq!(id, 20);
        assert_eq!(text, "y");
        assert!(sim > 0.9);
    }

    #[test]
    fn test_top_k_concepts() {
        let mut gen = SubstrateFirstGenerator::new(3);
        gen.register_concept(1, "a", vec![1.0, 0.0, 0.0]);
        gen.register_concept(2, "b", vec![0.0, 1.0, 0.0]);
        gen.register_concept(3, "c", vec![0.0, 0.0, 1.0]);

        let top = gen.top_k_concepts(&[0.8, 0.5, 0.1], 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, 1);
        assert_eq!(top[1].0, 2);
    }

    #[test]
    fn test_readout_entropy_low_when_deterministic() {
        let mut gen = SubstrateFirstGenerator::new(4);
        gen.register_concept(1, "c1", vec![1.0, 0.0, 0.0, 0.0]);
        gen.register_concept(2, "c2", vec![0.0, 1.0, 0.0, 0.0]);
        gen.register_concept(3, "c3", vec![0.0, 0.0, 1.0, 0.0]);

        let entropy = gen.readout_entropy(&[1.0, 0.0, 0.0, 0.0]);
        assert!(
            entropy < 0.5,
            "entropy should be low for deterministic readout, got {}",
            entropy
        );

        let entropy_high = gen.readout_entropy(&[0.5, 0.5, 0.5, 0.0]);
        assert!(
            entropy_high > entropy,
            "ambiguous state should have higher entropy"
        );
    }

    #[test]
    fn test_generate_multiple_tokens() {
        let mut gen = SubstrateFirstGenerator::new(4);
        gen.register_concept(1, "alpha", vec![1.0, 0.0, 0.0, 0.0]);
        gen.register_concept(2, "beta", vec![0.0, 1.0, 0.0, 0.0]);

        let t1 = gen.generate_from_ode(&[0.9, 0.1, 0.0, 0.0], GenerationMode::DirectReadout, 10);
        let t2 = gen.generate_from_ode(&[0.1, 0.9, 0.0, 0.0], GenerationMode::DirectReadout, 20);

        assert_eq!(t1.token_id, 0);
        assert_eq!(t2.token_id, 1);
        assert_eq!(t1.content, "alpha");
        assert_eq!(t2.content, "beta");
        assert_eq!(gen.generated.len(), 2);

        let history = gen.readout_history(5);
        assert_eq!(history.len(), 2);
    }
}
