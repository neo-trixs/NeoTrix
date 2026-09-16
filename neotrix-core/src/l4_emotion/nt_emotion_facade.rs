#![forbid(unsafe_code)]

//! L4 Emotion Facade — 唯一对外门面
//!
//! Wraps the internal `EmotionEngine` and exposes a simplified, stable API
//! for cross-layer consumers. All emotion processing flows through this facade.

use crate::l4_emotion::nt_feel::emotion_engine::{EmotionEngine, EmotionReport};

/// Emotional state snapshot — simplified view for external consumers.
#[derive(Debug, Clone)]
pub struct EmotionalSnapshot {
    pub primary: String,
    pub valence: f64,
    pub arousal: f64,
}

/// Writing tone suggestions derived from current emotional state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WritingTone {
    Neutral,
    Formal,
    Casual,
    Empathetic,
    Energetic,
}

/// Layer health report.
#[derive(Debug, Clone)]
pub struct LayerHealth {
    pub score: f64,
    pub status: String,
}

/// L4 Emotion Facade — 唯一对外门面
pub struct EmotionFacade {
    engine: EmotionEngine,
}

impl EmotionFacade {
    pub fn new() -> Self {
        Self {
            engine: EmotionEngine::new(),
        }
    }

    /// Detect emotion from text input and update internal state.
    pub fn process_input(&mut self, text: &str) -> EmotionalSnapshot {
        self.engine.detect_from_text(text);
        self.snapshot_from_report(self.engine.report())
    }

    /// Get current emotional state without mutation.
    pub fn current_state(&self) -> EmotionalSnapshot {
        self.snapshot_from_report(self.engine.report())
    }

    /// Suggest writing tone based on current emotion.
    pub fn suggest_tone(&self) -> WritingTone {
        let report = self.engine.report();
        Self::tone_from_report(&report)
    }

    /// Health check for the L4 Emotion layer.
    pub fn health(&self) -> LayerHealth {
        let report = self.engine.report();
        let score = report.confidence_score;
        let status = if score > 0.7 {
            "healthy".to_string()
        } else if score > 0.4 {
            "degraded".to_string()
        } else {
            "unhealthy".to_string()
        };
        LayerHealth { score, status }
    }

    fn snapshot_from_report(&self, report: EmotionReport) -> EmotionalSnapshot {
        EmotionalSnapshot {
            primary: report.emotion_label.label().to_string(),
            valence: report.valence,
            arousal: report.arousal,
        }
    }

    fn tone_from_report(report: &EmotionReport) -> WritingTone {
        match report.emotion_label {
            crate::core::nt_core_self::emotion_state::EmotionLabel::Joy => WritingTone::Energetic,
            crate::core::nt_core_self::emotion_state::EmotionLabel::Sadness => {
                WritingTone::Empathetic
            }
            crate::core::nt_core_self::emotion_state::EmotionLabel::Neutral => WritingTone::Neutral,
            crate::core::nt_core_self::emotion_state::EmotionLabel::Trust => WritingTone::Formal,
            crate::core::nt_core_self::emotion_state::EmotionLabel::Confused
            | crate::core::nt_core_self::emotion_state::EmotionLabel::Thinking => {
                WritingTone::Casual
            }
            _ => WritingTone::Neutral,
        }
    }
}

impl Default for EmotionFacade {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_facade_starts_neutral() {
        let facade = EmotionFacade::new();
        let state = facade.current_state();
        assert_eq!(state.primary, "neutral");
        assert!((state.valence - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_process_input_detects_joy() {
        let mut facade = EmotionFacade::new();
        let state = facade.process_input("Task completed successfully");
        assert_eq!(state.primary, "joy");
        assert!(state.valence > 0.5);
    }

    #[test]
    fn test_process_input_detects_sadness() {
        let mut facade = EmotionFacade::new();
        let state = facade.process_input("Error: build failed");
        assert_eq!(state.primary, "sadness");
    }

    #[test]
    fn test_process_input_detects_neutral() {
        let mut facade = EmotionFacade::new();
        let state = facade.process_input("The weather is okay");
        assert_eq!(state.primary, "neutral");
    }

    #[test]
    fn test_suggest_tone_joy() {
        let mut facade = EmotionFacade::new();
        facade.process_input("Task completed successfully");
        assert_eq!(facade.suggest_tone(), WritingTone::Energetic);
    }

    #[test]
    fn test_suggest_tone_neutral_default() {
        let facade = EmotionFacade::new();
        assert_eq!(facade.suggest_tone(), WritingTone::Neutral);
    }

    #[test]
    fn test_health_initial() {
        let facade = EmotionFacade::new();
        let health = facade.health();
        assert!(health.score > 0.0 && health.score <= 1.0);
        assert!(!health.status.is_empty());
    }

    #[test]
    fn test_health_healthy_after_positive_input() {
        let mut facade = EmotionFacade::new();
        facade.process_input("Task completed successfully");
        let health = facade.health();
        assert_eq!(health.status, "healthy");
    }

    #[test]
    fn test_writing_tone_debug() {
        let tone = WritingTone::Formal;
        assert_eq!(format!("{:?}", tone), "Formal");
    }

    #[test]
    fn test_emotional_snapshot_clone() {
        let snapshot = EmotionalSnapshot {
            primary: "joy".to_string(),
            valence: 0.8,
            arousal: 0.6,
        };
        let cloned = snapshot.clone();
        assert_eq!(cloned.primary, snapshot.primary);
        assert_eq!(cloned.valence, snapshot.valence);
    }

    #[test]
    fn test_facade_default() {
        let facade = EmotionFacade::default();
        let state = facade.current_state();
        assert_eq!(state.primary, "neutral");
    }
}
