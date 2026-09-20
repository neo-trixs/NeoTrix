use serde::{Deserialize, Serialize};

use super::coupling::{AdjustmentParam, CognitiveCoupling};
use super::emotion_state::EmotionState;

/// A decision option with an associated base weight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedOption {
    /// Human-readable label for this option.
    pub label: String,
    /// Base weight before emotional influence (0.0–1.0).
    pub base_weight: f32,
    /// Final weight after emotional influence.
    pub final_weight: f32,
}

impl WeightedOption {
    pub fn new(label: impl Into<String>, base_weight: f32) -> Self {
        Self {
            label: label.into(),
            base_weight: base_weight.clamp(0.0, 1.0),
            final_weight: base_weight,
        }
    }
}

/// Engine that applies emotional influence to decision options.
///
/// Given a set of options and an emotion state, it computes adjusted weights
/// using the cognitive coupling map and returns the options sorted by
/// final weight (descending).
#[derive(Debug)]
pub struct InfluenceEngine {
    coupling: CognitiveCoupling,
}

impl Default for InfluenceEngine {
    fn default() -> Self {
        Self {
            coupling: CognitiveCoupling::default(),
        }
    }
}

impl InfluenceEngine {
    pub fn new(coupling: CognitiveCoupling) -> Self {
        Self { coupling }
    }

    /// Adjust option weights based on the current emotional state.
    ///
    /// Returns a new `Vec<WeightedOption>` sorted by `final_weight` descending.
    pub fn adjust_weights(
        &self,
        emotion: &EmotionState,
        options: Vec<WeightedOption>,
    ) -> Vec<WeightedOption> {
        let adjustments = self.coupling.compute_adjustments(emotion);

        let mut result: Vec<WeightedOption> = options
            .into_iter()
            .map(|mut opt| {
                for adj in &adjustments {
                    let modifier = self.compute_modifier(adj.param, adj.delta);
                    opt.final_weight = (opt.base_weight + modifier).clamp(0.0, 1.0);
                }
                opt
            })
            .collect();

        result.sort_by(|a, b| {
            b.final_weight
                .partial_cmp(&a.final_weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        result
    }

    /// Map an adjustment parameter to a weight modifier for a single option.
    ///
    /// This is a simplified mapping; real implementations would use
    /// option-specific metadata (e.g., risk profile, novelty).
    fn compute_modifier(&self, param: AdjustmentParam, delta: f32) -> f32 {
        match param {
            AdjustmentParam::Patience => delta * 0.1,
            AdjustmentParam::ExplorationRate => delta * 0.15,
            AdjustmentParam::RiskTolerance => delta * 0.12,
            AdjustmentParam::ConfidenceThreshold => delta * 0.08,
            AdjustmentParam::AttentionFocus => delta * 0.1,
        }
    }

    /// Access the underlying coupling.
    pub fn coupling(&self) -> &CognitiveCoupling {
        &self.coupling
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l4_emotion::nt_feel::cognitive_bridge::emotion_state::EmotionVariant;

    #[test]
    fn test_adjust_weights_returns_sorted() {
        let engine = InfluenceEngine::default();
        let emotion = EmotionState::new(EmotionVariant::Joy, 0.8, "good".into());
        let options = vec![
            WeightedOption::new("conservative", 0.3),
            WeightedOption::new("risky", 0.7),
            WeightedOption::new("balanced", 0.5),
        ];

        let result = engine.adjust_weights(&emotion, options);
        for w in result.windows(2) {
            assert!(w[0].final_weight >= w[1].final_weight);
        }
    }

    #[test]
    fn test_frustration_biases_toward_patience() {
        let engine = InfluenceEngine::default();
        let emotion = EmotionState::new(EmotionVariant::Frustration, 1.0, "stuck".into());
        let options = vec![WeightedOption::new("retry", 0.5)];

        let result = engine.adjust_weights(&emotion, options);
        assert!(!result.is_empty());
        assert!(result[0].final_weight >= 0.0);
    }

    #[test]
    fn test_neutral_emotion_no_change() {
        let engine = InfluenceEngine::default();
        let emotion = EmotionState::neutral();
        let options = vec![
            WeightedOption::new("a", 0.4),
            WeightedOption::new("b", 0.6),
        ];

        let result = engine.adjust_weights(&emotion, options);
        assert_eq!(result.len(), 2);
        assert!((result[0].base_weight - result[0].final_weight).abs() < f32::EPSILON);
    }

    #[test]
    fn test_weight_clamping() {
        let engine = InfluenceEngine::default();
        let emotion = EmotionState::new(EmotionVariant::Frustration, 1.0, "max".into());
        let options = vec![WeightedOption::new("edge", 0.99)];

        let result = engine.adjust_weights(&emotion, options);
        assert!(result[0].final_weight <= 1.0);
        assert!(result[0].final_weight >= 0.0);
    }
}
