//! kb_trust — 从 `nt_memory_kb/mod.rs` 拆分 (行为零变更).
//! `impl KnowledgeBase` 按域搬运, 原文逐行, 仅补可见性/导入。


use super::KnowledgeBase;
use super::nt_memory_confidence;
use super::nt_memory_confidence::search_with_confidence;
use super::privacy::PrivacyConfig;
use super::nt_memory_store;
use super::privacy;
use super::shared_utils;
use super::{CommunityQueryMode, CommunityResult, KnowledgeNode, PrivacyEnforcer, PrivacyMode, RetrievalStrategy, UncertainResult};

impl KnowledgeBase {
    pub fn search_with_confidence(
        &self,
        query: &str,
        strategy: RetrievalStrategy,
        limit: usize,
    ) -> Result<Vec<UncertainResult>, String> {
        let store = self
            .confidence_store
            .read()
            .map_err(|e| format!("Lock: {}", e))?;
        search_with_confidence(self, &store, query, strategy, limit)
    }

    pub fn store_node_confidence(
        &self,
        node_id: &uuid::Uuid,
        epistemic: &nt_memory_confidence::EpistemicConfidence,
    ) -> Result<(), String> {
        let store = self
            .confidence_store
            .write()
            .map_err(|e| format!("Lock: {}", e))?;
        store.store_confidence(node_id, epistemic)
    }

    pub fn get_node_confidence(
        &self,
        node_id: &uuid::Uuid,
    ) -> Result<Option<nt_memory_confidence::EpistemicConfidence>, String> {
        let store = self
            .confidence_store
            .read()
            .map_err(|e| format!("Lock: {}", e))?;
        store.get_confidence(node_id)
    }

    pub fn persist_confidence_store(&self) -> Result<(), String> {
        let store = self
            .confidence_store
            .read()
            .map_err(|e| format!("Lock: {}", e))?;
        shared_utils::save_kv_state(self, "confidence", "store", &*store)
    }

    pub fn load_confidence_store(&self) -> Result<(), String> {
        let loaded: Option<nt_memory_confidence::ConfidenceStore> =
            shared_utils::load_kv_state(self, "confidence", "store")?;
        if let Some(data) = loaded {
            let mut store = self
                .confidence_store
                .write()
                .map_err(|e| format!("Lock: {}", e))?;
            *store = data;
        }
        Ok(())
    }

    // ── Community Detection ──

    pub fn detect_communities(&self, max_nodes: usize) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let nodes =
            nt_memory_store::get_all_nodes(&conn).map_err(|e| format!("list_nodes: {}", e))?;
        let nodes: Vec<_> = nodes.into_iter().take(max_nodes).collect();
        let mut edges = Vec::new();
        for node in &nodes {
            if let Ok(e) = nt_memory_store::get_edges_for_node(&conn, &node.id) {
                edges.extend(e);
            }
        }
        drop(conn);
        let mut cs = self
            .community_search
            .write()
            .map_err(|e| format!("Lock: {}", e))?;
        cs.detect(&nodes, &edges);
        Ok(cs.hierarchy().map(|h| h.total_communities()).unwrap_or(0))
    }

    pub fn search_community(
        &self,
        query: &str,
        mode: CommunityQueryMode,
        k: usize,
    ) -> Result<Vec<CommunityResult>, String> {
        let cs = self
            .community_search
            .read()
            .map_err(|e| format!("Lock: {}", e))?;
        cs.search_community(query, mode, k)
    }

    // ── Privacy ──

    pub fn insert_node_with_privacy(
        &self,
        node: &KnowledgeNode,
    ) -> Result<privacy::DataSovereigntyProof, String> {
        let enc = self.privacy.read().map_err(|e| format!("Lock: {}", e))?;
        enc.store_with_privacy(node)
    }

    pub fn set_privacy_mode(
        &self,
        mode: PrivacyMode,
        encryption_key: Option<String>,
        auto_export_path: Option<String>,
    ) -> Result<(), String> {
        let config = PrivacyConfig {
            mode,
            encryption_key,
            auto_export_path,
            data_retention_days: 90,
        };
        let mut privacy = self.privacy.write().map_err(|e| format!("Lock: {}", e))?;
        *privacy = PrivacyEnforcer::new(config);
        Ok(())
    }

    // ── Vector Adapter ──
}
