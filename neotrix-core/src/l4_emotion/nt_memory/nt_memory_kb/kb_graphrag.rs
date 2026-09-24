//! kb_graphrag — 从 `nt_memory_kb/mod.rs` 拆分 (行为零变更).
//! `impl KnowledgeBase` 按域搬运, 原文逐行, 仅补可见性/导入。


use super::KnowledgeBase;
use super::nt_memory_graphrag;
use super::shared_utils;
use super::{Community, EntityNode, GlobalSummary, GraphQueryMode, GraphRagStore, RelationEdge, SubgraphResult};

impl KnowledgeBase {
    pub fn init_graphrag(&self, config: nt_memory_graphrag::GraphRagConfig) -> Result<(), String> {
        let store = GraphRagStore::new(config);
        let mut gs = self
            .graphrag_store
            .write()
            .map_err(|e| format!("Lock: {}", e))?;
        *gs = Some(store);
        Ok(())
    }

    pub fn graphrag_extract(
        &self,
        text: &str,
        source_id: &str,
    ) -> Result<(Vec<EntityNode>, Vec<RelationEdge>), String> {
        let mut gs = self
            .graphrag_store
            .write()
            .map_err(|e| format!("Lock: {}", e))?;
        match gs.as_mut() {
            Some(store) => store.extract_entities(text, source_id),
            None => Err("GraphRAG not initialized. Call init_graphrag() first.".to_string()),
        }
    }

    pub fn graphrag_query(
        &self,
        seed_entity_ids: Vec<String>,
        mode: GraphQueryMode,
    ) -> Result<SubgraphResult, String> {
        let gs = self
            .graphrag_store
            .read()
            .map_err(|e| format!("Lock: {}", e))?;
        match gs.as_ref() {
            Some(store) => store.query(&seed_entity_ids, mode),
            None => Err("GraphRAG not initialized".to_string()),
        }
    }

    pub fn graphrag_query_by_text(
        &self,
        query_entities: Vec<String>,
        mode: GraphQueryMode,
    ) -> Result<SubgraphResult, String> {
        let gs = self
            .graphrag_store
            .read()
            .map_err(|e| format!("Lock: {}", e))?;
        match gs.as_ref() {
            Some(store) => {
                let refs: Vec<&str> = query_entities.iter().map(|s| s.as_str()).collect();
                store.query_by_text(&refs, mode)
            }
            None => Err("GraphRAG not initialized".to_string()),
        }
    }

    pub fn graphrag_search_local(
        &self,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<SubgraphResult>, String> {
        let gs = self
            .graphrag_store
            .read()
            .map_err(|e| format!("Lock: {}", e))?;
        match gs.as_ref() {
            Some(store) => Ok(store.search_local(query, top_k)),
            None => Err("GraphRAG not initialized".to_string()),
        }
    }

    pub fn graphrag_search_global(
        &self,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<GlobalSummary>, String> {
        let gs = self
            .graphrag_store
            .read()
            .map_err(|e| format!("Lock: {}", e))?;
        match gs.as_ref() {
            Some(store) => Ok(store.search_global(query, top_k)),
            None => Err("GraphRAG not initialized".to_string()),
        }
    }

    pub fn graphrag_community_summary(&self) -> Vec<Community> {
        self.graphrag_store
            .read()
            .map(|gs| {
                gs.as_ref()
                    .map(|s| s.community_summary())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    pub fn graphrag_stats(&self) -> Option<nt_memory_graphrag::GraphRagStats> {
        self.graphrag_store
            .read()
            .map(|gs| gs.as_ref().map(|s| s.stats().clone()))
            .unwrap_or(None)
    }

    pub fn save_graphrag(&self) -> Result<(), String> {
        let gs = self
            .graphrag_store
            .read()
            .map_err(|e| format!("Lock: {}", e))?;
        let store = gs.as_ref().ok_or("GraphRAG not initialized")?;
        shared_utils::save_kv_state(self, "graphrag", "store", store)
    }

    pub fn load_graphrag(&self) -> Result<(), String> {
        let loaded: Option<nt_memory_graphrag::GraphRagStore> =
            shared_utils::load_kv_state(self, "graphrag", "store")?;
        if let Some(data) = loaded {
            let mut gs = self
                .graphrag_store
                .write()
                .map_err(|e| format!("Lock: {}", e))?;
            *gs = Some(data);
        }
        Ok(())
    }
}
