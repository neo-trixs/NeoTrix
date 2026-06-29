use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use crate::core::nt_core_self::attention_head::AttentionDomain;
use std::collections::HashMap;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct DomainConfidence {
    pub domain: AttentionDomain,
    pub expected_success: f64,
    pub observed_success: f64,
    pub calibration_error: f64,
    pub sample_count: u64,
    pub uncertainty: f64,
}

#[derive(Debug, Clone)]
pub struct ConceptNode {
    pub id: u64,
    pub vector: Vec<u8>,
    pub label: String,
    pub confidence: f64,
    pub related_ids: Vec<u64>,
    pub last_verified: u64,
    pub verification_count: u64,
}

#[derive(Debug, Clone)]
pub struct EpistemicState {
    pub domain_confidence: HashMap<AttentionDomain, DomainConfidence>,
    pub concept_count: usize,
    pub average_calibration_error: f64,
    pub global_uncertainty: f64,
    pub total_samples: u64,
}

pub struct EpistemicConfig {
    pub min_samples_for_calibration: u64,
    pub calibration_learning_rate: f64,
    pub uncertainty_decay: f64,
    pub max_concepts: usize,
    pub novelty_threshold: f64,
}

impl Default for EpistemicConfig {
    fn default() -> Self {
        Self {
            min_samples_for_calibration: 5,
            calibration_learning_rate: 0.1,
            uncertainty_decay: 0.95,
            max_concepts: 1000,
            novelty_threshold: 0.35,
        }
    }
}

pub struct EpistemicSelfModel {
    domains: HashMap<AttentionDomain, DomainConfidence>,
    concepts: Vec<ConceptNode>,
    next_concept_id: u64,
    cycle: u64,
    pub config: EpistemicConfig,
    total_samples: u64,
    total_calibration_error: f64,
    calibration_samples: u64,
    factual_precision_history: VecDeque<f64>,
    factual_recall_history: VecDeque<f64>,
}

impl EpistemicSelfModel {
    pub fn new(config: EpistemicConfig) -> Self {
        let mut domains = HashMap::new();
        for domain in AttentionDomain::all() {
            domains.insert(
                domain,
                DomainConfidence {
                    domain,
                    expected_success: 0.5,
                    observed_success: 0.5,
                    calibration_error: 0.0,
                    sample_count: 0,
                    uncertainty: 0.8,
                },
            );
        }
        Self {
            domains,
            concepts: Vec::with_capacity(config.max_concepts),
            next_concept_id: 1,
            cycle: 0,
            config,
            total_samples: 0,
            total_calibration_error: 0.0,
            calibration_samples: 0,
            factual_precision_history: VecDeque::with_capacity(20),
            factual_recall_history: VecDeque::with_capacity(20),
        }
    }

    pub fn predict_success(&self, context_vec: &[u8]) -> f64 {
        let mut best_sim = 0.0f64;
        let mut best_domain = AttentionDomain::PatternMatch;
        for (domain, _state) in &self.domains {
            let domain_vec =
                QuantizedVSA::seeded_random(self.stable_hash(&format!("{:?}", domain)), 4096);
            let sim = QuantizedVSA::similarity(context_vec, &domain_vec);
            if sim > best_sim {
                best_sim = sim;
                best_domain = domain.clone();
            }
        }
        let domain_state = &self.domains[&best_domain];
        let novelty_penalty = 1.0 - best_sim;
        let base = domain_state.observed_success;
        let adjusted = base * (1.0 - novelty_penalty * self.config.novelty_threshold);
        adjusted * (1.0 - domain_state.uncertainty * 0.2)
    }

