#![forbid(unsafe_code)]

//! Memory compressor — reduces a memory set to a target count by keeping the most
//! valuable entries: highest access count, most recent, and most novel.

use super::distiller::MemoryEntry;

/// Compresses a memory set to a target count by scoring and selecting the best entries.
///
/// Scoring formula (R-P117, R-P118):
/// - Access weight: `access_count / max_access_count`
/// - Recency weight: `(now - created_at) / max_age` (inverted: newer = higher)
/// - Novelty weight: `novelty / max_novelty` (from embedding uniqueness)
///
/// Final score: `0.4 * access + 0.3 * recency + 0.3 * novelty`
#[derive(Debug)]
pub struct MemoryCompressor {
    /// Weight for access frequency.
    pub access_weight: f64,
    /// Weight for recency.
    pub recency_weight: f64,
    /// Weight for novelty (embedding uniqueness).
    pub novelty_weight: f64,
}

impl Default for MemoryCompressor {
    fn default() -> Self {
        Self {
            access_weight: 0.4,
            recency_weight: 0.3,
            novelty_weight: 0.3,
        }
    }
}

impl MemoryCompressor {
    pub fn new(access_weight: f64, recency_weight: f64, novelty_weight: f64) -> Self {
        Self {
            access_weight,
            recency_weight,
            novelty_weight,
        }
    }

    /// Compress `memories` down to `target_count` entries, keeping the most valuable.
    /// If `target_count >= memories.len()`, returns all memories unchanged.
    pub fn compress(&self, memories: Vec<MemoryEntry>, target_count: usize) -> Vec<MemoryEntry> {
        if memories.is_empty() || target_count == 0 {
            return Vec::new();
        }
        if target_count >= memories.len() {
            return memories;
        }

        let now = current_ts();
        let max_access = memories
            .iter()
            .map(|m| m.access_count)
            .max()
            .unwrap_or(1)
            .max(1);
        let max_age = memories
            .iter()
            .map(|m| now.saturating_sub(m.created_at))
            .max()
            .unwrap_or(1)
            .max(1);
        let novelty_scores: Vec<f64> = memories
            .iter()
            .map(|m| compute_novelty(m, &memories))
            .collect();
        let max_novelty = novelty_scores
            .iter()
            .cloned()
            .fold(0.0_f64, f64::max)
            .max(1e-9);

        let mut scored: Vec<(usize, f64)> = memories
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let access_score = m.access_count as f64 / max_access as f64;
                let age = now.saturating_sub(m.created_at);
                let recency_score = 1.0 - (age as f64 / max_age as f64);
                let novelty_score = novelty_scores[i] / max_novelty;

                let score = self.access_weight * access_score
                    + self.recency_weight * recency_score
                    + self.novelty_weight * novelty_score;
                (i, score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored
            .into_iter()
            .take(target_count)
            .map(|(i, _)| memories[i].clone())
            .collect()
    }
}

/// Compute novelty of a memory: average distance to all other memories.
/// Higher = more unique (less similar to others).
fn compute_novelty(memory: &MemoryEntry, all: &[MemoryEntry]) -> f64 {
    if all.len() <= 1 {
        return 1.0;
    }
    let total_sim: f64 = all
        .iter()
        .filter(|m| m.id != memory.id)
        .map(|m| super::distiller::cosine_similarity(&memory.embedding, &m.embedding))
        .sum();
    let count = all.len().saturating_sub(1) as f64;
    if count == 0.0 {
        return 1.0;
    }
    // Novelty = 1 - average_similarity (inverted: less similar = more novel)
    (1.0 - total_sim / count).max(0.0_f64)
}

fn current_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, access: u64, created_at: i64, embedding: Vec<f64>) -> MemoryEntry {
        MemoryEntry {
            id: id.to_string(),
            content: format!("content-{id}"),
            embedding,
            access_count: access,
            created_at,
            tags: Vec::new(),
        }
    }

    #[test]
    fn test_compress_keeps_target_count() {
        let memories = vec![
            entry("a", 10, 100, vec![1.0, 0.0]),
            entry("b", 5, 200, vec![0.5, 0.5]),
            entry("c", 1, 300, vec![0.0, 1.0]),
            entry("d", 8, 400, vec![0.9, 0.1]),
        ];
        let compressor = MemoryCompressor::default();
        let result = compressor.compress(memories, 2);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_compress_empty() {
        let compressor = MemoryCompressor::default();
        let result = compressor.compress(vec![], 5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_compress_zero_target() {
        let memories = vec![entry("a", 1, 100, vec![1.0])];
        let compressor = MemoryCompressor::default();
        let result = compressor.compress(memories, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_compress_target_ge_len() {
        let memories = vec![entry("a", 1, 100, vec![1.0]), entry("b", 2, 200, vec![0.5])];
        let compressor = MemoryCompressor::default();
        let result = compressor.compress(memories.clone(), 10);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_compress_prefers_high_access() {
        let now = current_ts();
        let memories = vec![
            entry("low", 1, now - 100, vec![1.0, 0.0]),
            entry("high", 100, now - 100, vec![0.9, 0.1]),
        ];
        let compressor = MemoryCompressor::default();
        let result = compressor.compress(memories, 1);
        assert_eq!(result[0].id, "high");
    }

    #[test]
    fn test_compress_prefers_recent() {
        let now = current_ts();
        let memories = vec![
            entry("old", 5, now - 10000, vec![1.0, 0.0]),
            entry("new", 5, now - 1, vec![0.9, 0.1]),
        ];
        let compressor = MemoryCompressor::default();
        let result = compressor.compress(memories, 1);
        assert_eq!(result[0].id, "new");
    }

    #[test]
    fn test_compress_prefers_novel() {
        let now = current_ts();
        // "novel" has very different embedding from the cluster
        let memories = vec![
            entry("cluster1", 5, now - 100, vec![1.0, 0.0, 0.0]),
            entry("cluster2", 5, now - 100, vec![0.9, 0.1, 0.0]),
            entry("novel", 5, now - 100, vec![0.0, 0.0, 1.0]),
        ];
        let compressor = MemoryCompressor::default();
        let result = compressor.compress(memories, 2);
        let ids: Vec<&str> = result.iter().map(|m| m.id.as_str()).collect();
        assert!(ids.contains(&"novel"));
    }

    #[test]
    fn test_compress_custom_weights() {
        let now = current_ts();
        let memories = vec![
            entry("accessed", 100, now - 10000, vec![1.0, 0.0]),
            entry("recent", 1, now, vec![0.5, 0.5]),
        ];
        // Heavy recency weight
        let compressor = MemoryCompressor::new(0.0, 0.9, 0.1);
        let result = compressor.compress(memories, 1);
        assert_eq!(result[0].id, "recent");
    }
}
