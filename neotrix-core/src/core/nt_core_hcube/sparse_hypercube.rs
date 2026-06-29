use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

const VSA_DIM: usize = 4096;
const TOP_K_CLUSTERS: usize = 4;
const MAX_CLUSTERS: usize = 64;
const CLUSTER_CAPACITY: usize = 256;
const HOT_THRESHOLD_ACCESS: u64 = 10;
const QUANTIZE_BITS: usize = 8;

#[derive(Debug, Clone)]
pub struct SparseHyperCube {
    clusters: Vec<SparseCluster>,
    cluster_centroids: Vec<Vec<u8>>,
    access_counters: Vec<u64>,
    total_vectors: usize,
}

#[derive(Debug, Clone)]
struct SparseCluster {
    vectors: Vec<Vec<u8>>,
    keys: Vec<String>,
    access_counts: Vec<u64>,
    level: CacheLevel,
}

#[derive(Debug, Clone, PartialEq)]
enum CacheLevel {
    Hot,
    Warm,
    Cold,
}

impl SparseCluster {
    fn new(level: CacheLevel) -> Self {
        Self {
            vectors: Vec::with_capacity(CLUSTER_CAPACITY),
            keys: Vec::with_capacity(CLUSTER_CAPACITY),
            access_counts: Vec::with_capacity(CLUSTER_CAPACITY),
            level,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SparseSearchResult {
    pub key: String,
    pub vector: Vec<u8>,
    pub distance: f64,
    pub level: &'static str,
}

impl SparseHyperCube {
    pub fn new() -> Self {
        Self {
            clusters: Vec::with_capacity(MAX_CLUSTERS),
            cluster_centroids: Vec::with_capacity(MAX_CLUSTERS),
            access_counters: Vec::with_capacity(MAX_CLUSTERS),
            total_vectors: 0,
        }
    }

    pub fn insert(&mut self, key: &str, vector: Vec<u8>) {
        if vector.len() != VSA_DIM {
            return;
        }
        let centroid_idx = self.find_nearest_cluster(&vector);
        if let Some(idx) = centroid_idx {
            self.clusters[idx].vectors.push(vector);
            self.clusters[idx].keys.push(key.to_string());
            self.clusters[idx].access_counts.push(0);
            self.total_vectors += 1;
            self.maybe_recluster();
        } else {
            self.create_cluster(&vector, key);
        }
    }

    pub fn search(&self, query: &[u8], top_k: usize) -> Vec<SparseSearchResult> {
        if self.clusters.is_empty() || query.len() != VSA_DIM {
            return Vec::new();
        }
        let mut scored: Vec<(f64, usize, usize)> = Vec::new();
        for (ci, centroid) in self.cluster_centroids.iter().enumerate() {
            let dist = 1.0 - QuantizedVSA::similarity(query, centroid);
            scored.push((dist, ci, 0));
        }
        scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let mut results: Vec<(f64, Vec<u8>, String, &'static str)> = Vec::new();
        for &(_dist, ci, _) in scored.iter().take(TOP_K_CLUSTERS) {
            let cluster = &self.clusters[ci];
            let level_tag = match cluster.level {
                CacheLevel::Hot => "hot",
                CacheLevel::Warm => "warm",
                CacheLevel::Cold => "cold",
            };
            for (vi, vector) in cluster.vectors.iter().enumerate() {
                let sim = QuantizedVSA::similarity(query, vector);
                results.push((sim, vector.clone(), cluster.keys[vi].clone(), level_tag));
            }
        }
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let mut seen = std::collections::HashSet::new();
        results
            .into_iter()
            .filter(|r| seen.insert(r.2.clone()))
            .take(top_k)
            .map(|(sim, vec, key, level)| SparseSearchResult {
                key,
                vector: vec,
                distance: 1.0 - sim,
                level,
            })
            .collect()
    }

    pub fn get(&mut self, key: &str) -> Option<&[u8]> {
        for (ci, cluster) in self.clusters.iter_mut().enumerate() {
            for (vi, k) in cluster.keys.iter().enumerate() {
                if k == key {
                    cluster.access_counts[vi] += 1;
                    self.access_counters[ci] += 1;
                    return Some(&cluster.vectors[vi]);
                }
            }
        }
        None
    }

    pub fn record_access(&mut self, key: &str) {
        for cluster in self.clusters.iter_mut() {
            for (vi, k) in cluster.keys.iter().enumerate() {
                if k == key {
                    cluster.access_counts[vi] += 1;
                    return;
                }
            }
        }
    }

    fn find_nearest_cluster(&self, vector: &[u8]) -> Option<usize> {
        if self.cluster_centroids.is_empty() {
            return None;
        }
        let mut best_idx = 0;
        let mut best_sim = 0.0f64;
        for (i, centroid) in self.cluster_centroids.iter().enumerate() {
            let sim = QuantizedVSA::similarity(vector, centroid);
            if sim > best_sim {
                best_sim = sim;
                best_idx = i;
            }
        }
        if best_sim > 0.4 {
            Some(best_idx)
        } else {
            None
        }
    }

    fn create_cluster(&mut self, seed: &[u8], key: &str) {
        if self.clusters.len() >= MAX_CLUSTERS {
            let coldest = self.find_coldest_cluster();
            if let Some(idx) = coldest {
                self.clusters.remove(idx);
                self.cluster_centroids.remove(idx);
                self.access_counters.remove(idx);
            } else {
                return;
            }
        }
        let level = if self.clusters.len() < 4 {
            CacheLevel::Hot
        } else if self.clusters.len() < 16 {
            CacheLevel::Warm
        } else {
            CacheLevel::Cold
        };
        let mut cluster = SparseCluster::new(level);
        cluster.vectors.push(seed.to_vec());
        cluster.keys.push(key.to_string());
        cluster.access_counts.push(0);
        self.clusters.push(cluster);
        self.cluster_centroids.push(seed.to_vec());
        self.access_counters.push(0);
        self.total_vectors += 1;
    }

    fn find_coldest_cluster(&self) -> Option<usize> {
        self.access_counters
            .iter()
            .enumerate()
            .min_by_key(|&(_, &count)| count)
            .map(|(i, _)| i)
    }

    fn maybe_recluster(&mut self) {
        if self.total_vectors % (CLUSTER_CAPACITY * 2) != 0 {
            return;
        }
        for (ci, cluster) in self.clusters.iter_mut().enumerate() {
            if cluster.vectors.is_empty() {
                continue;
            }
            let bundled: Vec<&[u8]> = cluster.vectors.iter().map(|v| v.as_slice()).collect();
            let new_centroid = QuantizedVSA::bundle(&bundled);
            self.cluster_centroids[ci] = new_centroid;
            let avg_access = cluster.access_counts.iter().sum::<u64>()
                / cluster.access_counts.len().max(1) as u64;
            cluster.level = if avg_access >= HOT_THRESHOLD_ACCESS {
                CacheLevel::Hot
            } else if avg_access >= HOT_THRESHOLD_ACCESS / 3 {
                CacheLevel::Warm
            } else {
                CacheLevel::Cold
            };
        }
    }

    pub fn quantize(&self, vector: &[u8]) -> Vec<u8> {
        if vector.len() < QUANTIZE_BITS {
            return vector.to_vec();
        }
        let step = vector.len() / QUANTIZE_BITS;
        let mut quantized = Vec::with_capacity(QUANTIZE_BITS);
        for chunk in vector.chunks(step) {
            let ones = chunk.iter().filter(|&&b| b > 0).count();
            let ratio = ones as f64 / chunk.len() as f64;
            quantized.push(if ratio > 0.5 { 1 } else { 0 });
        }
        quantized
    }

    pub fn cluster_count(&self) -> usize {
        self.clusters.len()
    }

    pub fn total_vectors(&self) -> usize {
        self.total_vectors
    }

    pub fn stats(&self) -> SparseHyperCubeStats {
        let hot = self
            .clusters
            .iter()
            .filter(|c| c.level == CacheLevel::Hot)
            .count();
        let warm = self
            .clusters
            .iter()
            .filter(|c| c.level == CacheLevel::Warm)
            .count();
        let cold = self
            .clusters
            .iter()
            .filter(|c| c.level == CacheLevel::Cold)
            .count();
        SparseHyperCubeStats {
            clusters: self.clusters.len(),
            total_vectors: self.total_vectors,
            hot_clusters: hot,
            warm_clusters: warm,
            cold_clusters: cold,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SparseHyperCubeStats {
    pub clusters: usize,
    pub total_vectors: usize,
    pub hot_clusters: usize,
    pub warm_clusters: usize,
    pub cold_clusters: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random_vsa() -> Vec<u8> {
        QuantizedVSA::random_binary()
    }

    #[test]
    fn test_new_hypercube_empty() {
        let hc = SparseHyperCube::new();
        assert_eq!(hc.cluster_count(), 0);
        assert_eq!(hc.total_vectors(), 0);
    }

    #[test]
    fn test_insert_and_search() {
        let mut hc = SparseHyperCube::new();
        let v1 = random_vsa();
        let v2 = random_vsa();
        hc.insert("key1", v1.clone());
        hc.insert("key2", v2.clone());
        assert_eq!(hc.total_vectors(), 2);
        let results = hc.search(&v1, 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].key, "key1");
    }

    #[test]
    fn test_get_returns_vector() {
        let mut hc = SparseHyperCube::new();
        let v = random_vsa();
        hc.insert("test", v.clone());
        let retrieved = hc.get("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &v);
    }

    #[test]
    fn test_get_nonexistent_returns_none() {
        let mut hc = SparseHyperCube::new();
        assert!(hc.get("nonexistent").is_none());
    }

    #[test]
    fn test_quantize_reduces_size() {
        let hc = SparseHyperCube::new();
        let v = random_vsa();
        let q = hc.quantize(&v);
        assert_eq!(q.len(), QUANTIZE_BITS);
        assert!(q.len() < v.len());
    }

    #[test]
    fn test_search_empty_returns_empty() {
        let hc = SparseHyperCube::new();
        let q = random_vsa();
        let results = hc.search(&q, 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_invalid_query_returns_empty() {
        let hc = SparseHyperCube::new();
        let results = hc.search(&[0u8; 100], 5);
        assert!(results.is_empty());
    }
}
