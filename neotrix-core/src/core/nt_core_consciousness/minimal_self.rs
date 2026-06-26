use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelfAttribution {
    SelfInitiated,
    OtherCaused,
    Ambiguous,
}

#[derive(Debug, Clone)]
pub struct AgencyRecord {
    pub action_vsa: Vec<u8>,
    pub outcome_vsa: Vec<u8>,
    pub attribution: SelfAttribution,
    pub agency_score: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct MinimalSelf {
    pub self_vector: Vec<u8>,
    pub agency_history: VecDeque<AgencyRecord>,
    pub current_agency: f64,
    pub current_ownership: f64,
    pub self_similarity_threshold: f64,
    pub max_history: usize,
    pub learning_rate: f64,
    pub cycle_count: u64,
    pub agency_window: VecDeque<f64>,
    pub first_person_anchor: Vec<u8>,
}

impl MinimalSelf {
    pub fn new() -> Self {
        let self_vector = QuantizedVSA::seeded_random(42, VSA_DIM);
        let first_person_anchor = QuantizedVSA::seeded_random(1, VSA_DIM);
        Self {
            self_vector,
            agency_history: VecDeque::with_capacity(100),
            current_agency: 0.5,
            current_ownership: 0.5,
            self_similarity_threshold: 0.6,
            max_history: 100,
            learning_rate: 0.1,
            cycle_count: 0,
            agency_window: VecDeque::with_capacity(100),
            first_person_anchor,
        }
    }

    pub fn sense_agency(&mut self, action_vsa: &[u8], outcome_vsa: &[u8]) -> f64 {
        let score = QuantizedVSA::similarity(action_vsa, outcome_vsa);
        let attribution = self.attribution(score);
        let record = AgencyRecord {
            action_vsa: action_vsa.to_vec(),
            outcome_vsa: outcome_vsa.to_vec(),
            attribution,
            agency_score: score,
            timestamp: self.cycle_count,
        };
        self.agency_history.push_back(record);
        if self.agency_history.len() > self.max_history {
            self.agency_history.pop_front();
        }
        self.agency_window.push_back(score);
        if self.agency_window.len() > self.max_history {
            self.agency_window.pop_front();
        }
        let sum: f64 = self.agency_window.iter().sum();
        let n = self.agency_window.len() as f64;
        self.current_agency = if n > 0.0 { sum / n } else { 0.5 };
        self.cycle_count += 1;
        score
    }

    pub fn sense_ownership(&mut self, thought_vsa: &[u8]) -> f64 {
        let sim = QuantizedVSA::similarity(thought_vsa, &self.self_vector);
        self.current_ownership =
            self.current_ownership * (1.0 - self.learning_rate) + sim * self.learning_rate;
        sim
    }

    pub fn update_self_vector(&mut self, experience_vsa: &[u8], strength: f64) {
        let weight = strength * self.learning_rate;
        let n_bundles = (weight * 10.0).max(1.0).round() as usize;
        let mut vectors = Vec::with_capacity(n_bundles + 1);
        vectors.push(&*self.self_vector);
        for _ in 0..n_bundles {
            vectors.push(experience_vsa);
        }
        let bundled = QuantizedVSA::bundle(&vectors);
        self.self_vector = QuantizedVSA::binarize(&bundled);
    }

    pub fn attribution(&self, agency_score: f64) -> SelfAttribution {
        if agency_score > 0.7 {
            SelfAttribution::SelfInitiated
        } else if agency_score < 0.3 {
            SelfAttribution::OtherCaused
        } else {
            SelfAttribution::Ambiguous
        }
    }

    pub fn self_similarity(&self, other_vsa: &[u8]) -> f64 {
        QuantizedVSA::similarity(other_vsa, &self.self_vector)
    }

    pub fn agency_volatility(&self) -> f64 {
        let n = self.agency_window.len();
        if n < 2 {
            return 0.0;
        }
        let sum: f64 = self.agency_window.iter().sum();
        let mean = sum / n as f64;
        let variance = self
            .agency_window
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / (n - 1) as f64;
        variance.sqrt()
    }

    pub fn is_coherent(&self) -> bool {
        self.current_agency > self.self_similarity_threshold
            && self.current_ownership > self.self_similarity_threshold
    }

