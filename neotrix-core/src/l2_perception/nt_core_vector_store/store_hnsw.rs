use std::collections::HashMap;
use std::sync::Mutex;

use instant_distance::{Builder, Search};

use super::float_vec::{FloatVec, bytes_to_f32s};
use super::store::VectorStore;
use super::types::{DistanceMetric, IndexConfig, VectorSearchResult, VectorRecord};

struct Inner {
    records: Vec<VectorRecord>,
    hnsw: Option<instant_distance::HnswMap<FloatVec, usize>>,
    config: IndexConfig,
}

/// The `instant_distance` graph is built exclusively from `FloatVec`, whose
/// `Point::distance` is cosine distance (see `float_vec`). The graph therefore
/// only ever produces a *cosine* ordering. Hamming has no graph, so it is
/// served by exact scan; reporting a Hamming distance next to a
/// cosine-ordered candidate list would contradict the ranking.
fn graph_supports(metric: DistanceMetric) -> bool {
    matches!(metric, DistanceMetric::Cosine | DistanceMetric::Euclidean)
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

    /// Exact brute-force path for metrics the cosine graph cannot rank by.
    /// Reported distance and ordering are produced by the same function, so the
    /// two can never disagree. `filter` is applied before ranking, keeping the
    /// result exact rather than a filtered sample of graph candidates.
    fn exact_hamming_scan(
        inner: &Inner,
        query: &[u8],
        k: usize,
        filter: Option<&HashMap<String, String>>,
    ) -> Vec<VectorSearchResult> {
        let mut results: Vec<VectorSearchResult> = inner
            .records
            .iter()
            .filter(|record| match filter {
                Some(filter) => filter
                    .iter()
                    .all(|(fk, fv)| record.metadata.get(fk).map(|mv| mv == fv).unwrap_or(false)),
                None => true,
            })
            .map(|record| VectorSearchResult {
                id: record.id.clone(),
                distance: hamming_distance_u8(query, &record.vector) as f64,
                metadata: record.metadata.clone(),
            })
            .collect();
        // Tie-break on id so equal distances (e.g. an all-zero query) yield a
        // stable, reproducible ordering instead of an arbitrary one.
        results.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        results.truncate(k);
        results
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

    fn search(&self, query: &[u8], k: usize) -> Vec<VectorSearchResult> {
        let inner = self.inner.lock().unwrap_or_else(|e| {
            log::warn!("[hnsw_vector_store] mutex poisoned: {}", e);
            e.into_inner()
        });

        if inner.records.is_empty() {
            return Vec::new();
        }

        if !graph_supports(inner.config.distance_metric) {
            return Self::exact_hamming_scan(&inner, query, k, None);
        }

        let hnsw = match &inner.hnsw {
            Some(h) => h,
            None => return Vec::new(),
        };

        let query_f32 = FloatVec(bytes_to_f32s(query));
        let mut search = Search::default();

        let mut results: Vec<VectorSearchResult> = hnsw
            .search(&query_f32, &mut search)
            .take(k)
            .filter_map(|item| {
                let record = inner.records.get(*item.value)?;
                // Same metric the graph ranked by: cosine distance, and for
                // Euclidean the monotone remap of that same cosine distance.
                let distance = match inner.config.distance_metric {
                    DistanceMetric::Hamming => hamming_distance_u8(query, &record.vector) as f64,
                    DistanceMetric::Cosine => item.distance as f64,
                    DistanceMetric::Euclidean => (item.distance as f64).powi(2) * query.len() as f64,
                };
                Some(VectorSearchResult {
                    id: record.id.clone(),
                    distance,
                    metadata: record.metadata.clone(),
                })
            })
            .collect();
        // A zero-norm query has cosine distance 1.0 against every record, so the
        // graph's own order among those ties is unspecified. Sort (id as
        // tie-break) so the degenerate case is still well defined.
        results.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        results
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
    ) -> Vec<VectorSearchResult> {
        let inner = self.inner.lock().unwrap_or_else(|e| {
            log::warn!("[hnsw_vector_store] mutex poisoned: {}", e);
            e.into_inner()
        });

        if inner.records.is_empty() {
            return Vec::new();
        }

        if !graph_supports(inner.config.distance_metric) {
            return Self::exact_hamming_scan(&inner, query, k, Some(filter));
        }

        let hnsw = match &inner.hnsw {
            Some(h) => h,
            None => return Vec::new(),
        };

        let query_f32 = FloatVec(bytes_to_f32s(query));
        let mut search = Search::default();
        let scan_limit = inner.records.len().max(k * 10);

        let mut filtered: Vec<VectorSearchResult> = hnsw
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
                Some(VectorSearchResult {
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
                .then_with(|| a.id.cmp(&b.id))
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
