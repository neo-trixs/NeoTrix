use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Emotional memory stores how things felt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalMemory {
    pub id: String,
    pub event_id: String,
    pub emotion: PlutchikEmotion,
    pub intensity: f64,
    pub valence: f64,        // positive/negative
    pub arousal: f64,        // calm/excited
    pub associated_learning: Option<String>,
    pub context: EmotionalContext,
    pub regulation_history: Vec<RegulationAttempt>,
}

/// Plutchik's wheel adapted for AI
/// 
/// NOTE: This is an extended emotion set for emotional memory analysis.
/// For the unified emotion label used across NeoTrix, see `nt_core_self::EmotionLabel`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlutchikEmotion {
    // Basic emotions (Plutchik's wheel)
    Joy,          // Satisfaction, completion
    Trust,        // Confidence, reliability
    Fear,         // Anxiety, uncertainty
    Surprise,     // Novelty, unexpected
    Sadness,      // Frustration, stuck
    Disgust,      // Rejection, bad patterns
    Anger,        // Determination, persistence
    Anticipation, // Curiosity, exploration
    
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

/// Context for the emotional experience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalContext {
    pub task_type: String,
    pub domain: String,
    pub complexity: f64,
    pub time_pressure: f64,
    pub social_context: Option<String>,
    pub previous_emotions: Vec<PlutchikEmotion>,
}

/// Attempt to regulate the emotion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulationAttempt {
    pub strategy: RegulationStrategy,
    pub effectiveness: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegulationStrategy {
    SituationSelection,    // Choose different task
    SituationModification, // Adjust task difficulty
    AttentionalDeployment, // Focus on different aspects
    CognitiveChange,       // Reappraise situation
    ResponseModulation,    // Adjust expression
}

/// Emotional memory store
pub struct EmotionalMemoryStore {
    memories: HashMap<String, EmotionalMemory>,
    index_by_emotion: HashMap<String, Vec<String>>,
    index_by_valence: HashMap<String, Vec<String>>,
    emotion_history: Vec<String>,
}

impl EmotionalMemoryStore {
    pub fn new() -> Self {
        Self {
            memories: HashMap::new(),
            index_by_emotion: HashMap::new(),
            index_by_valence: HashMap::new(),
            emotion_history: Vec::new(),
        }
    }

    /// Store a new emotional memory
    pub fn store(&mut self, memory: EmotionalMemory) {
        let id = memory.id.clone();
        let emotion = format!("{:?}", memory.emotion);
        let valence = if memory.valence >= 0.0 {
            "positive".to_string()
        } else {
            "negative".to_string()
        };

        // Update indexes
        self.index_by_emotion
            .entry(emotion)
            .or_insert_with(Vec::new)
            .push(id.clone());
        self.index_by_valence
            .entry(valence)
            .or_insert_with(Vec::new)
            .push(id.clone());
        self.emotion_history.push(id.clone());

        // Store memory
        self.memories.insert(id, memory);
    }

    /// Recall memories by emotion
    pub fn recall_by_emotion(&self, emotion: PlutchikEmotion, limit: usize) -> Vec<&EmotionalMemory> {
        let emotion_str = format!("{:?}", emotion);
        self.index_by_emotion
            .get(&emotion_str)
            .map(|ids| {
                ids.iter()
                    .rev()
                    .take(limit)
                    .filter_map(|id| self.memories.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Recall memories by valence
    pub fn recall_by_valence(&self, positive: bool, limit: usize) -> Vec<&EmotionalMemory> {
        let valence = if positive {
            "positive".to_string()
        } else {
            "negative".to_string()
        };
        self.index_by_valence
            .get(&valence)
            .map(|ids| {
                ids.iter()
                    .rev()
                    .take(limit)
                    .filter_map(|id| self.memories.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get emotional trajectory over time
    pub fn emotional_trajectory(&self, limit: usize) -> Vec<(&EmotionalMemory, DateTime<Utc>)> {
        self.emotion_history
            .iter()
            .rev()
            .take(limit)
            .filter_map(|id| {
                self.memories.get(id).map(|m| (m, m.context.previous_emotions.len() as i64))
            })
            .map(|(m, _)| (m, Utc::now()))
            .collect()
    }

    /// Calculate emotional intelligence metrics
    pub fn emotional_intelligence(&self) -> EmotionalIntelligence {
        let total_memories = self.memories.len() as f64;
        if total_memories == 0.0 {
            return EmotionalIntelligence::default();
        }

        // Self-awareness: range of emotions experienced
        let emotion_range = self.index_by_emotion.len() as f64 / 8.0; // 8 basic emotions
        
        // Self-regulation: success of regulation attempts
        let regulation_success = self.memories.values()
            .map(|m| {
                if m.regulation_history.is_empty() {
                    0.5 // No regulation attempted
                } else {
                    m.regulation_history.iter()
                        .map(|r| r.effectiveness)
                        .sum::<f64>() / m.regulation_history.len() as f64
                }
            })
            .sum::<f64>() / total_memories;

        // Motivation: positive emotion ratio
        let positive_count = self.index_by_valence.get("positive")
            .map(|v| v.len() as f64)
            .unwrap_or(0.0);
        let motivation = positive_count / total_memories;

        // Empathy: understanding of emotional context
        let empathy = self.memories.values()
            .filter(|m| m.context.social_context.is_some())
            .count() as f64 / total_memories;

        EmotionalIntelligence {
            self_awareness: emotion_range,
            self_regulation: regulation_success,
            motivation,
            empathy,
            social_skills: (empathy + regulation_success) / 2.0,
        }
    }

    /// Get memory count
    pub fn count(&self) -> usize {
        self.memories.len()
    }

    /// Get emotion distribution
    pub fn emotion_distribution(&self) -> HashMap<String, usize> {
        self.index_by_emotion.iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect()
    }
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
    fn test_emotional_memory_store() {
        let mut store = EmotionalMemoryStore::new();
        
        let memory = EmotionalMemory {
            id: "emotion-1".to_string(),
            event_id: "event-1".to_string(),
            emotion: EmotionLabel::Satisfaction,
            intensity: 0.8,
            valence: 0.9,
            arousal: 0.5,
            associated_learning: Some("lesson-1".to_string()),
            context: EmotionalContext {
                task_type: "bug_fix".to_string(),
                domain: "nt_core".to_string(),
                complexity: 0.7,
                time_pressure: 0.3,
                social_context: None,
                previous_emotions: vec![EmotionLabel::Frustration],
            },
            regulation_history: vec![],
        };

        store.store(memory);
        assert_eq!(store.count(), 1);

        let satisfaction = store.recall_by_emotion(EmotionLabel::Satisfaction, 10);
        assert_eq!(satisfaction.len(), 1);

        let positive = store.recall_by_valence(true, 10);
        assert_eq!(positive.len(), 1);
    }
}
