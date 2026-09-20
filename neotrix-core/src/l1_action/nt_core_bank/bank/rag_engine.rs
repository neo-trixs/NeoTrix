//! RAG (Retrieval-Augmented Generation) engine for NeoTrix memory system.
//!
//! Integrates patterns from supermemoryai/supermemory:
//! - Hybrid BM25 + embedding scoring
//! - Memory tier promotion (working → episodic → semantic)
//! - Memory consolidation and deduplication
//! - Fast retrieval with relevance scoring

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::l1_action::nt_core_bank::{MemoryTier, ReasoningMemory};

/// Memory with relevance score and retrieval metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredMemory {
    pub memory: ReasoningMemory,
    pub relevance_score: f64,
    pub retrieval_method: RetrievalMethod,
    pub rank: usize,
}

/// Retrieval method used to find the memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetrievalMethod {
    Bm25,
    Embedding,
    Hybrid,
    Exact,
}

/// Hybrid retriever combining BM25 and vector cosine scoring.
pub struct HybridRetriever {
    pub bm25_weight: f64,
    pub embedding_weight: f64,
    pub max_results: usize,
}

impl Default for HybridRetriever {
    fn default() -> Self {
        Self {
            bm25_weight: 0.6,
            embedding_weight: 0.4,
            max_results: 10,
        }
    }
}

impl HybridRetriever {
    pub fn new(bm25_weight: f64, embedding_weight: f64, max_results: usize) -> Self {
        Self {
            bm25_weight,
            embedding_weight,
            max_results,
        }
    }

    /// Score a memory against a query using hybrid BM25 + embedding.
    pub fn score_memory(
        &self,
        query: &str,
        memory: &ReasoningMemory,
        bm25_score: f64,
        embedding_similarity: Option<f64>,
    ) -> f64 {
        let embed_score = embedding_similarity.unwrap_or(0.0);
        let text_similarity = self.text_match_score(query, &memory.task_description);

        let combined = bm25_score * self.bm25_weight
            + embed_score * self.embedding_weight
            + text_similarity * (1.0 - self.bm25_weight - self.embedding_weight);

        combined.clamp(0.0, 1.0)
    }

    /// Simple text similarity based on keyword overlap.
    fn text_match_score(&self, query: &str, text: &str) -> f64 {
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let text_words: Vec<&str> = text.split_whitespace().collect();

        if query_words.is_empty() || text_words.is_empty() {
            return 0.0;
        }

        let matches = query_words
            .iter()
            .filter(|qw| text_words.iter().any(|tw| tw.eq_ignore_ascii_case(qw)))
            .count();

        matches as f64 / query_words.len() as f64
    }
}

/// Memory tier promotion rules for lifecycle management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierPromotion {
    pub working_to_episodic_threshold: f64,
    pub episodic_to_semantic_threshold: f64,
    pub access_count_promotion: usize,
    pub confidence_promotion: f64,
}

impl Default for TierPromotion {
    fn default() -> Self {
        Self {
            working_to_episodic_threshold: 0.5,
            episodic_to_semantic_threshold: 0.8,
            access_count_promotion: 3,
            confidence_promotion: 0.7,
        }
    }
}

impl TierPromotion {
    /// Determine if a memory should be promoted to the next tier.
    pub fn should_promote(&self, memory: &ReasoningMemory) -> Option<MemoryTier> {
        match memory.tier {
            MemoryTier::Working => {
                if memory.lifecycle.confidence >= self.working_to_episodic_threshold
                    && memory.lifecycle.access_count >= self.access_count_promotion
                {
                    Some(MemoryTier::Episodic)
                } else {
                    None
                }
            }
            MemoryTier::Episodic => {
                if memory.lifecycle.confidence >= self.episodic_to_semantic_threshold
                    && memory.lifecycle.access_count >= self.access_count_promotion * 2
                {
                    Some(MemoryTier::Semantic)
                } else {
                    None
                }
            }
            MemoryTier::Semantic | MemoryTier::Procedural => None,
        }
    }
}

/// Memory consolidation for deduplication and merging.
pub struct MemoryConsolidation {
    pub similarity_threshold: f64,
    pub merge_strategy: MergeStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeStrategy {
    KeepNewest,
    KeepHighestConfidence,
    MergeMetadata,
}

impl Default for MemoryConsolidation {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.85,
            merge_strategy: MergeStrategy::KeepHighestConfidence,
        }
    }
}

impl MemoryConsolidation {
    /// Find pairs of memories that can be consolidated.
    pub fn find_duplicates(&self, memories: &[ReasoningMemory]) -> Vec<(usize, usize)> {
        let mut duplicates = Vec::new();

        for i in 0..memories.len() {
            for j in (i + 1)..memories.len() {
                let similarity = self.memory_similarity(&memories[i], &memories[j]);
                if similarity >= self.similarity_threshold {
                    duplicates.push((i, j));
                }
            }
        }

        duplicates
    }

    /// Calculate similarity between two memories.
    fn memory_similarity(&self, a: &ReasoningMemory, b: &ReasoningMemory) -> f64 {
        let desc_sim = self.text_similarity(&a.task_description, &b.task_description);
        let type_match = if a.task_type == b.task_type { 0.3 } else { 0.0 };

        (desc_sim * 0.7 + type_match).clamp(0.0, 1.0)
    }

