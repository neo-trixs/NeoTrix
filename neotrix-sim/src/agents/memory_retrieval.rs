use std::collections::HashMap;

use crate::agents::memory_stream::{MemoryKind, MemoryNode};
use crate::feel::EmotionType;

/// Cue-based memory retrieval engine.
/// Supports text cues, embedding similarity, time range, and emotion-based retrieval.
pub struct MemoryRetriever {
    embedding_cache: HashMap<String, Vec<f32>>,
}

impl MemoryRetriever {
    pub fn new() -> Self {
        Self {
            embedding_cache: HashMap::new(),
        }
    }

    /// Cache an embedding for a given key
    pub fn cache_embedding(&mut self, key: String, embedding: Vec<f32>) {
        self.embedding_cache.insert(key, embedding);
    }

    /// Retrieve memories matching a text cue using keyword overlap
    pub fn retrieve_by_cue(
        &self,
        cue: &str,
        memories: &[MemoryNode],
        top_k: usize,
    ) -> Vec<MemoryNode> {
        let cue_lower = cue.to_lowercase();
        let cue_tokens: Vec<&str> = cue_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() >= 2)
            .collect();

        if cue_tokens.is_empty() {
            return memories.iter().take(top_k).cloned().collect();
        }

        let mut scored: Vec<(usize, f32)> = memories
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let desc_lower = m.description.to_lowercase();
                let desc_tokens: Vec<&str> = desc_lower
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|w| w.len() >= 2)
                    .collect();

                let keyword_overlap = cue_tokens
                    .iter()
                    .filter(|ct| desc_tokens.iter().any(|dt| dt == *ct))
                    .count() as f32
                    / cue_tokens.len().max(1) as f32;

                let keyword_match: f32 = m
                    .keywords
                    .iter()
                    .filter(|kw| cue_lower.contains(kw.as_str()))
                    .count() as f32
                    / m.keywords.len().max(1) as f32;

                let score = keyword_overlap * 0.6 + keyword_match * 0.3 + m.importance * 0.1;
                (i, score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .take(top_k)
            .filter_map(|(i, _)| memories.get(i).cloned())
            .collect()
    }

    /// Retrieve memories by embedding cosine similarity
    pub fn retrieve_by_similarity(
        &self,
        embedding: &[f32],
        memories: &[MemoryNode],
        top_k: usize,
    ) -> Vec<MemoryNode> {
        if embedding.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(usize, f32)> = memories
            .iter()
            .enumerate()
            .filter_map(|(i, m)| {
                m.embedding.map(|emb| {
                    let sim = cosine_similarity(embedding, &emb);
                    (i, sim)
                })
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .take(top_k)
            .filter_map(|(i, _)| memories.get(i).cloned())
            .collect()
    }

    /// Retrieve memories within a tick range
    pub fn retrieve_by_time(
        &self,
        tick_range: (u64, u64),
        memories: &[MemoryNode],
    ) -> Vec<MemoryNode> {
        memories
            .iter()
            .filter(|m| {
                m.created_tick >= tick_range.0 && m.created_tick <= tick_range.1
            })
            .cloned()
            .collect()
    }

    /// Retrieve memories by emotion label (mapped from MemoryKind)
    pub fn retrieve_by_emotion(
        &self,
        emotion: EmotionType,
        memories: &[MemoryNode],
    ) -> Vec<MemoryNode> {
        let target_kind = emotion_to_kind(emotion);
        memories
            .iter()
            .filter(|m| m.kind == target_kind)
            .cloned()
            .collect()
    }

    /// Combined retrieval: cue + recency + importance scoring
    pub fn retrieve_combined(
        &self,
        cue: &str,
        memories: &[MemoryNode],
        current_tick: u64,
        top_k: usize,
    ) -> Vec<MemoryNode> {
        let cue_lower = cue.to_lowercase();
        let cue_tokens: Vec<&str> = cue_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() >= 2)
            .collect();

        let mut scored: Vec<(usize, f32)> = memories
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let desc_lower = m.description.to_lowercase();
                let desc_tokens: Vec<&str> = desc_lower
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|w| w.len() >= 2)
                    .collect();

                let keyword_score = if cue_tokens.is_empty() {
                    0.0
                } else {
                    cue_tokens
                        .iter()
                        .filter(|ct| desc_tokens.iter().any(|dt| dt == *ct))
                        .count() as f32
                        / cue_tokens.len() as f32
                };

                let age = current_tick.saturating_sub(m.last_accessed_tick) as f32;
                let recency = (-0.001 * age).exp();

                let combined =
                    keyword_score * 3.0 + m.importance * 2.0 + recency * 0.5;
                (i, combined)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .take(top_k)
            .filter_map(|(i, _)| memories.get(i).cloned())
            .collect()
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    for i in 0..len {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    let denom = (norm_a * norm_b).sqrt();
    if denom < 1e-10 {
        0.0
    } else {
        dot / denom
    }
}

fn emotion_to_kind(emotion: EmotionType) -> MemoryKind {
    match emotion {
        EmotionType::Curiosity | EmotionType::Wonder => MemoryKind::Observation,
        EmotionType::Satisfaction | EmotionType::Pride => MemoryKind::Reflection,
        EmotionType::Trust | EmotionType::Resonance | EmotionType::Empathy => MemoryKind::Social,
        EmotionType::Frustration | EmotionType::Anxiety => MemoryKind::Thought,
        _ => MemoryKind::Observation,
    }
}

impl Default for MemoryRetriever {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mem(id: u64, desc: &str, importance: f32, tick: u64) -> MemoryNode {
        MemoryNode {
            id,
            kind: MemoryKind::Observation,
            agent_id: "agent_0".into(),
            created_tick: tick,
            last_accessed_tick: tick,
            description: desc.to_string(),
            importance,
            keywords: vec![],
            citations: vec![],
            embedding: None,
        }
    }

    #[test]
    fn retrieve_by_cue_finds_matching() {
        let retriever = MemoryRetriever::new();
        let mems = vec![
            make_mem(0, "saw a fire in the forest", 0.8, 0),
            make_mem(1, "found water near river", 0.6, 1),
            make_mem(2, "fire is dangerous", 0.9, 2),
        ];
        let results = retriever.retrieve_by_cue("fire", &mems, 10);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn retrieve_by_time_range() {
        let retriever = MemoryRetriever::new();
        let mems = vec![
            make_mem(0, "a", 0.5, 10),
            make_mem(1, "b", 0.5, 20),
            make_mem(2, "c", 0.5, 30),
        ];
        let results = retriever.retrieve_by_time((15, 25), &mems);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 1);
    }

    #[test]
    fn retrieve_by_emotion_filters() {
        let retriever = MemoryRetriever::new();
        let mems = vec![
            MemoryNode {
                id: 0,
                kind: MemoryKind::Social,
                agent_id: "a".into(),
                created_tick: 0,
                last_accessed_tick: 0,
                description: "talked".into(),
                importance: 0.5,
                keywords: vec![],
                citations: vec![],
                embedding: None,
            },
            MemoryNode {
                id: 1,
                kind: MemoryKind::Observation,
                agent_id: "a".into(),
                created_tick: 0,
                last_accessed_tick: 0,
                description: "saw".into(),
                importance: 0.5,
                keywords: vec![],
                citations: vec![],
                embedding: None,
            },
        ];
        let results = retriever.retrieve_by_emotion(EmotionType::Trust, &mems);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].kind, MemoryKind::Social);
    }

    #[test]
    fn combined_scoring_prefers_relevant() {
        let retriever = MemoryRetriever::new();
        let mems = vec![
            make_mem(0, "fire in the forest", 0.3, 0),
            make_mem(1, "fire is very dangerous and hot", 0.9, 100),
        ];
        let results = retriever.retrieve_combined("fire danger", &mems, 200, 10);
        assert!(!results.is_empty());
        assert!(results[0].description.contains("fire"));
    }

    #[test]
    fn cosine_sim_basic() {
        let a = [1.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);

        let c = [0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c)).abs() < 1e-6);
    }
}
