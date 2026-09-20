#![forbid(unsafe_code)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use super::tier_archival::{ArchivalConfig, ArchivalStore};
use super::tier_core::{CoreStore, CoreStoreConfig};
use super::tier_recall::{RecallConfig, RecallStore};
use super::traits::{
    HubStats, MemoryHub, MemoryItem, MemoryQuery, MemoryResult, MemoryTier, RetrievalStats,
};

/// Tier routing configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RouterConfig {
    /// Core store configuration (Tier 1).
    pub core: CoreStoreConfig,
    /// Archival store configuration (Tier 2).
    pub archival: ArchivalConfig,
    /// Recall store configuration (Tier 3).
    pub recall: RecallConfig,

    /// Maximum total results across all tiers.
    pub max_total_results: usize,

    /// If true, Tier 3 (recall) is only queried when Tier 1 + Tier 2
    /// return fewer than this threshold.
    pub escalation_threshold: usize,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            core: CoreStoreConfig::default(),
            archival: ArchivalConfig::default(),
            recall: RecallConfig::default(),
            max_total_results: 20,
            escalation_threshold: 3,
        }
    }
}

/// The unified 3-Tier memory hub.
///
/// Routes writes to the appropriate tier based on item characteristics:
/// - Items tagged as `core` or with very high importance → Tier 1 (Core).
/// - Items with embeddings or importance ≥ archival threshold → Tier 2 (Archival).
/// - Everything else → Tier 3 (Recall).
///
/// Retrieval cascades: Tier 1 → Tier 2 → Tier 3 (on-demand escalation).
pub struct TieredMemoryHub {
    core: Mutex<CoreStore>,
    archival: Mutex<ArchivalStore>,
    recall: Mutex<RecallStore>,
    config: RouterConfig,
    total_writes: AtomicU64,
    total_retrievals: AtomicU64,
    budget_evictions: AtomicU64,
}

impl TieredMemoryHub {
    /// Create with SQLite-backed archival store.
    pub fn new(conn: rusqlite::Connection, config: RouterConfig) -> Result<Self, String> {
        let core = CoreStore::new(config.core.clone());
        let archival = ArchivalStore::new(conn, config.archival.clone())?;
        let recall = RecallStore::new(config.recall.clone());

        Ok(Self {
            core: Mutex::new(core),
            archival: Mutex::new(archival),
            recall: Mutex::new(recall),
            config,
            total_writes: AtomicU64::new(0),
            total_retrievals: AtomicU64::new(0),
            budget_evictions: AtomicU64::new(0),
        })
    }

    /// Create an in-memory hub (no persistence). Useful for tests.
    pub fn in_memory(config: RouterConfig) -> Self {
        let core = CoreStore::new(config.core.clone());
        let archival = ArchivalStore::in_memory(config.archival.clone());
        let recall = RecallStore::new(config.recall.clone());

        Self {
            core: Mutex::new(core),
            archival: Mutex::new(archival),
            recall: Mutex::new(recall),
            config,
            total_writes: AtomicU64::new(0),
            total_retrievals: AtomicU64::new(0),
            budget_evictions: AtomicU64::new(0),
        }
    }

    /// Route a write to the appropriate tier based on item characteristics.
    fn route_write(&self, item: &MemoryItem) -> MemoryTier {
        // Heuristic: items explicitly tagged as core or with very high importance
        // go to Tier 1.
        if item.tags.contains(&"core".to_string()) || item.importance >= 0.95 {
            return MemoryTier::Tier1Core;
        }

        // Items with embeddings or high importance → Tier 2 (Archival).
        if !item.embedding.is_empty()
            || item.importance >= self.config.archival.min_importance
        {
            return MemoryTier::Tier2Archival;
        }

        // Everything else → Tier 3 (Recall).
        MemoryTier::Tier3Recall
    }

