use std::fs;
use std::path::Path;

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::types::*;

pub fn hamming_distance(a: &[u8], b: &[u8]) -> u64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x ^ y).count_ones() as u64)
        .sum()
}

pub fn cosine_similarity(a: &[u8], b: &[u8]) -> f64 {
    let hd = hamming_distance(a, b);
    let dim = (a.len().min(b.len()) * 8) as f64;
    if dim == 0.0 {
        return 0.0;
    }
    1.0 - 2.0 * hd as f64 / dim
}

pub fn euclidean_distance(a: &[u8], b: &[u8]) -> f64 {
    let hd = hamming_distance(a, b);
    (hd as f64).sqrt()
}

pub fn select_centroids(vectors: &[Vec<u8>], k: usize) -> Vec<Vec<u8>> {
    if vectors.is_empty() || k == 0 {
        return Vec::new();
    }
    let k = k.min(vectors.len());
    let mut rng = rand::thread_rng();
    let mut centroids: Vec<Vec<u8>> = Vec::with_capacity(k);

    let first_idx = rng.gen_range(0..vectors.len());
    centroids.push(vectors[first_idx].clone());

    let mut min_dists = vec![u64::MAX; vectors.len()];

    for _ in 1..k {
        let last = centroids.last().unwrap();
        let mut total_dist: u64 = 0;

        for (i, v) in vectors.iter().enumerate() {
            let d = hamming_distance(v, last);
            min_dists[i] = min_dists[i].min(d);
            total_dist += min_dists[i];
        }

        if total_dist == 0 {
            break;
        }

        let threshold = rng.gen_range(0..total_dist);
        let mut cumulative: u64 = 0;
        let mut selected = 0;
        for (i, d) in min_dists.iter().enumerate() {
            cumulative += d;
            if cumulative > threshold {
                selected = i;
                break;
            }
        }

        centroids.push(vectors[selected].clone());
    }

    centroids
}

pub fn assign_to_centroid(v: &[u8], centroids: &[Vec<u8>]) -> usize {
    let mut best = 0;
    let mut best_dist = u64::MAX;
    for (i, c) in centroids.iter().enumerate() {
        let d = hamming_distance(v, c);
        if d < best_dist {
            best_dist = d;
            best = i;
        }
    }
    best
}

