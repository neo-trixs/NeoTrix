#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::traits::{MemoryItem, MemoryQuery, MemoryResult, MemoryTier};

/// Tier 3 Recall — chronological conversation history with keyword indexing.
///
/// Design:
/// - Items stored in insertion order (chronological).
/// - Keyword index for fast text search (inverted index: word → item ids).
/// - Recency-weighted retrieval: recent items score higher.
/// - Bounded by max_items; oldest evicted when full.
/// - `snapshot_for_context()` returns the N most recent items for context injection.
#[derive(Debug)]
pub struct RecallStore {
    /// Items in insertion order.
    items: Vec<MemoryItem>,
    /// Keyword inverted index: lowercase word → set of item indices.
    keyword_index: HashMap<String, Vec<usize>>,
    /// Configuration.
    config: RecallConfig,
    /// Write counter.
    writes: u64,
    /// Retrieval counter.
    retrievals: u64,
}

/// Configuration for RecallStore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallConfig {
    /// Maximum number of conversation items (oldest evicted first).
    pub max_items: usize,
    /// Number of most recent items to include in context snapshot.
    pub snapshot_recent_count: usize,
    /// Recency weight for scoring (0.0-1.0).
    pub recency_weight: f64,
}

impl Default for RecallConfig {
    fn default() -> Self {
        Self {
            max_items: 1000,
            snapshot_recent_count: 20,
            recency_weight: 0.6,
        }
    }
}

impl RecallStore {
    pub fn new(config: RecallConfig) -> Self {
        Self {
            items: Vec::with_capacity(config.max_items),
            keyword_index: HashMap::new(),
            config,
            writes: 0,
            retrievals: 0,
        }
    }

    /// Append a conversation item (timestamp is auto-set).
    pub fn append(&mut self, mut item: MemoryItem) -> Result<(), String> {
        item.tier = MemoryTier::Tier3Recall;

        // Evict oldest if at capacity
        if self.items.len() >= self.config.max_items {
            self.evict_oldest(1);
        }

        let idx = self.items.len();

        // Update keyword index
        let words = extract_keywords(&item.content);
        for word in &words {
            self.keyword_index
                .entry(word.clone())
                .or_default()
                .push(idx);
        }

        self.items.push(item);
        self.writes += 1;
        Ok(())
    }

    /// Get item by id.
    pub fn get(&self, id: &str) -> Option<&MemoryItem> {
        self.items.iter().find(|i| i.id == id)
    }

    /// Get the N most recent items (for context snapshot).
    pub fn recent(&self, n: usize) -> Vec<&MemoryItem> {
        let start = self.items.len().saturating_sub(n);
        self.items[start..].iter().rev().collect()
    }

