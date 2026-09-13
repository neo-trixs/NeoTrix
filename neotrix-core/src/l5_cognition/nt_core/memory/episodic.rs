use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Episodic memory stores what happened in a specific context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicMemory {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub context: TaskContext,
    pub action: ActionRecord,
    pub outcome: OutcomeRecord,
    pub emotion: EmotionSnapshot,
    pub salience: f64,
    pub decay_rate: f64,
    pub consolidation_count: u32,
    pub tags: Vec<String>,
}

/// What was I doing when this happened?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub task_type: String,
    pub domain: String,
    pub complexity: f64,
    pub time_pressure: f64,
    pub emotional_state: String,
    pub recent_events: Vec<String>,
}

/// What did I do?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    pub action_type: String,
    pub description: String,
    pub strategy_used: String,
    pub confidence: f64,
    pub alternatives_considered: Vec<String>,
    pub time_spent_ms: u64,
}

/// What happened as a result?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeRecord {
    pub success: bool,
    pub quality_score: f64,
    pub unexpected_outcomes: Vec<String>,
    pub side_effects: Vec<String>,
    pub lessons_learned: Vec<String>,
}

/// How did it feel?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionSnapshot {
    pub primary_emotion: String,
    pub intensity: f64,
    pub valence: f64,  // positive/negative
    pub arousal: f64,  // calm/excited
    pub secondary_emotions: Vec<String>,
}

/// Episodic memory store
pub struct EpisodicMemoryStore {
    memories: HashMap<String, EpisodicMemory>,
    index_by_time: Vec<String>,
    index_by_emotion: HashMap<String, Vec<String>>,
    index_by_domain: HashMap<String, Vec<String>>,
}

impl EpisodicMemoryStore {
    /// Create a new empty episodic memory store with empty indexes.
    pub fn new() -> Self {
        Self {
            memories: HashMap::new(),
            index_by_time: Vec::new(),
            index_by_emotion: HashMap::new(),
            index_by_domain: HashMap::new(),
        }
    }

    /// Store a new episodic memory
    pub fn store(&mut self, memory: EpisodicMemory) {
        let id = memory.id.clone();
        let emotion = memory.emotion.primary_emotion.clone();
        let domain = memory.context.domain.clone();

        // Update indexes
        self.index_by_time.push(id.clone());
        self.index_by_emotion
            .entry(emotion)
            .or_insert_with(Vec::new)
            .push(id.clone());
        self.index_by_domain
            .entry(domain)
            .or_insert_with(Vec::new)
            .push(id.clone());

        // Store memory
        self.memories.insert(id, memory);
    }

    /// Recall memories by recency
    pub fn recall_by_recency(&self, limit: usize) -> Vec<&EpisodicMemory> {
        self.index_by_time
            .iter()
            .rev()
            .take(limit)
            .filter_map(|id| self.memories.get(id))
            .collect()
    }

    /// Recall memories by emotion
    pub fn recall_by_emotion(&self, emotion: &str, limit: usize) -> Vec<&EpisodicMemory> {
        self.index_by_emotion
            .get(emotion)
            .map(|ids| {
                ids.iter()
                    .rev()
                    .take(limit)
                    .filter_map(|id| self.memories.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Recall memories by domain
    pub fn recall_by_domain(&self, domain: &str, limit: usize) -> Vec<&EpisodicMemory> {
        self.index_by_domain
            .get(domain)
            .map(|ids| {
                ids.iter()
                    .rev()
                    .take(limit)
                    .filter_map(|id| self.memories.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Recall memories by salience (most important first)
    pub fn recall_by_salience(&self, limit: usize) -> Vec<&EpisodicMemory> {
        let mut memories: Vec<&EpisodicMemory> = self.memories.values().collect();
        memories.sort_by(|a, b| b.salience.partial_cmp(&a.salience).unwrap_or(std::cmp::Ordering::Equal));
        memories.into_iter().take(limit).collect()
    }

    /// Apply decay to all memories (call periodically)
    ///
    /// Note: Decay uses exponential formula `e^(-rate * age_hours / 24)`.
    /// Real implementation should also:
    /// - Prune memories below a salience threshold to bound memory usage
    /// - Support configurable decay functions (linear, power law)
    /// - Track access frequency to modulate decay (rehearsal slows decay)
    pub fn apply_decay(&mut self) {
        let now = Utc::now();
        for memory in self.memories.values_mut() {
            let age_hours = (now - memory.timestamp).num_hours() as f64;
            let decay = (-memory.decay_rate * age_hours / 24.0).exp();
            memory.salience *= decay;
        }
    }

    /// Consolidate frequently recalled memories
    ///
    /// Note: Returns memories with `consolidation_count >= threshold` and
    /// removes them from the store. Real implementation should merge similar
    /// episodes (not just threshold-gate) and preserve index consistency
    /// for emotion and domain indexes during removal.
    pub fn consolidate(&mut self, threshold: u32) -> Vec<EpisodicMemory> {
        let mut consolidated = Vec::new();
        let mut to_remove = Vec::new();

        for (id, memory) in &self.memories {
            if memory.consolidation_count >= threshold {
                consolidated.push(memory.clone());
                to_remove.push(id.clone());
            }
        }

        for id in to_remove {
            self.memories.remove(&id);
            self.index_by_time.retain(|x| x != &id);
        }

        consolidated
    }

    /// Get memory count
    pub fn count(&self) -> usize {
        self.memories.len()
    }

    /// Get average salience
    pub fn average_salience(&self) -> f64 {
        if self.memories.is_empty() {
            return 0.0;
        }
        let total: f64 = self.memories.values().map(|m| m.salience).sum();
        total / self.memories.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_episodic_memory_store() {
        let mut store = EpisodicMemoryStore::new();
        
        let memory = EpisodicMemory {
            id: "test-1".to_string(),
            timestamp: Utc::now(),
            context: TaskContext {
                task_type: "bug_fix".to_string(),
                domain: "nt_core".to_string(),
                complexity: 0.7,
                time_pressure: 0.3,
                emotional_state: "focused".to_string(),
                recent_events: vec![],
            },
            action: ActionRecord {
                action_type: "stub_fix".to_string(),
                description: "Fixed fabricated success pattern".to_string(),
                strategy_used: "honest_error_propagation".to_string(),
                confidence: 0.9,
                alternatives_considered: vec![],
                time_spent_ms: 5000,
            },
            outcome: OutcomeRecord {
                success: true,
                quality_score: 0.85,
                unexpected_outcomes: vec![],
                side_effects: vec![],
                lessons_learned: vec!["Stub detection is systematic".to_string()],
            },
            emotion: EmotionSnapshot {
                primary_emotion: "satisfaction".to_string(),
                intensity: 0.7,
                valence: 0.8,
                arousal: 0.5,
                secondary_emotions: vec!["pride".to_string()],
            },
            salience: 0.8,
            decay_rate: 0.1,
            consolidation_count: 0,
            tags: vec!["stub_fix".to_string(), "error_handling".to_string()],
        };

        store.store(memory);
        assert_eq!(store.count(), 1);

        let recent = store.recall_by_recency(10);
        assert_eq!(recent.len(), 1);

        let satisfaction = store.recall_by_emotion("satisfaction", 10);
        assert_eq!(satisfaction.len(), 1);

        let core_memories = store.recall_by_domain("nt_core", 10);
        assert_eq!(core_memories.len(), 1);
    }
}
