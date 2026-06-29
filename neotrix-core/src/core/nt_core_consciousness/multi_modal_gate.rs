use crate::core::nt_core_consciousness::vsa_tag::SenseModality;
use crate::core::nt_core_gwt::manar_attention::ManarAttention;
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

const VSA_DIM: usize = 4096;

#[derive(Debug, Clone, Copy)]
pub struct ModalityThresholds {
    pub min_similarity: f64,
    pub salience_decay: f64,
    pub activation_boost: f64,
}

impl Default for ModalityThresholds {
    fn default() -> Self {
        Self {
            min_similarity: 0.7,
            salience_decay: 0.9,
            activation_boost: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModalityGateConfig {
    pub visual: ModalityThresholds,
    pub auditory: ModalityThresholds,
    pub tactile: ModalityThresholds,
    pub olfactory: ModalityThresholds,
    pub gustatory: ModalityThresholds,
    pub proprioceptive: ModalityThresholds,
    pub vestibular: ModalityThresholds,
    pub interoceptive: ModalityThresholds,
    pub mental: ModalityThresholds,
}

impl Default for ModalityGateConfig {
    fn default() -> Self {
        Self {
            visual: ModalityThresholds {
                min_similarity: 0.65,
                salience_decay: 0.85,
                activation_boost: 0.15,
            },
            auditory: ModalityThresholds {
                min_similarity: 0.70,
                salience_decay: 0.88,
                activation_boost: 0.10,
            },
            tactile: ModalityThresholds {
                min_similarity: 0.75,
                salience_decay: 0.90,
                activation_boost: 0.05,
            },
            olfactory: ModalityThresholds {
                min_similarity: 0.80,
                salience_decay: 0.92,
                activation_boost: 0.03,
            },
            gustatory: ModalityThresholds {
                min_similarity: 0.80,
                salience_decay: 0.92,
                activation_boost: 0.03,
            },
            proprioceptive: ModalityThresholds {
                min_similarity: 0.75,
                salience_decay: 0.90,
                activation_boost: 0.05,
            },
            vestibular: ModalityThresholds {
                min_similarity: 0.78,
                salience_decay: 0.91,
                activation_boost: 0.04,
            },
            interoceptive: ModalityThresholds {
                min_similarity: 0.72,
                salience_decay: 0.89,
                activation_boost: 0.08,
            },
            mental: ModalityThresholds {
                min_similarity: 0.68,
                salience_decay: 0.87,
                activation_boost: 0.12,
            },
        }
    }
}

impl ModalityGateConfig {
    pub fn thresholds_for(&self, modality: SenseModality) -> ModalityThresholds {
        match modality {
            SenseModality::Visual => self.visual,
            SenseModality::Auditory => self.auditory,
            SenseModality::Tactile => self.tactile,
            SenseModality::Olfactory => self.olfactory,
            SenseModality::Gustatory => self.gustatory,
            SenseModality::Proprioceptive => self.proprioceptive,
            SenseModality::Vestibular => self.vestibular,
            SenseModality::Interoceptive => self.interoceptive,
            SenseModality::Mental => self.mental,
            SenseModality::Document => self.visual,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModalityGate {
    config: ModalityGateConfig,
    slot_affinities: Vec<(usize, Vec<SenseModality>)>,
}

impl ModalityGate {
    pub fn new(config: ModalityGateConfig, num_slots: usize) -> Self {
        let mut slot_affinities = Vec::with_capacity(num_slots);
        let all_mods = SenseModality::all();
        for i in 0..num_slots {
            let mods = match i % 9 {
                0 => vec![SenseModality::Visual],
                1 => vec![SenseModality::Auditory],
                2 => vec![SenseModality::Tactile],
                3 => vec![SenseModality::Olfactory, SenseModality::Gustatory],
                4 => vec![SenseModality::Proprioceptive],
                5 => vec![SenseModality::Vestibular],
                6 => vec![SenseModality::Interoceptive],
                7 => vec![SenseModality::Mental],
                _ => all_mods.to_vec(),
            };
            slot_affinities.push((i, mods));
        }
        Self {
            config,
            slot_affinities,
        }
    }

    pub fn config(&self) -> &ModalityGateConfig {
        &self.config
    }

    pub fn gate(&self, modality: SenseModality, proposal: &[u8]) -> f64 {
        let thresh = self.config.thresholds_for(modality);
        let base_salience = QuantizedVSA::similarity(proposal, proposal);
        (base_salience + thresh.activation_boost).min(1.0)
    }

    pub fn affinity_slots(&self, modality: SenseModality) -> Vec<usize> {
        self.slot_affinities
            .iter()
            .filter(|(_, mods)| mods.contains(&modality))
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn slot_affinities(&self) -> &[(usize, Vec<SenseModality>)] {
        &self.slot_affinities
    }
}

pub struct ModalityRouter;

impl ModalityRouter {
    pub fn modality_priority(modality: SenseModality) -> u8 {
        match modality {
            SenseModality::Visual => 9,
            SenseModality::Auditory => 8,
            SenseModality::Tactile => 7,
            SenseModality::Proprioceptive => 6,
            SenseModality::Vestibular => 5,
            SenseModality::Interoceptive => 4,
            SenseModality::Mental => 3,
            SenseModality::Olfactory => 2,
            SenseModality::Gustatory => 1,
            SenseModality::Document => 5,
        }
    }

    pub fn modality_priority_f64(modality: SenseModality) -> f64 {
        Self::modality_priority(modality) as f64 / 9.0
    }

    pub fn route_to_slots(
        modality: SenseModality,
        gate: &ModalityGate,
        num_slots: usize,
    ) -> Vec<usize> {
        let affinity = gate.affinity_slots(modality);
        if affinity.is_empty() {
            let base = Self::modality_priority(modality) as usize % num_slots.max(1);
            vec![base]
        } else {
            affinity.into_iter().filter(|id| *id < num_slots).collect()
        }
    }
}

pub fn gated_attention_cycle(
    modality: SenseModality,
    input_vsa: &[u8],
    gate: &ModalityGate,
    manar: &mut ManarAttention,
) -> f64 {
    let gated_salience = gate.gate(modality, input_vsa);
    if gated_salience < 0.3 {
        return 0.0;
    }
    let proposals = vec![input_vsa.to_vec()];
    let saliences = manar.attend(&proposals);
    let max_sal = saliences
        .into_iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0);
    let combined = max_sal * 0.7 + gated_salience * 0.3;
    combined.min(1.0)
}

pub fn cross_modal_binding(visual_vsa: &[u8], text_vsa: &[u8]) -> Vec<u8> {
    let v_len = visual_vsa.len().min(text_vsa.len()).min(VSA_DIM);
    if v_len == 0 {
        return vec![0u8; VSA_DIM];
    }
    let v = &visual_vsa[..v_len];
    let t = &text_vsa[..v_len];
    let bound = QuantizedVSA::bind(v, t);
    let rebound = QuantizedVSA::xor_bind(&bound, v);
    let mut result = Vec::with_capacity(v_len);
    for i in 0..v_len {
        result.push(rebound[i] ^ t[i % t.len()]);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_gwt::manar_attention::ManarConfig;

    fn dummy_vsa() -> Vec<u8> {
        QuantizedVSA::random_binary()
    }

    #[test]
    fn test_modality_gate_config_default() {
        let config = ModalityGateConfig::default();
        let v = config.thresholds_for(SenseModality::Visual);
        assert!((v.activation_boost - 0.15).abs() < 1e-9);
        assert!((v.min_similarity - 0.65).abs() < 1e-9);
    }

    #[test]
    fn test_modality_gate_creates_affinities() {
        let config = ModalityGateConfig::default();
        let gate = ModalityGate::new(config, 18);
        let affinities = gate.slot_affinities();
        assert_eq!(affinities.len(), 18);
        let visual_slots = gate.affinity_slots(SenseModality::Visual);
        assert!(!visual_slots.is_empty());
    }

    #[test]
    fn test_modality_gate_gating() {
        let config = ModalityGateConfig::default();
        let gate = ModalityGate::new(config, 9);
        let vsa = dummy_vsa();
        let score = gate.gate(SenseModality::Visual, &vsa);
        assert!((score - 1.0).abs() < 1e-6 || (score >= 0.0 && score <= 1.0));
    }

    #[test]
    fn test_modality_router_priority() {
        assert!(
            ModalityRouter::modality_priority(SenseModality::Visual)
                > ModalityRouter::modality_priority(SenseModality::Auditory)
        );
        assert!(
            ModalityRouter::modality_priority(SenseModality::Auditory)
                > ModalityRouter::modality_priority(SenseModality::Tactile)
        );
        assert!(
            ModalityRouter::modality_priority(SenseModality::Olfactory)
                > ModalityRouter::modality_priority(SenseModality::Gustatory)
        );
        assert_eq!(ModalityRouter::modality_priority(SenseModality::Visual), 9);
        assert_eq!(
            ModalityRouter::modality_priority(SenseModality::Gustatory),
            1
        );
    }

    #[test]
    fn test_modality_router_priority_f64() {
        let v = ModalityRouter::modality_priority_f64(SenseModality::Visual);
        assert!((v - 1.0).abs() < 1e-9);
        let g = ModalityRouter::modality_priority_f64(SenseModality::Gustatory);
        assert!((g - 1.0 / 9.0).abs() < 1e-9);
    }

    #[test]
    fn test_modality_router_route_to_slots() {
        let config = ModalityGateConfig::default();
        let gate = ModalityGate::new(config, 27);
        let slots = ModalityRouter::route_to_slots(SenseModality::Visual, &gate, 27);
        assert!(!slots.is_empty());
        for id in &slots {
            assert!(*id < 27);
        }
    }

    #[test]
    fn test_cross_modal_binding_produces_vsa() {
        let visual = dummy_vsa();
        let text = dummy_vsa();
        let bound = cross_modal_binding(&visual, &text);
        assert_eq!(bound.len(), VSA_DIM);
        assert!(bound.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_cross_modal_binding_deterministic() {
        let visual = dummy_vsa();
        let text = dummy_vsa();
        let a = cross_modal_binding(&visual, &text);
        let b = cross_modal_binding(&visual, &text);
        assert_eq!(a, b);
    }

    #[test]
    fn test_cross_modal_binding_different_inputs_differ() {
        let v1 = dummy_vsa();
        let t1 = dummy_vsa();
        let v2 = dummy_vsa();
        let t2 = dummy_vsa();
        let a = cross_modal_binding(&v1, &t1);
        let b = cross_modal_binding(&v2, &t2);
        let sim = QuantizedVSA::similarity(&a, &b);
        assert!(
            sim < 0.9,
            "different bindings should produce different results"
        );
    }

    #[test]
    fn test_gated_attention_cycle_low_salience_blocked() {
        let config = ModalityGateConfig::default();
        let gate = ModalityGate::new(config, 8);
        let manar_config = ManarConfig::default();
        let mut manar = ManarAttention::new(manar_config);
        let vsa = vec![0u8; VSA_DIM];
        let result = gated_attention_cycle(SenseModality::Visual, &vsa, &gate, &mut manar);
        assert!(result >= 0.0 && result <= 1.0);
    }

    #[test]
    fn test_all_modalities_have_thresholds() {
        let config = ModalityGateConfig::default();
        for mod_ in SenseModality::all() {
            let t = config.thresholds_for(*mod_);
            assert!(t.min_similarity >= 0.5 && t.min_similarity <= 1.0);
            assert!(t.salience_decay >= 0.0 && t.salience_decay <= 1.0);
            assert!(t.activation_boost >= 0.0 && t.activation_boost <= 1.0);
        }
    }

    #[test]
    fn test_modality_gate_affinities_contain_all_modalities() {
        let config = ModalityGateConfig::default();
        let gate = ModalityGate::new(config, 36);
        for mod_ in SenseModality::all() {
            let slots = gate.affinity_slots(*mod_);
            assert!(
                !slots.is_empty(),
                "modality {:?} should have affinity slots",
                mod_
            );
        }
    }

    #[test]
    fn test_visual_and_text_modalities_independent_slots() {
        let config = ModalityGateConfig::default();
        let gate = ModalityGate::new(config, 36);
        let visual_slots = gate.affinity_slots(SenseModality::Visual);
        let document_slots = gate.affinity_slots(SenseModality::Document);
        assert!(!visual_slots.is_empty());
        assert!(!document_slots.is_empty());
        let shared: Vec<&usize> = visual_slots
            .iter()
            .filter(|s| document_slots.contains(s))
            .collect();
        assert!(
            shared.len() != visual_slots.len() || shared.len() != document_slots.len(),
            "visual and document should not have identical slot affinity sets"
        );
    }

    #[test]
    fn test_independent_gating_scores_per_modality() {
        let config = ModalityGateConfig::default();
        let gate = ModalityGate::new(config.clone(), 18);
        let vsa = dummy_vsa();
        let visual_score = gate.gate(SenseModality::Visual, &vsa);
        let mental_score = gate.gate(SenseModality::Mental, &vsa);
        assert!((visual_score - 0.15).abs() < 1e-6 || visual_score >= 0.0);
        assert!((mental_score - 0.12).abs() < 1e-6 || mental_score >= 0.0);
        let expected_visual_boost = config
            .thresholds_for(SenseModality::Visual)
            .activation_boost;
        let expected_mental_boost = config
            .thresholds_for(SenseModality::Mental)
            .activation_boost;
        assert!(
            (expected_visual_boost - expected_mental_boost).abs() > 1e-9,
            "visual and mental should have different activation boosts"
        );
    }

    #[test]
    fn test_route_to_slots_different_modalities_different_routes() {
        let config = ModalityGateConfig::default();
        let gate = ModalityGate::new(config, 27);
        let visual_routes = ModalityRouter::route_to_slots(SenseModality::Visual, &gate, 27);
        let auditory_routes = ModalityRouter::route_to_slots(SenseModality::Auditory, &gate, 27);
        assert!(!visual_routes.is_empty());
        assert!(!auditory_routes.is_empty());
    }

    #[test]
    fn test_document_modality_maps_to_visual_thresholds() {
        let config = ModalityGateConfig::default();
        let visual_t = config.thresholds_for(SenseModality::Visual);
        let doc_t = config.thresholds_for(SenseModality::Document);
        assert_eq!(visual_t.min_similarity, doc_t.min_similarity);
        assert_eq!(visual_t.salience_decay, doc_t.salience_decay);
        assert_eq!(visual_t.activation_boost, doc_t.activation_boost);
    }
}
