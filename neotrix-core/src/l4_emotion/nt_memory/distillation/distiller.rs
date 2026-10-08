#![forbid(unsafe_code)]

//! Memory distiller — merges similar memories and extracts distilled knowledge.
//!
//! Rules (R-P117, R-MEM09):
//! - Memories with cosine similarity > threshold are merged.
//! - Each merged cluster produces a `DistilledMemory` with summary, key facts, and confidence.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A memory entry used by the distillation pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f64>,
    pub access_count: u64,
    pub created_at: i64,
    pub tags: Vec<String>,
}

/// Output of distillation — a compressed representation of similar memories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistilledMemory {
    pub summary: String,
    pub key_facts: Vec<String>,
    pub source_count: usize,
    pub confidence: f64,
}

/// Configuration for the distillation process.
#[derive(Debug, Clone)]
pub struct DistillerConfig {
    /// Cosine similarity threshold above which memories are merged.
    pub similarity_threshold: f64,
    /// Maximum key facts extracted per distilled memory.
    pub max_key_facts: usize,
}

impl Default for DistillerConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.8,
            max_key_facts: 5,
        }
    }
}

/// Merges similar memories and produces distilled knowledge units.
#[derive(Debug)]
pub struct MemoryDistiller {
    config: DistillerConfig,
}

impl MemoryDistiller {
    pub fn new(config: DistillerConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(DistillerConfig::default())
    }

    /// Cluster memories by cosine similarity using single-linkage clustering.
    fn cluster_by_similarity(&self, memories: &[MemoryEntry]) -> Vec<Vec<usize>> {
        let n = memories.len();
        let mut parent: Vec<usize> = (0..n).collect();

        for i in 0..n {
            for j in (i + 1)..n {
                let sim = cosine_similarity(&memories[i].embedding, &memories[j].embedding);
                if sim > self.config.similarity_threshold {
                    let ri = find(&mut parent, i);
                    let rj = find(&mut parent, j);
                    if ri != rj {
                        parent[ri] = rj;
                    }
                }
            }
        }

        let mut groups: std::collections::HashMap<usize, Vec<usize>> =
            std::collections::HashMap::new();
        for i in 0..n {
            let root = find(&mut parent, i);
            groups.entry(root).or_default().push(i);
        }
        groups.into_values().collect()
    }

    /// Distill a cluster with access to the original entries.
    fn distill_cluster_entries(&self, entries: &[&MemoryEntry]) -> DistilledMemory {
        let source_count = entries.len();

        // Merge key facts: deduplicate across entries
        let mut all_facts: Vec<String> = entries
            .iter()
            .flat_map(|e| extract_key_facts(&e.content))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        all_facts.truncate(self.config.max_key_facts);

        // Generate summary: use the longest content as base, prefix with merge note
        let summary = if source_count == 1 {
            entries[0].content.clone()
        } else {
            let base = entries
                .iter()
                .max_by_key(|e| e.content.len())
                .map(|e| e.content.as_str())
                .unwrap_or("");
            format!("[Merged from {source_count} memories] {base}")
        };

        // Confidence: higher when more sources agree, capped at 1.0
        let confidence = (0.5 + (source_count as f64 * 0.1)).min(1.0_f64);

        DistilledMemory {
            summary,
            key_facts: all_facts,
            source_count,
            confidence,
        }
    }

    /// Internal entry point that does the full distill with access to the slice.
    pub fn distill_with_entries(&self, memories: &[MemoryEntry]) -> Vec<DistilledMemory> {
        if memories.is_empty() {
            return Vec::new();
        }

        let clusters = self.cluster_by_similarity(memories);

        clusters
            .into_iter()
            .map(|cluster| {
                let refs: Vec<&MemoryEntry> = cluster.iter().map(|&i| &memories[i]).collect();
                self.distill_cluster_entries(&refs)
            })
            .collect()
    }
}

/// Cosine similarity between two vectors. Returns 0.0 if either is zero-length.
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let mag_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

/// Extract key facts from content (simple rule-based extraction).
fn extract_key_facts(content: &str) -> Vec<String> {
    let mut facts = Vec::new();
    for sentence in content.split(['。', '！', '？', '.', '!', '?', '\n']) {
        let s = sentence.trim();
        let len = s.chars().count();
        if (4..=200).contains(&len) {
            facts.push(s.to_string());
        }
    }
    facts
}

// Union-Find helpers
fn find(parent: &mut [usize], x: usize) -> usize {
    if parent[x] != x {
        parent[x] = find(parent, parent[x]);
    }
    parent[x]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, content: &str, embedding: Vec<f64>) -> MemoryEntry {
        MemoryEntry {
            id: id.to_string(),
            content: content.to_string(),
            embedding,
            access_count: 1,
            created_at: 1000,
            tags: Vec::new(),
        }
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b)).abs() < 1e-9);
    }

    #[test]
    fn test_cosine_similarity_empty() {
        assert_eq!(cosine_similarity(&[], &[]), 0.0);
        assert_eq!(cosine_similarity(&[1.0], &[]), 0.0);
    }

    #[test]
    fn test_distill_merges_similar() {
        let memories = vec![
            entry("m1", "Rust is a systems language.", vec![1.0, 0.0, 0.0]),
            entry(
                "m2",
                "Rust is a systems programming language.",
                vec![0.95, 0.1, 0.0],
            ),
            entry("m3", "Python is a scripting language.", vec![0.0, 0.0, 1.0]),
        ];
        let distiller = MemoryDistiller::with_defaults();
        let results = distiller.distill_with_entries(&memories);
        assert_eq!(results.len(), 2, "should produce 2 clusters");
        let merged = results.iter().find(|d| d.source_count > 1).unwrap();
        assert_eq!(merged.source_count, 2);
        assert!(merged.confidence > 0.5);
    }

    #[test]
    fn test_distill_empty() {
        let distiller = MemoryDistiller::with_defaults();
        let results = distiller.distill_with_entries(&[]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_distill_single_memory() {
        let memories = vec![entry("m1", "Single fact.", vec![1.0])];
        let distiller = MemoryDistiller::with_defaults();
        let results = distiller.distill_with_entries(&memories);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source_count, 1);
    }

    #[test]
    fn test_extract_key_facts() {
        let facts = extract_key_facts("First fact. Second fact. Ok.");
        assert_eq!(facts.len(), 2);
    }

    #[test]
    fn test_distill_custom_threshold() {
        let config = DistillerConfig {
            similarity_threshold: 0.99,
            ..Default::default()
        };
        let memories = vec![
            entry("m1", "Rust is great.", vec![1.0, 0.0]),
            entry("m2", "Rust is wonderful.", vec![0.8, 0.6]),
        ];
        let distiller = MemoryDistiller::new(config);
        let results = distiller.distill_with_entries(&memories);
        assert_eq!(results.len(), 2, "low sim should not merge");
    }
}