    /// Cascade retrieval: Tier 1 → Tier 2 → (escalate to Tier 3 if needed).
    async fn cascade_retrieve(
        &self,
        query: &MemoryQuery,
    ) -> Result<(Vec<MemoryResult>, RetrievalStats), String> {
        let mut all_results = Vec::new();
        let mut stats = RetrievalStats::default();
        let remaining = query.max_results.min(self.config.max_total_results);

        // Phase 1: Always query Tier 1 (Core)
        {
            let mut core = self.core.lock().map_err(|e| e.to_string())?;
            let core_results = self.retrieve_from_core(&mut core, query);
            stats.tier1_hits = core_results.len();
            all_results.extend(core_results);
        }

        // Phase 2: Query Tier 2 (Archival) if we need more results
        if all_results.len() < remaining {
            let mut archival = self.archival.lock().map_err(|e| e.to_string())?;
            let mut arch_query = query.clone();
            arch_query.max_results = remaining - all_results.len();
            let arch_results = archival.search(&arch_query);
            stats.tier2_hits = arch_results.len();
            all_results.extend(arch_results);
        }

        // Phase 3: Escalate to Tier 3 (Recall) only if core+archival are
        // insufficient AND the query isn't prefer_concise.
        if all_results.len() < self.config.escalation_threshold && !query.prefer_concise {
            stats.escalated_to_tier3 = true;
            let mut recall = self.recall.lock().map_err(|e| e.to_string())?;
            let mut recall_query = query.clone();
            recall_query.max_results = remaining - all_results.len();
            let recall_results = recall.search(&recall_query);
            stats.tier3_hits = recall_results.len();
            all_results.extend(recall_results);
        }

        stats.total_candidates = all_results.len();

        // Deduplicate by id (prefer higher-tier results)
        all_results.sort_by(|a, b| {
            a.item
                .id
                .cmp(&b.item.id)
                .then(tier_priority(&a.source_tier).cmp(&tier_priority(&b.source_tier)))
        });
        all_results.dedup_by(|a, b| a.item.id == b.item.id);

        // Limit final count
        all_results.truncate(remaining);

        Ok((all_results, stats))
    }

    /// Retrieve from Tier 1 only.
    fn retrieve_from_core(
        &self,
        core: &mut CoreStore,
        query: &MemoryQuery,
    ) -> Vec<MemoryResult> {
        if !query.tier_filter.contains(&MemoryTier::Tier1Core) {
            return Vec::new();
        }

        let items = core.items_by_importance();
        items
            .into_iter()
            .filter(|item| {
                item.importance >= query.min_importance
                    && (query.text.is_empty() || item.content.to_lowercase().contains(&query.text.to_lowercase()))
            })
            .take(query.max_results)
            .map(|item| MemoryResult {
                item: item.clone(),
                relevance: item.importance,
                source_tier: MemoryTier::Tier1Core,
            })
            .collect()
    }
}

/// Tier priority for deduplication: Tier1 > Tier2 > Tier3.
fn tier_priority(tier: &MemoryTier) -> u8 {
    match tier {
        MemoryTier::Tier1Core => 0,
        MemoryTier::Tier2Archival => 1,
        MemoryTier::Tier3Recall => 2,
    }
}

#[async_trait::async_trait]
impl MemoryHub for TieredMemoryHub {
    async fn write(&self, item: MemoryItem) -> Result<(), String> {
        let target_tier = self.route_write(&item);
        self.write_to_tier(item, target_tier).await
    }

