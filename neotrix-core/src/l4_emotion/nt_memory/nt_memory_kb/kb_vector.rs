//! kb_vector — 从 `nt_memory_kb/mod.rs` 拆分 (行为零变更).
//! `impl KnowledgeBase` 按域搬运, 原文逐行, 仅补可见性/导入。


use super::KnowledgeBase;
use super::vector_adapter;

impl KnowledgeBase {
    pub fn init_vector_adapter(&self) -> Result<(), String> {
        let adapter = vector_adapter::create_kb_vector_adapter(None);
        let mut va = self
            .vector_adapter
            .write()
            .map_err(|e| format!("Lock: {}", e))?;
        *va = Some(adapter);
        Ok(())
    }

    pub fn search_similar(
        &self,
        query_vector: &[u8],
        k: usize,
    ) -> Result<Vec<crate::l2_perception::nt_core_vector_store::types::VectorSearchResult>, String>
    {
        let va = self
            .vector_adapter
            .read()
            .map_err(|e| format!("Lock: {}", e))?;
        match va.as_ref() {
            Some(adapter) => Ok(adapter.search_similar_nodes(query_vector, k)),
            None => {
                Err("Vector adapter not initialized. Call init_vector_adapter() first.".to_string())
            }
        }
    }

    pub fn insert_embedding(
        &self,
        node_id: &str,
        vector: Vec<u8>,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<(), String> {
        let mut va = self
            .vector_adapter
            .write()
            .map_err(|e| format!("Lock: {}", e))?;
        match va.as_mut() {
            Some(adapter) => adapter.insert_node_embedding(node_id, vector, metadata),
            None => Err("Vector adapter not initialized".to_string()),
        }
    }

    // ── Store: basic CRUD ──
}
