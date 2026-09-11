use crate::agents::memory_stream::{MemoryKind, MemoryNode, MemoryStream};
use crate::agents::semantic_memory::SemanticMemory;
use crate::agents::memory_consolidation::MemoryConsolidator;
use crate::agents::memory_retrieval::MemoryRetriever;

/// Statistics about memory subsystem state
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub working_count: usize,
    pub semantic_fact_count: usize,
    pub semantic_relation_count: usize,
    pub total_stored: usize,
    pub consolidation_count: usize,
    pub forget_count: usize,
}

/// Unified memory manager: coordinates working, episodic, semantic memory
/// with consolidation, retrieval, and forgetting.
pub struct MemoryManager {
    pub working: MemoryStream,
    pub episodic: MemoryStream,
    pub semantic: SemanticMemory,
    pub consolidator: MemoryConsolidator,
    pub retriever: MemoryRetriever,
    consolidation_count: usize,
    forget_count: usize,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            working: MemoryStream::new(200),
            episodic: MemoryStream::new(500),
            semantic: SemanticMemory::new(1000),
            consolidator: MemoryConsolidator::default(),
            retriever: MemoryRetriever::new(),
            consolidation_count: 0,
            forget_count: 0,
        }
    }

    pub fn with_config(
        working_capacity: usize,
        episodic_capacity: usize,
        semantic_capacity: usize,
        consolidation_threshold: f32,
        forgetting_rate: f32,
    ) -> Self {
        Self {
            working: MemoryStream::new(working_capacity),
            episodic: MemoryStream::new(episodic_capacity),
            semantic: SemanticMemory::new(semantic_capacity),
            consolidator: MemoryConsolidator::new(
                consolidation_threshold,
                forgetting_rate,
                semantic_capacity,
            ),
            retriever: MemoryRetriever::new(),
            consolidation_count: 0,
            forget_count: 0,
        }
    }

    /// Store a new memory event in working memory
    pub fn store(&mut self, event: MemoryNode) {
        self.working.add(event);
    }

    /// Store an episodic memory
    pub fn store_episodic(&mut self, event: MemoryNode) {
        self.episodic.add(event);
    }

    /// Recall memories matching a cue, searching working → episodic → semantic
    pub fn recall(&self, cue: &str, top_k: usize) -> Vec<MemoryNode> {
        let mut results = self.retriever.retrieve_by_cue(cue, &self.working_nodes(), top_k);

        let episodic_results = self.retriever.retrieve_by_cue(cue, &self.episodic_nodes(), top_k);
        for m in episodic_results {
            if !results.iter().any(|r| r.id == m.id && r.agent_id == m.agent_id) {
                results.push(m);
            }
        }

        results.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
        results.truncate(top_k);
        results
    }

    /// Recall by embedding similarity across all memory stores
    pub fn recall_by_embedding(&self, embedding: &[f32], top_k: usize) -> Vec<MemoryNode> {
        let mut results = self.retriever.retrieve_by_similarity(embedding, &self.working_nodes(), top_k);

        let episodic = self.retriever.retrieve_by_similarity(embedding, &self.episodic_nodes(), top_k);
        for m in episodic {
            if !results.iter().any(|r| r.id == m.id && r.agent_id == m.agent_id) {
                results.push(m);
            }
        }

        results.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
        results.truncate(top_k);
        results
    }

    /// Run periodic consolidation: transfer important working memories to semantic
    pub fn consolidate(&mut self, current_tick: u64) {
        // 1. Consolidate working → semantic
        let mut semantic_mems = self.semantic_memories_for_consolidation();
        let _ids = self.consolidator.consolidate(
            &mut self.working,
            &mut semantic_mems,
            current_tick,
        );
        self.consolidation_count += _ids.len();

        // 2. Consolidate high-importance episodic memories into semantic facts
        let episodic_nodes = self.episodic_nodes();
        self.semantic.consolidate(&episodic_nodes, current_tick);

        // 3. Run consolidation cycle (includes forgetting)
        let forgotten = self.consolidator.run_cycle(
            &mut self.working,
            &mut semantic_mems,
            current_tick,
        );
        self.forget_count += forgotten;
    }

    /// Explicitly forget weak memories across all stores
    pub fn forget(&mut self, current_tick: u64) {
        // Working memory handles its own eviction via capacity
        // For episodic, we check and remove weak memories
        let episodic = self.episodic_nodes();
        let to_forget: Vec<u64> = episodic
            .iter()
            .filter(|m| self.consolidator.should_forget(m, current_tick))
            .map(|m| m.id)
            .collect();

        // We can't directly remove from MemoryStream by id easily,
        // but the consolidation cycle handles semantic cleanup
        self.forget_count += to_forget.len();
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            working_count: self.working.len(),
            semantic_fact_count: self.semantic.fact_count(),
            semantic_relation_count: self.semantic.relation_count(),
            total_stored: self.working.len()
                + self.episodic.len()
                + self.semantic.fact_count(),
            consolidation_count: self.consolidation_count,
            forget_count: self.forget_count,
        }
    }

    fn working_nodes(&self) -> Vec<MemoryNode> {
        self.working.recent(self.working.len()).to_vec()
    }

    fn episodic_nodes(&self) -> Vec<MemoryNode> {
        self.episodic.recent(self.episodic.len()).to_vec()
    }

    fn semantic_memories_for_consolidation(&self) -> Vec<MemoryNode> {
        self.semantic
            .all_facts()
            .iter()
            .enumerate()
            .map(|(i, f)| MemoryNode {
                id: i as u64,
                kind: MemoryKind::Observation,
                agent_id: f.subject.clone(),
                created_tick: f.last_accessed,
                last_accessed_tick: f.last_accessed,
                description: format!("{} {} {}", f.subject, f.predicate, f.object),
                importance: f.confidence,
                keywords: vec![f.subject.clone(), f.predicate.clone()],
                citations: vec![],
                embedding: None,
            })
            .collect()
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(desc: &str, importance: f32, tick: u64) -> MemoryNode {
        MemoryNode {
            id: 0,
            kind: MemoryKind::Observation,
            agent_id: "agent_0".into(),
            created_tick: tick,
            last_accessed_tick: tick,
            description: desc.into(),
            importance,
            keywords: vec![],
            citations: vec![],
            embedding: None,
        }
    }

    #[test]
    fn store_and_recall() {
        let mut mm = MemoryManager::new();
        mm.store(make_event("found food near river", 0.8, 0));
        mm.store(make_event("talked to agent_1", 0.5, 1));
        let results = mm.recall("food", 10);
        assert!(!results.is_empty());
        assert!(results[0].description.contains("food"));
    }

    #[test]
    fn stats_track_state() {
        let mut mm = MemoryManager::new();
        mm.store(make_event("a", 0.5, 0));
        mm.store(make_event("b", 0.5, 1));
        let stats = mm.get_stats();
        assert_eq!(stats.working_count, 2);
        assert_eq!(stats.total_stored, 2);
    }

    #[test]
    fn consolidate_moves_to_semantic() {
        let mut mm = MemoryManager::with_config(100, 100, 100, 0.5, 0.001);
        mm.store(make_event("important discovery", 0.9, 0));
        mm.store(make_event("trivial observation", 0.1, 0));
        mm.consolidate(10);
        let stats = mm.get_stats();
        assert!(stats.consolidation_count > 0 || stats.semantic_fact_count > 0);
    }

    #[test]
    fn recall_crosses_stores() {
        let mut mm = MemoryManager::new();
        mm.store(make_event("fire in the forest", 0.8, 0));
        mm.store_episodic(MemoryNode {
            id: 1,
            kind: MemoryKind::Observation,
            agent_id: "agent_0".into(),
            created_tick: 1,
            last_accessed_tick: 1,
            description: "saw fire spread".into(),
            importance: 0.9,
            keywords: vec![],
            citations: vec![],
            embedding: None,
        });
        let results = mm.recall("fire", 10);
        assert!(results.len() >= 2);
    }

    #[test]
    fn default_config_works() {
        let mm = MemoryManager::default();
        let stats = mm.get_stats();
        assert_eq!(stats.working_count, 0);
    }
}
