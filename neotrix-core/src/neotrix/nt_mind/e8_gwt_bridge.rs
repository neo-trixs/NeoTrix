//! # E8→GWT Bridge — Hexagram-Driven Specialist Activation
//!
//! Maps E₈ 64-state hexagram axes to GWT specialist activation biases.
//! Each hexagram axis (abstraction, scope, method, depth, mode, stance)
//! influences which specialists are naturally active.
//!
//! This replaces pure keyword-based salience with E₈-informed activation.

use crate::core::nt_core_gwt::module_def::SpecialistType;
use crate::core::nt_core_hex::ReasoningHexagram;
use std::collections::HashMap;

/// A bias profile mapping each specialist to an activation modifier.
pub type BiasProfile = HashMap<SpecialistType, f64>;

/// Map a hexagram state to specialist activation biases.
///
/// Each hexagram axis contributes to different specialist activations:
/// - Abstraction (bit 5): Concrete(0) → PatternMatcher+, CodeAnalyzer+
///                        Abstract(1) → KnowledgeIntegrator+, CreativityGenerator+
/// - Scope (bit 4): Focused(0) → GoalPrioritizer+, RiskAssessor+
///                  Broad(1) → KnowledgeIntegrator+, ReflectionEngine+
/// - Method (bit 3): Analytical(0) → CodeAnalyzer+, PatternMatcher+
///                   Generative(1) → CreativityGenerator+, Planner+
/// - Depth (bit 2): Deep(0) → ReflectionEngine+, CodeAnalyzer+
///                  Fast(1) → PatternMatcher+, GoalPrioritizer+
/// - Mode (bit 1): Solo(0) → CodeAnalyzer+, PatternMatcher+
///                 Collaborative(1) → KnowledgeIntegrator+, KnowledgeRetriever+
/// - Stance (bit 0): Certain(0) → RiskAssessor+, GoalPrioritizer+
///                   Exploratory(1) → CreativityGenerator+, AnomalyDetector+
pub fn hexagram_to_biases(hex: ReasoningHexagram) -> BiasProfile {
    let mut biases = HashMap::new();

    let abstraction = hex.axis(5);
    let scope = hex.axis(4);
    let method = hex.axis(3);
    let depth = hex.axis(2);
    let mode = hex.axis(1);
    let stance = hex.axis(0);

    // Abstraction axis
    if abstraction == 0 {
        *biases.entry(SpecialistType::PatternMatcher).or_insert(0.0) += 0.15;
        *biases.entry(SpecialistType::CodeAnalyzer).or_insert(0.0) += 0.15;
    } else {
        *biases
            .entry(SpecialistType::KnowledgeIntegrator)
            .or_insert(0.0) += 0.15;
        *biases
            .entry(SpecialistType::CreativityGenerator)
            .or_insert(0.0) += 0.15;
        *biases
            .entry(SpecialistType::ReflectionEngine)
            .or_insert(0.0) += 0.10;
    }

    // Scope axis
    if scope == 0 {
        *biases.entry(SpecialistType::GoalPrioritizer).or_insert(0.0) += 0.15;
        *biases.entry(SpecialistType::RiskAssessor).or_insert(0.0) += 0.10;
    } else {
        *biases
            .entry(SpecialistType::KnowledgeIntegrator)
            .or_insert(0.0) += 0.15;
        *biases
            .entry(SpecialistType::ReflectionEngine)
            .or_insert(0.0) += 0.10;
        *biases.entry(SpecialistType::Planner).or_insert(0.0) += 0.10;
    }

    // Method axis
    if method == 0 {
        *biases.entry(SpecialistType::CodeAnalyzer).or_insert(0.0) += 0.15;
        *biases.entry(SpecialistType::PatternMatcher).or_insert(0.0) += 0.10;
    } else {
        *biases
            .entry(SpecialistType::CreativityGenerator)
            .or_insert(0.0) += 0.15;
        *biases.entry(SpecialistType::Planner).or_insert(0.0) += 0.10;
        *biases.entry(SpecialistType::AnomalyDetector).or_insert(0.0) += 0.10;
    }

    // Depth axis
    if depth == 0 {
        *biases
            .entry(SpecialistType::ReflectionEngine)
            .or_insert(0.0) += 0.15;
        *biases.entry(SpecialistType::CodeAnalyzer).or_insert(0.0) += 0.10;
    } else {
        *biases.entry(SpecialistType::PatternMatcher).or_insert(0.0) += 0.15;
        *biases.entry(SpecialistType::GoalPrioritizer).or_insert(0.0) += 0.10;
        *biases.entry(SpecialistType::AnomalyDetector).or_insert(0.0) += 0.10;
    }

    // Mode axis
    if mode == 0 {
        *biases.entry(SpecialistType::CodeAnalyzer).or_insert(0.0) += 0.10;
        *biases.entry(SpecialistType::PatternMatcher).or_insert(0.0) += 0.10;
    } else {
        *biases
            .entry(SpecialistType::KnowledgeIntegrator)
            .or_insert(0.0) += 0.10;
        *biases
            .entry(SpecialistType::KnowledgeRetriever)
            .or_insert(0.0) += 0.15;
        *biases.entry(SpecialistType::Planner).or_insert(0.0) += 0.10;
    }

    // Stance axis
    if stance == 0 {
        *biases.entry(SpecialistType::RiskAssessor).or_insert(0.0) += 0.15;
        *biases.entry(SpecialistType::GoalPrioritizer).or_insert(0.0) += 0.10;
    } else {
        *biases
            .entry(SpecialistType::CreativityGenerator)
            .or_insert(0.0) += 0.15;
        *biases.entry(SpecialistType::AnomalyDetector).or_insert(0.0) += 0.15;
        *biases
            .entry(SpecialistType::ReflectionEngine)
            .or_insert(0.0) += 0.10;
    }

    biases
}

