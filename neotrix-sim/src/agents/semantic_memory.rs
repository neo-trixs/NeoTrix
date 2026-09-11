use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::agents::memory_stream::{MemoryKind, MemoryNode};

/// Source of a memory fact
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemorySource {
    Observed,
    Inferred,
    Learned,
    Social,
}

/// A single semantic fact: subject-predicate-object triple
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f32,
    pub source: MemorySource,
    pub access_count: u32,
    pub last_accessed: u64,
}

/// Semantic category for organizing facts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub facts: Vec<usize>,
    pub parent: Option<String>,
}

/// Relation type between concepts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationType {
    IsA,
    HasA,
    PartOf,
    Causes,
    LocatedAt,
    SimilarTo,
    OppositeOf,
}

/// A directed relation between two concepts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub from: String,
    pub to: String,
    pub rel_type: RelationType,
    pub weight: f32,
    pub confidence: f32,
}

/// Semantic memory store: facts, categories, and relations
pub struct SemanticMemory {
    facts: Vec<Fact>,
    categories: HashMap<String, Category>,
    relations: Vec<Relation>,
    fact_index: HashMap<String, Vec<usize>>,
    max_facts: usize,
}

impl SemanticMemory {
    pub fn new(max_facts: usize) -> Self {
        Self {
            facts: Vec::new(),
            categories: HashMap::new(),
            relations: Vec::new(),
            fact_index: HashMap::new(),
            max_facts,
        }
    }

    /// Learn a new fact, merging with existing if duplicate subject-predicate-object
    pub fn learn(&mut self, fact: Fact) {
        let key = format!("{}|{}", fact.subject, fact.predicate);
        if let Some(indices) = self.fact_index.get(&key) {
            if let Some(&idx) = indices.iter().find(|&&i| self.facts[i].object == fact.object) {
                let existing = &mut self.facts[idx];
                existing.confidence = (existing.confidence + fact.confidence).min(1.0);
                existing.access_count += 1;
                existing.last_accessed = fact.last_accessed;
                return;
            }
        }

        let idx = self.facts.len();
        self.facts.push(fact);
        self.fact_index.entry(key).or_default().push(idx);
        self.prune();
    }

    /// Query all facts about a subject
    pub fn query(&self, subject: &str) -> Vec<&Fact> {
        self.facts.iter().filter(|f| f.subject == subject).collect()
    }

    /// Query facts by subject and predicate
    pub fn query_pair(&self, subject: &str, predicate: &str) -> Vec<&Fact> {
        self.facts
            .iter()
            .filter(|f| f.subject == subject && f.predicate == predicate)
            .collect()
    }

    /// Infer the object for a given subject-predicate pair
    pub fn infer(&self, subject: &str, predicate: &str) -> Option<&Fact> {
        self.facts
            .iter()
            .filter(|f| f.subject == subject && f.predicate == predicate)
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
    }

    /// Add a category
    pub fn add_category(&mut self, name: &str, parent: Option<&str>) {
        self.categories.insert(
            name.to_string(),
            Category {
                name: name.to_string(),
                facts: Vec::new(),
                parent: parent.map(|s| s.to_string()),
            },
        );
    }

    /// Categorize a fact by index
    pub fn categorize(&mut self, fact_idx: usize, category: &str) {
        if let Some(cat) = self.categories.get_mut(category) {
            if !cat.facts.contains(&fact_idx) {
                cat.facts.push(fact_idx);
            }
        }
    }

    /// Add a relation between concepts
    pub fn add_relation(&mut self, relation: Relation) {
        let exists = self.relations.iter().any(|r| {
            r.from == relation.from && r.to == relation.to && r.rel_type == relation.rel_type
        });
        if !exists {
            self.relations.push(relation);
        }
    }

    /// Get all relations from a concept
    pub fn relations_from(&self, concept: &str) -> Vec<&Relation> {
        self.relations.iter().filter(|r| r.from == concept).collect()
    }

    /// Get all relations to a concept
    pub fn relations_to(&self, concept: &str) -> Vec<&Relation> {
        self.relations.iter().filter(|r| r.to == concept).collect()
    }

    /// Consolidate episodic memories into semantic facts
    pub fn consolidate(&mut self, events: &[MemoryNode], current_tick: u64) {
        for event in events {
            if event.importance < 0.3 {
                continue;
            }
            let fact = Fact {
                subject: event.agent_id.clone(),
                predicate: format!("{:?}", event.kind),
                object: event.description.clone(),
                confidence: event.importance,
                source: match event.kind {
                    MemoryKind::Observation => MemorySource::Observed,
                    MemoryKind::Reflection => MemorySource::Inferred,
                    MemoryKind::Social => MemorySource::Social,
                    _ => MemorySource::Learned,
                },
                access_count: 0,
                last_accessed: current_tick,
            };
            self.learn(fact);
        }
    }