fn compute_median_centroid(vectors: &[&[u8]], len: usize) -> Vec<u8> {
    if vectors.is_empty() {
        return vec![0; len];
    }
    let count = vectors.len();
    let mut centroid = vec![0u8; len];

    for byte_idx in 0..len {
        let mut byte_val = 0u8;
        for bit_idx in 0..8 {
            let mask = 1u8 << bit_idx;
            let ones = vectors.iter().filter(|v| (v[byte_idx] & mask) != 0).count();
            if ones > count / 2 {
                byte_val |= mask;
            }
        }
        centroid[byte_idx] = byte_val;
    }

    centroid
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IVFIndex {
    pub centroids: Vec<Vec<u8>>,
    pub partitions: Vec<Vec<VectorRecord>>,
    pub config: IndexConfig,
    pub nprobe: usize,
}

impl IVFIndex {
    pub fn new(config: IndexConfig) -> Self {
        let k = config.num_partitions;
        Self {
            centroids: Vec::with_capacity(k),
            partitions: Vec::with_capacity(k),
            config,
            nprobe: 2,
        }
    }

    pub fn len(&self) -> usize {
        self.partitions.iter().map(|p| p.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn build(&mut self, vectors: Vec<VectorRecord>) {
        if vectors.is_empty() {
            return;
        }

        let k = self.config.num_partitions.min(vectors.len());
        let vecs_only: Vec<Vec<u8>> = vectors.iter().map(|r| r.vector.clone()).collect();

        self.centroids = select_centroids(&vecs_only, k);
        self.partitions = vec![Vec::with_capacity(vectors.len() / k + 1); k];

        for record in vectors {
            let idx = assign_to_centroid(&record.vector, &self.centroids);
            self.partitions[idx].push(record);
        }

        self.recompute_centroids();
    }

    fn recompute_centroids(&mut self) {
        let len = match self.centroids.first() {
            Some(c) => c.len(),
            None => return,
        };

        for (i, partition) in self.partitions.iter().enumerate() {
            if partition.is_empty() {
                continue;
            }
            let vec_refs: Vec<&[u8]> = partition.iter().map(|r| r.vector.as_slice()).collect();
            let new_centroid = compute_median_centroid(&vec_refs, len);
            if new_centroid.len() == self.centroids[i].len() {
                self.centroids[i] = new_centroid;
            }
        }
    }

    pub fn search(&self, query: &[u8], k: usize) -> Vec<SearchResult> {
        if self.centroids.is_empty() || self.partitions.is_empty() {
            return Vec::new();
        }

        let dist_fn: DistanceFn = self.config.distance_metric.into();
        let nprobe = self.nprobe.min(self.centroids.len());

        let mut centroid_dists: Vec<(usize, f64)> = self
            .centroids
            .iter()
            .enumerate()
            .map(|(i, c)| (i, dist_fn.distance(query, c)))
            .collect();
        centroid_dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let mut results: Vec<SearchResult> = Vec::new();
        for &(pidx, _) in centroid_dists.iter().take(nprobe) {
            for record in &self.partitions[pidx] {
                let d = dist_fn.distance(query, &record.vector);
                results.push(SearchResult {
                    id: record.id.clone(),
                    distance: d,
                    metadata: record.metadata.clone(),
                });
            }
        }

        results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        results.truncate(k);
        results
    }

    pub fn insert(&mut self, record: VectorRecord) {
        if self.centroids.is_empty() {
            self.centroids.push(record.vector.clone());
            self.partitions.push(vec![record]);
            return;
        }

        let idx = assign_to_centroid(&record.vector, &self.centroids);
        self.partitions[idx].push(record);
    }

    pub fn remove(&mut self, id: &str) -> bool {
        for partition in &mut self.partitions {
            let before = partition.len();
            partition.retain(|r| r.id != id);
            if partition.len() < before {
                return true;
            }
        }
        false
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string(self).map_err(|e| e.to_string())?;
        fs::write(path, &json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let json = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }
}

enum DistanceFn {
    Hamming,
    Cosine,
    Euclidean,
}

impl DistanceFn {
    fn distance(&self, a: &[u8], b: &[u8]) -> f64 {
        match self {
            DistanceFn::Hamming => hamming_distance(a, b) as f64,
            DistanceFn::Cosine => 1.0 - cosine_similarity(a, b),
            DistanceFn::Euclidean => euclidean_distance(a, b),
        }
    }
}

impl From<DistanceMetric> for DistanceFn {
    fn from(m: DistanceMetric) -> Self {
        match m {
            DistanceMetric::Hamming => DistanceFn::Hamming,
            DistanceMetric::Cosine => DistanceFn::Cosine,
            DistanceMetric::Euclidean => DistanceFn::Euclidean,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random_binary_vector(len: usize, seed: u64) -> Vec<u8> {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};
        let mut rng = StdRng::seed_from_u64(seed);
        (0..len).map(|_| rng.gen::<u8>()).collect()
    }

    #[test]
    fn test_hamming_identical() {
        let v = vec![0b10101010, 0b11110000, 0b00001111];
        assert_eq!(hamming_distance(&v, &v), 0);
    }

    #[test]
    fn test_hamming_flipped_bits() {
        let a = vec![0b00000000, 0b00000000];
        let b = vec![0b11111111, 0b00000000];
        assert_eq!(hamming_distance(&a, &b), 8);
    }

    #[test]
    fn test_hamming_all_flipped() {
        let a = vec![0b00000000];
        let b = vec![0b11111111];
        assert_eq!(hamming_distance(&a, &b), 8);
    }

    #[test]
    fn test_cosine_identical() {
        let v = vec![0b10101010];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_opposite() {
        let a = vec![0b00000000];
        let b = vec![0b11111111];
        assert!((cosine_similarity(&a, &b) - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_half_different() {
        let a = vec![0b00001111];
        let b = vec![0b11111111];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_euclidean_identical() {
        let v = vec![0b10101010];
        assert!((euclidean_distance(&v, &v) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_select_centroids_basic() {
        let vectors = vec![
            vec![0b00000000],
            vec![0b11111111],
            vec![0b10101010],
            vec![0b01010101],
        ];
        let centroids = select_centroids(&vectors, 2);
        assert_eq!(centroids.len(), 2);
        for c in &centroids {
            assert_eq!(c.len(), 1);
        }
    }

    #[test]
    fn test_select_centroids_more_than_available() {
        let vectors = vec![vec![0b00000000], vec![0b11111111]];
        let centroids = select_centroids(&vectors, 10);
        assert_eq!(centroids.len(), 2);
    }

    #[test]
    fn test_assign_to_centroid() {
        let centroids = vec![vec![0b00000000], vec![0b11111111]];
        assert_eq!(assign_to_centroid(&vec![0b00000000], &centroids), 0);
        assert_eq!(assign_to_centroid(&vec![0b11111111], &centroids), 1);
        assert_eq!(assign_to_centroid(&vec![0b00001111], &centroids), 0);
    }

    #[test]
    fn test_ivf_new() {
        let config = IndexConfig::default();
        let index = IVFIndex::new(config);
        assert_eq!(index.partitions.len(), 0);
        assert_eq!(index.len(), 0);
        assert!(index.is_empty());
    }

    #[test]
    fn test_ivf_build_creates_partitions() {
        let config = IndexConfig {
            num_partitions: 4,
            ..IndexConfig::default()
        };
        let mut index = IVFIndex::new(config);

        let vectors: Vec<VectorRecord> = (0..20)
            .map(|i| VectorRecord::new(format!("id_{}", i), random_binary_vector(4, i as u64)))
            .collect();

        index.build(vectors);
        assert_eq!(index.partitions.len(), 4);
        assert_eq!(index.len(), 20);
        assert_eq!(index.centroids.len(), 4);
    }

    #[test]
    fn test_ivf_search_returns_nearest() {
        let config = IndexConfig::default();
        let mut index = IVFIndex::new(config);

        let mut vectors: Vec<VectorRecord> = (0..10)
            .map(|i| VectorRecord::new(format!("id_{}", i), random_binary_vector(4, i as u64)))
            .collect();

        let query_vec = random_binary_vector(4, 999);
        vectors.push(VectorRecord::new("target".to_string(), query_vec.clone()));

        index.build(vectors);
        let results = index.search(&query_vec, 3);

        assert!(!results.is_empty());
        assert_eq!(results[0].id, "target");
        assert!((results[0].distance).abs() < 1e-10);
    }

    #[test]
    fn test_ivf_insert_grows_index() {
        let config = IndexConfig {
            num_partitions: 2,
            ..IndexConfig::default()
        };
        let mut index = IVFIndex::new(config);

        let vectors: Vec<VectorRecord> = (0..5)
            .map(|i| VectorRecord::new(format!("id_{}", i), random_binary_vector(2, i as u64)))
            .collect();
        index.build(vectors);
        assert_eq!(index.len(), 5);

        index.insert(VectorRecord::new(
            "new_one".to_string(),
            random_binary_vector(2, 100),
        ));
        assert_eq!(index.len(), 6);
    }

    #[test]
    fn test_ivf_insert_when_empty() {
        let config = IndexConfig::default();
        let mut index = IVFIndex::new(config);
        assert!(index.is_empty());

        index.insert(VectorRecord::new("first".to_string(), vec![0b10101010]));
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_ivf_remove_shrinks_index() {
        let config = IndexConfig {
            num_partitions: 2,
            ..IndexConfig::default()
        };
        let mut index = IVFIndex::new(config);

        let vectors: Vec<VectorRecord> = (0..10)
            .map(|i| VectorRecord::new(format!("id_{}", i), random_binary_vector(2, i as u64)))
            .collect();
        index.build(vectors);
        assert_eq!(index.len(), 10);

        assert!(index.remove("id_0"));
        assert_eq!(index.len(), 9);

        assert!(!index.remove("nonexistent"));
        assert_eq!(index.len(), 9);
    }

    #[test]
    fn test_ivf_save_load_roundtrip() {
        let config = IndexConfig::default();
        let mut index = IVFIndex::new(config);

        let vectors: Vec<VectorRecord> = (0..10)
            .map(|i| VectorRecord::new(format!("id_{}", i), random_binary_vector(4, i as u64)))
            .collect();
        index.build(vectors);

        let dir = std::env::temp_dir().join("ivf_test");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("ivf_index.json");

        index.save(&path).unwrap();

        let loaded = IVFIndex::load(&path).unwrap();
        assert_eq!(loaded.len(), index.len());
        assert_eq!(loaded.centroids.len(), index.centroids.len());
        assert_eq!(loaded.partitions.len(), index.partitions.len());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_build_empty_vectors() {
        let config = IndexConfig::default();
        let mut index = IVFIndex::new(config);
        index.build(Vec::new());
        assert!(index.is_empty());
        assert!(index.search(&vec![0u8; 4], 5).is_empty());
    }

    #[test]
    fn test_distance_fn_from_metric() {
        let a = vec![0b00000000];
        let b = vec![0b11111111];

        let ham_fn: DistanceFn = DistanceMetric::Hamming.into();
        assert_eq!(ham_fn.distance(&a, &b), 8.0);

        let cos_fn: DistanceFn = DistanceMetric::Cosine.into();
        assert!((cos_fn.distance(&a, &b) - 2.0).abs() < 1e-10);

        let euc_fn: DistanceFn = DistanceMetric::Euclidean.into();
        assert!((euc_fn.distance(&a, &b) - (8.0f64).sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_similarity_big_vectors() {
        let a = vec![0xFF; 64];
        let b = vec![0xFF; 64];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-10);

        let c = vec![0x00; 64];
        assert!((cosine_similarity(&a, &c) - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_hamming_512_byte_vectors() {
        let a = vec![0xAB; 512];
        let b = vec![0xAB; 512];
        assert_eq!(hamming_distance(&a, &b), 0);

        let c = vec![0x00; 512];
        assert_eq!(hamming_distance(&a, &c), 512 * 4);
    }
}
