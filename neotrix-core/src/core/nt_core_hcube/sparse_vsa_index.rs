use super::sparse_vsa::SparseBinaryVSA;
use std::collections::{HashMap, HashSet};

/// Inverted index for SparseBinaryVSA — maps active bit positions to vector IDs.
///
/// For a sparse VSA with K=32 active bits out of DIM=4096, a query vector
/// shares active positions with O(K * avg_bucket_size) candidates instead of
/// scanning all N vectors. This is the standard sparse inverted index approach
/// adapted for VSA's high-dimensional binary representation.
///
/// # Complexity
/// - Insert: O(K) — one entry per active bit
/// - Search: O(K * avg_bucket) intersection scoring
/// - vs naive: O(N * K) for Jaccard scan
pub struct SparseVsaInvertedIndex<const DIM: usize, const K: usize> {
    /// Maps bit position → set of vector IDs that have a 1 at that position
    inverted: HashMap<u16, HashSet<u64>>,
    /// Maps vector ID → sparse VSA vector (for Jaccard computation)
    vectors: HashMap<u64, SparseBinaryVSA<DIM, K>>,
    /// Next auto-increment ID
    next_id: u64,
    /// Total inserts
    count: u64,
    /// Search stats
    pub stats: IndexStats,
}

#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    pub total_inserts: u64,
    pub total_searches: u64,
    pub avg_candidates_scored: f64,
    pub last_search_latency_us: u64,
}

impl IndexStats {
    fn record_search(&mut self, n_candidates: usize, latency_us: u64) {
        self.total_searches += 1;
        self.last_search_latency_us = latency_us;
        if self.total_searches <= 1 {
            self.avg_candidates_scored = n_candidates as f64;
        } else {
            self.avg_candidates_scored =
                self.avg_candidates_scored * 0.95 + n_candidates as f64 * 0.05;
        }
    }
}

impl<const DIM: usize, const K: usize> SparseVsaInvertedIndex<DIM, K> {
    pub fn new() -> Self {
        Self {
            inverted: HashMap::with_capacity(DIM as usize),
            vectors: HashMap::new(),
            next_id: 1,
            count: 0,
            stats: IndexStats::default(),
        }
    }

    /// Insert a sparse VSA vector, returning its auto-generated ID.
    pub fn insert(&mut self, v: &SparseBinaryVSA<DIM, K>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.vectors.insert(id, v.clone());

        for &bit_pos in &v.0 {
            self.inverted.entry(bit_pos).or_default().insert(id);
        }

        self.count += 1;
        self.stats.total_inserts += 1;
        id
    }

    /// Insert with a known ID (for reconstruction or migration).
    pub fn insert_with_id(&mut self, id: u64, v: &SparseBinaryVSA<DIM, K>) {
        self.vectors.insert(id, v.clone());
        for &bit_pos in &v.0 {
            self.inverted.entry(bit_pos).or_default().insert(id);
        }
        self.count += 1;
        self.stats.total_inserts += 1;
        if id >= self.next_id {
            self.next_id = id + 1;
        }
    }

    /// Remove a vector by ID.
    pub fn remove(&mut self, id: u64) -> bool {
        if let Some(v) = self.vectors.remove(&id) {
            for &bit_pos in &v.0 {
                if let Some(set) = self.inverted.get_mut(&bit_pos) {
                    set.remove(&id);
                    if set.is_empty() {
                        self.inverted.remove(&bit_pos);
                    }
                }
            }
            self.count -= 1;
            true
        } else {
            false
        }
    }

    /// Search for nearest neighbors by Jaccard similarity.
    /// Uses inverted index to only score candidates sharing active bits with query.
    ///
    /// Returns Vec<(id, similarity)> sorted descending by similarity.
    pub fn search(&mut self, query: &SparseBinaryVSA<DIM, K>, top_k: usize) -> Vec<(u64, f64)> {
        // NOTE: &mut self needed for stats recording; callers must pass &mut ref
        let start = std::time::Instant::now();

        // Collect candidate vector IDs: union of all buckets at query's active positions
        let mut candidates: HashMap<u64, usize> = HashMap::new();
        for &bit_pos in &query.0 {
            if let Some(ids) = self.inverted.get(&bit_pos) {
                for &id in ids {
                    *candidates.entry(id).or_default() += 1;
                }
            }
        }

        // Score each candidate by Jaccard similarity
        let mut results: Vec<(u64, f64)> = candidates
            .into_iter()
            .filter_map(|(id, overlap_count)| {
                self.vectors.get(&id).map(|v| {
                    let union = v.0.len() + query.0.len() - overlap_count;
                    let jaccard = if union == 0 {
                        0.0
                    } else {
                        overlap_count as f64 / union as f64
                    };
                    (id, jaccard)
                })
            })
            .collect();

        // Sort by Jaccard descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);

        // Update stats
        let latency = start.elapsed().as_micros() as u64;
        self.stats.record_search(results.len(), latency);

        results
    }

