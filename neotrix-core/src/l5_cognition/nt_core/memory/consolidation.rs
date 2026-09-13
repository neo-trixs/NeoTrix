use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::episodic::{EpisodicMemory, EpisodicMemoryStore};
use super::semantic::{SemanticMemory, SemanticMemoryStore, AbstractionLevel};
use super::emotional::{EmotionalMemory, EmotionalMemoryStore, EmotionLabel};

/// Memory consolidation pipeline
pub struct MemoryConsolidation {
    episodic_store: EpisodicMemoryStore,
    semantic_store: SemanticMemoryStore,
    emotional_store: EmotionalMemoryStore,
    consolidation_config: ConsolidationConfig,
    consolidation_history: Vec<ConsolidationEvent>,
}

/// Configuration for memory consolidation
#[derive(Debug, Clone)]
pub struct ConsolidationConfig {
    pub episodic_to_semantic_threshold: u32,  // How many episodes to create a pattern
    pub pattern_to_principle_threshold: u32,  // How many patterns to create a principle
    pub principle_to_wisdom_threshold: u32,   // How many principles to create wisdom
    pub emotional_tagging_enabled: bool,
    pub decay_interval_hours: u64,
    pub max_memories_per_type: usize,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            episodic_to_semantic_threshold: 10,
            pattern_to_principle_threshold: 5,
            principle_to_wisdom_threshold: 3,
            emotional_tagging_enabled: true,
            decay_interval_hours: 24,
            max_memories_per_type: 10000,
        }
    }
}

/// Event in the consolidation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: ConsolidationType,
    pub source_count: usize,
    pub target_concept: Option<String>,
    pub abstraction_level: Option<String>,
    pub success: bool,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsolidationType {
    EpisodeToPattern,
    PatternToPrinciple,
    PrincipleToWisdom,
    EmotionalTagging,
    DecayApplication,
    MemoryPruning,
}

impl MemoryConsolidation {
    pub fn new(config: ConsolidationConfig) -> Self {
        Self {
            episodic_store: EpisodicMemoryStore::new(),
            semantic_store: SemanticMemoryStore::new(),
            emotional_store: EmotionalMemoryStore::new(),
            consolidation_config: config,
            consolidation_history: Vec::new(),
        }
    }

    /// Store a new episodic memory
    pub fn store_episode(&mut self, memory: EpisodicMemory) {
        // Apply emotional tagging if enabled
        if self.consolidation_config.emotional_tagging_enabled {
            let emotional_memory = EmotionalMemory {
                id: format!("emotional-{}", memory.id),
                event_id: memory.id.clone(),
                emotion: EmotionLabel::Satisfaction, // Default, should be computed
                intensity: memory.salience,
                valence: if memory.outcome.success { 0.8 } else { -0.5 },
                arousal: memory.context.complexity,
                associated_learning: memory.outcome.lessons_learned.first().cloned(),
                context: super::emotional::EmotionalContext {
                    task_type: memory.context.task_type.clone(),
                    domain: memory.context.domain.clone(),
                    complexity: memory.context.complexity,
                    time_pressure: memory.context.time_pressure,
                    social_context: None,
                    previous_emotions: vec![],
                },
                regulation_history: vec![],
            };
            self.emotional_store.store(emotional_memory);
        }

        self.episodic_store.store(memory);
    }

