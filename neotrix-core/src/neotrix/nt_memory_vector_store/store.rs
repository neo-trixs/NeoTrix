use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use super::index::{self, IVFIndex};
use super::types::*;

pub trait VectorStore: Send + Sync {
    fn name(&self) -> &str;
    fn insert(&mut self, record: VectorRecord) -> Result<(), String>;
    fn search(&self, query: &[u8], k: usize) -> Vec<VectorSearchResult>;
    fn remove(&mut self, id: &str) -> Result<(), String>;
    fn len(&self) -> usize;
    fn is_healthy(&self) -> bool;
    fn search_with_filter(
        &self,
        query: &[u8],
        k: usize,
        filter: &HashMap<String, String>,
    ) -> Vec<VectorSearchResult>;
}

pub struct IvfVectorStore {
    pub index: Mutex<IVFIndex>,
    pub storage_path: Option<PathBuf>,
}

impl IvfVectorStore {
    pub fn new(config: IndexConfig) -> Self {
        Self {
            index: Mutex::new(IVFIndex::new(config)),
            storage_path: None,
        }
    }

    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.storage_path = Some(path);
        self
    }
}

impl VectorStore for IvfVectorStore {
    fn name(&self) -> &str {
        "ivf"
    }

    fn insert(&mut self, record: VectorRecord) -> Result<(), String> {
        let mut index = self.index.lock().map_err(|e| e.to_string())?;
        index.insert(record);
        Ok(())
    }

    fn search(&self, query: &[u8], k: usize) -> Vec<VectorSearchResult> {
        let index = self.index.lock().unwrap_or_else(|e| e.into_inner());
        index.search(query, k)
    }

    fn remove(&mut self, id: &str) -> Result<(), String> {
        let mut index = self.index.lock().map_err(|e| e.to_string())?;
        if index.remove(id) {
            Ok(())
        } else {
            Err(format!("id '{}' not found", id))
        }
    }

    fn len(&self) -> usize {
        self.index.lock().map(|i| i.len()).unwrap_or(0)
    }

    fn is_healthy(&self) -> bool {
        self.index.lock().is_ok()
    }

    fn search_with_filter(
        &self,
        query: &[u8],
        k: usize,
        filter: &HashMap<String, String>,
    ) -> Vec<VectorSearchResult> {
        let index = self.index.lock().unwrap_or_else(|e| e.into_inner());
        let all_results = index.search(query, self.len().max(k * 10));
        let mut filtered: Vec<VectorSearchResult> = all_results
            .into_iter()
            .filter(|r| {
                filter
                    .iter()
                    .all(|(fk, fv)| r.metadata.get(fk).map(|mv| mv == fv).unwrap_or(false))
            })
            .collect();
        filtered.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        filtered.truncate(k);
        filtered
    }
}

pub struct BruteForceVectorStore {
    records: Vec<VectorRecord>,
    config: IndexConfig,
}

impl BruteForceVectorStore {
    pub fn new(config: IndexConfig) -> Self {
        Self {
            records: Vec::new(),
            config,
        }
    }
}

impl VectorStore for BruteForceVectorStore {
    fn name(&self) -> &str {
        "bruteforce"
    }

    fn insert(&mut self, record: VectorRecord) -> Result<(), String> {
        self.records.push(record);
        Ok(())
    }

    fn search(&self, query: &[u8], k: usize) -> Vec<VectorSearchResult> {
        let mut results: Vec<VectorSearchResult> = self
            .records
            .iter()
            .map(|r| {
                let d = match self.config.distance_metric {
                    DistanceMetric::Hamming => index::hamming_distance(query, &r.vector) as f64,
                    DistanceMetric::Cosine => 1.0 - index::cosine_similarity(query, &r.vector),
                    DistanceMetric::Euclidean => index::euclidean_distance(query, &r.vector),
                };
                VectorSearchResult {
                    id: r.id.clone(),
                    distance: d,
                    metadata: r.metadata.clone(),
                }
            })
            .collect();
        results.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(k);
        results
    }

    fn remove(&mut self, id: &str) -> Result<(), String> {
        let before = self.records.len();
        self.records.retain(|r| r.id != id);
        if self.records.len() < before {
            Ok(())
        } else {
            Err(format!("id '{}' not found", id))
        }
    }

    fn len(&self) -> usize {
        self.records.len()
    }

    fn is_healthy(&self) -> bool {
        true
    }