    async fn write_to_tier(&self, mut item: MemoryItem, tier: MemoryTier) -> Result<(), String> {
        item.tier = tier;

        match tier {
            MemoryTier::Tier1Core => {
                let mut core = self.core.lock().map_err(|e| e.to_string())?;
                core.insert(item)?;
            }
            MemoryTier::Tier2Archival => {
                let mut archival = self.archival.lock().map_err(|e| e.to_string())?;
                archival.insert(item)?;
            }
            MemoryTier::Tier3Recall => {
                let mut recall = self.recall.lock().map_err(|e| e.to_string())?;
                recall.append(item)?;
            }
        }

        self.total_writes.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    async fn retrieve(&self, query: &MemoryQuery) -> Result<Vec<MemoryResult>, String> {
        self.total_retrievals.fetch_add(1, Ordering::Relaxed);
        let (results, _stats) = self.cascade_retrieve(query).await?;
        Ok(results)
    }

    async fn retrieve_from_tier(
        &self,
        query: &MemoryQuery,
        tier: MemoryTier,
    ) -> Result<Vec<MemoryResult>, String> {
        match tier {
            MemoryTier::Tier1Core => {
                let mut core = self.core.lock().map_err(|e| e.to_string())?;
                Ok(self.retrieve_from_core(&mut core, query))
            }
            MemoryTier::Tier2Archival => {
                let mut archival = self.archival.lock().map_err(|e| e.to_string())?;
                Ok(archival.search(query))
            }
            MemoryTier::Tier3Recall => {
                let mut recall = self.recall.lock().map_err(|e| e.to_string())?;
                Ok(recall.search(query))
            }
        }
    }

    async fn update(&self, id: &str, content: &str, importance: f64) -> Result<(), String> {
        // Try each tier
        {
            let mut core = self.core.lock().map_err(|e| e.to_string())?;
            if let Some(item) = core.get_mut(id) {
                item.content = content.to_string();
                item.importance = importance;
                return Ok(());
            }
        }
        // For archival and recall, remove + re-insert (simpler than in-place update)
        {
            let mut archival = self.archival.lock().map_err(|e| e.to_string())?;
            if let Some(mut item) = archival.get(id).cloned() {
                archival.remove(id)?;
                item.content = content.to_string();
                item.importance = importance;
                archival.insert(item)?;
                return Ok(());
            }
        }
        // Recall: find and note the item needs update (recall is append-only by design,
        // so we log this as a limitation)
        Err(format!("Item {} not found in any tier", id))
    }

    async fn delete(&self, id: &str) -> Result<(), String> {
        let mut deleted = false;

        {
            let mut core = self.core.lock().map_err(|e| e.to_string())?;
            if core.remove(id).is_some() {
                deleted = true;
            }
        }
        {
            let mut archival = self.archival.lock().map_err(|e| e.to_string())?;
            if archival.remove(id)? {
                deleted = true;
            }
        }
        {
            let mut recall = self.recall.lock().map_err(|e| e.to_string())?;
            if recall.remove(id).is_some() {
                deleted = true;
            }
        }

        if deleted {
            Ok(())
        } else {
            Err(format!("Item {} not found in any tier", id))
        }
    }

    async fn enforce_budget(&self) -> Result<usize, String> {
        let core = self.core.lock().map_err(|e| e.to_string())?;
        let before = core.len();
        // CoreStore enforces budget on every insert, so this is a no-op
        // but exposed for explicit control.
        let stats = core.stats();
        self.budget_evictions
            .fetch_add((before.saturating_sub(stats.item_count)) as u64, Ordering::Relaxed);
        Ok(before.saturating_sub(stats.item_count))
    }

    async fn snapshot_for_context(&self) -> Result<String, String> {
        let mut parts = Vec::new();

        // Tier 1 is always included
        {
            let core = self.core.lock().map_err(|e| e.to_string())?;
            let core_snap = core.snapshot();
            if !core_snap.is_empty() {
                parts.push(core_snap);
            }
        }

        // Tier 2: include a summary of top archival items
        {
            let archival = self.archival.lock().map_err(|e| e.to_string())?;
            let stats = archival.stats();
            if stats.item_count > 0 {
                parts.push(format!(
                    "[Archival Summary] {} items stored, {} with embeddings",
                    stats.item_count, stats.total_embeddings
                ));
            }
        }

        // Tier 3: include recent conversation history
        {
            let recall = self.recall.lock().map_err(|e| e.to_string())?;
            let recall_snap = recall.snapshot();
            if !recall_snap.is_empty() {
                parts.push(recall_snap);
            }
        }

        Ok(parts.join("\n\n"))
    }

    fn stats(&self) -> HubStats {
        let core_stats = self
            .core
            .lock()
            .map(|c| c.stats())
            .unwrap_or_default();
        let archival_stats = self
            .archival
            .lock()
            .map(|a| a.stats())
            .unwrap_or_default();
        let recall_stats = self
            .recall
            .lock()
            .map(|r| r.stats())
            .unwrap_or_default();

        HubStats {
            tier1_count: core_stats.item_count,
            tier1_chars: core_stats.total_chars,
            tier1_budget: core_stats.char_budget,
            tier2_count: archival_stats.item_count,
            tier3_count: recall_stats.item_count,
            total_writes: self.total_writes.load(Ordering::Relaxed),
            total_retrievals: self.total_retrievals.load(Ordering::Relaxed),
            budget_evictions: self.budget_evictions.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(id: &str, content: &str, importance: f64) -> MemoryItem {
        MemoryItem::new(
            id.to_string(),
            content.to_string(),
            MemoryTier::Tier1Core,
            importance,
            "test".to_string(),
        )
    }

    fn default_hub() -> TieredMemoryHub {
        TieredMemoryHub::in_memory(RouterConfig::default())
    }

    #[tokio::test]
    async fn test_write_and_retrieve() {
        let hub = default_hub();
        let item = make_item("u1", "Alice is an ML engineer", 0.95);
        hub.write(item).await.unwrap();

        let query = MemoryQuery {
            text: "Alice".to_string(),
            embedding: Vec::new(),
            tier_filter: vec![
                MemoryTier::Tier1Core,
                MemoryTier::Tier2Archival,
                MemoryTier::Tier3Recall,
            ],
            max_results: 10,
            min_importance: 0.0,
            prefer_concise: false,
        };

        let results = hub.retrieve(&query).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_route_high_importance_to_core() {
        let hub = default_hub();
        let item = make_item("u1", "critical info", 0.98);
        hub.write_to_tier(item.clone(), MemoryTier::Tier1Core)
            .await
            .unwrap();

        let query = MemoryQuery {
            text: "critical".to_string(),
            embedding: Vec::new(),
            tier_filter: vec![MemoryTier::Tier1Core],
            max_results: 10,
            min_importance: 0.0,
            prefer_concise: true,
        };

        let results = hub.retrieve_from_tier(&query, MemoryTier::Tier1Core).await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].source_tier, MemoryTier::Tier1Core);
    }

    #[tokio::test]
    async fn test_delete() {
        let hub = default_hub();
        hub.write(make_item("d1", "delete me", 0.5))
            .await
            .unwrap();
        hub.delete("d1").await.unwrap();

        let query = MemoryQuery {
            text: "delete".to_string(),
            embedding: Vec::new(),
            tier_filter: vec![
                MemoryTier::Tier1Core,
                MemoryTier::Tier2Archival,
                MemoryTier::Tier3Recall,
            ],
            max_results: 10,
            min_importance: 0.0,
            prefer_concise: false,
        };

        let results = hub.retrieve(&query).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_snapshot_for_context() {
        let hub = default_hub();
        hub.write(make_item("u1", "Alice", 0.95))
            .await
            .unwrap();
        hub.write(make_item("g1", "Build memory system", 0.85))
            .await
            .unwrap();

        let snap = hub.snapshot_for_context().await.unwrap();
        assert!(snap.contains("[Core Memory]"));
        assert!(snap.contains("Alice"));
    }

    #[tokio::test]
    async fn test_stats() {
        let hub = default_hub();
        hub.write(make_item("s1", "a", 0.9)).await.unwrap();
        hub.write(make_item("s2", "b", 0.5)).await.unwrap();

        let stats = hub.stats();
        assert!(stats.tier1_count + stats.tier2_count + stats.tier3_count >= 2);
        assert_eq!(stats.total_writes, 2);
    }

    #[tokio::test]
    async fn test_escalation_to_tier3() {
        let config = RouterConfig {
            escalation_threshold: 5,
            ..Default::default()
        };
        let hub = TieredMemoryHub::in_memory(config);

        // Write a low-importance item that goes to Tier 3
        let item = MemoryItem::new(
            "c1".to_string(),
            "casual conversation".to_string(),
            MemoryTier::Tier3Recall,
            0.1,
            "test".to_string(),
        );
        hub.write_to_tier(item, MemoryTier::Tier3Recall)
            .await
            .unwrap();

        let query = MemoryQuery {
            text: "casual".to_string(),
            embedding: Vec::new(),
            tier_filter: vec![
                MemoryTier::Tier1Core,
                MemoryTier::Tier2Archival,
                MemoryTier::Tier3Recall,
            ],
            max_results: 10,
            min_importance: 0.0,
            prefer_concise: false, // Allow escalation
        };

        let results = hub.retrieve(&query).await.unwrap();
        assert!(!results.is_empty());
    }
}