    pub fn calibrate(&mut self, domain: &AttentionDomain, expected: f64, actual: bool) {
        self.cycle += 1;
        let actual_f = if actual { 1.0 } else { 0.0 };
        let entry = self
            .domains
            .entry(domain.clone())
            .or_insert_with(|| DomainConfidence {
                domain: domain.clone(),
                expected_success: 0.0,
                observed_success: 0.0,
                calibration_error: 0.0,
                sample_count: 0,
                uncertainty: 0.5,
            });

        entry.sample_count += 1;
        entry.expected_success = entry.expected_success
            * (1.0 - self.config.calibration_learning_rate)
            + expected * self.config.calibration_learning_rate;
        entry.observed_success = entry.observed_success
            * (1.0 - self.config.calibration_learning_rate)
            + actual_f * self.config.calibration_learning_rate;

        let error = (expected - actual_f).abs();
        if entry.sample_count >= self.config.min_samples_for_calibration {
            entry.calibration_error = entry.calibration_error * 0.9 + error * 0.1;
        } else {
            entry.calibration_error = entry.calibration_error * 0.5 + error * 0.5;
        }

        let samples = entry.sample_count as f64;
        entry.uncertainty = 1.0 / (1.0 + samples * 0.1) + entry.calibration_error * 0.5;

        self.total_samples += 1;
        self.total_calibration_error += error;
        self.calibration_samples += 1;
    }

    pub fn track_concept(
        &mut self,
        label: &str,
        initial_confidence: f64,
        related_ids: &[u64],
    ) -> u64 {
        self.cycle += 1;
        let vec = QuantizedVSA::seeded_random(self.stable_hash(label), 4096);

        if let Some(existing) = self
            .concepts
            .iter()
            .find(|c| QuantizedVSA::similarity(&c.vector, &vec) > 0.85)
        {
            return existing.id;
        }

        if self.concepts.len() >= self.config.max_concepts {
            self.concepts.sort_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.concepts.remove(0);
        }

        let id = self.next_concept_id;
        self.next_concept_id += 1;
        self.concepts.push(ConceptNode {
            id,
            vector: vec,
            label: label.to_string(),
            confidence: initial_confidence,
            related_ids: related_ids.to_vec(),
            last_verified: self.cycle,
            verification_count: 1,
        });
        id
    }

    pub fn verify_concept(&mut self, concept_id: u64, success: bool) {
        if let Some(concept) = self.concepts.iter_mut().find(|c| c.id == concept_id) {
            concept.verification_count += 1;
            concept.last_verified = self.cycle;
            let delta = if success { 0.05 } else { -0.05 };
            concept.confidence = (concept.confidence + delta).clamp(0.0, 1.0);
        }
    }

    pub fn identify_gaps(&self, min_coverage: f64) -> Vec<&ConceptNode> {
        let mut gaps: Vec<&ConceptNode> = self
            .concepts
            .iter()
            .filter(|c| c.confidence < min_coverage && c.verification_count < 10)
            .collect();
        gaps.sort_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        gaps.truncate(20);
        gaps
    }

    pub fn compute_uncertainty_vector(&self) -> Vec<u8> {
        let mut weighted_vectors: Vec<Vec<u8>> = Vec::new();
        for (domain, state) in &self.domains {
            if state.uncertainty > 0.5 {
                let domain_vec = QuantizedVSA::seeded_random(
                    self.stable_hash(&format!("uncertain_{:?}", domain)),
                    4096,
                );
                let weighted = self.apply_weight(&domain_vec, state.uncertainty);
                weighted_vectors.push(weighted);
            }
        }
        if weighted_vectors.is_empty() {
            vec![0u8; 4096]
        } else {
            let refs: Vec<&[u8]> = weighted_vectors.iter().map(|v| v.as_slice()).collect();
            QuantizedVSA::bundle(&refs)
        }
    }

    fn apply_weight(&self, v: &[u8], weight: f64) -> Vec<u8> {
        v.iter()
            .enumerate()
            .map(|(i, &x)| {
                let keep = ((i as f64 * 0.6180339887498949 + weight).fract()) < weight;
                if keep {
                    x
                } else {
                    0
                }
            })
            .collect()
    }

    pub fn snapshot(&self) -> EpistemicState {
        let avg_cal = if self.calibration_samples > 0 {
            self.total_calibration_error / self.calibration_samples as f64
        } else {
            0.0
        };
        let global_uncertainty =
            self.domains.values().map(|d| d.uncertainty).sum::<f64>() / self.domains.len() as f64;
        EpistemicState {
            domain_confidence: self.domains.clone(),
            concept_count: self.concepts.len(),
            average_calibration_error: avg_cal,
            global_uncertainty,
            total_samples: self.total_samples,
        }
    }