    fn search_with_filter(
        &self,
        query: &[u8],
        k: usize,
        filter: &HashMap<String, String>,
    ) -> Vec<VectorSearchResult> {
        let mut results: Vec<VectorSearchResult> = self
            .records
            .iter()
            .filter(|r| {
                filter
                    .iter()
                    .all(|(fk, fv)| r.metadata.get(fk).map(|mv| mv == fv).unwrap_or(false))
            })
            .map(|r| {
                let d = match self.config.distance_metric {
                    DistanceMetric::Hamming => index::hamming_distance(query, &r.vector) as f64,
                    DistanceMetric::Cosine => 1.0 - index::cosine_similarity(query, &r.vector),
                    DistanceMetric::Euclidean => index::euclidean_distance(query, &r.vector),
                };
                VectorSearchResult {
                    id: r.id.clone(),
                    distance: d,
                    metadata: r.metadata.clone(),
                }
            })
            .collect();
        results.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(k);
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random_binary_vector(seed: u64) -> Vec<u8> {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};
        let mut rng = StdRng::seed_from_u64(seed);
        (0..8).map(|_| rng.gen::<u8>()).collect()
    }

    #[test]
    fn test_ivf_store_basic_ops() {
        let config = IndexConfig::default();
        let mut store = IvfVectorStore::new(config);
        assert_eq!(store.name(), "ivf");
        assert!(store.is_healthy());
        assert_eq!(store.len(), 0);

        let record = VectorRecord::new("test".to_string(), vec![0b10101010; 8]);
        store.insert(record).unwrap();
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_ivf_store_search() {
        let config = IndexConfig {
            num_partitions: 2,
            ..IndexConfig::default()
        };
        let mut store = IvfVectorStore::new(config);

        for i in 0..20 {
            let v = random_binary_vector(i as u64);
            store
                .insert(VectorRecord::new(format!("id_{}", i), v))
                .unwrap();
        }

        let query = random_binary_vector(100);
        let results = store.search(&query, 5);
        assert_eq!(results.len(), 5);
        assert!(results[0].distance <= results[1].distance);
    }

    #[test]
    fn test_ivf_store_remove() {
        let config = IndexConfig::default();
        let mut store = IvfVectorStore::new(config);
        store
            .insert(VectorRecord::new("a".to_string(), vec![0xFF; 4]))
            .unwrap();
        assert_eq!(store.len(), 1);

        store.remove("a").unwrap();
        assert_eq!(store.len(), 0);

        assert!(store.remove("nonexistent").is_err());
    }

    #[test]
    fn test_ivf_store_search_with_filter() {
        let config = IndexConfig::default();
        let mut store = IvfVectorStore::new(config);

        let mut meta_a = HashMap::new();
        meta_a.insert("domain".to_string(), "science".to_string());
        let mut meta_b = HashMap::new();
        meta_b.insert("domain".to_string(), "art".to_string());

        store
            .insert(
                VectorRecord::new("sci1".to_string(), vec![0x00; 4]).with_metadata(meta_a.clone()),
            )
            .unwrap();
        store
            .insert(VectorRecord::new("sci2".to_string(), vec![0x01; 4]).with_metadata(meta_a))
            .unwrap();
        store
            .insert(VectorRecord::new("art1".to_string(), vec![0xFF; 4]).with_metadata(meta_b))
            .unwrap();

        let mut filter = HashMap::new();
        filter.insert("domain".to_string(), "science".to_string());

        let query = vec![0x00; 4];
        let results = store.search_with_filter(&query, 10, &filter);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.id.starts_with("sci")));
    }

    #[test]
    fn test_bruteforce_store() {
        let config = IndexConfig::default();
        let mut store = BruteForceVectorStore::new(config);
        assert_eq!(store.name(), "bruteforce");
        assert!(store.is_healthy());

        store
            .insert(VectorRecord::new("a".to_string(), vec![0x00; 4]))
            .unwrap();
        store
            .insert(VectorRecord::new("b".to_string(), vec![0xFF; 4]))
            .unwrap();
        assert_eq!(store.len(), 2);

        let query = vec![0x00; 4];
        let results = store.search(&query, 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "a");

        store.remove("a").unwrap();
        assert_eq!(store.len(), 1);
        assert!(store.remove("nonexistent").is_err());
    }

    #[test]
    fn test_bruteforce_filter() {
        let config = IndexConfig::default();
        let mut store = BruteForceVectorStore::new(config);

        let mut meta = HashMap::new();
        meta.insert("lang".to_string(), "rust".to_string());

        store
            .insert(VectorRecord::new("r1".to_string(), vec![0x00; 4]).with_metadata(meta.clone()))
            .unwrap();
        store
            .insert(VectorRecord::new("other".to_string(), vec![0xFF; 4]))
            .unwrap();

        let mut filter = HashMap::new();
        filter.insert("lang".to_string(), "rust".to_string());
        let results = store.search_with_filter(&[0x00; 4], 10, &filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "r1");
    }

    #[test]
    fn test_vector_record_builder() {
        let mut meta = HashMap::new();
        meta.insert("k".to_string(), "v".to_string());

        let record = VectorRecord::new("id".to_string(), vec![0xAB; 8])
            .with_metadata(meta.clone())
            .with_timestamp(42);

        assert_eq!(record.id, "id");
        assert_eq!(record.metadata, meta);
        assert_eq!(record.timestamp, 42);
    }
}
