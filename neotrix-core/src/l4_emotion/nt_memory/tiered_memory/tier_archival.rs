#![forbid(unsafe_code)]

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::traits::{MemoryItem, MemoryQuery, MemoryResult, MemoryTier};

/// Tier 2 Archival — vector-backed long-term storage.
///
/// Design:
/// - Items stored with embedding vectors for similarity search.
/// - Retrieval via cosine similarity on embeddings.
/// - Fallback to keyword match when embeddings are unavailable.
/// - Items persisted to SQLite (cold) + in-memory index (hot).
/// - No capacity limit (bounded by disk), but items below importance threshold
///   are pruned during maintenance.
#[derive(Debug)]
pub struct ArchivalStore {
    /// In-memory items indexed by id.
    items: HashMap<String, MemoryItem>,
    /// SQLite connection for persistence.
    conn: Option<rusqlite::Connection>,
    /// Configuration.
    config: ArchivalConfig,
    /// Write counter for stats.
    writes: u64,
    /// Retrieval counter for stats.
    retrievals: u64,
}

/// Configuration for ArchivalStore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivalConfig {
    /// Minimum importance for archival storage (below this, items are skipped).
    pub min_importance: f64,
    /// Default top-K for similarity search.
    pub default_top_k: usize,
    /// Keyword search boost weight (0.0-1.0).
    pub keyword_boost: f64,
}

impl Default for ArchivalConfig {
    fn default() -> Self {
        Self {
            min_importance: 0.3,
            default_top_k: 10,
            keyword_boost: 0.2,
        }
    }
}

const ARCHIVAL_SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS archival_memory (
        id TEXT PRIMARY KEY,
        content TEXT NOT NULL,
        importance REAL NOT NULL,
        confidence REAL NOT NULL,
        timestamp INTEGER NOT NULL,
        access_count INTEGER NOT NULL DEFAULT 0,
        tags TEXT NOT NULL DEFAULT '[]',
        source TEXT NOT NULL DEFAULT '',
        embedding BLOB DEFAULT NULL,
        created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
    );
    CREATE INDEX IF NOT EXISTS idx_archival_importance ON archival_memory(importance);
    CREATE INDEX IF NOT EXISTS idx_archival_timestamp ON archival_memory(timestamp);
    CREATE VIRTUAL TABLE IF NOT EXISTS archival_fts USING fts5(content, tags, source, content_rowid=rowid);
";

impl ArchivalStore {
    pub fn new(conn: rusqlite::Connection, config: ArchivalConfig) -> Result<Self, String> {
        conn.execute_batch(ARCHIVAL_SCHEMA)
            .map_err(|e| e.to_string())?;

        let mut store = Self {
            items: HashMap::new(),
            conn: Some(conn),
            config,
            writes: 0,
            retrievals: 0,
        };

        store.load_from_db()?;
        Ok(store)
    }

    /// In-memory only (no persistence). Useful for tests.
    pub fn in_memory(config: ArchivalConfig) -> Self {
        Self {
            items: HashMap::new(),
            conn: None,
            config,
            writes: 0,
            retrievals: 0,
        }
    }

    /// Load all items from SQLite into memory index.
    fn load_from_db(&mut self) -> Result<(), String> {
        let conn = match &self.conn {
            Some(c) => c,
            None => return Ok(()),
        };

        let mut stmt = conn
            .prepare(
                "SELECT id, content, importance, confidence, timestamp, access_count, tags, source \
                 FROM archival_memory",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let tags_str: String = row.get(6).unwrap_or_else(|_| "[]".to_string());
                let tags: Vec<String> =
                    serde_json::from_str(&tags_str).unwrap_or_default();
                Ok(MemoryItem {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    tier: MemoryTier::Tier2Archival,
                    importance: row.get(2)?,
                    confidence: row.get(3)?,
                    timestamp: row.get(4)?,
                    access_count: row.get(5).unwrap_or(0),
                    tags,
                    source: row.get(7).unwrap_or_default(),
                    embedding: Vec::new(),
                })
            })
            .map_err(|e| e.to_string())?;

