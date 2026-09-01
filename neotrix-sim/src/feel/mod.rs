/// NT-FEEL Emotion Engine — "心弦调律者"
///
/// Migrates from emotion_state.rs (6-dim EMA) + intrinsic_motivation.rs (curiosity drive)
/// Extends to 15 emotions with conflict resolution and cross-domain modulation.

use std::collections::HashMap;

/// PAD vector: Valence, Arousal, Dominance
#[derive(Debug, Clone, Copy)]
pub struct PadVector {
    pub valence: f32,    // -1 (negative) to +1 (positive)
    pub arousal: f32,    // -1 (calm) to +1 (excited)
    pub dominance: f32,  // -1 (submissive) to +1 (dominant)
}

impl Default for PadVector {
    fn default() -> Self {
        Self { valence: 0.0, arousal: 0.0, dominance: 0.0 }
    }
}

/// Emotion types — extends emotion_state.rs 6-dim to 15 types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmotionType {
    // Exploration
    Curiosity,
    Wonder,
    Confusion,
    // Achievement
    Satisfaction,
    Flow,
    Pride,
    // Difficulty
    Frustration,
    Anxiety,
    Fatigue,
    // Social
    Trust,
    Resonance,
    Empathy,
    // Meta
    MetaAwareness,
    Acceptance,
    // Core (from emotion_state.rs migration)
    Joy,
}

impl EmotionType {
    pub fn default_decay_rate(&self) -> f32 {
        match self {
            Self::Curiosity => 0.008,
            Self::Wonder => 0.012,
            Self::Confusion => 0.01,
            Self::Satisfaction => 0.006,
            Self::Flow => 0.004,
            Self::Pride => 0.008,
            Self::Frustration => 0.012,
            Self::Anxiety => 0.01,
            Self::Fatigue => 0.004,
            Self::Trust => 0.002,
            Self::Resonance => 0.008,
            Self::Empathy => 0.006,
            Self::MetaAwareness => 0.004,
            Self::Acceptance => 0.004,
            Self::Joy => 0.008,
        }
    }

    /// Cross-domain modulation effect on GWT attention weights
    pub fn gwt_modulation(&self, intensity: f32) -> GwtModulation {
        match self {
            Self::Curiosity => GwtModulation {
                novelty_weight: intensity * 0.8,
                familiarity_weight: -intensity * 0.3,
                focus_width: 0.7, // wider focus
            },
            Self::Flow => GwtModulation {
                novelty_weight: 0.0,
                familiarity_weight: 0.0,
                focus_width: 0.3, // narrow focus
            },
            Self::Frustration => GwtModulation {
                novelty_weight: intensity * 0.5,
                familiarity_weight: -intensity * 0.5,
                focus_width: 0.9, // very wide (looking for alternatives)
            },
            Self::Anxiety => GwtModulation {
                novelty_weight: -intensity * 0.3,
                familiarity_weight: intensity * 0.5,
                focus_width: 0.4, // narrow (threat-focused)
            },
            Self::Fatigue => GwtModulation {
                novelty_weight: -intensity * 0.5,
                familiarity_weight: -intensity * 0.5,
                focus_width: 0.5,
            },
            _ => GwtModulation::default(),
        }
    }

    /// E8 hexagram transition bias
    pub fn e8_bias(&self, intensity: f32) -> f32 {
        match self {
            Self::Curiosity => intensity * 0.3,     // increase exploration
            Self::Flow => intensity * 0.1,           // slight stability
            Self::Frustration => intensity * 0.4,    // increase randomness
            Self::Anxiety => -intensity * 0.3,       // decrease risk
            Self::Fatigue => -intensity * 0.5,       // decrease all transitions
            _ => 0.0,
        }
    }

    /// Memory consolidation strategy
    pub fn memory_strategy(&self, intensity: f32) -> MemoryStrategy {
        match self {
            Self::Curiosity => MemoryStrategy {
                encoding_depth: 0.8 + intensity * 0.2,
                consolidation_priority: 0.7,
                emotional_tag: "curious".into(),
            },
            Self::Satisfaction => MemoryStrategy {
                encoding_depth: 0.6,
                consolidation_priority: 0.8 + intensity * 0.2,
                emotional_tag: "satisfied".into(),
            },
            Self::Frustration => MemoryStrategy {
                encoding_depth: 0.9, // deep encode failures
                consolidation_priority: 0.9,
                emotional_tag: "frustrated".into(),
            },
            Self::Fatigue => MemoryStrategy {
                encoding_depth: 0.3, // shallow encode
                consolidation_priority: 0.2,
                emotional_tag: "tired".into(),
            },
            _ => MemoryStrategy::default(),
        }
    }