    /// Run consolidation cycle
    pub fn consolidate(&mut self) -> Vec<ConsolidationEvent> {
        let mut events = Vec::new();

        // 1. Apply decay
        self.episodic_store.apply_decay();
        events.push(ConsolidationEvent {
            timestamp: Utc::now(),
            event_type: ConsolidationType::DecayApplication,
            source_count: self.episodic_store.count(),
            target_concept: None,
            abstraction_level: None,
            success: true,
            details: "Applied decay to episodic memories".to_string(),
        });

        // 2. Consolidate frequently recalled episodes into patterns
        let consolidated_episodes = self.episodic_store
            .consolidate(self.consolidation_config.episodic_to_semantic_threshold);
        
        for episode in consolidated_episodes {
            let pattern = self.episode_to_pattern(&episode);
            let semantic_id = format!("pattern-{}", pattern.concept.name);
            let mut semantic_memory = SemanticMemory {
                id: semantic_id,
                concept: pattern,
                relations: vec![],
                abstraction_level: AbstractionLevel::Pattern,
                confidence: episode.salience,
                source_episodes: vec![episode.id],
                last_consolidation: Utc::now(),
                usage_count: 1,
            };
            self.semantic_store.store(semantic_memory);
            
            events.push(ConsolidationEvent {
                timestamp: Utc::now(),
                event_type: ConsolidationType::EpisodeToPattern,
                source_count: 1,
                target_concept: Some(episode.outcome.lessons_learned.first().cloned().unwrap_or_default()),
                abstraction_level: Some("Pattern".to_string()),
                success: true,
                details: format!("Consolidated episode into pattern: {}", episode.id),
            });
        }

        // 3. Consolidate patterns into principles
        let patterns = self.semantic_store.find_by_level(AbstractionLevel::Pattern, 100);
        if patterns.len() >= self.consolidation_config.pattern_to_principle_threshold as usize {
            let principle = self.patterns_to_principle(&patterns);
            let semantic_id = format!("principle-{}", principle.concept.name);
            let mut semantic_memory = SemanticMemory {
                id: semantic_id,
                concept: principle,
                relations: vec![],
                abstraction_level: AbstractionLevel::Principle,
                confidence: 0.9,
                source_episodes: vec![],
                last_consolidation: Utc::now(),
                usage_count: 1,
            };
            self.semantic_store.store(semantic_memory);
            
            events.push(ConsolidationEvent {
                timestamp: Utc::now(),
                event_type: ConsolidationType::PatternToPrinciple,
                source_count: patterns.len(),
                target_concept: Some("New Principle".to_string()),
                abstraction_level: Some("Principle".to_string()),
                success: true,
                details: format!("Consolidated {} patterns into principle", patterns.len()),
            });
        }

        // 4. Consolidate principles into wisdom
        let principles = self.semantic_store.find_by_level(AbstractionLevel::Principle, 100);
        if principles.len() >= self.consolidation_config.principle_to_wisdom_threshold as usize {
            let wisdom = self.principles_to_wisdom(&principles);
            let semantic_id = format!("wisdom-{}", wisdom.concept.name);
            let mut semantic_memory = SemanticMemory {
                id: semantic_id,
                concept: wisdom,
                relations: vec![],
                abstraction_level: AbstractionLevel::Wisdom,
                confidence: 0.95,
                source_episodes: vec![],
                last_consolidation: Utc::now(),
                usage_count: 1,
            };
            self.semantic_store.store(semantic_memory);
            
            events.push(ConsolidationEvent {
                timestamp: Utc::now(),
                event_type: ConsolidationType::PrincipleToWisdom,
                source_count: principles.len(),
                target_concept: Some("New Wisdom".to_string()),
                abstraction_level: Some("Wisdom".to_string()),
                success: true,
                details: format!("Consolidated {} principles into wisdom", principles.len()),
            });
        }

        self.consolidation_history.extend(events.clone());
        events
    }

    /// Convert episode to pattern
    fn episode_to_pattern(&self, episode: &EpisodicMemory) -> super::semantic::ConceptNode {
        super::semantic::ConceptNode {
            name: episode.outcome.lessons_learned.first()
                .cloned()
                .unwrap_or_else(|| "Unknown Pattern".to_string()),
            description: format!("Pattern from: {}", episode.action.description),
            domain: episode.context.domain.clone(),
            examples: vec![episode.action.description.clone()],
            counter_examples: vec![],
            prerequisites: vec![],
            related_concepts: vec![],
        }
    }

