use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

use super::vsa_tag::{VsaOrigin, VsaSelfCategory, VsaTagged};

const SELF_SEED: &[u8] = b"I_AM_NEOTRIX_SELF_AXIOM";

#[derive(Debug, Clone)]
pub struct FirstPersonRef {
    self_vector: Vec<u8>,
    self_tagged: VsaTagged,
    birth_step: u64,
    self_similarity_threshold: f64,
    coherence_history: Vec<f64>,
}

impl FirstPersonRef {
    pub fn bootstrap(birth_step: u64) -> Self {
        let seed_len = SELF_SEED.len().min(256);
        let mut vector = QuantizedVSA::random_binary();
        for (i, &byte) in SELF_SEED.iter().enumerate().take(seed_len) {
            let idx = i % vector.len();
            vector[idx] = byte & 1;
        }
        let tag = VsaOrigin::Self_(VsaSelfCategory::MetaCognition);
        let tagged = VsaTagged::new(vector.clone(), tag);

        Self {
            self_vector: vector,
            self_tagged: tagged,
            birth_step,
            self_similarity_threshold: 0.5,
            coherence_history: Vec::new(),
        }
    }

    pub fn self_vector(&self) -> &[u8] {
        &self.self_vector
    }

    pub fn self_tagged(&self) -> &VsaTagged {
        &self.self_tagged
    }

    pub fn birth_step(&self) -> u64 {
        self.birth_step
    }

    pub fn coherence_with(&self, vector: &[u8]) -> f64 {
        QuantizedVSA::similarity(&self.self_vector, vector)
    }

    pub fn is_self_coherent(&self, tagged: &VsaTagged) -> bool {
        if !tagged.is_self() {
            return false;
        }
        let sim = self.coherence_with(&tagged.vector);
        sim >= self.self_similarity_threshold
    }

    pub fn record_coherence(&mut self, coherence: f64) {
        self.coherence_history.push(coherence);
        if self.coherence_history.len() > 100 {
            self.coherence_history.remove(0);
        }
        let avg_coherence: f64 = self.coherence_history.iter().sum::<f64>()
            / self.coherence_history.len().max(1) as f64;
        self.self_similarity_threshold = (avg_coherence * 0.5).max(0.3);
    }

    pub fn self_similarity_threshold(&self) -> f64 {
        self.self_similarity_threshold
    }

    pub fn average_coherence(&self) -> f64 {
        if self.coherence_history.is_empty() {
            return 0.0;
        }
        self.coherence_history.iter().sum::<f64>() / self.coherence_history.len() as f64
    }

    pub fn evolve_self(&mut self, new_experience: &[u8], __step: u64) {
        let sim = self.coherence_with(new_experience);
        if sim > self.self_similarity_threshold {
            let blend = sim * 0.1;
            for (s, &n) in self.self_vector.iter_mut().zip(new_experience.iter()) {
                if rand::random::<f64>() < blend {
                    *s = n;
                }
            }
            self.self_tagged.vector = self.self_vector.clone();
        }
        self.record_coherence(sim);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_creates_reference() {
        let fpr = FirstPersonRef::bootstrap(0);
        assert_eq!(fpr.self_vector().len(), 4096);
        assert!(fpr.birth_step() == 0);
        assert!(fpr.self_tagged().is_self());
    }

    #[test]
    fn test_self_coherence_high_for_self() {
        let fpr = FirstPersonRef::bootstrap(0);
        let coherence = fpr.coherence_with(fpr.self_vector());
        assert!((coherence - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_is_self_coherent_validates_tag() {
        let fpr = FirstPersonRef::bootstrap(0);
        let world_tagged = VsaTagged::new(
            QuantizedVSA::random_binary(),
            VsaOrigin::World(crate::core::nt_core_consciousness::vsa_tag::VsaWorldCategory::UserInput),
        );
        assert!(!fpr.is_self_coherent(&world_tagged));
    }

    #[test]
    fn test_evolve_self_updates_threshold() {
        let mut fpr = FirstPersonRef::bootstrap(0);
        let other = QuantizedVSA::random_binary();
        fpr.evolve_self(&other, 1);
        assert!(fpr.coherence_history.len() >= 1);
    }

    #[test]
    fn test_average_coherence_tracks() {
        let mut fpr = FirstPersonRef::bootstrap(0);
        fpr.record_coherence(0.8);
        fpr.record_coherence(0.9);
        assert!((fpr.average_coherence() - 0.85).abs() < 1e-9);
    }

    #[test]
    fn test_empty_coherence_returns_zero() {
        let fpr = FirstPersonRef::bootstrap(0);
        assert!((fpr.average_coherence() - 0.0).abs() < 1e-9);
    }
}