    /// ConsciousnessTree branch weight modulation
    pub fn ct_branch_modulation(&self, intensity: f32) -> HashMap<String, f32> {
        let mut weights = HashMap::new();
        match self {
            Self::Curiosity => {
                weights.insert("NT-WORLD".into(), 1.0 + intensity * 0.5);
            }
            Self::Flow => {
                weights.insert("NT-CORE".into(), 1.0 + intensity * 0.3);
            }
            Self::Frustration => {
                weights.insert("NT-REPAIR".into(), 1.0 + intensity * 0.6);
            }
            Self::Anxiety => {
                weights.insert("NT-SHIELD".into(), 1.0 + intensity * 0.5);
            }
            Self::Satisfaction => {
                weights.insert("NT-MEMORY".into(), 1.0 + intensity * 0.3);
            }
            Self::Trust => {
                weights.insert("NT-NEXUS".into(), 1.0 + intensity * 0.4);
            }
            _ => {}
        }
        weights
    }
}

#[derive(Debug, Clone, Default)]
pub struct GwtModulation {
    pub novelty_weight: f32,
    pub familiarity_weight: f32,
    pub focus_width: f32,
}

#[derive(Debug, Clone)]
pub struct MemoryStrategy {
    pub encoding_depth: f32,
    pub consolidation_priority: f32,
    pub emotional_tag: String,
}

impl Default for MemoryStrategy {
    fn default() -> Self {
        Self {
            encoding_depth: 0.5,
            consolidation_priority: 0.5,
            emotional_tag: "neutral".into(),
        }
    }
}

/// Single emotion instance
#[derive(Debug, Clone)]
pub struct Emotion {
    pub type_: EmotionType,
    pub intensity: f32,
}

/// System events that trigger emotions
#[derive(Debug, Clone)]
pub enum SystemEvent {
    NoveltyDetected { score: f32 },
    UnexpectedOutcome { surprise: f32 },
    GoalAchieved { difficulty: f32 },
    GoalBlocked { attempts: u32 },
    TaskInProgress { duration: f32, difficulty: f32 },
    BatteryLow { level: f32 },
    ErrorRate { rate: f32 },
    RecoverySuccess,
    PeerConnected { id: String },
    ConsensusReached { agreement: f32 },
    PeerStruggling { peer_id: String },
}

/// Conflict rule: two emotions that suppress each other
pub struct ConflictRule {
    pub a: EmotionType,
    pub b: EmotionType,
    pub resolver: fn(&mut Emotion, &mut Emotion),
}

/// The emotion engine — heart of NT-FEEL
pub struct EmotionEngine {
    pub emotions: Vec<Emotion>,
    pub pad: PadVector,
    decay_rates: HashMap<EmotionType, f32>,
    conflict_rules: Vec<ConflictRule>,
}

impl EmotionEngine {
    pub fn new() -> Self {
        let decay_rates: HashMap<EmotionType, f32> = EmotionType::iter()
            .map(|e| (e, e.default_decay_rate()))
            .collect();

        let conflict_rules = vec![
            ConflictRule {
                a: EmotionType::Flow,
                b: EmotionType::Frustration,
                resolver: |a, b| {
                    if a.intensity > b.intensity {
                        b.intensity *= 0.5;
                    } else {
                        a.intensity *= 0.5;
                    }
                },
            },
            ConflictRule {
                a: EmotionType::Satisfaction,
                b: EmotionType::Frustration,
                resolver: |a, b| {
                    let avg = (a.intensity + b.intensity) * 0.5;
                    a.intensity = avg;
                    b.intensity = avg;
                },
            },
            ConflictRule {
                a: EmotionType::Acceptance,
                b: EmotionType::Anxiety,
                resolver: |a, b| {
                    if a.intensity > 0.5 {
                        b.intensity *= 0.3;
                    }
                },
            },
        ];

        Self {
            emotions: Vec::new(),
            pad: PadVector::default(),
            decay_rates,
            conflict_rules,
        }
    }

    pub fn process_events(&mut self, events: &[SystemEvent]) {
        // 1. Trigger emotions from events
        for event in events {
            let deltas = self.evaluate(event);
            for delta in deltas {
                self.apply_delta(delta);
            }
        }

        // 2. Homeostasis: decay toward baseline
        self.decay();

        // 3. Conflict resolution
        self.resolve_conflicts();

        // 4. Threshold cutoff
        self.emotions.retain(|e| e.intensity > 0.05);

        // 5. Update PAD from active emotions
        self.update_pad();
    }

