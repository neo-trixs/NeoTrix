use serde::{Deserialize, Serialize};

use super::emotion_state::{EmotionState, EmotionVariant};

/// Reasoning adjustment triggered by an emotional state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningAdjustment {
    /// Which parameter to adjust.
    pub param: AdjustmentParam,
    /// Signed adjustment: positive increases, negative decreases.
    pub delta: f32,
}

/// Tunable reasoning parameters affected by emotion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdjustmentParam {
    Patience,
    ExplorationRate,
    RiskTolerance,
    ConfidenceThreshold,
    AttentionFocus,
}

impl std::fmt::Display for AdjustmentParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Patience => "Patience",
            Self::ExplorationRate => "ExplorationRate",
            Self::RiskTolerance => "RiskTolerance",
            Self::ConfidenceThreshold => "ConfidenceThreshold",
            Self::AttentionFocus => "AttentionFocus",
        };
        write!(f, "{name}")
    }
}

/// Maps emotion states to reasoning adjustments.
///
/// Each mapping is a (variant, intensity_threshold) → adjustment rule.
/// When an emotion state exceeds the threshold, the corresponding
/// reasoning adjustment is applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveCoupling {
    /// Coupling rules: (variant, threshold, adjustments).
    rules: Vec<CouplingRule>,
    /// Strength multiplier (0.0–1.0) applied to all adjustments.
    strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CouplingRule {
    variant: EmotionVariant,
    threshold: f32,
    adjustments: Vec<ReasoningAdjustment>,
}

impl Default for CognitiveCoupling {
    fn default() -> Self {
        let mut coupling = Self {
            rules: Vec::new(),
            strength: 0.7,
        };
        coupling.add_default_rules();
        coupling
    }
}

impl CognitiveCoupling {
    pub fn new(strength: f32) -> Self {
        let mut coupling = Self {
            rules: Vec::new(),
            strength: strength.clamp(0.0, 1.0),
        };
        coupling.add_default_rules();
        coupling
    }

    fn add_default_rules(&mut self) {
        self.add_rule(
            EmotionVariant::Frustration,
            0.5,
            vec![
                ReasoningAdjustment {
                    param: AdjustmentParam::Patience,
                    delta: 0.3,
                },
                ReasoningAdjustment {
                    param: AdjustmentParam::ExplorationRate,
                    delta: -0.2,
                },
                ReasoningAdjustment {
                    param: AdjustmentParam::RiskTolerance,
                    delta: -0.1,
                },
            ],
        );
        self.add_rule(
            EmotionVariant::Curiosity,
            0.3,
            vec![
                ReasoningAdjustment {
                    param: AdjustmentParam::ExplorationRate,
                    delta: 0.4,
                },
                ReasoningAdjustment {
                    param: AdjustmentParam::AttentionFocus,
                    delta: 0.2,
                },
            ],
        );
        self.add_rule(
            EmotionVariant::Joy,
            0.4,
            vec![
                ReasoningAdjustment {
                    param: AdjustmentParam::ConfidenceThreshold,
                    delta: 0.2,
                },
                ReasoningAdjustment {
                    param: AdjustmentParam::RiskTolerance,
                    delta: 0.1,
                },
            ],
        );
        self.add_rule(
            EmotionVariant::Fear,
            0.4,
            vec![
                ReasoningAdjustment {
                    param: AdjustmentParam::RiskTolerance,
                    delta: -0.3,
                },
                ReasoningAdjustment {
                    param: AdjustmentParam::ConfidenceThreshold,
                    delta: 0.2,
                },
                ReasoningAdjustment {
                    param: AdjustmentParam::Patience,
                    delta: 0.2,
                },
            ],
        );
        self.add_rule(
            EmotionVariant::Sadness,
            0.5,
            vec![
                ReasoningAdjustment {
                    param: AdjustmentParam::ExplorationRate,
                    delta: -0.2,
                },
                ReasoningAdjustment {
                    param: AdjustmentParam::ConfidenceThreshold,
                    delta: -0.1,
                },
            ],
        );
        self.add_rule(
            EmotionVariant::Anger,
            0.5,
            vec![
                ReasoningAdjustment {
                    param: AdjustmentParam::RiskTolerance,
                    delta: 0.3,
                },
                ReasoningAdjustment {
                    param: AdjustmentParam::Patience,
                    delta: -0.3,
                },
            ],
        );
        self.add_rule(
            EmotionVariant::Surprise,
            0.3,
            vec![ReasoningAdjustment {
                param: AdjustmentParam::AttentionFocus,
                delta: 0.3,
            }],
        );
        self.add_rule(
            EmotionVariant::Satisfaction,
            0.4,
            vec![ReasoningAdjustment {
                param: AdjustmentParam::ConfidenceThreshold,
                delta: 0.15,
            }],
        );
    }

