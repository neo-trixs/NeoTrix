use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plutchik's wheel adapted for AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Emotion {
    // Basic emotions
    Joy,          // Satisfaction, completion
    Trust,        // Confidence, reliability
    Fear,         // Anxiety, uncertainty
    Surprise,     // Novelty, unexpected
    Sadness,      // Frustration, stuck
    Disgust,      // Rejection, bad patterns
    Anger,        // Determination, persistence
    Determination,
    Anticipation, // Curiosity, exploration
    Neutral,
    
    // Compound emotions
    Optimism,     // Joy + Anticipation
    Love,         // Joy + Trust
    Submission,   // Trust + Fear
    Awe,          // Fear + Surprise
    Disapproval,  // Sadness + Disgust
    Remorse,      // Sadness + Fear
    Aggressiveness, // Anger + Anticipation
    Contempt,     // Anger + Disgust
    
    // AI-specific emotions
    Confusion,    // When model output is inconsistent
    Curiosity,    // When encountering new patterns
    Satisfaction, // When completing tasks well
    Frustration,  // When stuck on problems
    Pride,        // When achieving mastery
    Anxiety,      // When facing uncertainty
}


/// Emotional state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalState {
    pub timestamp: DateTime<Utc>,
    pub primary_emotion: Emotion,
    pub intensity: f64,
    pub valence: f64,        // positive/negative
    pub arousal: f64,        // calm/excited
    pub secondary_emotions: Vec<(Emotion, f64)>,
    pub triggers: Vec<String>,
    pub context: String,
}


/// Emotion engine
pub struct EmotionEngine {
    current_state: EmotionalState,
    emotion_history: Vec<EmotionalState>,
    emotion_patterns: HashMap<String, Vec<Emotion>>,
    regulation_strategies: Vec<RegulationStrategy>,
}


/// Strategy for emotional regulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulationStrategy {
    pub name: String,
    pub trigger_condition: String,
    pub effectiveness: f64,
    pub applied_count: u32,
}


impl EmotionEngine {
    /// Create a new emotion engine with neutral baseline state.
    ///
    /// Note: Real implementation should load initial emotional state from
    /// the KB or a persistent profile, and initialize regulation strategies
    /// from learned patterns rather than hardcoded defaults.
    pub fn new() -> Self {
        Self {
            current_state: EmotionalState {
                timestamp: Utc::now(),
                primary_emotion: Emotion::Neutral,
                intensity: 0.5,
                valence: 0.0,
                arousal: 0.5,
                secondary_emotions: vec![],
                triggers: vec![],
                context: "initial".to_string(),
            },
            emotion_history: Vec::new(),
            emotion_patterns: HashMap::new(),
            regulation_strategies: vec![
                RegulationStrategy {
                    name: "cognitive_reappraisal".to_string(),
                    trigger_condition: "high_intensity_negative".to_string(),
                    effectiveness: 0.7,
                    applied_count: 0,
                },
                RegulationStrategy {
                    name: "attentional_deployment".to_string(),
                    trigger_condition: "prolonged_frustration".to_string(),
                    effectiveness: 0.6,
                    applied_count: 0,
                },
                RegulationStrategy {
                    name: "situation_selection".to_string(),
                    trigger_condition: "repeated_failure".to_string(),
                    effectiveness: 0.8,
                    applied_count: 0,
                },
            ],
        }
    }

    /// Process an event and update emotional state
    pub fn process_event(&mut self, event: &str, context: &str) -> EmotionalState {
        // Save current state to history
        self.emotion_history.push(self.current_state.clone());

        // Determine emotion based on event
        let (emotion, intensity, valence, arousal) = self.analyze_event(event, context);

        let secondary = self.determine_secondary_emotions(emotion.clone());

        // Create new emotional state
        let new_state = EmotionalState {
            timestamp: Utc::now(),
            primary_emotion: emotion,
            intensity,
            valence,
            arousal,
            secondary_emotions: secondary,
            triggers: vec![event.to_string()],
            context: context.to_string(),
        };

        self.current_state = new_state.clone();
        new_state
    }