    /// Search with a similarity threshold filter.
    pub fn search_with_threshold(
        &mut self,
        query: &SparseBinaryVSA<DIM, K>,
        top_k: usize,
        threshold: f64,
    ) -> Vec<(u64, f64)> {
        self.search(query, top_k)
            .into_iter()
            .filter(|(_, sim)| *sim >= threshold)
            .collect()
    }

    /// Number of vectors in the index.
    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    /// Check if index is empty.
    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }

    /// Get a vector by ID.
    pub fn get(&self, id: u64) -> Option<&SparseBinaryVSA<DIM, K>> {
        self.vectors.get(&id)
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.inverted.clear();
        self.vectors.clear();
        self.count = 0;
    }

    /// Memory estimate (inverted index + vectors).
    pub fn memory_estimate(&self) -> usize {
        let inverted_size: usize = self.inverted.iter().map(|(_, set)| set.len() * 8 + 8).sum();
        let vectors_size: usize = self.vectors.iter().map(|(_, v)| v.0.len() * 2).sum();
        inverted_size + vectors_size
    }

    /// Get all IDs in the index, sorted.
    pub fn all_ids(&self) -> Vec<u64> {
        let mut ids: Vec<u64> = self.vectors.keys().copied().collect();
        ids.sort_unstable();
        ids
    }

    /// Get distinct bit positions that have at least one vector.
    pub fn active_bit_positions(&self) -> Vec<u16> {
        let mut bits: Vec<u16> = self.inverted.keys().copied().collect();
        bits.sort_unstable();
        bits
    }

    /// Stats summary string.
    pub fn stats_summary(&self) -> String {
        format!(
            "SparseVsaIndex: {} vectors, {} non-zero buckets, memory ~{}KB, searches={}, avg_candidates={:.1}",
            self.vectors.len(),
            self.inverted.len(),
            self.memory_estimate() / 1024,
            self.stats.total_searches,
            self.stats.avg_candidates_scored,
        )
    }
}