    pub fn add_rule(
        &mut self,
        variant: EmotionVariant,
        threshold: f32,
        adjustments: Vec<ReasoningAdjustment>,
    ) {
        self.rules.push(CouplingRule {
            variant,
            threshold: threshold.clamp(0.0, 1.0),
            adjustments,
        });
    }

    /// Compute the set of reasoning adjustments for a given emotional state.
    ///
    /// Returns empty vec if the emotion is below all rule thresholds or is neutral.
    pub fn compute_adjustments(&self, state: &EmotionState) -> Vec<ReasoningAdjustment> {
        if state.intensity < 0.01 {
            return Vec::new();
        }

        self.rules
            .iter()
            .filter(|r| r.variant == state.variant && state.intensity >= r.threshold)
            .flat_map(|r| {
                let scale = self.strength * state.intensity;
                r.adjustments.iter().map(move |adj| ReasoningAdjustment {
                    param: adj.param,
                    delta: adj.delta * scale,
                })
            })
            .collect()
    }

    /// Get the coupling strength.
    pub fn strength(&self) -> f32 {
        self.strength
    }

    /// Set the coupling strength.
    pub fn set_strength(&mut self, strength: f32) {
        self.strength = strength.clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_coupling_has_rules() {
        let coupling = CognitiveCoupling::default();
        assert!(!coupling.rules.is_empty());
    }

    #[test]
    fn test_frustration_increases_patience() {
        let coupling = CognitiveCoupling::default();
        let state = EmotionState::new(EmotionVariant::Frustration, 0.8, "blocked".into());
        let adjustments = coupling.compute_adjustments(&state);
        let patience = adjustments.iter().find(|a| a.param == AdjustmentParam::Patience);
        assert!(patience.is_some());
        assert!(patience.unwrap().delta > 0.0);
    }

    #[test]
    fn test_curiosity_increases_exploration() {
        let coupling = CognitiveCoupling::default();
        let state = EmotionState::new(EmotionVariant::Curiosity, 0.6, "novel".into());
        let adjustments = coupling.compute_adjustments(&state);
        let exploration = adjustments
            .iter()
            .find(|a| a.param == AdjustmentParam::ExplorationRate);
        assert!(exploration.is_some());
        assert!(exploration.unwrap().delta > 0.0);
    }

    #[test]
    fn test_neutral_state_no_adjustments() {
        let coupling = CognitiveCoupling::default();
        let state = EmotionState::neutral();
        let adjustments = coupling.compute_adjustments(&state);
        assert!(adjustments.is_empty());
    }

    #[test]
    fn test_below_threshold_no_adjustments() {
        let coupling = CognitiveCoupling::default();
        let state = EmotionState::new(EmotionVariant::Frustration, 0.2, "minor".into());
        let adjustments = coupling.compute_adjustments(&state);
        assert!(adjustments.is_empty());
    }

    #[test]
    fn test_strength_scales_adjustments() {
        let mut coupling = CognitiveCoupling::new(1.0);
        let state = EmotionState::new(EmotionVariant::Curiosity, 1.0, "strong".into());
        let full = coupling.compute_adjustments(&state);

        coupling.set_strength(0.5);
        let half = coupling.compute_adjustments(&state);

        assert_eq!(full.len(), half.len());
        for (a, b) in full.iter().zip(half.iter()) {
            assert!((a.delta - b.delta * 2.0).abs() < f32::EPSILON);
        }
    }
}
