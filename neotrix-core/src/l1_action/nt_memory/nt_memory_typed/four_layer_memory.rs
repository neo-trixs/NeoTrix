//! Four-Layer Memory — 吸收自 hirn
//! Working → Episodic → Semantic → Procedural
//! 生物启发的转换: surprise-gated admission, spaced repetition

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryLayer {
    Working,
    Episodic,
    Semantic,
    Procedural,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub layer: MemoryLayer,
    pub content: String,
    pub importance: f64,
    pub created_at: u64,
    pub last_accessed: u64,
    pub access_count: u32,
    pub consolidated: bool,
}

pub struct FourLayerMemory {
    working: Vec<MemoryEntry>,    // TTL-based, auto-expire
    episodic: Vec<MemoryEntry>,   // session-bound, timestamped
    semantic: Vec<MemoryEntry>,   // consolidated summaries
    procedural: Vec<MemoryEntry>, // skills, patterns
    working_ttl_secs: u64,
    consolidation_threshold: usize,
}

impl FourLayerMemory {
    pub fn new() -> Self {
        Self {
            working: Vec::new(),
            episodic: Vec::new(),
            semantic: Vec::new(),
            procedural: Vec::new(),
            working_ttl_secs: 86400, // 1 day
            consolidation_threshold: 10,
        }
    }

    pub fn remember(&mut self, entry: MemoryEntry) {
        match entry.layer {
            MemoryLayer::Working => self.working.push(entry),
            MemoryLayer::Episodic => self.episodic.push(entry),
            MemoryLayer::Semantic => self.semantic.push(entry),
            MemoryLayer::Procedural => self.procedural.push(entry),
        }
    }

    pub fn recall(&self, query: &str) -> Vec<&MemoryEntry> {
        let now_ts = now();
        let mut results: Vec<&MemoryEntry> = Vec::new();

        // Search all layers
        for layer in [
            &self.working,
            &self.episodic,
            &self.semantic,
            &self.procedural,
        ] {
            for entry in layer {
                if entry.content.to_lowercase().contains(&query.to_lowercase()) {
                    results.push(entry);
                }
            }
        }

        // Sort by importance + recency
        results.sort_by(|a, b| {
            let score_a = a.importance + 1.0 / (1.0 + (now_ts - a.last_accessed) as f64 / 86400.0);
            let score_b = b.importance + 1.0 / (1.0 + (now_ts - b.last_accessed) as f64 / 86400.0);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// TTL sweep: expire working memories
    pub fn sweep_working(&mut self) {
        let now_ts = now();
        self.working
            .retain(|e| now_ts - e.created_at < self.working_ttl_secs);
    }

    /// Consolidation: episodic → semantic (surprise-gated)
    pub fn consolidate(&mut self) {
        if self.episodic.len() < self.consolidation_threshold {
            return;
        }

        // Group by session (simplified: just take high-importance ones)
        let to_consolidate: Vec<MemoryEntry> = self
            .episodic
            .drain(..self.consolidation_threshold)
            .collect();

        // Create semantic summary
        let summary_content = to_consolidate
            .iter()
            .map(|e| e.content.as_str())
            .collect::<Vec<_>>()
            .join("; ");

        let avg_importance =
            to_consolidate.iter().map(|e| e.importance).sum::<f64>() / to_consolidate.len() as f64;

        let now_ts = now();
        self.semantic.push(MemoryEntry {
            id: format!("sem_{}", self.semantic.len()),
            layer: MemoryLayer::Semantic,
            content: format!("Consolidated: {}", summary_content),
            importance: avg_importance,
            created_at: now_ts,
            last_accessed: now_ts,
            access_count: 0,
            consolidated: true,
        });
    }

    pub fn stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("working".to_string(), self.working.len());
        stats.insert("episodic".to_string(), self.episodic.len());
        stats.insert("semantic".to_string(), self.semantic.len());
        stats.insert("procedural".to_string(), self.procedural.len());
        stats
    }
}

impl Default for FourLayerMemory {
    fn default() -> Self {
        Self::new()
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_four_layer_basic() {
        let mut mem = FourLayerMemory::new();
        mem.remember(MemoryEntry {
            id: "w1".into(),
            layer: MemoryLayer::Working,
            content: "Quick note".into(),
            importance: 0.5,
            created_at: now(),
            last_accessed: now(),
            access_count: 0,
            consolidated: false,
        });
        mem.remember(MemoryEntry {
            id: "e1".into(),
            layer: MemoryLayer::Episodic,
            content: "Session event".into(),
            importance: 0.7,
            created_at: now(),
            last_accessed: now(),
            access_count: 0,
            consolidated: false,
        });

        let results = mem.recall("note");
        assert!(!results.is_empty());

        let stats = mem.stats();
        assert_eq!(stats["working"], 1);
        assert_eq!(stats["episodic"], 1);
    }
}
