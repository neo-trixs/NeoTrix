use serde::{Deserialize, Serialize};

const VALENCE_DECAY: f64 = 0.05;
const AROUSAL_DECAY: f64 = 0.08;
const HISTORY_SIZE: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NamedEmotion {
    Excitement,
    Curiosity,
    Satisfaction,
    Frustration,
    Confusion,
    Calm,
    Gratitude,
    Humor,
    Neutral,
}

impl NamedEmotion {
    pub fn from_valence_arousal(valence: f64, arousal: f64) -> Self {
        match () {
            _ if valence > 0.5 && arousal > 0.6 => NamedEmotion::Excitement,
            _ if valence > 0.2 && arousal > 0.5 => NamedEmotion::Curiosity,
            _ if valence > 0.5 && arousal < 0.4 => NamedEmotion::Satisfaction,
            _ if valence > 0.3 && valence < 0.7 && arousal > 0.2 && arousal < 0.6 => NamedEmotion::Gratitude,
            _ if valence < -0.3 && arousal > 0.4 => NamedEmotion::Frustration,
            _ if valence.abs() < 0.2 && arousal > 0.5 => NamedEmotion::Confusion,
            _ if arousal <= 0.25 => NamedEmotion::Calm,
            _ if valence > 0.3 && arousal > 0.3 => NamedEmotion::Humor,
            _ => NamedEmotion::Neutral,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValenceAxis {
    pub valence: f64,
    pub arousal: f64,
    pub history: Vec<(f64, f64)>,
    // Evolution counters
    pub accumulation_count: u64,
    pub generalization_strength: f64,
    pub differentiation_level: f64,
    pub regulation_strength: f64,
    // Anticipated
    pub predicted_valence: f64,
    pub predicted_arousal: f64,
}

impl Default for ValenceAxis {
    fn default() -> Self {
        Self::new()
    }
}

impl ValenceAxis {
    pub fn new() -> Self {
        Self {
            valence: 0.0,
            arousal: 0.3,
            history: Vec::with_capacity(HISTORY_SIZE),
            accumulation_count: 0,
            generalization_strength: 0.0,
            differentiation_level: 0.0,
            regulation_strength: 0.0,
            predicted_valence: 0.0,
            predicted_arousal: 0.3,
        }
    }

    pub fn current_emotion(&self) -> NamedEmotion {
        NamedEmotion::from_valence_arousal(self.valence, self.arousal)
    }

    pub fn apply_reward(&mut self, reward: f64) {
        let valence_shift = reward.clamp(-0.3, 0.3);
        let arousal_shift = reward.abs().clamp(0.0, 0.2);
        self.shift(valence_shift, arousal_shift);
    }

    pub fn apply_negentropy(&mut self, delta_n: f64) {
        let valence_shift = (delta_n * 0.5).clamp(-0.3, 0.3);
        let arousal_shift = (delta_n.abs() * 0.3).clamp(0.0, 0.2);
        self.shift(valence_shift, arousal_shift);
    }

    pub fn apply_curiosity(&mut self, curiosity: f64) {
        let arousal_shift = curiosity.clamp(0.0, 0.3);
        let valence_shift = curiosity * 0.2;
        self.shift(valence_shift, arousal_shift);
    }

    pub fn apply_failure(&mut self) {
        self.shift(-0.15, 0.1);
    }

    pub fn apply_success(&mut self) {
        self.shift(0.1, -0.05);
    }

    pub fn shift(&mut self, valence_delta: f64, arousal_delta: f64) {
        self.accumulation_count += 1;

        // Accumulation: repeated similar triggers reinforce
        let accel = 1.0 + self.accumulation_count as f64 * 0.001;
        let v = (self.valence + valence_delta * accel).clamp(-1.0, 1.0);
        let a = (self.arousal + arousal_delta * accel).clamp(0.0, 1.0);

        // Differentiation: over time, same triggers produce finer-grained responses
        let diff = self.differentiation_level * 0.1;
        let v = (v * (1.0 - diff) + valence_delta * diff).clamp(-1.0, 1.0);
        let a = (a * (1.0 - diff) + arousal_delta * diff).clamp(0.0, 1.0);

        // Regulation: with experience, extreme swings dampen
        self.regulation_strength = (self.regulation_strength + 0.003).min(1.0);
        let reg_strength = self.regulation_strength * 0.8;
        let v = if v.abs() > 0.6 { v * (1.0 - reg_strength) + v.signum() * 0.6 * reg_strength } else { v };
        let a = if a > 0.8 { a * (1.0 - reg_strength) + 0.8 * reg_strength } else { a };

        // Generalization: similar contexts start to evoke similar emotions
        self.generalization_strength = (self.generalization_strength + 0.0005).min(1.0);

        self.valence = v;
        self.arousal = a;

        self.history.push((v, a));
        if self.history.len() > HISTORY_SIZE {
            self.history.remove(0);
        }

        // Update prediction as moving average
        self.predicted_valence = self.predicted_valence * 0.9 + v * 0.1;
        self.predicted_arousal = self.predicted_arousal * 0.9 + a * 0.1;
    }

    pub fn tick(&mut self) {
        // Natural decay toward neutral
        self.valence *= 1.0 - VALENCE_DECAY;
        self.arousal *= 1.0 - AROUSAL_DECAY;
        if self.arousal < 0.05 {
            self.arousal = 0.05;
        }
    }

    pub fn emotional_intensity(&self) -> f64 {
        self.valence.abs() * 0.5 + self.arousal * 0.5
    }

    pub fn coherence(&self) -> f64 {
        let recent = &self.history;
        if recent.len() < 5 {
            return 1.0;
        }
        let last_5 = &recent[recent.len().saturating_sub(5)..];
        let mean_v: f64 = last_5.iter().map(|(v, _)| v).sum::<f64>() / last_5.len() as f64;
        let mean_a: f64 = last_5.iter().map(|(_, a)| a).sum::<f64>() / last_5.len() as f64;
        let var_v: f64 = last_5.iter().map(|(v, _)| (v - mean_v).powi(2)).sum::<f64>() / last_5.len() as f64;
        let var_a: f64 = last_5.iter().map(|(_, a)| (a - mean_a).powi(2)).sum::<f64>() / last_5.len() as f64;
        (1.0 - (var_v.sqrt().min(1.0)) * 0.5 + (1.0 - var_a.sqrt().min(1.0)) * 0.5).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let v = ValenceAxis::new();
        assert!((v.valence - 0.0).abs() < 0.001);
        assert!((v.arousal - 0.3).abs() < 0.001);
        assert_eq!(v.current_emotion(), NamedEmotion::Neutral);
    }

    #[test]
    fn test_apply_negentropy_positive_delta() {
        let mut v = ValenceAxis::new();
        v.apply_negentropy(0.5);
        assert!(v.valence > 0.0, "positive ΔN should increase valence");
        assert!(v.arousal > 0.3, "positive ΔN should increase arousal");
    }

    #[test]
    fn test_apply_negentropy_negative_delta() {
        let mut v = ValenceAxis::new();
        v.apply_negentropy(-0.5);
        assert!(v.valence < 0.0, "negative ΔN should decrease valence");
    }

    #[test]
    fn test_negentropy_calibrates_emotion() {
        let mut v = ValenceAxis::new();
        v.apply_negentropy(0.8);
        let emotion = v.current_emotion();
        // Positive ΔN should map to positive emotions
        assert!(matches!(emotion, NamedEmotion::Excitement | NamedEmotion::Curiosity | NamedEmotion::Satisfaction));
    }

    #[test]
    fn test_negentropy_frustration() {
        let mut v = ValenceAxis::new();
        // Sustained negative ΔN builds frustration
        for _ in 0..5 {
            v.apply_negentropy(-0.3);
        }
        assert!(v.valence < -0.1, "sustained negative ΔN should cause negative valence");
    }

    #[test]
    fn test_apply_reward_shifts_valence() {
        let mut v = ValenceAxis::new();
        v.apply_reward(0.8);
        assert!(v.valence > 0.0, "positive reward should increase valence");
    }

    #[test]
    fn test_apply_failure_shifts_negative() {
        let mut v = ValenceAxis::new();
        v.apply_failure();
        assert!(v.valence < 0.0, "failure should decrease valence");
    }

    #[test]
    fn test_emotion_mapping() {
        assert_eq!(NamedEmotion::from_valence_arousal(0.8, 0.9), NamedEmotion::Excitement);
        assert_eq!(NamedEmotion::from_valence_arousal(0.3, 0.6), NamedEmotion::Curiosity);
        assert_eq!(NamedEmotion::from_valence_arousal(0.7, 0.2), NamedEmotion::Satisfaction);
        assert_eq!(NamedEmotion::from_valence_arousal(-0.5, 0.6), NamedEmotion::Frustration);
        assert_eq!(NamedEmotion::from_valence_arousal(0.0, 0.2), NamedEmotion::Calm);
    }

    #[test]
    fn test_tick_decays() {
        let mut v = ValenceAxis::new();
        v.valence = 0.8;
        v.arousal = 0.9;
        v.tick();
        assert!(v.valence < 0.8, "valence should decay toward 0");
        assert!(v.arousal < 0.9, "arousal should decay toward 0");
    }

    #[test]
    fn test_emotional_intensity() {
        let mut v = ValenceAxis::new();
        assert!(v.emotional_intensity() < 0.3);
        v.valence = 0.8;
        v.arousal = 0.9;
        let intensity = v.emotional_intensity();
        assert!(intensity > 0.5);
    }

    #[test]
    fn test_accumulation_amplifies() {
        let mut v = ValenceAxis::new();
        for _ in 0..10 {
            v.apply_reward(0.3);
        }
        assert!(v.accumulation_count >= 10);
        assert!(v.valence > 0.3, "accumulated rewards should build up");
    }

    #[test]
    fn test_coherence_high_when_stable() {
        let mut v = ValenceAxis::new();
        for _ in 0..10 {
            v.apply_reward(0.5);
        }
        let c = v.coherence();
        assert!(c > 0.5, "stable emotion should have high coherence");
    }

    #[test]
    fn test_prediction_moving_average() {
        let mut v = ValenceAxis::new();
        for _ in 0..10 {
            v.apply_reward(0.5);
        }
        assert!(v.predicted_valence > 0.0);
        assert!(v.predicted_arousal > 0.0);
    }

    #[test]
    fn test_shift_clamps() {
        let mut v = ValenceAxis::new();
        v.shift(10.0, 10.0);
        assert!(v.valence <= 1.0);
        assert!(v.arousal <= 1.0);
        v.shift(-10.0, -10.0);
        assert!(v.valence >= -1.0);
        assert!(v.arousal >= 0.0);
    }

    #[test]
    fn test_generalization_grows() {
        let mut v = ValenceAxis::new();
        for _ in 0..100 {
            v.apply_reward(0.1);
        }
        assert!(v.generalization_strength > 0.0);
    }

    #[test]
    fn test_regulation_dampens_extremes() {
        let mut v = ValenceAxis::new();
        for _ in 0..200 {
            v.apply_reward(1.0);
        }
        assert!(v.regulation_strength > 0.0);
        // After strong regulation, extreme should be dampened
        assert!(v.valence <= 0.9);
    }
}
