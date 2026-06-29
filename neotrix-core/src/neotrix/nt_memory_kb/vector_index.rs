use chrono::{DateTime, Utc};
use rand::Rng;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::neotrix::nt_act_earn::knowledge_arbitrage::KnowledgeGraph;
use crate::neotrix::nt_memory_kb::nt_memory_embed;

/// A single vector entry in the index.
#[derive(Debug, Clone)]
pub struct VectorEntry {
    pub id: u64,
    pub key: String,
    pub vector: Vec<f64>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

/// Search result from vector index.
#[derive(Debug, Clone)]
pub struct VectorIndexResult {
    pub entry: VectorEntry,
    pub score: f64,
}

/// Vector index configuration.
#[derive(Debug, Clone)]
pub struct VectorIndexConfig {
    pub dimensions: usize,
    pub max_entries: usize,
    pub index_type: IndexType,
    pub distance_metric: DistanceMetric,
}

impl Default for VectorIndexConfig {
    fn default() -> Self {
        Self {
            dimensions: 4096,
            max_entries: 100_000,
            index_type: IndexType::Flat,
            distance_metric: DistanceMetric::Cosine,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexType {
    Flat,
    Ivf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistanceMetric {
    Cosine,
    Dot,
    Euclidean,
}

/// In-process vector index using VSA cosine similarity.
/// Models LanceDB's API for easy migration to real LanceDB later.
pub struct VectorIndex {
    config: VectorIndexConfig,
    entries: Vec<VectorEntry>,
    next_id: u64,
    centroids: Option<Vec<Vec<f64>>>,
    inverted_lists: Option<Vec<Vec<usize>>>,
    nprobes: usize,
}

impl VectorIndex {
    pub fn new(config: VectorIndexConfig) -> Self {
        Self {
            config,
            entries: Vec::new(),
            next_id: 1,
            centroids: None,
            inverted_lists: None,
            nprobes: 8,
        }
    }

    pub fn set_nprobes(&mut self, nprobes: usize) {
        self.nprobes = nprobes;
    }

    pub fn upsert(
        &mut self,
        key: &str,
        vector: Vec<f64>,
        metadata: HashMap<String, String>,
    ) -> u64 {
        if let Some(existing) = self.entries.iter_mut().find(|e| e.key == key) {
            existing.vector = vector;
            existing.metadata = metadata;
            existing.created_at = Utc::now();
            self.centroids = None;
            self.inverted_lists = None;
            existing.id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.entries.push(VectorEntry {
                id,
                key: key.to_string(),
                vector,
                metadata,
                created_at: Utc::now(),
            });
            self.centroids = None;
            self.inverted_lists = None;
            id
        }
    }

    pub fn search(&self, query: &[f64], k: usize) -> Vec<VectorIndexResult> {
        match self.config.index_type {
            IndexType::Ivf if self.centroids.is_some() => self.search_ivf(query, k),
            _ => self.search_flat(query, k),
        }
    }

    pub fn search_by_key(&self, key: &str, k: usize) -> Result<Vec<VectorIndexResult>, String> {
        let entry = self
            .entries
            .iter()
            .find(|e| e.key == key)
            .ok_or_else(|| format!("Key '{}' not found in vector index", key))?;
        Ok(self.search(&entry.vector, k))
    }

    pub fn delete(&mut self, id: u64) -> bool {
        if let Some(pos) = self.entries.iter().position(|e| e.id == id) {
            self.entries.remove(pos);
            self.centroids = None;
            self.inverted_lists = None;
            true
        } else {
            false
        }
    }

    pub fn get(&self, key: &str) -> Option<&VectorEntry> {
        self.entries.iter().find(|e| e.key == key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn build_ivf(&mut self, num_clusters: usize) {
        if self.entries.is_empty() {
            return;
        }
        let k = num_clusters.clamp(1, self.entries.len());
        let data: Vec<Vec<f64>> = self.entries.iter().map(|e| e.vector.clone()).collect();
        let (centroids, inverted) = kmeans(&data, k, 50);
        self.centroids = Some(centroids);
        self.inverted_lists = Some(inverted);
    }

    pub fn search_with_filter<F>(
        &self,
        query: &[f64],
        k: usize,
        filter: F,
    ) -> Vec<VectorIndexResult>
    where
        F: Fn(&VectorEntry) -> bool,
    {
        let mut scored: Vec<VectorIndexResult> = self
            .entries
            .iter()
            .filter(|e| filter(e))
            .map(|e| {
                let score = self.compute_similarity(query, &e.vector);
                VectorIndexResult {
                    entry: e.clone(),
                    score,
                }
            })
            .collect();
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(k);
        scored
    }

    fn compute_similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        match self.config.distance_metric {
            DistanceMetric::Cosine => {
                let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
                let na: f64 = a.iter().map(|x| x * x).sum();
                let nb: f64 = b.iter().map(|x| x * x).sum();
                let norm = na.sqrt() * nb.sqrt();
                if norm < 1e-12 {
                    0.0
                } else {
                    dot / norm
                }
            }
            DistanceMetric::Dot => a.iter().zip(b.iter()).map(|(x, y)| x * y).sum(),
            DistanceMetric::Euclidean => {
                let dist: f64 = a
                    .iter()
                    .zip(b.iter())
                    .map(|(x, y)| (x - y) * (x - y))
                    .sum::<f64>()
                    .sqrt();
                1.0 / (1.0 + dist)
            }
        }
    }

    fn search_flat(&self, query: &[f64], k: usize) -> Vec<VectorIndexResult> {
        let mut scored: Vec<VectorIndexResult> = self
            .entries
            .iter()
            .map(|e| {
                let score = self.compute_similarity(query, &e.vector);
                VectorIndexResult {
                    entry: e.clone(),
                    score,
                }
            })
            .collect();
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(k);
        scored
    }

    fn search_ivf(&self, query: &[f64], k: usize) -> Vec<VectorIndexResult> {
        let centroids = self.centroids.as_ref().expect("IVF not built");
        let lists = self.inverted_lists.as_ref().expect("IVF not built");

        let mut centroid_scores: Vec<(f64, usize)> = centroids
            .iter()
            .enumerate()
            .map(|(i, c)| (self.compute_similarity(query, c), i))
            .collect();
        centroid_scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let nprobes = self.nprobes.min(centroids.len());
        let mut seen = vec![false; self.entries.len()];
        let mut candidates: Vec<usize> = Vec::new();
        for i in 0..nprobes {
            for &idx in &lists[centroid_scores[i].1] {
                if !seen[idx] {
                    seen[idx] = true;
                    candidates.push(idx);
                }
            }
        }

        let mut results: Vec<VectorIndexResult> = candidates
            .iter()
            .map(|&idx| {
                let score = self.compute_similarity(query, &self.entries[idx].vector);
                VectorIndexResult {
                    entry: self.entries[idx].clone(),
                    score,
                }
            })
            .collect();
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(k);
        results
    }
}

// ---------------------------------------------------------------------------
// K-Means clustering
// ---------------------------------------------------------------------------

fn euclidean_distance_sq(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y) * (x - y)).sum()
}

fn closest_centroid(point: &[f64], centroids: &[Vec<f64>]) -> (usize, f64) {
    centroids
        .iter()
        .enumerate()
        .map(|(i, c)| (i, euclidean_distance_sq(point, c)))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or((0, f64::MAX))
}

fn kmeans_plus_plus_init(data: &[Vec<f64>], k: usize) -> Vec<Vec<f64>> {
    let n = data.len();
    let mut rng = rand::thread_rng();
    let mut centroids: Vec<Vec<f64>> = Vec::with_capacity(k);

    centroids.push(data[rng.gen_range(0..n)].clone());

    let mut min_dists = vec![f64::MAX; n];

    while centroids.len() < k {
        let total: f64 = min_dists
            .iter_mut()
            .enumerate()
            .map(|(i, d)| {
                let d_sq = euclidean_distance_sq(&data[i], centroids.last().unwrap());
                *d = (*d).min(d_sq);
                *d
            })
            .sum();

        if total < 1e-30 {
            break;
        }

        let threshold = rng.gen::<f64>() * total;
        let mut cumulative = 0.0;
        for i in 0..n {
            cumulative += min_dists[i];
            if cumulative >= threshold {
                centroids.push(data[i].clone());
                break;
            }
        }
    }

    centroids
}

fn kmeans(data: &[Vec<f64>], k: usize, max_iter: usize) -> (Vec<Vec<f64>>, Vec<Vec<usize>>) {
    let n = data.len();
    if k >= n {
        let centroids = data.to_vec();
        let inverted: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
        return (centroids, inverted);
    }

    let mut centroids = kmeans_plus_plus_init(data, k);
    let mut assignments = vec![0usize; n];

    for _iter in 0..max_iter {
        let mut changed = false;

        for (i, point) in data.iter().enumerate() {
            let (best, _) = closest_centroid(point, &centroids);
            if best != assignments[i] {
                assignments[i] = best;
                changed = true;
            }
        }

        if !changed {
            break;
        }

        let mut new_centroids = vec![vec![0.0_f64; data[0].len()]; centroids.len()];
        let mut counts = vec![0usize; centroids.len()];
        for (i, point) in data.iter().enumerate() {
            let c = assignments[i];
            for (j, &v) in point.iter().enumerate() {
                new_centroids[c][j] += v;
            }
            counts[c] += 1;
        }

        for (c, centroid) in new_centroids.iter_mut().enumerate() {
            if counts[c] > 0 {
                let inv = counts[c] as f64;
                for v in centroid.iter_mut() {
                    *v /= inv;
                }
            } else {
                *centroid = data[rand::thread_rng().gen_range(0..n)].clone();
            }
        }

        centroids = new_centroids;
    }

    let mut inverted: Vec<Vec<usize>> = vec![Vec::new(); centroids.len()];
    for (i, &c) in assignments.iter().enumerate() {
        inverted[c].push(i);
    }

    (centroids, inverted)
}

// ---------------------------------------------------------------------------
// Bridge: VectorIndex ↔ KnowledgeGraph
// ---------------------------------------------------------------------------

/// Bridges VectorIndex with the KnowledgeGraph.
pub struct KnowledgeVectorBridge {
    graph: Arc<RwLock<KnowledgeGraph>>,
    index: Arc<RwLock<VectorIndex>>,
}

impl KnowledgeVectorBridge {
    pub fn new(graph: Arc<RwLock<KnowledgeGraph>>) -> Self {
        Self {
            graph,
            index: Arc::new(RwLock::new(VectorIndex::new(VectorIndexConfig::default()))),
        }
    }

    pub fn index_node(&mut self, node_id: &str, text: &str) -> Result<u64, String> {
        let config = nt_memory_embed::EmbeddingConfig::default();
        let vec_f32 = nt_memory_embed::embed_text(&config, text)?;
        let vector: Vec<f64> = vec_f32.iter().map(|&v| v as f64).collect();

        let mut metadata = HashMap::new();
        metadata.insert("node_id".to_string(), node_id.to_string());

        let mut index = self.index.write().map_err(|e| format!("Lock: {}", e))?;
        Ok(index.upsert(node_id, vector, metadata))
    }

    pub fn search_knowledge(&self, query: &str, k: usize) -> Result<Vec<(String, f64)>, String> {
        let config = nt_memory_embed::EmbeddingConfig::default();
        let vec_f32 = nt_memory_embed::embed_text(&config, query)?;
        let query_vec: Vec<f64> = vec_f32.iter().map(|&v| v as f64).collect();

        let index = self.index.read().map_err(|e| format!("Lock: {}", e))?;
        let results = index.search(&query_vec, k);
        Ok(results
            .into_iter()
            .map(|r| (r.entry.key, r.score))
            .collect())
    }

    pub fn rebuild_from_graph(&mut self) -> Result<usize, String> {
        let graph = self.graph.read().map_err(|e| format!("Lock: {}", e))?;
        let config = nt_memory_embed::EmbeddingConfig::default();

        let texts: Vec<(String, String)> = graph
            .entities
            .iter()
            .map(|e| {
                let text = if e.attributes.is_empty() {
                    e.name.clone()
                } else {
                    let attrs: Vec<String> = e
                        .attributes
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect();
                    format!("{} {}", e.name, attrs.join("; "))
                };
                (e.id.clone(), text)
            })
            .collect();

        drop(graph);

        let mut count = 0usize;
        for (id, text) in &texts {
            if let Ok(vec_f32) = nt_memory_embed::embed_text(&config, text) {
                let vector: Vec<f64> = vec_f32.iter().map(|&v| v as f64).collect();
                let mut metadata = HashMap::new();
                metadata.insert("node_id".to_string(), id.clone());
                let mut index = self.index.write().map_err(|e| format!("Lock: {}", e))?;
                index.upsert(id, vector, metadata);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn remove_node(&mut self, key: &str) -> bool {
        let mut index = match self.index.write() {
            Ok(i) => i,
            Err(_) => return false,
        };
        if let Some(entry) = index.get(key) {
            let id = entry.id;
            index.delete(id)
        } else {
            false
        }
    }

    pub fn index_ref(&self) -> &Arc<RwLock<VectorIndex>> {
        &self.index
    }
}

// ---------------------------------------------------------------------------
// Embedded tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_vector(dim: usize, seed: f64) -> Vec<f64> {
        (0..dim).map(|i| (i as f64 + seed).sin()).collect()
    }

    fn default_index() -> VectorIndex {
        let config = VectorIndexConfig {
            dimensions: 128,
            max_entries: 1000,
            index_type: IndexType::Flat,
            distance_metric: DistanceMetric::Cosine,
        };
        VectorIndex::new(config)
    }

    #[test]
    fn test_insert_and_search() {
        let mut idx = default_index();
        let v1 = make_vector(128, 1.0);
        let v2 = make_vector(128, 2.0);
        let v3 = make_vector(128, 3.0);

        idx.upsert("a", v1.clone(), HashMap::new());
        idx.upsert("b", v2.clone(), HashMap::new());
        idx.upsert("c", v3.clone(), HashMap::new());

        assert_eq!(idx.len(), 3);

        let results = idx.search(&v1, 5);
        assert!(!results.is_empty(), "Should return results");
        assert_eq!(
            results[0].entry.key, "a",
            "Closest should be 'a' (same vector)"
        );
        assert!(
            (results[0].score - 1.0).abs() < 1e-10,
            "Self-similarity should be 1.0"
        );
    }

    #[test]
    fn test_search_returns_closest() {
        let mut idx = default_index();
        let base = make_vector(128, 42.0);
        let close = make_vector(128, 42.1);
        let far = make_vector(128, 99.0);

        idx.upsert("base", base.clone(), HashMap::new());
        idx.upsert("close", close.clone(), HashMap::new());
        idx.upsert("far", far.clone(), HashMap::new());

        let results = idx.search(&base, 5);
        assert_eq!(results[0].entry.key, "base");
        assert_eq!(results[1].entry.key, "close");
        assert_eq!(results[2].entry.key, "far");
        assert!(results[0].score > results[1].score);
        assert!(results[1].score > results[2].score);
    }

    #[test]
    fn test_delete_removes_entry() {
        let mut idx = default_index();
        let v = make_vector(128, 1.0);
        let id = idx.upsert("to_delete", v.clone(), HashMap::new());
        assert_eq!(idx.len(), 1);

        assert!(idx.delete(id));
        assert_eq!(idx.len(), 0);
        assert!(idx.is_empty());

        assert!(!idx.delete(999));
    }

    #[test]
    fn test_upsert_updates_existing() {
        let mut idx = default_index();
        let v1 = make_vector(128, 1.0);
        let v2 = make_vector(128, 2.0);

        let id1 = idx.upsert("same_key", v1.clone(), HashMap::new());
        let id2 = idx.upsert("same_key", v2.clone(), HashMap::new());

        assert_eq!(id1, id2, "Upsert of existing key should return same id");
        assert_eq!(idx.len(), 1, "Should still have 1 entry");

        let results = idx.search(&v2, 5);
        assert_eq!(results[0].entry.key, "same_key");
        assert!((results[0].score - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_search_with_filter() {
        let mut idx = default_index();
        let v1 = make_vector(128, 1.0);
        let v2 = make_vector(128, 2.0);
        let v3 = make_vector(128, 3.0);

        let mut m1 = HashMap::new();
        m1.insert("type".to_string(), "fruit".to_string());
        let mut m2 = HashMap::new();
        m2.insert("type".to_string(), "fruit".to_string());
        let mut m3 = HashMap::new();
        m3.insert("type".to_string(), "animal".to_string());

        idx.upsert("apple", v1.clone(), m1);
        idx.upsert("banana", v2.clone(), m2);
        idx.upsert("cat", v3.clone(), m3);

        let results = idx.search_with_filter(&v1, 5, |e| {
            e.metadata
                .get("type")
                .map(|s| s == "fruit")
                .unwrap_or(false)
        });

        assert_eq!(results.len(), 2, "Should find 2 fruit entries");
        for r in &results {
            assert_eq!(r.entry.metadata.get("type").unwrap(), "fruit");
        }
    }

    #[test]
    fn test_empty_index_search() {
        let idx = default_index();
        let v = make_vector(128, 1.0);
        let results = idx.search(&v, 5);
        assert!(results.is_empty(), "Empty index should return no results");
    }

    #[test]
    fn test_search_by_key() {
        let mut idx = default_index();
        let v1 = make_vector(128, 1.0);
        let v2 = make_vector(128, 1.1);

        idx.upsert("target", v1.clone(), HashMap::new());
        idx.upsert("other", v2.clone(), HashMap::new());

        let results = idx.search_by_key("target", 5).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].entry.key, "target");
        assert!((results[0].score - 1.0).abs() < 1e-10);

        assert!(idx.search_by_key("nonexistent", 5).is_err());
    }

    #[test]
    fn test_ivf_build_and_search() {
        let mut idx = VectorIndex::new(VectorIndexConfig {
            dimensions: 128,
            max_entries: 1000,
            index_type: IndexType::Ivf,
            distance_metric: DistanceMetric::Cosine,
        });

        for i in 0..50 {
            let v = make_vector(128, i as f64);
            idx.upsert(&format!("item_{}", i), v, HashMap::new());
        }

        idx.build_ivf(5);
        assert!(idx.centroids.is_some());
        assert!(idx.inverted_lists.is_some());

        let query = make_vector(128, 0.0);
        let results = idx.search(&query, 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].entry.key, "item_0");
    }

    #[test]
    fn test_ivf_vs_flat_accuracy() {
        let mut flat = VectorIndex::new(VectorIndexConfig {
            dimensions: 64,
            max_entries: 1000,
            index_type: IndexType::Flat,
            distance_metric: DistanceMetric::Cosine,
        });

        let mut ivf = VectorIndex::new(VectorIndexConfig {
            dimensions: 64,
            max_entries: 1000,
            index_type: IndexType::Ivf,
            distance_metric: DistanceMetric::Cosine,
        });

        for i in 0..100 {
            let v = make_vector(64, i as f64);
            flat.upsert(&format!("k{}", i), v.clone(), HashMap::new());
            ivf.upsert(&format!("k{}", i), v, HashMap::new());
        }

        ivf.build_ivf(10);
        ivf.set_nprobes(3);

        let query = make_vector(64, 50.5);
        let flat_results = flat.search(&query, 5);
        let ivf_results = ivf.search(&query, 5);

        assert!(!flat_results.is_empty());
        assert!(!ivf_results.is_empty());

        let _flat_top = flat_results[0].entry.key.clone();
        let _ivf_top = ivf_results[0].entry.key.clone();

        let overlap = flat_results
            .iter()
            .filter(|fr| ivf_results.iter().any(|ir| ir.entry.key == fr.entry.key))
            .count();

        assert!(
            overlap >= 3,
            "IVF should share at least 3/5 top results with flat; got {}",
            overlap
        );
    }

    #[test]
    fn test_dot_metric() {
        let config = VectorIndexConfig {
            dimensions: 128,
            max_entries: 1000,
            index_type: IndexType::Flat,
            distance_metric: DistanceMetric::Dot,
        };
        let mut idx = VectorIndex::new(config);
        let v1 = make_vector(128, 1.0);
        let v2 = make_vector(128, 2.0);
        idx.upsert("a", v1.clone(), HashMap::new());
        idx.upsert("b", v2.clone(), HashMap::new());
        let results = idx.search(&v1, 5);
        assert_eq!(results[0].entry.key, "a");
    }

    #[test]
    fn test_euclidean_metric() {
        let config = VectorIndexConfig {
            dimensions: 128,
            max_entries: 1000,
            index_type: IndexType::Flat,
            distance_metric: DistanceMetric::Euclidean,
        };
        let mut idx = VectorIndex::new(config);
        let v1 = make_vector(128, 1.0);
        let v2 = make_vector(128, 100.0);
        idx.upsert("a", v1.clone(), HashMap::new());
        idx.upsert("b", v2.clone(), HashMap::new());
        let results = idx.search(&v1, 5);
        assert_eq!(results[0].entry.key, "a");
        let results_b = idx.search(&v2, 5);
        assert_eq!(results_b[0].entry.key, "b");
    }

    #[test]
    fn test_get_entry() {
        let mut idx = default_index();
        let v = make_vector(128, 1.0);
        idx.upsert("find_me", v.clone(), HashMap::new());
        let entry = idx.get("find_me");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().key, "find_me");
        assert!(idx.get("missing").is_none());
    }

    #[test]
    fn test_max_entries_respected() {
        let mut idx = VectorIndex::new(VectorIndexConfig {
            dimensions: 16,
            max_entries: 5,
            index_type: IndexType::Flat,
            distance_metric: DistanceMetric::Cosine,
        });
        for i in 0..10 {
            let v = make_vector(16, i as f64);
            let id = idx.upsert(&format!("k{}", i), v, HashMap::new());
            assert!(id > 0);
        }
        assert_eq!(idx.len(), 10);
    }

    #[test]
    fn test_kmeans_small_dataset() {
        let data: Vec<Vec<f64>> = (0..10).map(|i| make_vector(32, i as f64 * 10.0)).collect();
        let (centroids, inverted) = kmeans(&data, 3, 20);
        assert_eq!(centroids.len(), 3);
        let total_assigned: usize = inverted.iter().map(|l| l.len()).sum();
        assert_eq!(total_assigned, 10);
    }

    #[test]
    fn test_kmeans_k_equals_n() {
        let data: Vec<Vec<f64>> = (0..5).map(|i| make_vector(8, i as f64)).collect();
        let (centroids, inverted) = kmeans(&data, 5, 10);
        assert_eq!(centroids.len(), 5);
        let total: usize = inverted.iter().map(|l| l.len()).sum();
        assert_eq!(total, 5);
    }
}