    fn evaluate(&self, event: &SystemEvent) -> Vec<(EmotionType, f32)> {
        match event {
            SystemEvent::NoveltyDetected { score } => {
                vec![(EmotionType::Curiosity, score * 0.8)]
            }
            SystemEvent::UnexpectedOutcome { surprise } => {
                if *surprise > 0.0 {
                    vec![(EmotionType::Wonder, surprise * 0.6)]
                } else {
                    vec![(EmotionType::Confusion, surprise.abs() * 0.6)]
                }
            }
            SystemEvent::GoalAchieved { difficulty } => {
                vec![
                    (EmotionType::Satisfaction, difficulty * 0.7),
                    (EmotionType::Joy, difficulty * 0.5),
                ]
            }
            SystemEvent::GoalBlocked { attempts } => {
                vec![(EmotionType::Frustration, (*attempts as f32 * 0.1).min(1.0))]
            }
            SystemEvent::TaskInProgress { difficulty, .. } => {
                // Flow: challenge ≈ skill (simplified)
                let balance = 1.0 - (difficulty - 0.5).abs();
                if balance > 0.7 {
                    vec![(EmotionType::Flow, balance * 0.5)]
                } else {
                    vec![]
                }
            }
            SystemEvent::BatteryLow { level } => {
                vec![(EmotionType::Fatigue, (1.0 - level) * 0.9)]
            }
            SystemEvent::ErrorRate { rate } => {
                vec![(EmotionType::Anxiety, rate * 0.8)]
            }
            SystemEvent::RecoverySuccess => {
                vec![
                    (EmotionType::Satisfaction, 0.6),
                    (EmotionType::Frustration, -0.4), // decrease
                ]
            }
            SystemEvent::PeerConnected { .. } => {
                vec![(EmotionType::Trust, 0.5)]
            }
            SystemEvent::ConsensusReached { agreement } => {
                vec![(EmotionType::Resonance, *agreement)]
            }
            SystemEvent::PeerStruggling { .. } => {
                vec![(EmotionType::Empathy, 0.4)]
            }
        }
    }

    fn apply_delta(&mut self, (type_, delta): (EmotionType, f32)) {
        if let Some(emotion) = self.emotions.iter_mut().find(|e| e.type_ == type_) {
            emotion.intensity = (emotion.intensity + delta).clamp(0.0, 1.0);
        } else if delta > 0.0 {
            self.emotions.push(Emotion { type_, intensity: delta.min(1.0) });
        }
    }

    fn decay(&mut self) {
        for emotion in &mut self.emotions {
            let decay = self.decay_rates.get(&emotion.type_).unwrap_or(&0.02);
            emotion.intensity *= 1.0 - decay;
        }
    }

    fn resolve_conflicts(&mut self) {
        for rule in &self.conflict_rules {
            let a_idx = self.emotions.iter().position(|e| e.type_ == rule.a);
            let b_idx = self.emotions.iter().position(|e| e.type_ == rule.b);

            if let (Some(ai), Some(bi)) = (a_idx, b_idx) {
                // SAFETY: ai != bi because a != b (different emotion types)
                if ai < bi {
                    let (left, right) = self.emotions.split_at_mut(bi);
                    (rule.resolver)(&mut left[ai], &mut right[0]);
                } else {
                    let (left, right) = self.emotions.split_at_mut(ai);
                    (rule.resolver)(&mut right[0], &mut left[bi]);
                }
            }
        }
    }

    fn update_pad(&mut self) {
        let mut valence = 0.0f32;
        let mut arousal = 0.0f32;
        let mut dominance = 0.0f32;
        let mut total = 0.0f32;

        for emotion in &self.emotions {
            let (v, a, d) = emotion_to_pad(&emotion.type_);
            valence += v * emotion.intensity;
            arousal += a * emotion.intensity;
            dominance += d * emotion.intensity;
            total += emotion.intensity;
        }

        if total > 0.01 {
            self.pad = PadVector {
                valence: (valence / total).clamp(-1.0, 1.0),
                arousal: (arousal / total).clamp(-1.0, 1.0),
                dominance: (dominance / total).clamp(-1.0, 1.0),
            };
        } else {
            self.pad = PadVector::default();
        }
    }

    pub fn dominant_emotion(&self) -> Option<EmotionType> {
        self.emotions.iter()
            .max_by(|a, b| a.intensity.partial_cmp(&b.intensity).unwrap())
            .map(|e| e.type_)
    }

    pub fn get_emotion(&self, type_: EmotionType) -> f32 {
        self.emotions.iter()
            .find(|e| e.type_ == type_)
            .map(|e| e.intensity)
            .unwrap_or(0.0)
    }
}