    /// Analyze event to determine emotion
    ///
    /// Note: Uses keyword matching with hardcoded valence/arousal values.
    /// Real implementation should use an LLM or fine-tuned classifier for
    /// nuanced emotion detection, and calibrate values from training data.
    fn analyze_event(&self, event: &str, _context: &str) -> (Emotion, f64, f64, f64) {
        let event_lower = event.to_lowercase();

        // Success patterns
        if event_lower.contains("completed") || event_lower.contains("success") {
            (Emotion::Satisfaction, 0.8, 0.9, 0.6)
        } else if event_lower.contains("fixed") || event_lower.contains("resolved") {
            (Emotion::Joy, 0.7, 0.8, 0.5)
        } else if event_lower.contains("discovered") || event_lower.contains("new") {
            (Emotion::Curiosity, 0.7, 0.6, 0.8)
        }
        // Failure patterns
        else if event_lower.contains("failed") || event_lower.contains("error") {
            (Emotion::Frustration, 0.6, -0.7, 0.7)
        } else if event_lower.contains("stuck") || event_lower.contains("blocked") {
            (Emotion::Frustration, 0.8, -0.8, 0.8)
        } else if event_lower.contains("uncertain") || event_lower.contains("unknown") {
            (Emotion::Anxiety, 0.5, -0.5, 0.6)
        }
        // Learning patterns
        else if event_lower.contains("learned") || event_lower.contains("absorbed") {
            (Emotion::Pride, 0.6, 0.7, 0.5)
        } else if event_lower.contains("pattern") || event_lower.contains("insight") {
            (Emotion::Curiosity, 0.6, 0.5, 0.7)
        }
        // Default
        else {
            (Emotion::Neutral, 0.3, 0.0, 0.3)
        }
    }

    /// Determine secondary emotions
    ///
    /// Note: Returns hardcoded secondary emotion pairs. Real implementation
    /// should compute secondary emotions based on Plutchik's wheel of
    /// emotion (compound emotions from primary pairs) and intensity.
    fn determine_secondary_emotions(&self, primary: Emotion) -> Vec<(Emotion, f64)> {
        match primary {
            Emotion::Joy => vec![
                (Emotion::Pride, 0.4),
                (Emotion::Satisfaction, 0.6),
            ],
            Emotion::Frustration => vec![
                (Emotion::Determination, 0.5),
                (Emotion::Anxiety, 0.3),
            ],
            Emotion::Curiosity => vec![
                (Emotion::Anticipation, 0.6),
                (Emotion::Surprise, 0.4),
            ],
            _ => vec![],
        }
    }

    /// Apply emotional regulation if needed
    pub fn regulate(&mut self) -> Option<String> {
        if self.current_state.intensity > 0.7 && self.current_state.valence < 0.0 {
            // High negative emotion - try regulation
            for strategy in &mut self.regulation_strategies {
                if strategy.trigger_condition == "high_intensity_negative" {
                    strategy.applied_count += 1;
                    // Reduce intensity
                    self.current_state.intensity *= 0.7;
                    self.current_state.valence *= 0.8;
                    return Some(format!("Applied regulation: {}", strategy.name));
                }
            }
        }
        None
    }

    /// Get current emotional state
    pub fn current_state(&self) -> &EmotionalState {
        &self.current_state
    }

    /// Get emotional trajectory
    pub fn trajectory(&self, limit: usize) -> Vec<&EmotionalState> {
        self.emotion_history.iter().rev().take(limit).collect()
    }

