use std::collections::HashMap;
use std::sync::Mutex;

use instant_distance::{Builder, Search};

use super::float_vec::{FloatVec, bytes_to_f32s};
use super::store::VectorStore;
use super::types::{DistanceMetric, IndexConfig, SearchResult, VectorRecord};

struct Inner {
    records: Vec<VectorRecord>,
    hnsw: Option<instant_distance::HnswMap<FloatVec, usize>>,
    config: IndexConfig,
}

pub struct HnswVectorStore {
    inner: Mutex<Inner>,
}

impl HnswVectorStore {
    pub fn new(config: IndexConfig) -> Self {
        Self {
            inner: Mutex::new(Inner {
                records: Vec::new(),
                hnsw: None,
                config,
            }),
        }
    }

    fn rebuild(inner: &mut Inner) {
        if inner.records.is_empty() {
            inner.hnsw = None;
            return;
        }
        let points: Vec<FloatVec> = inner
            .records
            .iter()
            .map(|r| FloatVec(bytes_to_f32s(&r.vector)))
            .collect();
        let values: Vec<usize> = (0..inner.records.len()).collect();
        inner.hnsw = Some(Builder::default().build(points, values));
    }
}

fn hamming_distance_u8(a: &[u8], b: &[u8]) -> u64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x ^ y).count_ones() as u64)
        .sum()
}

impl VectorStore for HnswVectorStore {
    fn name(&self) -> &str {
        "hnsw"
    }

    fn insert(&mut self, record: VectorRecord) -> Result<(), String> {
        let mut inner = self.inner.lock().map_err(|e| e.to_string())?;
        inner.records.push(record);
        Self::rebuild(&mut inner);
        Ok(())
    }

    fn search(&self, query: &[u8], k: usize) -> Vec<SearchResult> {
        let inner = self.inner.lock().unwrap_or_else(|e| {
            log::warn!("[hnsw_vector_store] mutex poisoned: {}", e);
            e.into_inner()
        });

        let hnsw = match &inner.hnsw {
            Some(h) => h,
            None => return Vec::new(),
        };

        let query_f32 = FloatVec(bytes_to_f32s(query));
        let mut search = Search::default();

        hnsw.search(&query_f32, &mut search)
            .take(k)
            .filter_map(|item| {
                let record = inner.records.get(*item.value)?;
                let distance = match inner.config.distance_metric {
                    DistanceMetric::Hamming => hamming_distance_u8(query, &record.vector) as f64,
                    DistanceMetric::Cosine => item.distance as f64,
                    DistanceMetric::Euclidean => (item.distance as f64).powi(2) * query.len() as f64,
                };
                Some(SearchResult {
                    id: record.id.clone(),
                    distance,
                    metadata: record.metadata.clone(),
                })
            })
            .collect()
    }

    fn remove(&mut self, id: &str) -> Result<(), String> {
        let mut inner = self.inner.lock().map_err(|e| e.to_string())?;
        let before = inner.records.len();
        inner.records.retain(|r| r.id != id);
        if inner.records.len() < before {
            Self::rebuild(&mut inner);
            Ok(())
        } else {
            Err(format!("id '{}' not found", id))
        }
    }

    fn len(&self) -> usize {
        self.inner.lock().map(|i| i.records.len()).unwrap_or(0)
    }

    fn is_healthy(&self) -> bool {
        self.inner.lock().is_ok()
    }

    fn search_with_filter(
        &self,
        query: &[u8],
        k: usize,
        filter: &HashMap<String, String>,
    ) -> Vec<SearchResult> {
        let inner = self.inner.lock().unwrap_or_else(|e| {
            log::warn!("[hnsw_vector_store] mutex poisoned: {}", e);
            e.into_inner()
        });

        let hnsw = match &inner.hnsw {
            Some(h) => h,
            None => return Vec::new(),
        };

        let query_f32 = FloatVec(bytes_to_f32s(query));
        let mut search = Search::default();
        let scan_limit = inner.records.len().max(k * 10);

        let mut filtered: Vec<SearchResult> = hnsw
            .search(&query_f32, &mut search)
            .take(scan_limit)
            .filter_map(|item| {
                let record = inner.records.get(*item.value)?;
                let matches = filter
                    .iter()
                    .all(|(fk, fv)| record.metadata.get(fk).map(|mv| mv == fv).unwrap_or(false));
                if !matches {
                    return None;
                }
                let distance = match inner.config.distance_metric {
                    DistanceMetric::Hamming => hamming_distance_u8(query, &record.vector) as f64,
                    DistanceMetric::Cosine => item.distance as f64,
                    DistanceMetric::Euclidean => (item.distance as f64).powi(2) * query.len() as f64,
                };
                Some(SearchResult {
                    id: record.id.clone(),
                    distance,
                    metadata: record.metadata.clone(),
                })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hnsw_store_basic_ops() {
        let config = IndexConfig::default();
        let mut store = HnswVectorStore::new(config);
        assert_eq!(store.name(), "hnsw");
        assert!(store.is_healthy());
        assert_eq!(store.len(), 0);

        let record = VectorRecord::new("test".to_string(), vec![0b10101010; 8]);
        store.insert(record).unwrap();
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_hnsw_store_search() {
        let config = IndexConfig::default();
        let mut store = HnswVectorStore::new(config);

        for i in 0..20 {
            let v: Vec<u8> = (0..8).map(|j| ((i * 7 + j * 3) % 256) as u8).collect();
            store.insert(VectorRecord::new(format!("id_{}", i), v)).unwrap();
        }

        let query = vec![0x00u8; 8];
        let results = store.search(&query, 5);
        assert_eq!(results.len(), 5);
        assert!(results[0].distance <= results[1].distance);
    }

    #[test]
    fn test_hnsw_store_remove() {
        let config = IndexConfig::default();
        let mut store = HnswVectorStore::new(config);
        store.insert(VectorRecord::new("a".to_string(), vec![0xFF; 4])).unwrap();
        assert_eq!(store.len(), 1);

        store.remove("a").unwrap();
        assert_eq!(store.len(), 0);
        assert!(store.remove("nonexistent").is_err());
    }

    #[test]
    fn test_hnsw_empty_store() {
        let config = IndexConfig::default();
        let store = HnswVectorStore::new(config);
        let results = store.search(&vec![0u8; 8], 5);
        assert!(results.is_empty());
    }
}