    /// Convert patterns to principle
    fn patterns_to_principle(&self, patterns: &[&SemanticMemory]) -> super::semantic::ConceptNode {
        let domain = patterns.first()
            .map(|p| p.concept.domain.clone())
            .unwrap_or_else(|| "general".to_string());
        
        super::semantic::ConceptNode {
            name: format!("{} Principle", domain),
            description: format!("Principle derived from {} patterns", patterns.len()),
            domain,
            examples: patterns.iter().map(|p| p.concept.name.clone()).collect(),
            counter_examples: vec![],
            prerequisites: vec![],
            related_concepts: patterns.iter().map(|p| p.id.clone()).collect(),
        }
    }

    /// Convert principles to wisdom
    fn principles_to_wisdom(&self, principles: &[&SemanticMemory]) -> super::semantic::ConceptNode {
        super::semantic::ConceptNode {
            name: "Core Wisdom".to_string(),
            description: format!("Wisdom derived from {} principles", principles.len()),
            domain: "meta".to_string(),
            examples: principles.iter().map(|p| p.concept.name.clone()).collect(),
            counter_examples: vec![],
            prerequisites: vec![],
            related_concepts: principles.iter().map(|p| p.id.clone()).collect(),
        }
    }

    /// Get consolidation statistics
    pub fn statistics(&self) -> ConsolidationStatistics {
        ConsolidationStatistics {
            episodic_count: self.episodic_store.count(),
            semantic_count: self.semantic_store.count(),
            emotional_count: self.emotional_store.count(),
            abstraction_distribution: self.semantic_store.abstraction_distribution(),
            emotion_distribution: self.emotional_store.emotion_distribution(),
            total_consolidations: self.consolidation_history.len(),
            recent_consolidations: self.consolidation_history.iter()
                .rev()
                .take(10)
                .cloned()
                .collect(),
        }
    }

    /// Get episodic store reference
    pub fn episodic(&self) -> &EpisodicMemoryStore {
        &self.episodic_store
    }

    /// Get semantic store reference
    pub fn semantic(&self) -> &SemanticMemoryStore {
        &self.semantic_store
    }

    /// Get emotional store reference
    pub fn emotional(&self) -> &EmotionalMemoryStore {
        &self.emotional_store
    }
}

/// Statistics about memory consolidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationStatistics {
    pub episodic_count: usize,
    pub semantic_count: usize,
    pub emotional_count: usize,
    pub abstraction_distribution: HashMap<String, usize>,
    pub emotion_distribution: HashMap<String, usize>,
    pub total_consolidations: usize,
    pub recent_consolidations: Vec<ConsolidationEvent>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::episodic::{TaskContext, ActionRecord, OutcomeRecord, EmotionSnapshot};

    #[test]
    fn test_memory_consolidation() {
        let mut consolidation = MemoryConsolidation::new(ConsolidationConfig {
            episodic_to_semantic_threshold: 2, // Lower for testing
            ..Default::default()
        });

        // Store 3 episodes
        for i in 0..3 {
            let memory = EpisodicMemory {
                id: format!("episode-{}", i),
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
                    description: format!("Fixed stub {}", i),
                    strategy_used: "honest_error".to_string(),
                    confidence: 0.9,
                    alternatives_considered: vec![],
                    time_spent_ms: 5000,
                },
                outcome: OutcomeRecord {
                    success: true,
                    quality_score: 0.85,
                    unexpected_outcomes: vec![],
                    side_effects: vec![],
                    lessons_learned: vec!["Lesson learned".to_string()],
                },
                emotion: EmotionSnapshot {
                    primary_emotion: "satisfaction".to_string(),
                    intensity: 0.7,
                    valence: 0.8,
                    arousal: 0.5,
                    secondary_emotions: vec![],
                },
                salience: 0.8,
                decay_rate: 0.1,
                consolidation_count: 3, // Above threshold
                tags: vec![],
            };
            consolidation.store_episode(memory);
        }

        // Run consolidation
        let events = consolidation.consolidate();
        
        // Should have at least decay and episode-to-pattern events
        assert!(events.len() >= 2);
        
        // Check statistics
        let stats = consolidation.statistics();
        assert!(stats.episodic_count > 0 || stats.semantic_count > 0);
    }
}
