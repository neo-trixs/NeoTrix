use serde::{Deserialize, Serialize};

/// The 8 core emotions used for cognition coupling.
///
/// These map to a subset of Plutchik's wheel plus AI-specific states,
/// providing a focused signal for reasoning adjustments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmotionVariant {
    Joy,
    Sadness,
    Anger,
    Fear,
    Surprise,
    Curiosity,
    Frustration,
    Satisfaction,
}

impl EmotionVariant {
    /// Base valence for this emotion (-1.0 negative .. +1.0 positive).
    pub fn base_valence(self) -> f32 {
        match self {
            Self::Joy | Self::Satisfaction => 1.0,
            Self::Surprise | Self::Curiosity => 0.3,
            Self::Sadness | Self::Frustration | Self::Anger | Self::Fear => -1.0,
        }
    }

    /// Base arousal level (0.0 calm .. 1.0 activated).
    pub fn base_arousal(self) -> f32 {
        match self {
            Self::Joy | Self::Surprise | Self::Frustration | Self::Anger => 0.8,
            Self::Fear | Self::Curiosity => 0.6,
            Self::Satisfaction => 0.4,
            Self::Sadness => 0.3,
        }
    }

    /// All variants in canonical order.
    pub fn all() -> &'static [EmotionVariant] {
        &[
            Self::Joy,
            Self::Sadness,
            Self::Anger,
            Self::Fear,
            Self::Surprise,
            Self::Curiosity,
            Self::Frustration,
            Self::Satisfaction,
        ]
    }
}

impl std::fmt::Display for EmotionVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Joy => "Joy",
            Self::Sadness => "Sadness",
            Self::Anger => "Anger",
            Self::Fear => "Fear",
            Self::Surprise => "Surprise",
            Self::Curiosity => "Curiosity",
            Self::Frustration => "Frustration",
            Self::Satisfaction => "Satisfaction",
        };
        write!(f, "{name}")
    }
}

/// Current emotional state snapshot for coupling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionState {
    /// Which emotion is dominant.
    pub variant: EmotionVariant,
    /// Intensity 0.0–1.0.
    pub intensity: f32,
    /// Valence derived from variant * intensity.
    pub valence: f32,
    /// Arousal derived from variant * intensity.
    pub arousal: f32,
    /// Free-form context that triggered this state.
    pub trigger: String,
}

impl EmotionState {
    pub fn new(variant: EmotionVariant, intensity: f32, trigger: String) -> Self {
        let clamped = intensity.clamp(0.0, 1.0);
        Self {
            variant,
            intensity: clamped,
            valence: variant.base_valence() * clamped,
            arousal: variant.base_arousal() * clamped,
            trigger,
        }
    }

    /// Neutral baseline state.
    pub fn neutral() -> Self {
        Self::new(EmotionVariant::Joy, 0.0, "baseline".into())
    }

    /// Is this state effectively neutral (below threshold)?
    pub fn is_neutral(&self, threshold: f32) -> bool {
        self.intensity < threshold
    }

    /// Apply intensity decay toward a baseline.
    pub fn decay(&mut self, rate: f32, baseline: f32) {
        let diff = self.intensity - baseline;
        self.intensity = baseline + diff * (1.0 - rate).clamp(0.0, 1.0);
        self.valence = self.variant.base_valence() * self.intensity;
        self.arousal = self.variant.base_arousal() * self.intensity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotion_variant_count() {
        assert_eq!(EmotionVariant::all().len(), 8);
    }

    #[test]
    fn test_valence_sign() {
        assert!(EmotionVariant::Joy.base_valence() > 0.0);
        assert!(EmotionVariant::Sadness.base_valence() < 0.0);
    }

    #[test]
    fn test_emotion_state_creation() {
        let state = EmotionState::new(EmotionVariant::Frustration, 0.8, "blocked".into());
        assert_eq!(state.variant, EmotionVariant::Frustration);
        assert!((state.intensity - 0.8).abs() < f32::EPSILON);
        assert!(state.valence < 0.0);
        assert_eq!(state.trigger, "blocked");
    }

    #[test]
    fn test_intensity_clamping() {
        let state = EmotionState::new(EmotionVariant::Joy, 5.0, "overload".into());
        assert!((state.intensity - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_neutral_state() {
        let state = EmotionState::neutral();
        assert!(state.is_neutral(0.1));
    }

    #[test]
    fn test_decay() {
        let mut state = EmotionState::new(EmotionVariant::Anger, 1.0, "triggered".into());
        state.decay(0.5, 0.0);
        assert!((state.intensity - 0.5).abs() < f32::EPSILON);
        assert!((state.valence - EmotionVariant::Anger.base_valence() * 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_display() {
        assert_eq!(EmotionVariant::Curiosity.to_string(), "Curiosity");
    }
}