    /// Search by keywords with recency weighting.
    pub fn search(&mut self, query: &MemoryQuery) -> Vec<MemoryResult> {
        self.retrievals += 1;

        let query_words = extract_keywords(&query.text);
        if query_words.is_empty() && query.embedding.is_empty() {
            // No keywords — return most recent items
            return self.recent(query.max_results)
                .into_iter()
                .enumerate()
                .map(|(i, item)| MemoryResult {
                    item: item.clone(),
                    relevance: 1.0 - (i as f64 * 0.01),
                    source_tier: MemoryTier::Tier3Recall,
                })
                .collect();
        }

        // Score items based on keyword overlap + recency
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let max_age = self
            .items
            .iter()
            .map(|i| (now - i.timestamp).max(0))
            .max()
            .unwrap_or(1);

        // Collect candidate indices from keyword index
        let mut candidate_indices: Vec<usize> = Vec::new();
        let mut keyword_hits: HashMap<usize, usize> = HashMap::new();

        for word in &query_words {
            if let Some(indices) = self.keyword_index.get(word) {
                for &idx in indices {
                    *keyword_hits.entry(idx).or_insert(0) += 1;
                    candidate_indices.push(idx);
                }
            }
        }

        // Deduplicate
        candidate_indices.sort();
        candidate_indices.dedup();

        // Score each candidate
        let mut scored: Vec<(f64, &MemoryItem)> = candidate_indices
            .iter()
            .filter_map(|&idx| {
                if idx < self.items.len() {
                    let item = &self.items[idx];
                    if item.importance < query.min_importance {
                        return None;
                    }

                    let hits = keyword_hits.get(&idx).copied().unwrap_or(0) as f64;
                    let keyword_score = hits / query_words.len().max(1) as f64;

                    let age = (now - item.timestamp).max(0) as f64;
                    let recency_score = 1.0 - (age / max_age as f64);

                    let score = keyword_score * (1.0 - self.config.recency_weight)
                        + recency_score * self.config.recency_weight;

                    Some((score, item))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored
            .into_iter()
            .take(query.max_results)
            .map(|(relevance, item)| MemoryResult {
                item: item.clone(),
                relevance,
                source_tier: MemoryTier::Tier3Recall,
            })
            .collect()
    }

    /// Remove an item by id.
    pub fn remove(&mut self, id: &str) -> Option<MemoryItem> {
        let pos = self.items.iter().position(|i| i.id == id)?;
        let removed = self.items.remove(pos);

        // Rebuild keyword index (simpler than incremental update for correctness)
        self.rebuild_keyword_index();

        Some(removed)
    }

    /// Format recent items for context injection.
    pub fn snapshot(&self) -> String {
        let recent = self.recent(self.config.snapshot_recent_count);
        if recent.is_empty() {
            return String::new();
        }

        let mut lines = vec!["[Recall History]".to_string()];
        for item in recent.iter().rev() {
            let time_str = format_timestamp(item.timestamp);
            let truncated = if item.content.len() > 200 {
                format!("{}...", &item.content[..197])
            } else {
                item.content.clone()
            };
            lines.push(format!("  [{}] {}", time_str, truncated));
        }
        lines.join("\n")
    }

    /// Evict oldest items.
    fn evict_oldest(&mut self, n: usize) {
        for _ in 0..n {
            if !self.items.is_empty() {
                self.items.remove(0);
            }
        }
        self.rebuild_keyword_index();
    }

    /// Rebuild keyword index from scratch.
    fn rebuild_keyword_index(&mut self) {
        self.keyword_index.clear();
        for (idx, item) in self.items.iter().enumerate() {
            let words = extract_keywords(&item.content);
            for word in words {
                self.keyword_index.entry(word).or_default().push(idx);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn stats(&self) -> RecallStats {
        RecallStats {
            item_count: self.items.len(),
            keyword_index_size: self.keyword_index.len(),
            writes: self.writes,
            retrievals: self.retrievals,
        }
    }
}

/// Extract keywords from text for indexing.
///
/// Lowercases, splits on whitespace, filters out very short words.
fn extract_keywords(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| w.len() >= 2)
        .map(String::from)
        .collect()
}

/// Format a Unix timestamp as a short time string.
fn format_timestamp(ts: i64) -> String {
    // Simple YYYY-MM-DD HH:MM format without chrono dependency
    let days = ts / 86400;
    let secs_in_day = ts % 86400;
    let hours = secs_in_day / 3600;
    let minutes = (secs_in_day % 3600) / 60;

    // Approximate date from epoch days (1970-01-01 is day 0)
    let year = 1970 + days / 365;
    let day_of_year = days % 365;
    let month = (day_of_year / 30 + 1).min(12);
    let day = (day_of_year % 30 + 1).min(31);

    format!("{:04}-{:02}-{:02} {:02}:{:02}", year, month, day, hours, minutes)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecallStats {
    pub item_count: usize,
    pub keyword_index_size: usize,
    pub writes: u64,
    pub retrievals: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(id: &str, content: &str) -> MemoryItem {
        MemoryItem::new(
            id.to_string(),
            content.to_string(),
            MemoryTier::Tier3Recall,
            0.5,
            "test".to_string(),
        )
    }

    #[test]
    fn test_append_and_get() {
        let mut store = RecallStore::new(RecallConfig::default());
        store.append(make_item("m1", "hello world")).unwrap();
        assert_eq!(store.len(), 1);
        assert!(store.get("m1").is_some());
    }

    #[test]
    fn test_recent_items() {
        let mut store = RecallStore::new(RecallConfig::default());
        store.append(make_item("m1", "first")).unwrap();
        store.append(make_item("m2", "second")).unwrap();
        store.append(make_item("m3", "third")).unwrap();

        let recent = store.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].id, "m3"); // most recent first
        assert_eq!(recent[1].id, "m2");
    }

    #[test]
    fn test_capacity_eviction() {
        let config = RecallConfig {
            max_items: 3,
            ..Default::default()
        };
        let mut store = RecallStore::new(config);
        store.append(make_item("m1", "a")).unwrap();
        store.append(make_item("m2", "b")).unwrap();
        store.append(make_item("m3", "c")).unwrap();
        store.append(make_item("m4", "d")).unwrap();

        assert_eq!(store.len(), 3);
        assert!(store.get("m1").is_none()); // oldest evicted
        assert!(store.get("m4").is_some());
    }

    #[test]
    fn test_keyword_search() {
        let mut store = RecallStore::new(RecallConfig::default());
        store
            .append(make_item("m1", "Rust memory safety"))
            .unwrap();
        store
            .append(make_item("m2", "Python web framework"))
            .unwrap();

        let query = MemoryQuery {
            text: "memory safety".to_string(),
            embedding: Vec::new(),
            tier_filter: vec![MemoryTier::Tier3Recall],
            max_results: 5,
            min_importance: 0.0,
            prefer_concise: false,
        };

        let results = store.search(&query);
        assert!(!results.is_empty());
        assert_eq!(results[0].item.id, "m1");
    }

    #[test]
    fn test_extract_keywords() {
        let keywords = extract_keywords("Hello, World! This is a test.");
        assert!(keywords.contains(&"hello".to_string()));
        assert!(keywords.contains(&"world".to_string()));
        assert!(keywords.contains(&"test".to_string()));
        assert!(!keywords.contains(&"a".to_string())); // too short
    }

    #[test]
    fn test_snapshot_format() {
        let mut store = RecallStore::new(RecallConfig {
            snapshot_recent_count: 5,
            ..Default::default()
        });
        store.append(make_item("m1", "first message")).unwrap();
        store.append(make_item("m2", "second message")).unwrap();

        let snap = store.snapshot();
        assert!(snap.starts_with("[Recall History]"));
        assert!(snap.contains("first message"));
    }

    #[test]
    fn test_remove() {
        let mut store = RecallStore::new(RecallConfig::default());
        store.append(make_item("m1", "hello")).unwrap();
        store.append(make_item("m2", "world")).unwrap();

        let removed = store.remove("m1");
        assert!(removed.is_some());
        assert_eq!(store.len(), 1);
        assert!(store.get("m1").is_none());
    }

    #[test]
    fn test_empty_query_returns_recent() {
        let mut store = RecallStore::new(RecallConfig::default());
        store.append(make_item("m1", "a")).unwrap();
        store.append(make_item("m2", "b")).unwrap();

        let query = MemoryQuery {
            text: String::new(),
            embedding: Vec::new(),
            tier_filter: vec![MemoryTier::Tier3Recall],
            max_results: 5,
            min_importance: 0.0,
            prefer_concise: false,
        };

        let results = store.search(&query);
        assert_eq!(results.len(), 2);
    }
}
