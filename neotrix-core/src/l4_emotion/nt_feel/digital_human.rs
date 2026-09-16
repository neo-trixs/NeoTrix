#![forbid(unsafe_code)]

//! Digital Human emotion types — animation-key mapping and text-based detection.
//!
//! Canonical home for the digital human `Emotion` enum and lightweight
//! `EmotionEngine`.  These were originally in `l1_action::nt_io::nt_io_digital_human`
//! and migrated here to consolidate all emotion domain types under L4.

use std::collections::VecDeque;
use std::time::Instant;

/// Digital human animation emotion — maps 1:1 to avatar animation keys.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Emotion {
    Neutral,
    Happy,
    Sad,
    Angry,
    Surprised,
    Confused,
    Thinking,
}

impl Emotion {
    pub fn animation_key(&self) -> &'static str {
        match self {
            Emotion::Neutral => "idle",
            Emotion::Happy => "smile",
            Emotion::Sad => "frown",
            Emotion::Angry => "fury",
            Emotion::Surprised => "shock",
            Emotion::Confused => "tilt",
            Emotion::Thinking => "look_up",
        }
    }
}

/// Bridge affective expression keys to digital human `Emotion` enum.
pub fn emotion_from_expression(expression: &str) -> Emotion {
    match expression {
        "smile" => Emotion::Happy,
        "frown" => Emotion::Sad,
        "fury" => Emotion::Angry,
        "shock" => Emotion::Surprised,
        "tilt" => Emotion::Confused,
        "look_up" => Emotion::Thinking,
        _ => Emotion::Neutral,
    }
}

/// Lightweight text-based emotion engine for digital human pipeline.
#[derive(Debug, Clone)]
pub struct EmotionEngine {
    current: Emotion,
    intensity: f64,
    history: VecDeque<(Emotion, Instant)>,
}

impl Default for EmotionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EmotionEngine {
    pub fn new() -> Self {
        Self {
            current: Emotion::Neutral,
            intensity: 0.5,
            history: VecDeque::new(),
        }
    }

    pub fn detect_from_text(&mut self, text: &str) -> Emotion {
        let lower = text.to_lowercase();
        let emotion = if lower.contains("happy")
            || lower.contains("great")
            || lower.contains("thank")
        {
            Emotion::Happy
        } else if lower.contains("sad") || lower.contains("sorry") || lower.contains("bad") {
            Emotion::Sad
        } else if lower.contains("angry") || lower.contains("mad") || lower.contains("furious")
        {
            Emotion::Angry
        } else if lower.contains("wow")
            || lower.contains("amazing")
            || lower.contains("unexpected")
        {
            Emotion::Surprised
        } else if lower.contains("hmm")
            || lower.contains("maybe")
            || lower.chars().any(|c| c == '?')
        {
            Emotion::Confused
        } else {
            Emotion::Neutral
        };
        self.current = emotion;
        self.history.push_back((emotion, Instant::now()));
        if self.history.len() > 100 {
            self.history.pop_front();
        }
        emotion
    }

    pub fn set_intensity(&mut self, intensity: f64) {
        self.intensity = intensity.max(0.0).min(1.0);
    }

    pub fn current_emotion(&self) -> Emotion {
        self.current
    }

    pub fn intensity(&self) -> f64 {
        self.intensity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotion_detection() {
        let mut ee = EmotionEngine::new();
        assert_eq!(ee.detect_from_text("thank you very much"), Emotion::Happy);
        assert_eq!(ee.detect_from_text("I am so angry"), Emotion::Angry);
        assert_eq!(ee.detect_from_text("ordinary text"), Emotion::Neutral);
    }

    #[test]
    fn test_emotion_intensity_clamping() {
        let mut ee = EmotionEngine::new();
        ee.set_intensity(1.5);
        assert!((ee.intensity() - 1.0).abs() < 0.01);
        ee.set_intensity(-0.5);
        assert!((ee.intensity() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_emotion_animation_keys() {
        assert_eq!(Emotion::Neutral.animation_key(), "idle");
        assert_eq!(Emotion::Confused.animation_key(), "tilt");
        assert_eq!(Emotion::Thinking.animation_key(), "look_up");
    }

    #[test]
    fn test_emotion_from_expression() {
        assert_eq!(emotion_from_expression("fury"), Emotion::Angry);
        assert_eq!(emotion_from_expression("idle"), Emotion::Neutral);
        assert_eq!(emotion_from_expression("look_up"), Emotion::Thinking);
    }
}
