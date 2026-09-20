#![forbid(unsafe_code)]

//! Cross-session memory bridge — find related memories across different sessions.

use super::distiller::{cosine_similarity, MemoryEntry};

/// A memory from another session with its relevance score.
#[derive(Debug, Clone)]
pub struct RelatedMemory {
    pub memory: MemoryEntry,
    pub session_id: String,
    pub relevance: f64,
}

/// Cross-session bridge: discovers related memories across session boundaries.
#[derive(Debug)]
pub struct CrossSessionBridge {
    /// Minimum relevance threshold to include in results.
    pub min_relevance: f64,
}

impl CrossSessionBridge {
    pub fn new(min_relevance: f64) -> Self {
        Self { min_relevance }
    }

    pub fn with_defaults() -> Self {
        Self { min_relevance: 0.5 }
    }

    /// Find memories from `other_sessions` related to `memory`.
    ///
    /// Each session is a `(session_id, memories)` pair.
    /// Results are sorted by relevance (descending).
    pub fn find_related(
        &self,
        memory: &MemoryEntry,
        other_sessions: Vec<(&str, Vec<MemoryEntry>)>,
    ) -> Vec<RelatedMemory> {
        let mut results: Vec<RelatedMemory> = Vec::new();

        for (session_id, entries) in other_sessions {
            for entry in entries {
                let relevance = cosine_similarity(&memory.embedding, &entry.embedding);
                if relevance >= self.min_relevance {
                    results.push(RelatedMemory {
                        memory: entry,
                        session_id: session_id.to_string(),
                        relevance,
                    });
                }
            }
        }

        results.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    /// Find top-K related memories across sessions.
    pub fn find_top_k(
        &self,
        memory: &MemoryEntry,
        other_sessions: Vec<(&str, Vec<MemoryEntry>)>,
        k: usize,
    ) -> Vec<RelatedMemory> {
        let mut results = self.find_related(memory, other_sessions);
        results.truncate(k);
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, embedding: Vec<f64>) -> MemoryEntry {
        MemoryEntry {
            id: id.to_string(),
            content: format!("content-{id}"),
            embedding,
            access_count: 1,
            created_at: 1000,
            tags: Vec::new(),
        }
    }

    #[test]
    fn test_find_related_similar() {
        let target = entry("target", vec![1.0, 0.0, 0.0]);
        let sessions = vec![(
            "s1",
            vec![
                entry("a", vec![0.95, 0.1, 0.0]),
                entry("b", vec![0.0, 0.0, 1.0]),
            ],
        )];
        let bridge = CrossSessionBridge::with_defaults();
        let related = bridge.find_related(&target, sessions);
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].memory.id, "a");
        assert_eq!(related[0].session_id, "s1");
        assert!(related[0].relevance > 0.9);
    }

    #[test]
    fn test_find_related_none_above_threshold() {
        let target = entry("target", vec![1.0, 0.0]);
        let sessions = vec![("s1", vec![entry("a", vec![0.0, 1.0])])];
        let bridge = CrossSessionBridge::new(0.9);
        let related = bridge.find_related(&target, sessions);
        assert!(related.is_empty());
    }

    #[test]
    fn test_find_top_k() {
        let target = entry("t", vec![1.0, 0.0, 0.0]);
        let sessions = vec![(
            "s",
            vec![
                entry("a", vec![0.9, 0.1, 0.0]),
                entry("b", vec![0.8, 0.2, 0.0]),
                entry("c", vec![0.7, 0.3, 0.0]),
            ],
        )];
        let bridge = CrossSessionBridge::with_defaults();
        let top = bridge.find_top_k(&target, sessions, 2);
        assert_eq!(top.len(), 2);
        assert!(top[0].relevance >= top[1].relevance);
    }

    #[test]
    fn test_find_related_sorted_by_relevance() {
        let target = entry("t", vec![1.0, 0.0]);
        let sessions = vec![(
            "s",
            vec![
                entry("low", vec![0.5, 0.5]),
                entry("high", vec![0.99, 0.01]),
            ],
        )];
        let bridge = CrossSessionBridge::new(0.3);
        let related = bridge.find_related(&target, sessions);
        assert_eq!(related.len(), 2);
        assert_eq!(related[0].memory.id, "high");
        assert_eq!(related[1].memory.id, "low");
    }

    #[test]
    fn test_find_related_multiple_sessions() {
        let target = entry("t", vec![1.0, 0.0]);
        let sessions = vec![
            ("s1", vec![entry("a", vec![0.9, 0.1])]),
            ("s2", vec![entry("b", vec![0.8, 0.2])]),
        ];
        let bridge = CrossSessionBridge::with_defaults();
        let related = bridge.find_related(&target, sessions);
        assert_eq!(related.len(), 2);
        assert!(related.iter().any(|r| r.session_id == "s1"));
        assert!(related.iter().any(|r| r.session_id == "s2"));
    }

    #[test]
    fn test_find_related_empty_sessions() {
        let target = entry("t", vec![1.0, 0.0]);
        let bridge = CrossSessionBridge::with_defaults();
        let related = bridge.find_related(&target, vec![]);
        assert!(related.is_empty());
    }
}