    pub fn best_domains(&self, top_k: usize) -> Vec<(&AttentionDomain, f64)> {
        let mut scored: Vec<(&AttentionDomain, f64)> = self
            .domains
            .iter()
            .map(|(d, s)| (d, s.observed_success * (1.0 - s.uncertainty)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    pub fn weakest_domains(&self, top_k: usize) -> Vec<(&AttentionDomain, f64)> {
        let mut scored: Vec<(&AttentionDomain, f64)> = self
            .domains
            .iter()
            .map(|(d, s)| (d, s.observed_success * (1.0 + s.uncertainty)))
            .collect();
        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    pub fn record_factual_f1(&mut self, precision: f64, recall: f64) {
        let p = precision.clamp(0.0, 1.0);
        let r = recall.clamp(0.0, 1.0);
        if self.factual_precision_history.len() >= 20 {
            self.factual_precision_history.pop_front();
        }
        if self.factual_recall_history.len() >= 20 {
            self.factual_recall_history.pop_front();
        }
        self.factual_precision_history.push_back(p);
        self.factual_recall_history.push_back(r);

        // Update calibration error based on (predicted_confidence - factual_f1)
        let f1 = self.factual_f1();
        for domain_state in self.domains.values_mut() {
            if domain_state.sample_count > 0 {
                let error = (domain_state.expected_success - f1).abs();
                domain_state.calibration_error = domain_state.calibration_error * 0.9 + error * 0.1;
            }
        }
    }

    pub fn factual_precision_score(&self) -> f64 {
        if self.factual_precision_history.is_empty() {
            return 0.0;
        }
        self.factual_precision_history.iter().sum::<f64>()
            / self.factual_precision_history.len() as f64
    }

    pub fn factual_recall_score(&self) -> f64 {
        if self.factual_recall_history.is_empty() {
            return 0.0;
        }
        self.factual_recall_history.iter().sum::<f64>() / self.factual_recall_history.len() as f64
    }

    pub fn factual_f1(&self) -> f64 {
        let p = self.factual_precision_score();
        let r = self.factual_recall_score();
        if (p + r) < 1e-10 {
            return 0.0;
        }
        2.0 * p * r / (p + r)
    }

    fn stable_hash(&self, s: &str) -> u64 {
        let mut h: u64 = 0xe8e8_e8e8_e8e8_e8e8u64;
        for b in s.bytes() {
            h = h.wrapping_mul(0x9e3779b97f4a7c15u64);
            h ^= b as u64;
            h = h.rotate_left(11);
        }
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_model() -> EpistemicSelfModel {
        EpistemicSelfModel::new(EpistemicConfig {
            min_samples_for_calibration: 2,
            calibration_learning_rate: 0.1,
            uncertainty_decay: 0.95,
            max_concepts: 100,
            novelty_threshold: 0.35,
        })
    }

    #[test]
    fn test_calibrate_updates_expected_and_observed() {
        let mut model = make_model();
        let expected_before = model.domains[&AttentionDomain::Code].expected_success;
        model.calibrate(&AttentionDomain::Code, 0.8, true);
        let entry = &model.domains[&AttentionDomain::Code];
        assert!(entry.sample_count >= 1);
        assert!(entry.expected_success != expected_before);
        assert!(entry.observed_success > 0.5);
    }

    #[test]
    fn test_calibrate_reduces_uncertainty() {
        let mut model = make_model();
        let uncert_before = model.domains[&AttentionDomain::Code].uncertainty;
        for _ in 0..10 {
            model.calibrate(&AttentionDomain::Code, 0.7, true);
        }
        assert!(model.domains[&AttentionDomain::Code].uncertainty < uncert_before);
    }

    #[test]
    fn test_calibrate_tracks_calibration_error() {
        let mut model = make_model();
        model.calibrate(&AttentionDomain::Code, 1.0, false);
        model.calibrate(&AttentionDomain::Code, 1.0, false);
        assert!(model.domains[&AttentionDomain::Code].calibration_error > 0.0);
    }

    #[test]
    fn test_calibrate_increments_total_samples() {
        let mut model = make_model();
        model.calibrate(&AttentionDomain::Code, 0.5, true);
        model.calibrate(&AttentionDomain::Semantic, 0.5, false);
        assert_eq!(model.total_samples, 2);
    }

    #[test]
    fn test_track_concept_creates_node() {
        let mut model = make_model();
        let id = model.track_concept("recursion", 0.6, &[]);
        assert!(id > 0);
        assert_eq!(model.concepts.len(), 1);
        assert_eq!(model.concepts[0].label, "recursion");
    }

    #[test]
    fn test_track_concept_reuses_existing() {
        let mut model = make_model();
        let id1 = model.track_concept("same_concept", 0.6, &[]);
        let id2 = model.track_concept("same_concept", 0.6, &[]);
        assert_eq!(id1, id2, "duplicate concept should reuse existing id");
        assert_eq!(model.concepts.len(), 1);
    }

    #[test]
    fn test_verify_concept_increases_confidence() {
        let mut model = make_model();
        let id = model.track_concept("test_concept", 0.5, &[]);
        model.verify_concept(id, true);
        model.verify_concept(id, true);
        let c = model.concepts.iter().find(|c| c.id == id).unwrap();
        assert!(c.confidence > 0.5);
        assert_eq!(c.verification_count, 3);
    }

    #[test]
    fn test_verify_concept_decreases_on_failure() {
        let mut model = make_model();
        let id = model.track_concept("test_concept", 0.7, &[]);
        model.verify_concept(id, false);
        let c = model.concepts.iter().find(|c| c.id == id).unwrap();
        assert!(c.confidence < 0.7);
    }

    #[test]
    fn test_identify_gaps_returns_low_confidence() {
        let mut model = make_model();
        model.track_concept("weak_concept", 0.2, &[]);
        model.track_concept("strong_concept", 0.8, &[]);
        let gaps = model.identify_gaps(0.5);
        assert!(gaps.iter().any(|c| c.label == "weak_concept"));
        assert!(!gaps.iter().any(|c| c.label == "strong_concept"));
    }

    #[test]
    fn test_predict_success_returns_value() {
        let model = make_model();
        let vec = QuantizedVSA::random_vector();
        let pred = model.predict_success(&vec);
        assert!(pred >= 0.0 && pred <= 1.0);
    }

    #[test]
    fn test_snapshot_returns_correct_state() {
        let model = make_model();
        let snap = model.snapshot();
        assert_eq!(snap.domain_confidence.len(), 9);
        assert_eq!(snap.concept_count, 0);
        assert!(snap.global_uncertainty > 0.0);
    }

    #[test]
    fn test_best_domains_returns_sorted() {
        let mut model = make_model();
        model.calibrate(&AttentionDomain::Code, 0.9, true);
        model.calibrate(&AttentionDomain::Creativity, 0.3, false);
        let best = model.best_domains(3);
        assert!(!best.is_empty());
        assert!(best[0].1 >= best[1].1);
    }

    #[test]
    fn test_weakest_domains_returns_sorted() {
        let mut model = make_model();
        model.calibrate(&AttentionDomain::Creativity, 0.3, false);
        model.calibrate(&AttentionDomain::Code, 0.9, true);
        let weakest = model.weakest_domains(3);
        assert!(!weakest.is_empty());
        assert!(weakest[0].1 <= weakest[1].1);
    }

    #[test]
    fn test_compute_uncertainty_vector_succeeds() {
        let model = make_model();
        let vec = model.compute_uncertainty_vector();
        assert_eq!(vec.len(), 4096);
    }

    #[test]
    fn test_initial_domains_all_present() {
        let model = make_model();
        for domain in AttentionDomain::all() {
            assert!(model.domains.contains_key(&domain));
        }
    }
}