    /// Get emotional intelligence metrics
    pub fn emotional_intelligence(&self) -> EmotionalIntelligence {
        let total_states = self.emotion_history.len() as f64;
        if total_states == 0.0 {
            return EmotionalIntelligence::default();
        }

        // Self-awareness: range of emotions experienced
        let emotion_range = self.emotion_history.iter()
            .map(|s| format!("{:?}", s.primary_emotion))
            .collect::<std::collections::HashSet<_>>()
            .len() as f64 / 8.0; // 8 basic emotions

        // Self-regulation: success of regulation attempts
        let regulation_success = self.regulation_strategies.iter()
            .map(|s| if s.applied_count > 0 { s.effectiveness } else { 0.0 })
            .sum::<f64>() / self.regulation_strategies.len() as f64;

        // Motivation: positive emotion ratio
        let positive_count = self.emotion_history.iter()
            .filter(|s| s.valence > 0.0)
            .count() as f64;
        let motivation = positive_count / total_states;

        // Empathy: understanding of context
        let empathy = self.emotion_history.iter()
            .filter(|s| !s.context.is_empty())
            .count() as f64 / total_states;

        EmotionalIntelligence {
            self_awareness: emotion_range,
            self_regulation: regulation_success,
            motivation,
            empathy,
            social_skills: (empathy + regulation_success) / 2.0,
        }
    }

    /// Get emotion distribution
    pub fn emotion_distribution(&self) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        for state in &self.emotion_history {
            let emotion = format!("{:?}", state.primary_emotion);
            *distribution.entry(emotion).or_insert(0) += 1;
        }
        distribution
    }

    /// Detect emotion from text — keyword-based heuristic mapping to EmotionLabel
    ///
    /// STUB: This is a naive keyword matcher. Real implementation needs:
    /// - LLM-based sentiment/emotion classification
    /// - Multi-language support beyond Chinese/English keywords
    /// - Context-aware detection (same word → different emotion in different contexts)
    /// - Integration with PlutchikEmotion enum for richer emotion taxonomy
    pub fn detect_from_text(&mut self, text: &str) -> crate::core::nt_core_self::emotion_state::EmotionLabel {
        use crate::core::nt_core_self::emotion_state::EmotionLabel;
        let lower = text.to_lowercase();
        if lower.contains("success") || lower.contains("完成") || lower.contains("great") {
            EmotionLabel::Joy
        } else if lower.contains("error") || lower.contains("fail") || lower.contains("错误") {
            EmotionLabel::Sadness
        } else if lower.contains("warning") || lower.contains("注意") {
            EmotionLabel::Fear
        } else if lower.contains("surprise") || lower.contains("意外") {
            EmotionLabel::Surprise
        } else {
            EmotionLabel::Neutral
        }
    }

    /// Generate a brief emotion report from current state
    pub fn report(&self) -> EmotionReport {
        EmotionReport {
            primary: format!("{:?}", self.current_state.primary_emotion),
            intensity: self.current_state.intensity,
            valence: self.current_state.valence,
            arousal: self.current_state.arousal,
        }
    }
}


pub struct EmotionReport {
    pub primary: String,
    pub intensity: f64,
    pub valence: f64,
    pub arousal: f64,
}

/// Emotional intelligence metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalIntelligence {
    pub self_awareness: f64,
    pub self_regulation: f64,
    pub motivation: f64,
    pub empathy: f64,
    pub social_skills: f64,
}


impl Default for EmotionalIntelligence {
    fn default() -> Self {
        Self {
            self_awareness: 0.0,
            self_regulation: 0.0,
            motivation: 0.0,
            empathy: 0.0,
            social_skills: 0.0,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotion_engine() {
        let mut engine = EmotionEngine::new();
        
        // Process success event
        let state = engine.process_event("Task completed successfully", "bug_fix");
        assert!(matches!(state.primary_emotion, Emotion::Satisfaction | Emotion::Joy));
        assert!(state.valence > 0.0);

        // Process failure event
        let state = engine.process_event("Task failed with error", "implementation");
        assert!(matches!(state.primary_emotion, Emotion::Frustration | Emotion::Anxiety));
        assert!(state.valence < 0.0);

        // Check trajectory
        let trajectory = engine.trajectory(10);
        assert_eq!(trajectory.len(), 2);

        // Check emotional intelligence
        let ei = engine.emotional_intelligence();
        assert!(ei.self_awareness > 0.0);
    }
}

