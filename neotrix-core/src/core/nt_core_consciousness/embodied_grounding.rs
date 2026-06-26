use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SensoryPrimitive {
    pub primitive_vsa: Vec<u8>,
    pub modality: PrimitiveModality,
    pub label: String,
    pub intensity: f64,
    pub polarity: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveModality {
    Proprioception,
    Interoception,
    Touch,
    Temperature,
    Pain,
    Pressure,
    Spatial,
    Kinesthetic,
    Visceral,
}

#[derive(Debug, Clone)]
pub struct GroundedConcept {
    pub concept_vsa: Vec<u8>,
    pub label: String,
    pub primitives: Vec<(SensoryPrimitive, f64)>,
    pub abstraction_level: f64,
    pub last_grounded: u64,
}

#[derive(Debug, Clone)]
pub struct EmbodiedGroundingSystem {
    pub primitive_library: HashMap<String, SensoryPrimitive>,
    pub grounded_concepts: Vec<GroundedConcept>,
    pub max_concepts: usize,
    pub grounding_threshold: f64,
    pub decay_rate: f64,
    pub cross_modal_binding_key: Vec<u8>,
    pub cycle_count: u64,
}

impl EmbodiedGroundingSystem {
    pub fn new() -> Self {
        let mut primitive_library = HashMap::new();
        let pairs: [(&str, PrimitiveModality, u64, f64, f64); 20] = [
            ("push", PrimitiveModality::Kinesthetic, 1000, 0.8, 0.6),
            ("pull", PrimitiveModality::Kinesthetic, 1001, 0.7, -0.4),
            ("warm", PrimitiveModality::Temperature, 1002, 0.6, 0.7),
            ("cold", PrimitiveModality::Temperature, 1003, 0.7, -0.6),
            (
                "pressure_light",
                PrimitiveModality::Pressure,
                1004,
                0.3,
                0.3,
            ),
            (
                "pressure_heavy",
                PrimitiveModality::Pressure,
                1005,
                0.9,
                -0.5,
            ),
            ("near", PrimitiveModality::Spatial, 1006, 0.5, 0.4),
            ("far", PrimitiveModality::Spatial, 1007, 0.4, -0.2),
            ("up", PrimitiveModality::Spatial, 1008, 0.5, 0.5),
            ("down", PrimitiveModality::Spatial, 1009, 0.5, -0.3),
            ("expand", PrimitiveModality::Interoception, 1010, 0.6, 0.8),
            (
                "contract",
                PrimitiveModality::Interoception,
                1011,
                0.6,
                -0.7,
            ),
            ("fast", PrimitiveModality::Kinesthetic, 1012, 0.8, 0.5),
            ("slow", PrimitiveModality::Kinesthetic, 1013, 0.4, -0.1),
            ("toward", PrimitiveModality::Spatial, 1014, 0.5, 0.6),
            ("away", PrimitiveModality::Spatial, 1015, 0.5, -0.5),
            ("tight", PrimitiveModality::Proprioception, 1016, 0.7, -0.3),
            ("loose", PrimitiveModality::Proprioception, 1017, 0.4, 0.5),
            ("sweet", PrimitiveModality::Visceral, 1018, 0.5, 0.8),
            ("bitter", PrimitiveModality::Visceral, 1019, 0.6, -0.7),
        ];
        for (label, modality, seed, intensity, polarity) in pairs {
            let vsa = QuantizedVSA::seeded_random(seed, VSA_DIM);
            primitive_library.insert(
                label.to_string(),
                SensoryPrimitive {
                    primitive_vsa: vsa,
                    modality,
                    label: label.to_string(),
                    intensity,
                    polarity,
                },
            );
        }
        let cross_modal_binding_key = QuantizedVSA::seeded_random(9999, VSA_DIM);
        EmbodiedGroundingSystem {
            primitive_library,
            grounded_concepts: Vec::new(),
            max_concepts: 200,
            grounding_threshold: 0.3,
            decay_rate: 0.02,
            cross_modal_binding_key,
            cycle_count: 0,
        }
    }

    pub fn ground_concept(
        &mut self,
        concept_vsa: Vec<u8>,
        label: &str,
        activated_primitives: &[(&str, f64)],
    ) -> GroundedConcept {
        if self.grounded_concepts.len() >= self.max_concepts {
            return GroundedConcept {
                concept_vsa,
                label: label.to_string(),
                primitives: Vec::new(),
                abstraction_level: 1.0,
                last_grounded: self.cycle_count,
            };
        }
        let mut primitives = Vec::new();
        let mut total_strength = 0.0;
        for (prim_label, strength) in activated_primitives {
            if let Some(primitive) = self.primitive_library.get(*prim_label) {
                let grounded_strength = strength.max(0.0);
                total_strength += grounded_strength;
                primitives.push((primitive.clone(), grounded_strength));
            }
        }
        let abstraction_level = if primitives.is_empty() {
            1.0
        } else {
            let avg = total_strength / primitives.len() as f64;
            (1.0 - avg).clamp(0.0, 1.0)
        };
        let gc = GroundedConcept {
            concept_vsa,
            label: label.to_string(),
            primitives,
            abstraction_level,
            last_grounded: self.cycle_count,
        };
        self.grounded_concepts.push(gc.clone());
        gc
    }

    pub fn find_similar_concepts(&self, concept_vsa: &[u8], n: usize) -> Vec<&GroundedConcept> {
        let mut scored: Vec<(&GroundedConcept, f64)> = self
            .grounded_concepts
            .iter()
            .map(|gc| (gc, QuantizedVSA::similarity(concept_vsa, &gc.concept_vsa)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(n).map(|(gc, _)| gc).collect()
    }

    pub fn primitive_similarity(&self, concept_vsa: &[u8], primitive_label: &str) -> f64 {
        for gc in &self.grounded_concepts {
            if QuantizedVSA::similarity(concept_vsa, &gc.concept_vsa).abs() > 0.99 {
                for (prim, strength) in &gc.primitives {
                    if prim.label == primitive_label {
                        return *strength;
                    }
                }
                return 0.0;
            }
        }
        0.0
    }

    pub fn abstractness(&self, concept_label: &str) -> Option<f64> {
        self.grounded_concepts
            .iter()
            .find(|gc| gc.label == concept_label)
            .map(|gc| gc.abstraction_level)
    }

    pub fn concepts_by_modality(&self, modality: PrimitiveModality) -> Vec<&GroundedConcept> {
        self.grounded_concepts
            .iter()
            .filter(|gc| gc.primitives.iter().any(|(p, _)| p.modality == modality))
            .collect()
    }

    pub fn decay(&mut self) {
        for gc in &mut self.grounded_concepts {
            for (_, strength) in &mut gc.primitives {
                *strength = (*strength - self.decay_rate).max(0.0);
            }
        }
    }

    pub fn reset(&mut self) {
        self.grounded_concepts.clear();
        self.cycle_count = 0;
    }
}

impl Default for EmbodiedGroundingSystem {
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
    fn test_new_system_defaults() {
        let egs = EmbodiedGroundingSystem::new();
        assert_eq!(egs.max_concepts, 200);
        assert!((egs.grounding_threshold - 0.3).abs() < 1e-6);
        assert!((egs.decay_rate - 0.02).abs() < 1e-6);
        assert!(egs.grounded_concepts.is_empty());
        assert_eq!(egs.primitive_library.len(), 20);
    }

    #[test]
    fn test_ground_concept_stores_concept() {
        let mut egs = EmbodiedGroundingSystem::new();
        let vsa = make_vsa(42);
        let gc = egs.ground_concept(vsa.clone(), "love", &[("warm", 0.8), ("expand", 0.6)]);
        assert_eq!(gc.label, "love");
        assert_eq!(egs.grounded_concepts.len(), 1);
        assert_eq!(egs.grounded_concepts[0].label, "love");
    }

    #[test]
    fn test_ground_concept_correct_primitives() {
        let mut egs = EmbodiedGroundingSystem::new();
        let vsa = make_vsa(7);
        egs.ground_concept(
            vsa,
            "freedom",
            &[("expand", 0.9), ("up", 0.7), ("fast", 0.5)],
        );
        let gc = &egs.grounded_concepts[0];
        assert_eq!(gc.primitives.len(), 3);
        assert!(gc.primitives.iter().any(|(p, _)| p.label == "expand"));
        assert!(gc.primitives.iter().any(|(p, _)| p.label == "up"));
        assert!(gc.primitives.iter().any(|(p, _)| p.label == "fast"));
    }

    #[test]
    fn test_ground_concept_abstraction_level() {
        let mut egs = EmbodiedGroundingSystem::new();
        let vsa = make_vsa(10);
        egs.ground_concept(vsa, "hot_coal", &[("warm", 1.0), ("pressure_heavy", 1.0)]);
        let gc = &egs.grounded_concepts[0];
        assert!(gc.abstraction_level < 0.1);
    }

    #[test]
    fn test_ground_concept_high_abstraction() {
        let mut egs = EmbodiedGroundingSystem::new();
        let vsa = make_vsa(20);
        egs.ground_concept(vsa, "truth", &[]);
        let gc = &egs.grounded_concepts[0];
        assert!((gc.abstraction_level - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_find_similar_concepts_by_vsa() {
        let mut egs = EmbodiedGroundingSystem::new();
        let vsa_a = make_vsa(100);
        let vsa_b = make_vsa(200);
        egs.ground_concept(vsa_a.clone(), "concept_a", &[("warm", 0.5)]);
        egs.ground_concept(vsa_b, "concept_b", &[("cold", 0.5)]);
        let similar = egs.find_similar_concepts(&vsa_a, 5);
        assert_eq!(similar[0].label, "concept_a");
    }

    #[test]
    fn test_find_similar_concepts_returns_n() {
        let mut egs = EmbodiedGroundingSystem::new();
        for i in 0..10 {
            egs.ground_concept(make_vsa(i), &format!("c{}", i), &[("warm", 0.5)]);
        }
        let query = make_vsa(0);
        let similar = egs.find_similar_concepts(&query, 3);
        assert_eq!(similar.len(), 3);
    }

    #[test]
    fn test_primitive_similarity() {
        let mut egs = EmbodiedGroundingSystem::new();
        let vsa = make_vsa(50);
        egs.ground_concept(vsa.clone(), "courage", &[("push", 0.7), ("expand", 0.8)]);
        let sim = egs.primitive_similarity(&vsa, "expand");
        assert!((sim - 0.8).abs() < 1e-6);
        let missing = egs.primitive_similarity(&vsa, "cold");
        assert!((missing - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_abstractness_returns_level() {
        let mut egs = EmbodiedGroundingSystem::new();
        let vsa = make_vsa(77);
        egs.ground_concept(vsa, "abstract_idea", &[]);
        let level = egs.abstractness("abstract_idea");
        assert!(level.is_some());
        assert!((level.unwrap() - 1.0).abs() < 1e-6);
        assert!(egs.abstractness("nonexistent").is_none());
    }

    #[test]
    fn test_concepts_by_modality() {
        let mut egs = EmbodiedGroundingSystem::new();
        egs.ground_concept(make_vsa(1), "heat", &[("warm", 0.8)]);
        egs.ground_concept(make_vsa(2), "coldness", &[("cold", 0.7)]);
        egs.ground_concept(make_vsa(3), "speed", &[("fast", 0.9)]);
        let temp_concepts = egs.concepts_by_modality(PrimitiveModality::Temperature);
        assert_eq!(temp_concepts.len(), 2);
        let kinetic = egs.concepts_by_modality(PrimitiveModality::Kinesthetic);
        assert_eq!(kinetic.len(), 1);
    }

    #[test]
    fn test_decay_reduces_grounding() {
        let mut egs = EmbodiedGroundingSystem::new();
        let vsa = make_vsa(30);
        egs.ground_concept(vsa, "strong_feeling", &[("warm", 0.9), ("expand", 0.8)]);
        let before: Vec<f64> = egs.grounded_concepts[0]
            .primitives
            .iter()
            .map(|(_, s)| *s)
            .collect();
        egs.decay();
        let after: Vec<f64> = egs.grounded_concepts[0]
            .primitives
            .iter()
            .map(|(_, s)| *s)
            .collect();
        for (b, a) in before.iter().zip(after.iter()) {
            assert!((a - (b - egs.decay_rate)).abs() < 1e-6);
        }
    }

    #[test]
    fn test_primitives_have_unique_vsas() {
        let egs = EmbodiedGroundingSystem::new();
        let vsas: Vec<&Vec<u8>> = egs
            .primitive_library
            .values()
            .map(|p| &p.primitive_vsa)
            .collect();
        for i in 0..vsas.len() {
            for j in i + 1..vsas.len() {
                assert_ne!(
                    vsas[i], vsas[j],
                    "primitives at {} and {} share same VSA",
                    i, j
                );
            }
        }
    }

    #[test]
    fn test_max_concepts_enforced() {
        let mut egs = EmbodiedGroundingSystem::new();
        egs.max_concepts = 3;
        for i in 0..5 {
            egs.ground_concept(make_vsa(i), &format!("c{}", i), &[("push", 0.5)]);
        }
        assert_eq!(egs.grounded_concepts.len(), 3);
    }

    #[test]
    fn test_reset() {
        let mut egs = EmbodiedGroundingSystem::new();
        egs.ground_concept(make_vsa(1), "a", &[("warm", 0.5)]);
        egs.ground_concept(make_vsa(2), "b", &[("cold", 0.5)]);
        egs.cycle_count = 99;
        egs.reset();
        assert!(egs.grounded_concepts.is_empty());
        assert_eq!(egs.cycle_count, 0);
    }
}