    pub fn reset(&mut self) {
        self.agency_history.clear();
        self.current_agency = 0.5;
        self.current_ownership = 0.5;
        self.cycle_count = 0;
        self.agency_window.clear();
    }
}

impl Default for MinimalSelf {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vsa(seed: u64) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed, VSA_DIM)
    }

    #[test]
    fn test_new_minimal_self_defaults() {
        let ms = MinimalSelf::new();
        assert_eq!(ms.self_vector.len(), VSA_DIM);
        assert_eq!(ms.first_person_anchor.len(), VSA_DIM);
        assert!((ms.current_agency - 0.5).abs() < 1e-6);
        assert!((ms.current_ownership - 0.5).abs() < 1e-6);
        assert!((ms.self_similarity_threshold - 0.6).abs() < 1e-6);
        assert_eq!(ms.max_history, 100);
        assert!((ms.learning_rate - 0.1).abs() < 1e-6);
        assert_eq!(ms.cycle_count, 0);
        assert!(ms.agency_history.is_empty());
        assert!(ms.agency_window.is_empty());
    }

    #[test]
    fn test_sense_agency_high_similarity() {
        let mut ms = MinimalSelf::new();
        let v = make_vsa(100);
        let score = ms.sense_agency(&v, &v);
        assert!(
            score > 0.95,
            "identical vectors should have near-1 similarity, got {}",
            score
        );
        assert!(ms.current_agency > 0.9);
    }

    #[test]
    fn test_sense_agency_low_similarity() {
        let mut ms = MinimalSelf::new();
        let a = make_vsa(100);
        let b = make_vsa(200);
        let score = ms.sense_agency(&a, &b);
        assert!(
            score < 0.2,
            "independent vectors should have near-0 similarity, got {}",
            score
        );
    }

    #[test]
    fn test_sense_agency_records_history() {
        let mut ms = MinimalSelf::new();
        let v = make_vsa(100);
        ms.sense_agency(&v, &v);
        assert_eq!(ms.agency_history.len(), 1);
        assert_eq!(ms.agency_window.len(), 1);
        let rec = &ms.agency_history[0];
        assert_eq!(rec.action_vsa, v);
        assert_eq!(rec.outcome_vsa, v);
        assert_eq!(rec.attribution, SelfAttribution::SelfInitiated);
        assert!(rec.agency_score > 0.95);
    }

    #[test]
    fn test_sense_ownership_self_similar() {
        let mut ms = MinimalSelf::new();
        let sv = ms.self_vector.clone();
        let score = ms.sense_ownership(&sv);
        assert!(
            score > 0.95,
            "self-vector should have high ownership, got {}",
            score
        );
        assert!(ms.current_ownership > 0.9);
    }

    #[test]
    fn test_sense_ownership_dissimilar() {
        let mut ms = MinimalSelf::new();
        let other = make_vsa(999);
        let score = ms.sense_ownership(&other);
        assert!(
            score < 0.2,
            "random vector should have low ownership, got {}",
            score
        );
    }

    #[test]
    fn test_update_self_vector_changes_self() {
        let mut ms = MinimalSelf::new();
        let original = ms.self_vector.clone();
        let experience = make_vsa(500);
        ms.update_self_vector(&experience, 1.0);
        let sim = QuantizedVSA::similarity(&original, &ms.self_vector);
        assert!(sim < 1.0, "self-vector should change after update");
        assert!(
            sim > 0.3,
            "self-vector should remain somewhat similar, got {}",
            sim
        );
    }

    #[test]
    fn test_attribution_self() {
        let ms = MinimalSelf::new();
        assert_eq!(ms.attribution(0.8), SelfAttribution::SelfInitiated);
        assert_eq!(ms.attribution(1.0), SelfAttribution::SelfInitiated);
    }

    #[test]
    fn test_attribution_other() {
        let ms = MinimalSelf::new();
        assert_eq!(ms.attribution(0.2), SelfAttribution::OtherCaused);
        assert_eq!(ms.attribution(0.0), SelfAttribution::OtherCaused);
    }

    #[test]
    fn test_attribution_ambiguous() {
        let ms = MinimalSelf::new();
        assert_eq!(ms.attribution(0.5), SelfAttribution::Ambiguous);
        assert_eq!(ms.attribution(0.4), SelfAttribution::Ambiguous);
        assert_eq!(ms.attribution(0.6), SelfAttribution::Ambiguous);
    }

    #[test]
    fn test_self_similarity() {
        let ms = MinimalSelf::new();
        let sim_self = ms.self_similarity(&ms.self_vector);
        assert!(sim_self > 0.95);
        let other = make_vsa(777);
        let sim_other = ms.self_similarity(&other);
        assert!(sim_other < 0.2);
    }

    #[test]
    fn test_agency_volatility() {
        let mut ms = MinimalSelf::new();
        let v = make_vsa(100);
        for _ in 0..10 {
            ms.sense_agency(&v, &v);
        }
        let vol = ms.agency_volatility();
        assert!(
            vol < 0.05,
            "consistent scores should have low volatility, got {}",
            vol
        );
    }

    #[test]
    fn test_is_coherent() {
        let mut ms = MinimalSelf::new();
        let v = make_vsa(100);
        ms.sense_agency(&v, &v);
        let sv = ms.self_vector.clone();
        ms.sense_ownership(&sv);
        assert!(
            ms.is_coherent(),
            "should be coherent with high agency and ownership"
        );
    }

    #[test]
    fn test_reset_keeps_self_vector() {
        let mut ms = MinimalSelf::new();
        let saved_self = ms.self_vector.clone();
        let saved_anchor = ms.first_person_anchor.clone();
        let v = make_vsa(100);
        ms.sense_agency(&v, &v);
        ms.sense_ownership(&make_vsa(200));
        assert!(ms.cycle_count > 0);
        ms.reset();
        assert_eq!(ms.self_vector, saved_self);
        assert_eq!(ms.first_person_anchor, saved_anchor);
        assert_eq!(ms.cycle_count, 0);
        assert!(ms.agency_history.is_empty());
        assert!(ms.agency_window.is_empty());
        assert!((ms.current_agency - 0.5).abs() < 1e-6);
        assert!((ms.current_ownership - 0.5).abs() < 1e-6);
    }
}
