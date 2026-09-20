//! Emotion-to-Reasoning Bridge — absorbed from neotrix-sim
//!
//! Maps PAD emotional state to GWT attention weights, E8 transition
//! biases, and ConsciousnessTree branch priorities.
//!
//! Source: archive/neotrix-sim/src/core_bridge/mod.rs

/// NT-CORE Bridge — connects NT-FEEL emotions to NT-CORE reasoning
///
/// Emotional states modulate:
/// - GWT attention weights (what gets broadcast)
/// - E8 hexagram transition probabilities (what gets reasoned about)
/// - ConsciousnessTree branch priorities (which domain gets focus)

// Local stubs for absorbed types (originally from neotrix-sim::feel)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmotionType {
    Curiosity, Wonder, Confusion, Satisfaction, Flow, Pride,
    Frustration, Anxiety, Fatigue, Trust, Resonance, Empathy,
    MetaAwareness, Acceptance, Joy,
}

/// GWT modulation weights derived from an emotion
#[derive(Debug, Clone, Default)]
pub struct GwtModulation {
    pub novelty_weight: f32,
    pub focus_width: f32,
}

impl EmotionType {
    pub fn gwt_modulation(&self, intensity: f64) -> GwtModulation {
        let i = intensity as f32;
        match self {
            EmotionType::Curiosity => GwtModulation { novelty_weight: i * 0.8, focus_width: 0.3 + i * 0.4 },
            EmotionType::Flow => GwtModulation { novelty_weight: i * 0.3, focus_width: 0.9 - i * 0.3 },
            EmotionType::Anxiety => GwtModulation { novelty_weight: -i * 0.2, focus_width: 0.1 + i * 0.2 },
            _ => GwtModulation { novelty_weight: i * 0.2, focus_width: 0.5 },
        }
    }
    pub fn e8_bias(&self, intensity: f64) -> f32 {
        let i = intensity as f32;
        match self {
            EmotionType::Curiosity => i * 0.3,
            EmotionType::Frustration => -i * 0.2,
            EmotionType::Flow => i * 0.1,
            _ => 0.0,
        }
    }
    pub fn ct_branch_modulation(&self, intensity: f64) -> Vec<(String, f32)> {
        let i = intensity as f32;
        match self {
            EmotionType::Curiosity => vec![("NT-WORLD".into(), 1.0 + i * 0.5)],
            EmotionType::Flow => vec![("NT-CORE".into(), 1.0 + i * 0.3)],
            EmotionType::Frustration => vec![("NT-REPAIR".into(), 1.0 + i * 0.4)],
            EmotionType::Anxiety => vec![("NT-SHIELD".into(), 1.0 + i * 0.4)],
            _ => vec![],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PadVector {
    pub valence: f64,
    pub arousal: f64,
    pub dominance: f64,
}

#[derive(Debug, Clone, Default)]
pub struct EmotionEngine {
    pub emotions: std::collections::HashMap<EmotionType, f64>,
    pub pad: PadVector,
}

impl EmotionEngine {
    pub fn new() -> Self {
        Self {
            pad: PadVector::default(),
            emotions: std::collections::HashMap::new(),
        }
    }

    pub fn get_pad(&self) -> &PadVector { &self.pad }
    pub fn get_emotion(&self, et: EmotionType) -> f64 {
        self.emotions.get(&et).copied().unwrap_or(0.0)
    }

    pub fn process_events(&mut self, events: &[SystemEvent]) {
        for event in events {
            match event {
                SystemEvent::NoveltyDetected { score } => {
                    self.pad.valence += score * 0.3;
                    self.pad.arousal += score * 0.2;
                }
                SystemEvent::GoalBlocked { attempts } => {
                    self.pad.valence -= (*attempts as f64) * 0.1;
                }
                SystemEvent::GoalCompleted { success } => {
                    if *success {
                        self.pad.valence += 0.2;
                    } else {
                        self.pad.valence -= 0.2;
                    }
                }
                SystemEvent::SocialInteraction { positive } => {
                    if *positive {
                        self.pad.valence += 0.15;
                    } else {
                        self.pad.valence -= 0.15;
                    }
                }
                SystemEvent::ThreatDetected => {
                    self.pad.arousal += 0.5;
                    self.pad.dominance -= 0.3;
                }
                SystemEvent::LearningProgress { .. } => {
                    self.pad.valence += 0.05;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum SystemEvent {
    GoalCompleted { success: bool },
    SocialInteraction { positive: bool },
    ThreatDetected,
    NoveltyDetected { score: f64 },
    GoalBlocked { attempts: u32 },
    LearningProgress { amount: f64 },
}

/// Attention weight biases from emotions
#[derive(Debug, Clone, Default)]
pub struct AttentionBias {
    /// How much to weight novel signals vs familiar
    pub novelty_weight: f32,
    /// Focus width: 0.0 = laser focus, 1.0 = broad survey
    pub focus_width: f32,
    /// Domain-specific boosts
    pub domain_boosts: [(String, f32); 4],
}

/// E8 transition bias from emotions
#[derive(Debug, Clone, Default)]
pub struct TransitionBias {
    /// Bias toward exploration (positive) or exploitation (negative)
    pub exploration_bias: f32,
    /// Bias toward risk-taking (positive) or safety (negative)
    pub risk_bias: f32,
    /// Transition probability modifier
    pub probability_modifier: f32,
}

/// ConsciousnessTree branch priorities from emotions
#[derive(Debug, Clone, Default)]
pub struct CtBranchPriorities {
    /// Branch name → weight multiplier
    pub weights: Vec<(String, f32)>,
}

/// The bridge between emotion and reasoning
pub struct CoreBridge {
    pub attention_bias: AttentionBias,
    pub transition_bias: TransitionBias,
    pub ct_priorities: CtBranchPriorities,
    pub current_pad: PadVector,
}

impl CoreBridge {
    pub fn new() -> Self {
        Self {
            attention_bias: AttentionBias::default(),
            transition_bias: TransitionBias::default(),
            ct_priorities: CtBranchPriorities::default(),
            current_pad: PadVector::default(),
        }
    }

    /// Update all bridges from current emotional state
    pub fn update_from_feel(&mut self, feel: &EmotionEngine) {
        self.current_pad = feel.pad.clone();
        self.update_attention_bias(feel);
        self.update_transition_bias(feel);
        self.update_ct_priorities(feel);
    }

    fn update_attention_bias(&mut self, feel: &EmotionEngine) {
        let mut novelty = 0.0f32;
        let mut focus = 0.5f32;

        for (emotion_type, intensity) in &feel.emotions {
            let mod_ = emotion_type.gwt_modulation(*intensity);
            novelty += mod_.novelty_weight * *intensity as f32;
            focus += (mod_.focus_width - 0.5) * *intensity as f32;
        }

        self.attention_bias = AttentionBias {
            novelty_weight: novelty.clamp(-1.0, 1.0),
            focus_width: focus.clamp(0.1, 1.0),
            domain_boosts: [
                ("NT-CORE".into(), feel.get_emotion(EmotionType::Flow) as f32),
                ("NT-WORLD".into(), feel.get_emotion(EmotionType::Curiosity) as f32),
                ("NT-REPAIR".into(), feel.get_emotion(EmotionType::Frustration) as f32),
                ("NT-SHIELD".into(), feel.get_emotion(EmotionType::Anxiety) as f32),
            ],
        };
    }

    fn update_transition_bias(&mut self, feel: &EmotionEngine) {
        let mut exploration = 0.0f32;
        let mut risk = 0.0f32;

        for (emotion_type, intensity) in &feel.emotions {
            let bias = emotion_type.e8_bias(*intensity);
            exploration += bias;
            risk += match emotion_type {
                EmotionType::Anxiety => -*intensity as f32 * 0.3,
                EmotionType::Curiosity => *intensity as f32 * 0.2,
                _ => 0.0,
            };
        }

        self.transition_bias = TransitionBias {
            exploration_bias: exploration.clamp(-1.0, 1.0),
            risk_bias: risk.clamp(-1.0, 1.0),
            probability_modifier: (exploration * 0.1).clamp(-0.5, 0.5),
        };
    }

    fn update_ct_priorities(&mut self, feel: &EmotionEngine) {
        let mut weights = std::collections::HashMap::new();

        for (emotion_type, intensity) in &feel.emotions {
            let branch_weights = emotion_type.ct_branch_modulation(*intensity);
            for (branch, weight) in branch_weights {
                *weights.entry(branch).or_insert(1.0) += weight - 1.0;
            }
        }

        let mut sorted: Vec<_> = weights.into_iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sorted.truncate(6);

        self.ct_priorities = CtBranchPriorities { weights: sorted };
    }

    /// Get the modulated transition probability for a specific E8 hexagram
    pub fn modulate_transition(&self, base_probability: f32) -> f32 {
        (base_probability + self.transition_bias.probability_modifier).clamp(0.0, 1.0)
    }

    /// Get attention weight for a specific domain
    pub fn domain_weight(&self, domain: &str) -> f32 {
        for (name, weight) in &self.ct_priorities.weights {
            if name == domain {
                return *weight;
            }
        }
        1.0 // default neutral weight
    }

    /// Get novelty search weight
    pub fn novelty_weight(&self) -> f32 {
        self.attention_bias.novelty_weight
    }

    /// Get focus width (for GWT competition gate)
    pub fn focus_width(&self) -> f32 {
        self.attention_bias.focus_width
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // SystemEvent defined locally above

    #[test]
    fn bridge_updates_from_feel() {
        let mut feel = EmotionEngine::new();
        feel.process_events(&[SystemEvent::NoveltyDetected { score: 0.9 }]);

        let mut bridge = CoreBridge::new();
        bridge.update_from_feel(&feel);

        // Curiosity should increase novelty weight
        assert!(bridge.novelty_weight() > 0.0);
    }

    #[test]
    fn transition_modulation() {
        let bridge = CoreBridge {
            transition_bias: TransitionBias {
                exploration_bias: 0.5,
                risk_bias: 0.0,
                probability_modifier: 0.1,
            },
            ..CoreBridge::new()
        };

        let modulated = bridge.modulate_transition(0.5);
        assert!((modulated - 0.6).abs() < 1e-6);
    }

    #[test]
    fn domain_weight_default() {
        let bridge = CoreBridge::new();
        assert!((bridge.domain_weight("NT-CORE") - 1.0).abs() < 1e-6);
    }

    #[test]
    fn frustration_increases_repair_weight() {
        let mut feel = EmotionEngine::new();
        // GoalBlocked triggers Frustration
        feel.process_events(&[SystemEvent::GoalBlocked { attempts: 5 }]);

        let mut bridge = CoreBridge::new();
        bridge.update_from_feel(&feel);

        let repair_weight = bridge.domain_weight("NT-REPAIR");
        assert!(repair_weight > 1.0);
    }
}