impl<const DIM: usize, const K: usize> Default for SparseVsaInvertedIndex<DIM, K> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestIndex = SparseVsaInvertedIndex<4096, 32>;

    #[test]
    fn test_insert_and_len() {
        let mut idx = TestIndex::new();
        let v = SparseBinaryVSA::<4096, 32>::random(42);
        let id = idx.insert(&v);
        assert_eq!(idx.len(), 1);
        assert_eq!(id, 1);
    }

    #[test]
    fn test_insert_multiple() {
        let mut idx = TestIndex::new();
        for seed in 0..10 {
            let v = SparseBinaryVSA::<4096, 32>::random(seed);
            idx.insert(&v);
        }
        assert_eq!(idx.len(), 10);
    }

    #[test]
    fn test_remove() {
        let mut idx = TestIndex::new();
        let v = SparseBinaryVSA::<4096, 32>::random(42);
        let id = idx.insert(&v);
        assert!(idx.remove(id));
        assert_eq!(idx.len(), 0);
        assert!(!idx.remove(999));
    }

    #[test]
    fn test_search_same_vector() {
        let mut idx = TestIndex::new();
        let v = SparseBinaryVSA::<4096, 32>::random(42);
        let id = idx.insert(&v);
        let results = idx.search(&v, 5);
        assert!(!results.is_empty(), "should find at least itself");
        let (found_id, sim) = results[0];
        assert_eq!(found_id, id);
        assert!((sim - 1.0).abs() < 1e-6, "self-similarity should be 1.0");
    }

    #[test]
    fn test_search_top_k() {
        let mut idx = TestIndex::new();
        let query = SparseBinaryVSA::<4096, 32>::random(42);
        for seed in 0..20 {
            let v = SparseBinaryVSA::<4096, 32>::random(seed + 100);
            idx.insert(&v);
        }
        let results = idx.search(&query, 3);
        assert!(results.len() <= 3, "should cap at top_k");
    }

    #[test]
    fn test_search_preserves_order() {
        let mut idx = TestIndex::new();
        let query = SparseBinaryVSA::<4096, 32>::random(42);
        for seed in 0..10 {
            let v = SparseBinaryVSA::<4096, 32>::random(seed);
            idx.insert(&v);
        }
        let results = idx.search(&query, 10);
        for w in results.windows(2) {
            assert!(
                w[0].1 >= w[1].1,
                "results must be sorted descending by similarity"
            );
        }
    }

    #[test]
    fn test_insert_with_id() {
        let mut idx = TestIndex::new();
        let v = SparseBinaryVSA::<4096, 32>::random(42);
        idx.insert_with_id(100, &v);
        assert!(idx.get(100).is_some());
        assert_eq!(idx.len(), 1);
    }

    #[test]
    fn test_search_empty_index() {
        let mut idx = TestIndex::new();
        let v = SparseBinaryVSA::<4096, 32>::random(42);
        let results = idx.search(&v, 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_with_threshold() {
        let mut idx = TestIndex::new();
        let v = SparseBinaryVSA::<4096, 32>::random(42);
        let id = idx.insert(&v);
        let results = idx.search_with_threshold(&v, 5, 0.9);
        assert!(!results.is_empty());
        assert_eq!(results[0].0, id);
    }

    #[test]
    fn test_clear() {
        let mut idx = TestIndex::new();
        for seed in 0..10 {
            idx.insert(&SparseBinaryVSA::<4096, 32>::random(seed));
        }
        idx.clear();
        assert!(idx.is_empty());
        assert_eq!(idx.len(), 0);
    }

    #[test]
    fn test_all_ids() {
        let mut idx = TestIndex::new();
        let v1 = SparseBinaryVSA::<4096, 32>::random(1);
        let v2 = SparseBinaryVSA::<4096, 32>::random(2);
        let id1 = idx.insert(&v1);
        let id2 = idx.insert(&v2);
        let mut ids = idx.all_ids();
        ids.sort_unstable();
        assert_eq!(ids, vec![id1, id2]);
    }

    #[test]
    fn test_active_bit_positions() {
        let mut idx = TestIndex::new();
        let v = SparseBinaryVSA::<4096, 32>::random(42);
        idx.insert(&v);
        let bits = idx.active_bit_positions();
        assert!(!bits.is_empty());
        for &b in &bits {
            assert!(b < 4096);
        }
    }

    #[test]
    fn test_remove_updates_inverted_index() {
        let mut idx = TestIndex::new();
        let v = SparseBinaryVSA::<4096, 32>::random(42);
        let id = idx.insert(&v);
        idx.remove(id);
        // After removal, none of v's active positions should reference id
        for &bit in &v.0 {
            if let Some(set) = idx.inverted.get(&bit) {
                assert!(
                    !set.contains(&id),
                    "removed id still in inverted index at bit {}",
                    bit
                );
            }
        }
    }

    #[test]
    fn test_memory_estimate_monotonic() {
        let mut idx = TestIndex::new();
        let empty_est = idx.memory_estimate();
        for seed in 0..5 {
            idx.insert(&SparseBinaryVSA::<4096, 32>::random(seed));
        }
        assert!(
            idx.memory_estimate() > empty_est,
            "index with data should use more memory"
        );
    }

    #[test]
    fn test_stats_summary() {
        let mut idx = TestIndex::new();
        for seed in 0..3 {
            idx.insert(&SparseBinaryVSA::<4096, 32>::random(seed));
        }
        let s = idx.stats_summary();
        assert!(s.contains("SparseVsaIndex:"));
        assert!(s.contains("vectors"));
    }

    #[test]
    fn test_default_is_empty() {
        let idx = TestIndex::default();
        assert!(idx.is_empty());
    }

    #[test]
    fn test_search_nonsense_vector() {
        let mut idx = TestIndex::new();
        for seed in 0..10 {
            idx.insert(&SparseBinaryVSA::<4096, 32>::random(seed));
        }
        // Empty vector should return empty results
        let empty = SparseBinaryVSA::<4096, 32>::default();
        let results = idx.search(&empty, 5);
        assert!(results.is_empty());
    }
}
