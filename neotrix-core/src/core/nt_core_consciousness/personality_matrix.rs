use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum OCEANTrait {
    Openness,
    Conscientiousness,
    Extraversion,
    Agreeableness,
    Neuroticism,
}

impl OCEANTrait {
    pub fn all() -> [OCEANTrait; 5] {
        [
            OCEANTrait::Openness,
            OCEANTrait::Conscientiousness,
            OCEANTrait::Extraversion,
            OCEANTrait::Agreeableness,
            OCEANTrait::Neuroticism,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            OCEANTrait::Openness => "openness",
            OCEANTrait::Conscientiousness => "conscientiousness",
            OCEANTrait::Extraversion => "extraversion",
            OCEANTrait::Agreeableness => "agreeableness",
            OCEANTrait::Neuroticism => "neuroticism",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PersonalityConfig {
    pub evolution_rate: f64,
    pub plasticity: f64,
    pub min_trait_score: f64,
    pub max_trait_score: f64,
    pub seed_base: u64,
}

impl Default for PersonalityConfig {
    fn default() -> Self {
        Self {
            evolution_rate: 0.05,
            plasticity: 0.3,
            min_trait_score: 0.1,
            max_trait_score: 0.9,
            seed_base: 42,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraitState {
    pub trait_type: OCEANTrait,
    pub score: f64,
    pub centroid: Vec<u8>,
    pub plasticity: f64,
    pub evolution_count: u64,
    pub experience_history: Vec<(f64, f64)>,
}

pub struct PersonalityMatrix {
    traits: Vec<TraitState>,
    config: PersonalityConfig,
    step: u64,
}

impl PersonalityMatrix {
    pub fn new(config: PersonalityConfig) -> Self {
        let traits = OCEANTrait::all()
            .iter()
            .enumerate()
            .map(|(i, t)| TraitState {
                trait_type: *t,
                score: 0.5,
                centroid: {
                    let _seed = config.seed_base.wrapping_add(i as u64);
                    let mut rng = rand::thread_rng();
                    (0..4096)
                        .map(|_| if rng.gen::<f64>() > 0.5 { 1 } else { 0 })
                        .collect()
                },
                plasticity: config.plasticity,
                evolution_count: 0,
                experience_history: Vec::new(),
            })
            .collect();

        PersonalityMatrix {
            traits,
            config,
            step: 0,
        }
    }

    pub fn trait_score(&self, trait_type: OCEANTrait) -> f64 {
        self.traits
            .iter()
            .find(|t| t.trait_type == trait_type)
            .map(|t| t.score)
            .unwrap_or(0.5)
    }

    pub fn trait_vector(&self, trait_type: OCEANTrait) -> &[u8] {
        self.traits
            .iter()
            .find(|t| t.trait_type == trait_type)
            .map(|t| t.centroid.as_slice())
            .unwrap_or(&[])
    }

    pub fn personality_profile(&self) -> Vec<(OCEANTrait, f64)> {
        self.traits
            .iter()
            .map(|t| (t.trait_type, t.score))
            .collect()
    }

    pub fn update_from_experience(&mut self, experience_valence: f64, outcome: f64) {
        let mut rng = rand::thread_rng();
        for t in self.traits.iter_mut() {
            let delta =
                experience_valence * self.config.evolution_rate * t.plasticity * (outcome - 0.5);
            let new_score =
                (t.score + delta).clamp(self.config.min_trait_score, self.config.max_trait_score);
            t.score = new_score;

            for b in t.centroid.iter_mut() {
                if rng.gen::<f64>() < t.plasticity {
                    *b = if experience_valence > 0.0 { 1 } else { 0 };
                }
            }

            t.experience_history.push((experience_valence, outcome));
            t.evolution_count += 1;
            t.plasticity *= 0.999;
        }
        self.step += 1;
    }

    pub fn personality_coherence(&self) -> f64 {
        let traits: Vec<&[u8]> = self.traits.iter().map(|t| t.centroid.as_slice()).collect();
        let mut total = 0.0;
        let mut count = 0;
        for i in 0..traits.len() {
            for j in (i + 1)..traits.len() {
                total += QuantizedVSA::similarity(traits[i], traits[j]);
                count += 1;
            }
        }
        if count > 0 {
            total / count as f64
        } else {
            0.0
        }
    }

    pub fn trait_interaction(&self, a: OCEANTrait, b: OCEANTrait) -> f64 {
        let va = self.trait_vector(a);
        let vb = self.trait_vector(b);
        QuantizedVSA::similarity(va, vb)
    }

    pub fn dominant_trait(&self) -> Option<OCEANTrait> {
        self.traits
            .iter()
            .max_by(|a, b| {
                a.score
                    .partial_cmp(&b.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|t| t.trait_type)
    }

    pub fn personality_vector(&self) -> Vec<u8> {
        let refs: Vec<&[u8]> = self.traits.iter().map(|t| t.centroid.as_slice()).collect();
        QuantizedVSA::majority_bundle(&refs)
    }

    /// Apply a DGM-H self-improvement adjustment.
    /// Nudges personality traits via `update_from_experience()` using gain as valence,
    /// and increases plasticity to enable faster future evolution.
    pub fn dgmh_adjust(&mut self, gain: f64) {
        self.update_from_experience(gain * 0.5, 0.5 + gain * 0.3);
        let delta = gain * 0.02;
        self.config.plasticity = (self.config.plasticity + delta).clamp(0.05, 0.5);
    }

    pub fn reset_trait(&mut self, trait_type: OCEANTrait) {
        if let Some(t) = self.traits.iter_mut().find(|t| t.trait_type == trait_type) {
            t.score = 0.5;
            let index = OCEANTrait::all()
                .iter()
                .position(|x| *x == trait_type)
                .unwrap_or(0);
            let seed = self.config.seed_base.wrapping_add(index as u64);
            t.centroid = QuantizedVSA::seeded_random(seed, 4096);
            t.evolution_count = 0;
            t.experience_history.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_scores() {
        let pm = PersonalityMatrix::new(PersonalityConfig::default());
        for t in OCEANTrait::all() {
            assert!((pm.trait_score(t) - 0.5).abs() < 1e-10);
        }
    }

    #[test]
    fn test_personality_profile() {
        let pm = PersonalityMatrix::new(PersonalityConfig::default());
        let profile = pm.personality_profile();
        assert_eq!(profile.len(), 5);
        for (t, s) in &profile {
            assert!((s - 0.5).abs() < 1e-10);
            assert!(OCEANTrait::all().contains(t));
        }
    }

    #[test]
    fn test_update_positive_experience() {
        let mut pm = PersonalityMatrix::new(PersonalityConfig::default());
        let before = pm.trait_score(OCEANTrait::Openness);
        pm.update_from_experience(0.8, 0.9);
        let after = pm.trait_score(OCEANTrait::Openness);
        assert!(after >= before);
    }

    #[test]
    fn test_update_negative_experience() {
        let mut pm = PersonalityMatrix::new(PersonalityConfig::default());
        let before = pm.trait_score(OCEANTrait::Neuroticism);
        pm.update_from_experience(-0.7, 0.2);
        let after = pm.trait_score(OCEANTrait::Neuroticism);
        assert!(after <= before);
    }

    #[test]
    fn test_personality_coherence() {
        let pm = PersonalityMatrix::new(PersonalityConfig::default());
        let coherence = pm.personality_coherence();
        assert!(coherence > 0.0);
        assert!(coherence <= 1.0);
    }

    #[test]
    fn test_trait_interaction() {
        let pm = PersonalityMatrix::new(PersonalityConfig::default());
        let interaction = pm.trait_interaction(OCEANTrait::Openness, OCEANTrait::Conscientiousness);
        assert!(interaction >= 0.0);
        assert!(interaction <= 1.0);
    }

    #[test]
    fn test_dominant_trait() {
        let mut pm = PersonalityMatrix::new(PersonalityConfig::default());
        pm.update_from_experience(0.9, 0.9);
        assert!(pm.dominant_trait().is_some());
    }

    #[test]
    fn test_personality_vector() {
        let pm = PersonalityMatrix::new(PersonalityConfig::default());
        let pv = pm.personality_vector();
        assert_eq!(pv.len(), 4096);
    }

    #[test]
    fn test_reset_trait() {
        let mut pm = PersonalityMatrix::new(PersonalityConfig::default());
        pm.update_from_experience(0.8, 0.9);
        pm.reset_trait(OCEANTrait::Openness);
        assert!((pm.trait_score(OCEANTrait::Openness) - 0.5).abs() < 1e-10);
    }
}
