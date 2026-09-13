use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Semantic memory stores what things mean
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemory {
    pub id: String,
    pub concept: ConceptNode,
    pub relations: Vec<ConceptRelation>,
    pub abstraction_level: AbstractionLevel,
    pub confidence: f64,
    pub source_episodes: Vec<String>,
    pub last_consolidation: DateTime<Utc>,
    pub usage_count: u32,
}

/// A concept in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptNode {
    pub name: String,
    pub description: String,
    pub domain: String,
    pub examples: Vec<String>,
    pub counter_examples: Vec<String>,
    pub prerequisites: Vec<String>,
    pub related_concepts: Vec<String>,
}

/// Relationship between concepts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptRelation {
    pub from: String,
    pub to: String,
    pub relation_type: RelationType,
    pub strength: f64,
    pub evidence_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationType {
    IsA,           // inheritance
    PartOf,        // composition
    Causes,        // causation
    Enables,       // enablement
    Contradicts,   // conflict
    SimilarTo,     // analogy
    PrerequisiteFor, // dependency
}

/// Abstraction levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbstractionLevel {
    Concrete,    // specific instance
    Pattern,     // recurring pattern
    Principle,   // general rule
    Wisdom,      // deep insight
}

/// Semantic memory store
pub struct SemanticMemoryStore {
    concepts: HashMap<String, SemanticMemory>,
    index_by_domain: HashMap<String, Vec<String>>,
    index_by_level: HashMap<String, Vec<String>>,
    relation_graph: HashMap<String, Vec<String>>,
}

impl SemanticMemoryStore {
    /// Create a new empty semantic memory store.
    pub fn new() -> Self {
        Self {
            concepts: HashMap::new(),
            index_by_domain: HashMap::new(),
            index_by_level: HashMap::new(),
            relation_graph: HashMap::new(),
        }
    }

    /// Store a new semantic memory
    pub fn store(&mut self, memory: SemanticMemory) {
        let id = memory.id.clone();
        let domain = memory.concept.domain.clone();
        let level = format!("{:?}", memory.abstraction_level);

        // Update indexes
        self.index_by_domain
            .entry(domain)
            .or_insert_with(Vec::new)
            .push(id.clone());
        self.index_by_level
            .entry(level)
            .or_insert_with(Vec::new)
            .push(id.clone());

        // Update relation graph
        for relation in &memory.relations {
            self.relation_graph
                .entry(relation.from.clone())
                .or_insert_with(Vec::new)
                .push(relation.to.clone());
        }

        // Store memory
        self.concepts.insert(id, memory);
    }

    /// Find concepts by domain
    pub fn find_by_domain(&self, domain: &str, limit: usize) -> Vec<&SemanticMemory> {
        self.index_by_domain
            .get(domain)
            .map(|ids| {
                ids.iter()
                    .take(limit)
                    .filter_map(|id| self.concepts.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find concepts by abstraction level
    pub fn find_by_level(&self, level: AbstractionLevel, limit: usize) -> Vec<&SemanticMemory> {
        let level_str = format!("{:?}", level);
        self.index_by_level
            .get(&level_str)
            .map(|ids| {
                ids.iter()
                    .take(limit)
                    .filter_map(|id| self.concepts.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find related concepts
    pub fn find_related(&self, concept_id: &str, limit: usize) -> Vec<&SemanticMemory> {
        self.relation_graph
            .get(concept_id)
            .map(|related| {
                related.iter()
                    .take(limit)
                    .filter_map(|id| self.concepts.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find concept by name
    pub fn find_by_name(&self, name: &str) -> Option<&SemanticMemory> {
        self.concepts.values().find(|m| m.concept.name == name)
    }

    /// Abstraction pipeline: episode → pattern → principle → wisdom
    ///
    /// Note: Confidence is boosted by 10% on each abstraction step. Real
    /// implementation should also update relation graph edges, propagate
    /// confidence to related concepts, and merge overlapping abstractions.
    pub fn abstract_up(&mut self, concept_id: &str, new_level: AbstractionLevel) -> bool {
        if let Some(concept) = self.concepts.get_mut(concept_id) {
            concept.abstraction_level = new_level;
            concept.confidence *= 1.1; // Abstraction increases confidence
            true
        } else {
            false
        }
    }

    /// Get concept count
    pub fn count(&self) -> usize {
        self.concepts.len()
    }

    /// Get concepts at each abstraction level
    pub fn abstraction_distribution(&self) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        for concept in self.concepts.values() {
            let level = format!("{:?}", concept.abstraction_level);
            *distribution.entry(level).or_insert(0) += 1;
        }
        distribution
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_memory_store() {
        let mut store = SemanticMemoryStore::new();
        
        let memory = SemanticMemory {
            id: "concept-1".to_string(),
            concept: ConceptNode {
                name: "Stub Detection".to_string(),
                description: "Identifying functions that return fabricated success".to_string(),
                domain: "code_quality".to_string(),
                examples: vec!["Ok(default) pattern".to_string()],
                counter_examples: vec!["Err propagation".to_string()],
                prerequisites: vec![],
                related_concepts: vec!["Error Handling".to_string()],
            },
            relations: vec![],
            abstraction_level: AbstractionLevel::Pattern,
            confidence: 0.8,
            source_episodes: vec!["episode-1".to_string()],
            last_consolidation: Utc::now(),
            usage_count: 10,
        };

        store.store(memory);
        assert_eq!(store.count(), 1);

        let code_quality = store.find_by_domain("code_quality", 10);
        assert_eq!(code_quality.len(), 1);

        let patterns = store.find_by_level(AbstractionLevel::Pattern, 10);
        assert_eq!(patterns.len(), 1);
    }
}
