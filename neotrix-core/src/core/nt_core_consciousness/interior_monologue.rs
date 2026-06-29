// REVIVED Task 1 — dead_code removed 2026-06-24

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum MonologueType {
    Reflective,
    Analytical,
    Creative,
    Social,
    Procedural,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmotionalState {
    pub valence: f64,
    pub arousal: f64,
    pub dominance: f64,
    pub emotions: HashMap<String, f64>,
}

impl EmotionalState {
    pub fn new(valence: f64, arousal: f64, dominance: f64) -> Self {
        Self {
            valence,
            arousal,
            dominance,
            emotions: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InteriorMonologue {
    pub monologue_type: MonologueType,
    pub emotional_state: EmotionalState,
    pub content: String,
    pub timestamp: u64,
    pub coherence: f64,
}

impl InteriorMonologue {
    pub fn new(
        monologue_type: MonologueType,
        emotional_state: EmotionalState,
        content: String,
        timestamp: u64,
    ) -> Self {
        Self {
            monologue_type,
            emotional_state,
            content,
            timestamp,
            coherence: 0.5,
        }
    }
}