        for row in rows.flatten() {
            self.items.insert(row.id.clone(), row);
        }
        Ok(())
    }

    /// Insert or update an archival item.
    pub fn insert(&mut self, mut item: MemoryItem) -> Result<(), String> {
        // Skip items below importance threshold
        if item.importance < self.config.min_importance {
            return Ok(());
        }

        item.tier = MemoryTier::Tier2Archival;
        let tags_json =
            serde_json::to_string(&item.tags).unwrap_or_else(|_| "[]".to_string());
        let embedding_bytes = serialize_embedding(&item.embedding);

        // Upsert to SQLite
        if let Some(conn) = &self.conn {
            conn.execute(
                "INSERT OR REPLACE INTO archival_memory \
                 (id, content, importance, confidence, timestamp, access_count, tags, source, embedding) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    item.id,
                    item.content,
                    item.importance,
                    item.confidence,
                    item.timestamp,
                    item.access_count,
                    tags_json,
                    item.source,
                    embedding_bytes,
                ],
            )
            .map_err(|e| e.to_string())?;
        }

        self.items.insert(item.id.clone(), item);
        self.writes += 1;
        Ok(())
    }

    /// Remove an item.
    pub fn remove(&mut self, id: &str) -> Result<bool, String> {
        if let Some(conn) = &self.conn {
            let count = conn
                .execute("DELETE FROM archival_memory WHERE id=?1", rusqlite::params![id])
                .map_err(|e| e.to_string())?;
            Ok(count > 0)
        } else {
            Ok(self.items.remove(id).is_some())
        }
    }

    /// Retrieve by cosine similarity of embeddings.
    ///
    /// If query has no embedding, falls back to keyword matching.
    pub fn search(&mut self, query: &MemoryQuery) -> Vec<MemoryResult> {
        self.retrievals += 1;

        if !query.embedding.is_empty() {
            self.search_by_embedding(query)
        } else {
            self.search_by_keyword(query)
        }
    }

    /// Cosine similarity search on embeddings.
    fn search_by_embedding(&self, query: &MemoryQuery) -> Vec<MemoryResult> {
        let mut scored: Vec<(f64, &MemoryItem)> = self
            .items
            .values()
            .filter(|item| {
                !item.embedding.is_empty()
                    && item.importance >= query.min_importance
                    && query
                        .tier_filter
                        .contains(&MemoryTier::Tier2Archival)
            })
            .map(|item| {
                let sim = cosine_similarity(&query.embedding, &item.embedding);
                (sim, item)
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored
            .into_iter()
            .take(query.max_results)
            .map(|(relevance, item)| MemoryResult {
                item: item.clone(),
                relevance,
                source_tier: MemoryTier::Tier2Archival,
            })
            .collect()
    }

    /// Keyword fallback search (BM25-lite: simple TF scoring).
    fn search_by_keyword(&self, query: &MemoryQuery) -> Vec<MemoryResult> {
        let query_lower = query.text.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        if query_words.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(f64, &MemoryItem)> = self
            .items
            .values()
            .filter(|item| {
                item.importance >= query.min_importance
                    && query
                        .tier_filter
                        .contains(&MemoryTier::Tier2Archival)
            })
            .map(|item| {
                let content_lower = item.content.to_lowercase();
                let tag_text = item.tags.join(" ").to_lowercase();
                let combined = format!("{} {}", content_lower, tag_text);

                let tf_score: f64 = query_words
                    .iter()
                    .map(|w| {
                        let count = combined.matches(w).count() as f64;
                        count / (1.0 + count) // TF normalization
                    })
                    .sum::<f64>()
                    / query_words.len() as f64;

                let keyword_boost = if query.prefer_concise {
                    self.config.keyword_boost
                } else {
                    self.config.keyword_boost * 0.5
                };
                let final_score = tf_score * (1.0 - keyword_boost)
                    + item.importance * keyword_boost;

                (final_score, item)
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored
            .into_iter()
            .take(query.max_results)
            .map(|(relevance, item)| MemoryResult {
                item: item.clone(),
                relevance,
                source_tier: MemoryTier::Tier2Archival,
            })
            .collect()
    }

    /// Get item by id.
    pub fn get(&self, id: &str) -> Option<&MemoryItem> {
        self.items.get(id)
    }

    /// Prune items below importance threshold.
    pub fn prune(&mut self, threshold: f64) -> usize {
        let to_remove: Vec<String> = self
            .items
            .values()
            .filter(|item| item.importance < threshold)
            .map(|item| item.id.clone())
            .collect();

        // 2026-09-30: 原为 `let _ = self.remove(id);` 循环完返回 `to_remove.len()`。
        // `remove` 走 SQLite（SQLITE_BUSY / 只读 / 磁盘满均可失败），失败被吞，
        // 而返回值是**候选数**不是**删除数** ⇒ 「已清理 N 项」在 0 项被删时同样成立。
        // 附带：该文件的 `#[cfg(test)]` 用 `ArchivalStore::in_memory`，
        // **恰好只覆盖了错误不可能发生的那条分支**（in_memory 的 remove 不返回 Err），
        // 这就是 12,209 测试全绿却抓不到它的原因。
        // ⇒ 按实际删除数计数，并对失败给出可观测通道。
        let mut removed = 0usize;
        for id in &to_remove {
            match self.remove(id) {
                Ok(true) => removed += 1,
                Ok(false) => {}
                Err(e) => log::warn!("[archival] 清理失败 {id}: {e}"),
            }
        }
        removed
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn stats(&self) -> ArchivalStats {
        ArchivalStats {
            item_count: self.items.len(),
            total_embeddings: self.items.values().filter(|i| !i.embedding.is_empty()).count(),
            writes: self.writes,
            retrievals: self.retrievals,
        }
    }
}

/// Cosine similarity between two vectors.
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

/// Serialize embedding vector to bytes for SQLite storage.
fn serialize_embedding(embedding: &[f64]) -> Option<Vec<u8>> {
    if embedding.is_empty() {
        return None;
    }
    let bytes: Vec<u8> = embedding
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect();
    Some(bytes)
}

/// Deserialize embedding vector from SQLite bytes.
pub fn deserialize_embedding(bytes: &[u8]) -> Vec<f64> {
    bytes
        .chunks_exact(8)
        .map(|chunk| {
            let arr: [u8; 8] = [chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7]];
            f64::from_le_bytes(arr)
        })
        .collect()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArchivalStats {
    pub item_count: usize,
    pub total_embeddings: usize,
    pub writes: u64,
    pub retrievals: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(id: &str, content: &str, importance: f64) -> MemoryItem {
        MemoryItem::new(
            id.to_string(),
            content.to_string(),
            MemoryTier::Tier2Archival,
            importance,
            "test".to_string(),
        )
    }

    #[test]
    fn test_insert_and_get() {
        let mut store = ArchivalStore::in_memory(ArchivalConfig::default());
        let item = make_item("doc-1", "Rust is a systems language", 0.8);
        store.insert(item).unwrap();
        assert_eq!(store.len(), 1);
        assert!(store.get("doc-1").is_some());
    }

    #[test]
    fn test_importance_threshold() {
        let config = ArchivalConfig {
            min_importance: 0.5,
            ..Default::default()
        };
        let mut store = ArchivalStore::in_memory(config);
        store.insert(make_item("low", "trivial", 0.2)).unwrap();
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_keyword_search() {
        let mut store = ArchivalStore::in_memory(ArchivalConfig::default());
        store
            .insert(make_item(
                "doc-1",
                "Rust memory safety guarantees",
                0.8,
            ))
            .unwrap();
        store
            .insert(make_item(
                "doc-2",
                "Python is a scripting language",
                0.7,
            ))
            .unwrap();

        let query = MemoryQuery {
            text: "memory safety".to_string(),
            embedding: Vec::new(),
            tier_filter: vec![MemoryTier::Tier2Archival],
            max_results: 5,
            min_importance: 0.0,
            prefer_concise: false,
        };

        let results = store.search(&query);
        assert!(!results.is_empty());
        assert_eq!(results[0].item.id, "doc-1");
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c)).abs() < 1e-6);
    }

    #[test]
    fn test_prune() {
        let mut store = ArchivalStore::in_memory(ArchivalConfig::default());
        store.insert(make_item("a", "a", 0.9)).unwrap();
        store.insert(make_item("b", "b", 0.1)).unwrap();
        let pruned = store.prune(0.5);
        assert_eq!(pruned, 1);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut store = ArchivalStore::in_memory(ArchivalConfig::default());
        store.insert(make_item("a", "a", 0.5)).unwrap();
        let removed = store.remove("a").unwrap();
        assert!(removed);
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_embedding_search() {
        let mut store = ArchivalStore::in_memory(ArchivalConfig::default());

        let mut item1 = make_item("a", "hello world", 0.8);
        item1.embedding = vec![1.0, 0.0, 0.0];
        store.insert(item1).unwrap();

        let mut item2 = make_item("b", "goodbye world", 0.7);
        item2.embedding = vec![0.0, 1.0, 0.0];
        store.insert(item2).unwrap();

        let query = MemoryQuery {
            text: String::new(),
            embedding: vec![0.9, 0.1, 0.0],
            tier_filter: vec![MemoryTier::Tier2Archival],
            max_results: 5,
            min_importance: 0.0,
            prefer_concise: false,
        };

        let results = store.search(&query);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].item.id, "a"); // cosine with a is higher
    }
}