/// Map emotion type to PAD coordinates
fn emotion_to_pad(type_: &EmotionType) -> (f32, f32, f32) {
    match type_ {
        EmotionType::Curiosity => (0.3, 0.6, 0.5),
        EmotionType::Wonder => (0.7, 0.8, 0.3),
        EmotionType::Confusion => (-0.3, 0.4, -0.2),
        EmotionType::Satisfaction => (0.6, -0.2, 0.4),
        EmotionType::Flow => (0.4, 0.3, 0.6),
        EmotionType::Pride => (0.5, 0.3, 0.7),
        EmotionType::Frustration => (-0.5, 0.6, -0.3),
        EmotionType::Anxiety => (-0.4, 0.7, -0.5),
        EmotionType::Fatigue => (-0.3, -0.5, -0.4),
        EmotionType::Trust => (0.4, -0.1, 0.3),
        EmotionType::Resonance => (0.5, 0.2, 0.4),
        EmotionType::Empathy => (0.2, 0.1, -0.1),
        EmotionType::MetaAwareness => (0.1, 0.2, 0.5),
        EmotionType::Acceptance => (0.3, -0.3, 0.2),
        EmotionType::Joy => (0.8, 0.5, 0.4),
    }
}

/// Iterator over all emotion types
pub struct EmotionTypeIter {
    idx: usize,
}

impl EmotionType {
    pub fn iter() -> EmotionTypeIter {
        EmotionTypeIter { idx: 0 }
    }
}

impl Iterator for EmotionTypeIter {
    type Item = EmotionType;

    fn next(&mut self) -> Option<Self::Item> {
        let all = [
            EmotionType::Curiosity, EmotionType::Wonder, EmotionType::Confusion,
            EmotionType::Satisfaction, EmotionType::Flow, EmotionType::Pride,
            EmotionType::Frustration, EmotionType::Anxiety, EmotionType::Fatigue,
            EmotionType::Trust, EmotionType::Resonance, EmotionType::Empathy,
            EmotionType::MetaAwareness, EmotionType::Acceptance, EmotionType::Joy,
        ];
        if self.idx < all.len() {
            let item = all[self.idx];
            self.idx += 1;
            Some(item)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emotion_creation() {
        let engine = EmotionEngine::new();
        assert!(engine.emotions.is_empty());
    }

    #[test]
    fn curiosity_trigger() {
        let mut engine = EmotionEngine::new();
        engine.process_events(&[SystemEvent::NoveltyDetected { score: 0.9 }]);
        let curiosity = engine.get_emotion(EmotionType::Curiosity);
        assert!(curiosity > 0.5);
    }

    #[test]
    fn decay_reduces_intensity() {
        let mut engine = EmotionEngine::new();
        engine.process_events(&[SystemEvent::NoveltyDetected { score: 1.0 }]);
        let before = engine.get_emotion(EmotionType::Curiosity);
        engine.process_events(&[]);
        let after = engine.get_emotion(EmotionType::Curiosity);
        assert!(after < before);
    }

    #[test]
    fn flow_suppresses_frustration() {
        let mut engine = EmotionEngine::new();
        // Trigger both flow and frustration
        engine.emotions.push(Emotion { type_: EmotionType::Flow, intensity: 0.8 });
        engine.emotions.push(Emotion { type_: EmotionType::Frustration, intensity: 0.6 });
        engine.resolve_conflicts();
        // The stronger one should survive
        let flow = engine.get_emotion(EmotionType::Flow);
        let frustration = engine.get_emotion(EmotionType::Frustration);
        assert!(flow > frustration);
    }

    #[test]
    fn pad_updates() {
        let mut engine = EmotionEngine::new();
        engine.process_events(&[SystemEvent::NoveltyDetected { score: 1.0 }]);
        // Curiosity has positive valence
        assert!(engine.pad.valence > 0.0);
    }

    #[test]
    fn dominant_emotion() {
        let mut engine = EmotionEngine::new();
        engine.emotions.push(Emotion { type_: EmotionType::Joy, intensity: 0.9 });
        engine.emotions.push(Emotion { type_: EmotionType::Curiosity, intensity: 0.3 });
        assert_eq!(engine.dominant_emotion(), Some(EmotionType::Joy));
    }

    #[test]
    fn gwt_modulation_curiosity() {
        let mod_ = EmotionType::Curiosity.gwt_modulation(0.8);
        assert!(mod_.novelty_weight > 0.0);
        assert!(mod_.focus_width > 0.5); // wide focus
    }

    #[test]
    fn e8_bias_frustration() {
        let bias = EmotionType::Frustration.e8_bias(0.7);
        assert!(bias > 0.0); // increases randomness
    }

    #[test]
    fn memory_strategy_curiosity() {
        let strat = EmotionType::Curiosity.memory_strategy(0.8);
        assert!(strat.encoding_depth > 0.7); // deep encoding
    }
}