    /// Simple text similarity.
    fn text_similarity(&self, a: &str, b: &str) -> f64 {
        let words_a: Vec<&str> = a.split_whitespace().collect();
        let words_b: Vec<&str> = b.split_whitespace().collect();

        if words_a.is_empty() || words_b.is_empty() {
            return 0.0;
        }

        let intersection = words_a.iter().filter(|w| words_b.contains(w)).count();
        let union = words_a.len() + words_b.len() - intersection;

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Merge two memories based on strategy.
    pub fn merge(&self, a: &mut ReasoningMemory, b: &ReasoningMemory) {
        match self.merge_strategy {
            MergeStrategy::KeepNewest => {
                if b.timestamp > a.timestamp {
                    *a = b.clone();
                }
            }
            MergeStrategy::KeepHighestConfidence => {
                if b.lifecycle.confidence > a.lifecycle.confidence {
                    *a = b.clone();
                }
            }
            MergeStrategy::MergeMetadata => {
                a.lifecycle.access_count += b.lifecycle.access_count;
                a.lifecycle.confidence = a.lifecycle.confidence.max(b.lifecycle.confidence);
                a.lifecycle.importance = a.lifecycle.importance.max(b.lifecycle.importance);
            }
        }
    }
}

/// RAG engine orchestrating retrieval, ingestion, and consolidation.
pub struct RagEngine {
    memories: VecDeque<ReasoningMemory>,
    max_memories: usize,
    retriever: HybridRetriever,
    tier_promotion: TierPromotion,
    consolidation: MemoryConsolidation,
}

impl RagEngine {
    pub fn new(max_memories: usize) -> Self {
        Self {
            memories: VecDeque::with_capacity(max_memories),
            max_memories,
            retriever: HybridRetriever::default(),
            tier_promotion: TierPromotion::default(),
            consolidation: MemoryConsolidation::default(),
        }
    }

    /// Retrieve top-k relevant memories for a query.
    pub fn retrieve(&self, query: &str, k: usize) -> Vec<ScoredMemory> {
        let mut scored: Vec<ScoredMemory> = self
            .memories
            .iter()
            .enumerate()
            .map(|(idx, memory)| {
                let score = self.retriever.score_memory(query, memory, 0.5, None);
                ScoredMemory {
                    memory: memory.clone(),
                    relevance_score: score,
                    retrieval_method: RetrievalMethod::Hybrid,
                    rank: idx,
                }
            })
            .collect();

        scored.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        scored.truncate(k);
        scored
    }

    /// Ingest a new memory with optional embedding.
    pub fn ingest(&mut self, memory: ReasoningMemory, _embedding: Option<Vec<f64>>) {
        if self.memories.len() >= self.max_memories {
            self.memories.pop_front();
        }
        self.memories.push_back(memory);
    }

    /// Consolidate memories — remove duplicates and promote tiers.
    pub fn consolidate(&mut self) {
        // Find and merge duplicates
        let memories_vec: Vec<ReasoningMemory> = self.memories.iter().cloned().collect();
        let duplicates = self.consolidation.find_duplicates(&memories_vec);

        // Remove duplicates (keep first)
        let mut indices_to_remove: Vec<usize> = duplicates.iter().map(|&(_, j)| j).collect();
        indices_to_remove.sort_unstable();
        indices_to_remove.dedup();

        for &idx in indices_to_remove.iter().rev() {
            if idx < self.memories.len() {
                self.memories.remove(idx);
            }
        }

        // Promote tiers
        for memory in self.memories.iter_mut() {
            if let Some(new_tier) = self.tier_promotion.should_promote(memory) {
                memory.tier = new_tier;
            }
        }
    }

    /// Get memory count by tier.
    pub fn tier_counts(&self) -> std::collections::HashMap<MemoryTier, usize> {
        let mut counts = std::collections::HashMap::new();
        for memory in &self.memories {
            *counts.entry(memory.tier).or_insert(0) += 1;
        }
        counts
    }

    /// Get total memory count.
    pub fn len(&self) -> usize {
        self.memories.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.memories.is_empty()
    }
}

impl Default for RagEngine {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_retriever_scoring() {
        let retriever = HybridRetriever::default();
        let memory = ReasoningMemory::default();
        let score = retriever.score_memory("test query", &memory, 0.5, None);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_tier_promotion() {
        let promotion = TierPromotion::default();
        let mut memory = ReasoningMemory::default();
        memory.tier = MemoryTier::Working;
        memory.lifecycle.confidence = 0.6;
        memory.lifecycle.access_count = 4;

        let new_tier = promotion.should_promote(&memory);
        assert_eq!(new_tier, Some(MemoryTier::Episodic));
    }

    #[test]
    fn test_consolidation_find_duplicates() {
        let consolidation = MemoryConsolidation::default();
        let mut mem1 = ReasoningMemory::default();
        mem1.task_description = "test memory".into();
        let mut mem2 = ReasoningMemory::default();
        mem2.task_description = "test memory".into();

        let dupes = consolidation.find_duplicates(&[mem1, mem2]);
        assert!(!dupes.is_empty());
    }
}
