use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Debug, Clone)]
pub struct Hypothesis {
    pub id: String,
    pub vsa_vector: Vec<u8>,
    pub confidence: f64,
    pub source: String,
}

impl Hypothesis {
    pub fn new(
        id: impl Into<String>,
        vsa_vector: Vec<u8>,
        confidence: f64,
        source: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            vsa_vector,
            confidence: confidence.clamp(0.0, 1.0),
            source: source.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HypothesisSet {
    pub name: String,
    pub hypotheses: Vec<Hypothesis>,
}

impl HypothesisSet {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            hypotheses: Vec::new(),
        }
    }

    pub fn add(&mut self, h: Hypothesis) {
        self.hypotheses.push(h);
    }

    pub fn len(&self) -> usize {
        self.hypotheses.len()
    }

    pub fn is_empty(&self) -> bool {
        self.hypotheses.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct HypothesisEvaluationConfig {
    pub capacity: usize,
    pub diversity_threshold: f64,
    pub min_confidence: f64,
    pub enable_synthesis: bool,
}

impl Default for HypothesisEvaluationConfig {
    fn default() -> Self {
        Self {
            capacity: 64,
            diversity_threshold: 0.3,
            min_confidence: 0.1,
            enable_synthesis: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScoredHypothesis {
    pub hypothesis: Hypothesis,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub struct ParallelHypothesisEvaluator {
    config: HypothesisEvaluationConfig,
    pool: Vec<Hypothesis>,
    scored: Vec<ScoredHypothesis>,
    evaluated: bool,
}

impl ParallelHypothesisEvaluator {
    pub fn new(config: HypothesisEvaluationConfig) -> Self {
        Self {
            config,
            pool: Vec::new(),
            scored: Vec::new(),
            evaluated: false,
        }
    }

    pub fn add_hypothesis(&mut self, h: Hypothesis) {
        if self.pool.len() < self.config.capacity {
            self.pool.push(h);
            self.evaluated = false;
        }
    }

    pub fn evaluate_all(&mut self) {
        let n = self.pool.len();
        if n == 0 {
            self.scored = Vec::new();
            self.evaluated = true;
            return;
        }

        let mut scores = vec![0.0f64; n];
        let min_conf = self.config.min_confidence;

        for i in 0..n {
            let mut agreement_sum = 0.0;
            let mut count = 0;
            for j in 0..n {
                if i == j {
                    continue;
                }
                if self.pool[j].confidence < min_conf {
                    continue;
                }
                let sim = QuantizedVSA::cosine(&self.pool[i].vsa_vector, &self.pool[j].vsa_vector);
                let weighted = sim * self.pool[j].confidence;
                agreement_sum += weighted.max(0.0);
                count += 1;
            }
            let agreement = if count > 0 {
                agreement_sum / count as f64
            } else {
                0.0
            };
            scores[i] = self.pool[i].confidence * 0.6 + agreement * 0.4;
        }

        self.scored = self
            .pool
            .drain(..)
            .zip(scores.iter())
            .map(|(h, &s)| ScoredHypothesis {
                hypothesis: h,
                score: s,
            })
            .collect();

        self.scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.evaluated = true;
    }

    pub fn ranked_results(&self) -> &[ScoredHypothesis] {
        &self.scored
    }

    pub fn winner(&self) -> Option<&ScoredHypothesis> {
        self.scored.first()
    }

    pub fn diversity_score(&self, set: &[&Hypothesis]) -> f64 {
        if set.len() < 2 {
            return 1.0;
        }
        let mut total_dist = 0.0;
        let mut pairs = 0;
        for i in 0..set.len() {
            for j in (i + 1)..set.len() {
                let sim = QuantizedVSA::cosine(&set[i].vsa_vector, &set[j].vsa_vector);
                total_dist += 1.0 - sim.max(0.0);
                pairs += 1;
            }
        }
        total_dist / pairs as f64
    }

    pub fn consensus_set(&self) -> HypothesisSet {
        if self.scored.is_empty() {
            return HypothesisSet::new("consensus");
        }
        let threshold = self.config.diversity_threshold;
        let mut selected: Vec<&Hypothesis> = Vec::new();
        for sh in &self.scored {
            if sh.hypothesis.confidence < self.config.min_confidence {
                continue;
            }
            let mut too_close = false;
            for existing in &selected {
                let sim = QuantizedVSA::cosine(&sh.hypothesis.vsa_vector, &existing.vsa_vector);
                if sim > 1.0 - threshold {
                    too_close = true;
                    break;
                }
            }
            if !too_close {
                selected.push(&sh.hypothesis);
            }
        }
        let mut set = HypothesisSet::new("consensus");
        for h in selected {
            set.add(h.clone());
        }
        set
    }

    pub fn hypothesis_synthesis(&self, consensus_set: &HypothesisSet) -> Option<Hypothesis> {
        if consensus_set.is_empty() || !self.config.enable_synthesis {
            return None;
        }
        if consensus_set.len() == 1 {
            return Some(consensus_set.hypotheses[0].clone());
        }
        let vsa_refs: Vec<&[u8]> = consensus_set
            .hypotheses
            .iter()
            .map(|h| h.vsa_vector.as_slice())
            .collect();
        let bundled = QuantizedVSA::bundle(&vsa_refs);
        let avg_conf: f64 = consensus_set
            .hypotheses
            .iter()
            .map(|h| h.confidence)
            .sum::<f64>()
            / consensus_set.len() as f64;
        let sources: Vec<&str> = consensus_set
            .hypotheses
            .iter()
            .map(|h| h.source.as_str())
            .collect();
        let source_str = sources.join("+");
        Some(Hypothesis::new("synthesis", bundled, avg_conf, source_str))
    }

    pub fn config(&self) -> &HypothesisEvaluationConfig {
        &self.config
    }

    pub fn pool_len(&self) -> usize {
        self.pool.len() + self.scored.len()
    }

    pub fn clear(&mut self) {
        self.pool.clear();
        self.scored.clear();
        self.evaluated = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::{QuantizedVSA, VSA_DIM};

    fn make_hypothesis(id: &str, seed: u64, confidence: f64, source: &str) -> Hypothesis {
        let vsa = QuantizedVSA::seeded_random(seed, VSA_DIM);
        Hypothesis::new(id, vsa, confidence, source)
    }

    #[test]
    fn test_hypothesis_new_clamps_confidence() {
        let h = Hypothesis::new("h1", vec![1; 16], 1.5, "test");
        assert!((h.confidence - 1.0).abs() < 1e-9);
        let h2 = Hypothesis::new("h2", vec![0; 16], -0.5, "test");
        assert!((h2.confidence - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_empty_evaluator() {
        let mut ev = ParallelHypothesisEvaluator::new(HypothesisEvaluationConfig::default());
        ev.evaluate_all();
        assert!(ev.ranked_results().is_empty());
        assert!(ev.winner().is_none());
    }

    #[test]
    fn test_single_hypothesis_wins() {
        let mut ev = ParallelHypothesisEvaluator::new(HypothesisEvaluationConfig::default());
        ev.add_hypothesis(make_hypothesis("h1", 42, 0.9, "reasoning"));
        ev.evaluate_all();
        let w = ev.winner().unwrap();
        assert_eq!(w.hypothesis.id, "h1");
        assert_eq!(w.hypothesis.source, "reasoning");
    }

    #[test]
    fn test_ranking_orders_by_score() {
        let mut ev = ParallelHypothesisEvaluator::new(HypothesisEvaluationConfig::default());
        ev.add_hypothesis(make_hypothesis("low", 1, 0.3, "a"));
        ev.add_hypothesis(make_hypothesis("high", 2, 0.9, "b"));
        ev.add_hypothesis(make_hypothesis("mid", 3, 0.6, "c"));
        ev.evaluate_all();
        let ranked = ev.ranked_results();
        assert_eq!(ranked[0].hypothesis.id, "high");
        assert_eq!(ranked[1].hypothesis.id, "mid");
        assert_eq!(ranked[2].hypothesis.id, "low");
    }

    #[test]
    fn test_diversity_score_identical_vectors() {
        let v = vec![1; 64];
        let h1 = Hypothesis::new("a", v.clone(), 0.8, "src");
        let h2 = Hypothesis::new("b", v, 0.8, "src");
        let set = vec![&h1, &h2];
        let mut ev = ParallelHypothesisEvaluator::new(HypothesisEvaluationConfig::default());
        let div = ev.diversity_score(&set);
        assert!((div - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_diversity_score_orthogonal_vectors() {
        let v1 = vec![1; 64];
        let v2 = vec![0; 64];
        let h1 = Hypothesis::new("a", v1, 0.8, "src");
        let h2 = Hypothesis::new("b", v2, 0.8, "src");
        let set = vec![&h1, &h2];
        let mut ev = ParallelHypothesisEvaluator::new(HypothesisEvaluationConfig::default());
        let div = ev.diversity_score(&set);
        assert!(div > 0.99);
    }

    #[test]
    fn test_diversity_score_single() {
        let h = make_hypothesis("h", 1, 0.8, "src");
        let set = vec![&h];
        let mut ev = ParallelHypothesisEvaluator::new(HypothesisEvaluationConfig::default());
        assert!((ev.diversity_score(&set) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_consensus_set_filters_close_hypotheses() {
        let mut ev = ParallelHypothesisEvaluator::new(HypothesisEvaluationConfig {
            diversity_threshold: 0.3,
            min_confidence: 0.0,
            enable_synthesis: true,
            capacity: 64,
        });
        ev.add_hypothesis(make_hypothesis("h1", 100, 0.9, "src1"));
        ev.add_hypothesis(make_hypothesis("h2", 100, 0.8, "src2"));
        ev.add_hypothesis(make_hypothesis("h3", 200, 0.7, "src3"));
        ev.evaluate_all();
        let consensus = ev.consensus_set();
        assert!(consensus.len() <= 2);
    }

    #[test]
    fn test_hypothesis_synthesis_bundles_vectors() {
        let mut ev = ParallelHypothesisEvaluator::new(HypothesisEvaluationConfig {
            diversity_threshold: 0.1,
            min_confidence: 0.0,
            enable_synthesis: true,
            capacity: 64,
        });
        ev.add_hypothesis(make_hypothesis("h1", 100, 0.8, "src1"));
        ev.add_hypothesis(make_hypothesis("h2", 101, 0.9, "src2"));
        ev.evaluate_all();
        let consensus = ev.consensus_set();
        let synth = ev.hypothesis_synthesis(&consensus);
        assert!(synth.is_some());
        let s = synth.unwrap();
        assert_eq!(s.id, "synthesis");
        assert!(s.source.contains("src1"));
        assert!(s.source.contains("src2"));
        assert_eq!(s.vsa_vector.len(), VSA_DIM);
    }

    #[test]
    fn test_hypothesis_synthesis_single_returns_copy() {
        let mut ev = ParallelHypothesisEvaluator::new(HypothesisEvaluationConfig::default());
        ev.add_hypothesis(make_hypothesis("single", 42, 0.9, "alone"));
        ev.evaluate_all();
        let consensus = ev.consensus_set();
        let synth = ev.hypothesis_synthesis(&consensus);
        assert!(synth.is_some());
        assert_eq!(synth.unwrap().id, "single");
    }

    #[test]
    fn test_hypothesis_synthesis_disabled_returns_none() {
        let mut ev = ParallelHypothesisEvaluator::new(HypothesisEvaluationConfig {
            enable_synthesis: false,
            ..Default::default()
        });
        ev.add_hypothesis(make_hypothesis("h1", 100, 0.8, "src1"));
        ev.evaluate_all();
        let consensus = ev.consensus_set();
        assert!(ev.hypothesis_synthesis(&consensus).is_none());
    }

    #[test]
    fn test_hypothesis_synthesis_empty_set_returns_none() {
        let ev = ParallelHypothesisEvaluator::new(HypothesisEvaluationConfig::default());
        let empty = HypothesisSet::new("empty");
        assert!(ev.hypothesis_synthesis(&empty).is_none());
    }

    #[test]
    fn test_clear_resets_evaluator() {
        let mut ev = ParallelHypothesisEvaluator::new(HypothesisEvaluationConfig::default());
        ev.add_hypothesis(make_hypothesis("h", 1, 0.5, "s"));
        ev.evaluate_all();
        assert_eq!(ev.pool_len(), 1);
        ev.clear();
        assert_eq!(ev.pool_len(), 0);
        assert!(ev.winner().is_none());
    }

    #[test]
    fn test_hypothesis_set_basic() {
        let mut set = HypothesisSet::new("test");
        assert!(set.is_empty());
        set.add(make_hypothesis("h1", 10, 0.7, "src"));
        assert_eq!(set.len(), 1);
        assert!(!set.is_empty());
    }

    #[test]
    fn test_capacity_limits_pool() {
        let mut ev = ParallelHypothesisEvaluator::new(HypothesisEvaluationConfig {
            capacity: 2,
            ..Default::default()
        });
        ev.add_hypothesis(make_hypothesis("h1", 1, 0.5, "s"));
        ev.add_hypothesis(make_hypothesis("h2", 2, 0.5, "s"));
        ev.add_hypothesis(make_hypothesis("h3", 3, 0.5, "s"));
        ev.evaluate_all();
        assert_eq!(ev.ranked_results().len(), 2);
    }
}