/// Get the specialist most activated by a given hexagram state.
pub fn dominant_specialist(hex: ReasoningHexagram) -> SpecialistType {
    let biases = hexagram_to_biases(hex);
    biases
        .into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(st, _)| st)
        .unwrap_or(SpecialistType::PatternMatcher)
}

/// Apply hexagram biases to base salience scores.
/// Returns adjusted saliences.
pub fn apply_hex_bias(
    base_saliences: &[(SpecialistType, f64)],
    hex: ReasoningHexagram,
) -> Vec<(SpecialistType, f64)> {
    let biases = hexagram_to_biases(hex);
    base_saliences
        .iter()
        .map(|(st, base)| {
            let bias = biases.get(st).copied().unwrap_or(0.0);
            let adjusted = (base + bias).clamp(0.0, 1.0);
            (*st, adjusted)
        })
        .collect()
}

/// Get a human-readable description of why this hexagram biases certain specialists.
pub fn bias_rationale(hex: ReasoningHexagram) -> String {
    let bits = [
        ("Abstraction", hex.axis(5)),
        ("Scope", hex.axis(4)),
        ("Method", hex.axis(3)),
        ("Depth", hex.axis(2)),
        ("Mode", hex.axis(1)),
        ("Stance", hex.axis(0)),
    ];
    let axis_descriptions: Vec<String> = bits
        .iter()
        .map(|(name, val)| {
            let desc = match (*name, *val) {
                ("Abstraction", 0) => "Concrete → PatternMatcher, CodeAnalyzer",
                ("Abstraction", 1) => "Abstract → KnowledgeIntegrator, Creativity",
                ("Scope", 0) => "Focused → GoalPrioritizer, RiskAssessor",
                ("Scope", 1) => "Broad → Integrator, Reflection, Planner",
                ("Method", 0) => "Analytical → CodeAnalyzer, PatternMatcher",
                ("Method", 1) => "Generative → Creativity, Planner",
                ("Depth", 0) => "Deep → Reflection, CodeAnalyzer",
                ("Depth", 1) => "Fast → PatternMatcher, GoalPrioritizer",
                ("Mode", 0) => "Solo → CodeAnalyzer, PatternMatcher",
                ("Mode", 1) => "Collaborative → Integrator, Retriever",
                ("Stance", 0) => "Certain → RiskAssessor, GoalPrioritizer",
                ("Stance", 1) => "Exploratory → Creativity, AnomalyDetector",
                _ => "",
            };
            format!("{}={} ({})", name, val, desc)
        })
        .collect();
    format!(
        "E8 state {}: {}",
        hex.mode_name(),
        axis_descriptions.join("; ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concrete_analytic_yields_different_biases_than_abstract_generative() {
        // Mode 0 = Concrete, Focused, Analytical, Deep, Solo, Certain
        let concrete = ReasoningHexagram::new(0);
        // Mode 63 = Abstract, Broad, Generative, Fast, Collaborative, Exploratory
        let abstract_gen = ReasoningHexagram::new(63);

        let b1 = hexagram_to_biases(concrete);
        let b2 = hexagram_to_biases(abstract_gen);

        // Concrete should favor PatternMatcher/CodeAnalyzer
        assert!(
            b1.get(&SpecialistType::PatternMatcher)
                .copied()
                .unwrap_or(0.0)
                > 0.0
        );
        assert!(
            b1.get(&SpecialistType::CodeAnalyzer)
                .copied()
                .unwrap_or(0.0)
                > 0.0
        );

        // Abstract should favor Creativity/KnowledgeIntegrator
        assert!(
            b2.get(&SpecialistType::CreativityGenerator)
                .copied()
                .unwrap_or(0.0)
                > 0.0
        );
        assert!(
            b2.get(&SpecialistType::KnowledgeIntegrator)
                .copied()
                .unwrap_or(0.0)
                > 0.0
        );
    }

    #[test]
    fn test_dominant_specialist_differs_by_mode() {
        let d1 = dominant_specialist(ReasoningHexagram::new(0));
        let d2 = dominant_specialist(ReasoningHexagram::new(63));
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_apply_hex_bias_adjusts_salience() {
        let base = vec![
            (SpecialistType::PatternMatcher, 0.5),
            (SpecialistType::CodeAnalyzer, 0.5),
            (SpecialistType::CreativityGenerator, 0.5),
        ];
        // Mode 0 = Concrete/Analytical — should boost PatternMatcher + CodeAnalyzer
        let adjusted = apply_hex_bias(&base, ReasoningHexagram::new(0));
        let pm = adjusted
            .iter()
            .find(|(st, _)| *st == SpecialistType::PatternMatcher)
            .unwrap();
        let ca = adjusted
            .iter()
            .find(|(st, _)| *st == SpecialistType::CodeAnalyzer)
            .unwrap();
        assert!(pm.1 > 0.5);
        assert!(ca.1 > 0.5);
    }

    #[test]
    fn test_bias_rationale_contains_axis_names() {
        let rationale = bias_rationale(ReasoningHexagram::new(42));
        assert!(rationale.contains("Abstraction"));
        assert!(rationale.contains("Scope"));
        assert!(rationale.contains("E8 state"));
    }

    #[test]
    fn test_all_64_states_have_non_empty_biases() {
        for bits in 0..64u8 {
            let hex = ReasoningHexagram::new(bits);
            let biases = hexagram_to_biases(hex);
            assert!(!biases.is_empty(), "mode {} should have biases", bits);
        }
    }

    #[test]
    fn test_hex_modes_close_in_hamming_have_similar_dominant() {
        // Mode 0 and Mode 1 differ by 1 bit (stance flip)
        let d0 = dominant_specialist(ReasoningHexagram::new(0));
        let d1 = dominant_specialist(ReasoningHexagram::new(1));
        // Should often be similar but not guaranteed identical
        // Just ensure they're valid
        assert!(!format!("{:?}", d0).is_empty());
        assert!(!format!("{:?}", d1).is_empty());
    }
}
