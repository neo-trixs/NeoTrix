use super::store::{BruteForceVectorStore, IvfVectorStore, VectorStore};
use super::types::IndexConfig;

pub enum StoreBackend {
    IVF,
    BruteForce,
}

pub fn create_store(backend: StoreBackend, config: IndexConfig) -> Box<dyn VectorStore> {
    match backend {
        StoreBackend::IVF => Box::new(IvfVectorStore::new(config)),
        StoreBackend::BruteForce => Box::new(BruteForceVectorStore::new(config)),
    }
}

pub fn create_default_store() -> Box<dyn VectorStore> {
    Box::new(IvfVectorStore::new(IndexConfig::default()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_vector_store::types::VectorRecord;

    #[test]
    fn test_create_ivf_store() {
        let mut store = create_store(StoreBackend::IVF, IndexConfig::default());
        assert_eq!(store.name(), "ivf");
        assert!(store.is_healthy());

        let record = VectorRecord::new("test".to_string(), vec![0xAB; 8]);
        store.insert(record).unwrap();
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_create_bruteforce_store() {
        let store = create_store(StoreBackend::BruteForce, IndexConfig::default());
        assert_eq!(store.name(), "bruteforce");
        assert!(store.is_healthy());
    }

    #[test]
    fn test_create_default_store() {
        let mut store = create_default_store();
        assert_eq!(store.name(), "ivf");
        assert!(store.is_healthy());

        let record = VectorRecord::new("test".to_string(), vec![0xAB; 8]);
        store.insert(record).unwrap();
        assert_eq!(store.len(), 1);
    }
}
