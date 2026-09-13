use std::collections::HashMap;

use crate::core::nt_core_vector_store::store::VectorStore;
use crate::core::nt_core_vector_store::types::{IndexConfig, VectorRecord, VectorSearchResult};

pub struct KbVectorAdapter {
    pub store: Box<dyn VectorStore>,
}

impl KbVectorAdapter {
    pub fn new(store: Box<dyn VectorStore>) -> Self {
        Self { store }
    }

    pub fn search_similar_nodes(&self, query_vector: &[u8], k: usize) -> Vec<VectorSearchResult> {
        self.store.search(query_vector, k)
    }

    pub fn insert_node_embedding(
        &mut self,
        node_id: &str,
        vector: Vec<u8>,
        metadata: HashMap<String, String>,
    ) -> Result<(), String> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let record = VectorRecord {
            id: node_id.to_string(),
            vector,
            metadata,
            timestamp: ts,
        };

        self.store.insert(record)
    }

    pub fn remove_node(&mut self, node_id: &str) -> Result<(), String> {
        self.store.remove(node_id)
    }

    pub fn total_vectors(&self) -> usize {
        self.store.len()
    }

    pub fn is_healthy(&self) -> bool {
        self.store.is_healthy()
    }
}

pub fn create_kb_vector_adapter(config: Option<IndexConfig>) -> KbVectorAdapter {
    let cfg = config.unwrap_or_default();
    let store = crate::core::nt_core_vector_store::factory::create_store(
        crate::core::nt_core_vector_store::factory::StoreBackend::IVF,
        cfg,
    );

    KbVectorAdapter::new(store)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_creation() {
        let config = IndexConfig::default();
        let store = crate::core::nt_core_vector_store::factory::create_store(
            crate::core::nt_core_vector_store::factory::StoreBackend::BruteForce,
            config,
        );

        let adapter = KbVectorAdapter::new(store);
        assert!(adapter.is_healthy());
        assert_eq!(adapter.total_vectors(), 0);
    }

    #[test]
    fn test_adapter_insert_search() {
        let config = IndexConfig::default();
        let store = crate::core::nt_core_vector_store::factory::create_store(
            crate::core::nt_core_vector_store::factory::StoreBackend::BruteForce,
            config,
        );

        let mut adapter = KbVectorAdapter::new(store);

        let mut meta = HashMap::new();
        meta.insert("domain".to_string(), "test".to_string());

        adapter
            .insert_node_embedding("node_1", vec![0x00; 8], meta.clone())
            .unwrap();
        adapter
            .insert_node_embedding("node_2", vec![0xFF; 8], HashMap::new())
            .unwrap();

        assert_eq!(adapter.total_vectors(), 2);

        let results = adapter.search_similar_nodes(&[0x00; 8], 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "node_1");
        assert!((results[0].distance).abs() < 1e-10);
    }
}