    /// Get all facts
    pub fn all_facts(&self) -> &[Fact] {
        &self.facts
    }

    /// Get all relations
    pub fn all_relations(&self) -> &[Relation] {
        &self.relations
    }

    /// Number of facts
    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }

    /// Number of relations
    pub fn relation_count(&self) -> usize {
        self.relations.len()
    }

    fn prune(&mut self) {
        if self.facts.len() <= self.max_facts {
            return;
        }
        self.facts.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap()
                .then(b.access_count.cmp(&a.access_count))
        });
        self.facts.truncate(self.max_facts * 8 / 10);
        self.rebuild_index();
    }

    fn rebuild_index(&mut self) {
        self.fact_index.clear();
        for (i, fact) in self.facts.iter().enumerate() {
            let key = format!("{}|{}", fact.subject, fact.predicate);
            self.fact_index.entry(key).or_default().push(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fact(subject: &str, predicate: &str, object: &str, confidence: f32) -> Fact {
        Fact {
            subject: subject.to_string(),
            predicate: predicate.to_string(),
            object: object.to_string(),
            confidence,
            source: MemorySource::Observed,
            access_count: 0,
            last_accessed: 0,
        }
    }

    #[test]
    fn learn_and_query() {
        let mut mem = SemanticMemory::new(100);
        mem.learn(make_fact("agent_0", "likes", "fish", 0.8));
        mem.learn(make_fact("agent_0", "likes", "berries", 0.6));
        let facts = mem.query("agent_0");
        assert_eq!(facts.len(), 2);
    }

    #[test]
    fn infer_returns_highest_confidence() {
        let mut mem = SemanticMemory::new(100);
        mem.learn(make_fact("agent_0", "knows", "forest", 0.5));
        mem.learn(make_fact("agent_0", "knows", "river", 0.9));
        let best = mem.infer("agent_0", "knows").unwrap();
        assert_eq!(best.object, "river");
    }

    #[test]
    fn merge_duplicate_facts() {
        let mut mem = SemanticMemory::new(100);
        mem.learn(make_fact("agent_0", "likes", "fish", 0.5));
        mem.learn(make_fact("agent_0", "likes", "fish", 0.7));
        assert_eq!(mem.fact_count(), 1);
        assert!(mem.all_facts()[0].confidence > 0.5);
    }

    #[test]
    fn relations_work() {
        let mut mem = SemanticMemory::new(100);
        mem.add_relation(Relation {
            from: "forest".to_string(),
            to: "trees".to_string(),
            rel_type: RelationType::HasA,
            weight: 0.9,
            confidence: 0.8,
        });
        assert_eq!(mem.relation_count(), 1);
        assert_eq!(mem.relations_from("forest").len(), 1);
    }

    #[test]
    fn categories_work() {
        let mut mem = SemanticMemory::new(100);
        mem.add_category("locations", None);
        mem.learn(make_fact("forest", "is_a", "location", 0.9));
        mem.categorize(0, "locations");
        assert_eq!(mem.categories["locations"].facts.len(), 1);
    }

    #[test]
    fn prune_respects_limit() {
        let mut mem = SemanticMemory::new(5);
        for i in 0..10 {
            mem.learn(make_fact("agent_0", "knows", &format!("thing_{}", i), i as f32 * 0.1));
        }
        assert!(mem.fact_count() <= 5);
    }

    #[test]
    fn consolidate_from_episodic() {
        let mut mem = SemanticMemory::new(100);
        let events = vec![
            MemoryNode {
                id: 0,
                kind: MemoryKind::Observation,
                agent_id: "agent_0".into(),
                created_tick: 0,
                last_accessed_tick: 0,
                description: "saw fire".into(),
                importance: 0.8,
                keywords: vec![],
                citations: vec![],
                embedding: None,
            },
            MemoryNode {
                id: 1,
                kind: MemoryKind::Reflection,
                agent_id: "agent_0".into(),
                created_tick: 1,
                last_accessed_tick: 1,
                description: "fire is dangerous".into(),
                importance: 0.9,
                keywords: vec![],
                citations: vec![],
                embedding: None,
            },
        ];
        mem.consolidate(&events, 10);
        assert!(mem.fact_count() >= 2);
    }
}
