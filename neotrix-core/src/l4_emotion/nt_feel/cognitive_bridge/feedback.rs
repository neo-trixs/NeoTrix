use serde::{Deserialize, Serialize};

use super::emotion_state::{EmotionState, EmotionVariant};

/// An outcome from a reasoning or decision cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    /// Whether the outcome was positive.
    pub success: bool,
    /// How surprising was the outcome (0.0 expected .. 1.0 very surprising).
    pub surprise_level: f32,
    /// Optional description of the outcome.
    pub description: String,
}

impl Outcome {
    pub fn success(description: impl Into<String>) -> Self {
        Self {
            success: true,
            surprise_level: 0.0,
            description: description.into(),
        }
    }

    pub fn failure(description: impl Into<String>) -> Self {
        Self {
            success: false,
            surprise_level: 0.3,
            description: description.into(),
        }
    }

    pub fn surprising(description: impl Into<String>, level: f32) -> Self {
        Self {
            success: true,
            surprise_level: level.clamp(0.0, 1.0),
            description: description.into(),
        }
    }
}

/// Feedback loop that updates emotion state based on outcomes.
///
/// Success decreases frustration and increases satisfaction/joy.
/// Failure increases frustration and decreases satisfaction.
/// Surprise modulates the current emotion's intensity.
#[derive(Debug)]
pub struct FeedbackLoop {
    /// How strongly outcomes influence emotions (0.0–1.0).
    influence_strength: f32,
    /// Decay rate applied to emotion between updates.
    decay_rate: f32,
    /// Baseline intensity toward which emotions decay.
    baseline: f32,
}

impl Default for FeedbackLoop {
    fn default() -> Self {
        Self {
            influence_strength: 0.3,
            decay_rate: 0.05,
            baseline: 0.3,
        }
    }
}

impl FeedbackLoop {
    pub fn new(influence_strength: f32, decay_rate: f32, baseline: f32) -> Self {
        Self {
            influence_strength: influence_strength.clamp(0.0, 1.0),
            decay_rate: decay_rate.clamp(0.0, 1.0),
            baseline: baseline.clamp(0.0, 1.0),
        }
    }

    /// Observe an outcome and update the emotion state accordingly.
    pub fn observe_outcome(&self, emotion: &mut EmotionState, outcome: &Outcome) {
        emotion.decay(self.decay_rate, self.baseline);

        let shift = self.influence_strength * outcome.surprise_level;

        if outcome.success {
            match emotion.variant {
                EmotionVariant::Frustration => {
                    emotion.intensity = (emotion.intensity - self.influence_strength).max(0.0);
                }
                EmotionVariant::Sadness => {
                    emotion.intensity = (emotion.intensity - self.influence_strength * 0.5).max(0.0);
                }
                _ => {
                    if outcome.surprise_level > 0.5 {
                        emotion.variant = EmotionVariant::Surprise;
                        emotion.intensity = (emotion.intensity + shift).min(1.0);
                    } else {
                        emotion.intensity = (emotion.intensity + self.influence_strength * 0.2).min(1.0);
                    }
                }
            }
        } else {
            match emotion.variant {
                EmotionVariant::Satisfaction | EmotionVariant::Joy => {
                    emotion.intensity = (emotion.intensity - self.influence_strength).max(0.0);
                }
                EmotionVariant::Curiosity => {
                    emotion.intensity = (emotion.intensity + self.influence_strength * 0.1).min(1.0);
                }
                _ => {
                    emotion.variant = EmotionVariant::Frustration;
                    emotion.intensity = (emotion.intensity + self.influence_strength).min(1.0);
                }
            }
        }

        emotion.valence = emotion.variant.base_valence() * emotion.intensity;
        emotion.arousal = emotion.variant.base_arousal() * emotion.intensity;
    }

    /// Observe multiple outcomes sequentially.
    pub fn observe_outcomes(&self, emotion: &mut EmotionState, outcomes: &[Outcome]) {
        for outcome in outcomes {
            self.observe_outcome(emotion, outcome);
        }
    }

    pub fn influence_strength(&self) -> f32 {
        self.influence_strength
    }

    pub fn decay_rate(&self) -> f32 {
        self.decay_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_reduces_frustration() {
        let fb = FeedbackLoop::default();
        let mut emotion = EmotionState::new(EmotionVariant::Frustration, 0.8, "stuck".into());
        let outcome = Outcome::success("worked");

        fb.observe_outcome(&mut emotion, &outcome);
        assert!(emotion.intensity < 0.8);
    }

    #[test]
    fn test_failure_increases_frustration() {
        let fb = FeedbackLoop::default();
        let mut emotion = EmotionState::new(EmotionVariant::Satisfaction, 0.5, "done".into());
        let outcome = Outcome::failure("broke");

        fb.observe_outcome(&mut emotion, &outcome);
        assert_eq!(emotion.variant, EmotionVariant::Frustration);
    }

    #[test]
    fn test_surprising_success_triggers_surprise() {
        let fb = FeedbackLoop::new(0.5, 0.0, 0.0);
        let mut emotion = EmotionState::new(EmotionVariant::Joy, 0.5, "ok".into());
        let outcome = Outcome::surprising("wow", 0.9);

        fb.observe_outcome(&mut emotion, &outcome);
        assert_eq!(emotion.variant, EmotionVariant::Surprise);
    }

    #[test]
    fn test_neutral_emotion_on_failure() {
        let fb = FeedbackLoop::default();
        let mut emotion = EmotionState::neutral();
        let outcome = Outcome::failure("miss");

        fb.observe_outcome(&mut emotion, &outcome);
        assert_eq!(emotion.variant, EmotionVariant::Frustration);
        assert!(emotion.intensity > 0.0);
    }

    #[test]
    fn test_multiple_outcomes() {
        let fb = FeedbackLoop::default();
        let mut emotion = EmotionState::new(EmotionVariant::Frustration, 0.9, "start".into());
        let outcomes = vec![
            Outcome::success("step1"),
            Outcome::success("step2"),
            Outcome::failure("step3"),
        ];

        fb.observe_outcomes(&mut emotion, &outcomes);
        assert!(emotion.intensity >= 0.0);
        assert!(emotion.intensity <= 1.0);
    }

    #[test]
    fn test_intensity_stays_in_bounds() {
        let fb = FeedbackLoop::new(1.0, 0.0, 0.0);
        let mut emotion = EmotionState::new(EmotionVariant::Joy, 1.0, "max".into());
        let outcome = Outcome::success("extreme");

        fb.observe_outcome(&mut emotion, &outcome);
        assert!(emotion.intensity >= 0.0);
        assert!(emotion.intensity <= 1.0);
    }
}
