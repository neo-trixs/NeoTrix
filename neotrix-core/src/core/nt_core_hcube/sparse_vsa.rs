use crate::core::nt_core_hcube::vsa::BinaryVsaBackend;
use rand::Rng;
use std::collections::HashSet;

/// Sparse Binary VSA — only k≈32 bits out of DIM=4096 are 1.
///
/// Operations are set-theoretic:
/// - Bind (XOR) = symmetric difference of index sets
/// - Bundle = majority voting on index presence
/// - Negate = complement
/// - Similarity = Jaccard overlap |∩|/|∪|
///
/// # Type Parameters
/// - `DIM`: dimension (total bits, default 4096)
/// - `K`: number of active (1) bits (default 32)
#[derive(Debug, Clone, PartialEq)]
pub struct SparseBinaryVSA<const DIM: usize, const K: usize>(pub Vec<u16>);

impl<const DIM: usize, const K: usize> Default for SparseBinaryVSA<DIM, K> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<const DIM: usize, const K: usize> SparseBinaryVSA<DIM, K> {
    /// Create a sparse VSA from raw indices, ensuring sorted+unique+≤K.
    pub fn from_indices(mut indices: Vec<u16>) -> Self {
        indices.sort_unstable();
        indices.dedup();
        indices.retain(|&i| (i as usize) < DIM);
        indices.truncate(K);
        Self(indices)
    }

    /// Create sparse VSA from a random number generator (RNG API, spec-compliant).
    pub fn new(dim: usize, rng: &mut impl Rng) -> Self {
        debug_assert_eq!(dim, DIM, "dim must match DIM const generic");
        let mut indices = Vec::with_capacity(K);
        let mut seen = HashSet::new();
        while indices.len() < K {
            let idx = rng.gen_range(0..DIM as u16);
            if seen.insert(idx) {
                indices.push(idx);
            }
        }
        indices.sort_unstable();
        Self(indices)
    }

    /// Seeded random generation of K sparse positions.
    pub fn random(seed: u64) -> Self {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut indices = Vec::with_capacity(K);
        let mut seen = HashSet::new();
        while indices.len() < K {
            let idx = rng.gen_range(0..DIM as u16);
            if seen.insert(idx) {
                indices.push(idx);
            }
        }
        indices.sort_unstable();
        Self(indices)
    }

    /// XOR bind = symmetric difference of index sets: (A ∪ B) \ (A ∩ B).
    pub fn bind(a: &Self, b: &Self) -> Self {
        let result = Self::symmetric_difference(&a.0, &b.0);
        Self(Self::maintain_density(result))
    }

    /// XOR is its own inverse.
    pub fn unbind(c: &Self, a: &Self) -> Self {
        Self::bind(c, a)
    }

    /// Majority voting over sparse positions.
    pub fn bundle(vectors: &[&Self]) -> Self {
        if vectors.is_empty() {
            return Self(Vec::new());
        }
        let n = vectors.len();
        // Count occurrences of each index across all vectors
        // Use sorted insert into counts for merge efficiency
        let mut counts: Vec<(u16, usize)> = Vec::new();
        for v in vectors {
            for &idx in &v.0 {
                match counts.binary_search_by(|(i, _)| i.cmp(&idx)) {
                    Ok(pos) => counts[pos].1 += 1,
                    Err(pos) => counts.insert(pos, (idx, 1)),
                }
            }
        }
        // Keep indices where count > n/2 (strict majority)
        let result: Vec<u16> = counts
            .into_iter()
            .filter(|(_, cnt)| *cnt > n / 2)
            .map(|(idx, _)| idx)
            .collect();
        Self(Self::maintain_density(result))
    }

    /// Rotate all indices by shift (mod DIM).
    pub fn permute(v: &Self, shift: isize) -> Self {
        let indices: Vec<u16> =
            v.0.iter()
                .map(|&idx| ((idx as isize + shift).rem_euclid(DIM as isize)) as u16)
                .collect();
        Self(Self::maintain_density(indices))
    }

    /// Complement: select K indices NOT in the current set.
    pub fn negate(v: &Self) -> Self {
        let present: HashSet<u16> = v.0.iter().copied().collect();
        let candidates: Vec<u16> = (0..DIM as u16).filter(|i| !present.contains(i)).collect();
        let mut selected = Self::sample_k(&candidates, K);
        selected.sort_unstable();
        Self(selected)
    }

    /// Jaccard similarity: |intersection| / |union|.
    pub fn similarity(a: &Self, b: &Self) -> f64 {
        let intersection = Self::intersection_count(&a.0, &b.0);
        let union = a.0.len() + b.0.len() - intersection;
        if union == 0 {
            return 1.0;
        }
        intersection as f64 / union as f64
    }

    /// Convert to dense Vec<u8> of 0/1 values, length DIM.
    pub fn to_dense(&self) -> Vec<u8> {
        let mut dense = vec![0u8; DIM];
        for &idx in &self.0 {
            dense[idx as usize] = 1;
        }
        dense
    }

    /// Extract sparse indices from dense Vec<u8> of 0/1 values.
    pub fn from_dense(dense: &[u8]) -> Self {
        let mut indices: Vec<u16> = dense
            .iter()
            .enumerate()
            .filter(|(_, &v)| v != 0)
            .take(K)
            .map(|(i, _)| i as u16)
            .collect();
        indices.sort_unstable();
        Self(indices)
    }

    /// Count of shared active positions = intersection size.
    pub fn intersection_size(a: &Self, b: &Self) -> u32 {
        Self::intersection_count(&a.0, &b.0) as u32
    }

    /// Sparse density: K_actual / DIM.
    pub fn density(&self) -> f64 {
        if DIM == 0 {
            return 0.0;
        }
        self.0.len() as f64 / DIM as f64
    }

    /// Access the sorted unique indices.
    pub fn indices(&self) -> &[u16] {
        &self.0
    }

    // === Internal helpers ===

    fn symmetric_difference(a: &[u16], b: &[u16]) -> Vec<u16> {
        let mut result = Vec::with_capacity(a.len() + b.len());
        let mut i = 0;
        let mut j = 0;
        while i < a.len() && j < b.len() {
            if a[i] < b[j] {
                result.push(a[i]);
                i += 1;
            } else if b[j] < a[i] {
                result.push(b[j]);
                j += 1;
            } else {
                // Equal → skip both (XOR cancellation)
                i += 1;
                j += 1;
            }
        }
        result.extend_from_slice(&a[i..]);
        result.extend_from_slice(&b[j..]);
        result
    }

    fn intersection_count(a: &[u16], b: &[u16]) -> usize {
        let mut count = 0;
        let mut i = 0;
        let mut j = 0;
        while i < a.len() && j < b.len() {
            if a[i] < b[j] {
                i += 1;
            } else if b[j] < a[i] {
                j += 1;
            } else {
                count += 1;
                i += 1;
                j += 1;
            }
        }
        count
    }

    /// Ensure exactly K elements: add random unique or truncate.
    fn maintain_density(mut indices: Vec<u16>) -> Vec<u16> {
        indices.sort_unstable();
        indices.dedup();
        if indices.len() > K {
            // Randomly subsample to K via partial Fisher-Yates
            let mut rng = rand::thread_rng();
            for i in (K..indices.len()).rev() {
                let j = rng.gen_range(0..=i);
                indices.swap(i, j);
            }
            indices.truncate(K);
            indices.sort_unstable();
            indices
        } else if indices.len() < K && K > 0 {
            let mut set: HashSet<u16> = indices.iter().copied().collect();
            let mut rng = rand::thread_rng();
            while set.len() < K {
                set.insert(rng.gen_range(0..DIM as u16));
            }
            let mut result: Vec<u16> = set.into_iter().collect();
            result.sort_unstable();
            result
        } else {
            indices
        }
    }

    /// Randomly sample k elements from a slice (Fisher-Yates partial).
    fn sample_k(items: &[u16], k: usize) -> Vec<u16> {
        if k >= items.len() {
            return items.to_vec();
        }
        if k == 0 {
            return Vec::new();
        }
        let mut rng = rand::thread_rng();
        let mut result = items.to_vec();
        for i in (result.len() - k..result.len()).rev() {
            let j = rng.gen_range(0..=i);
            result.swap(i, j);
        }
        result[result.len() - k..].to_vec()
    }
}

impl<const DIM: usize, const K: usize> BinaryVsaBackend for SparseBinaryVSA<DIM, K> {
    fn bind(&self, a: &[u8], b: &[u8]) -> Vec<u8> {
        let a_dense: Vec<u8> = a.to_vec();
        let b_dense: Vec<u8> = b.to_vec();
        a_dense.iter().zip(&b_dense).map(|(x, y)| x ^ y).collect()
    }

    fn unbind(&self, c: &[u8], a: &[u8]) -> Vec<u8> {
        self.bind(c, a)
    }

    fn bundle(&self, vectors: &[&[u8]]) -> Vec<u8> {
        if vectors.is_empty() {
            return vec![0u8; DIM];
        }
        let mut result = vec![0u8; DIM];
        for v in vectors {
            for (r, x) in result.iter_mut().zip(v.iter()) {
                *r ^= x;
            }
        }
        result
    }

    fn permute(&self, v: &[u8], shift: isize) -> Vec<u8> {
        let len = v.len();
        let mut result = vec![0u8; len];
        for (i, item) in result.iter_mut().enumerate() {
            let src = ((i as isize - shift).rem_euclid(len as isize)) as usize;
            *item = v[src];
        }
        result
    }

    fn similarity(&self, a: &[u8], b: &[u8]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let same = a.iter().zip(b).filter(|(x, y)| x == y).count();
        same as f64 / a.len() as f64
    }

    fn dimensions(&self) -> usize {
        DIM
    }

    fn name(&self) -> &str {
        "sparse-binary-vsa"
    }

    fn to_bits(&self, v: &[u8]) -> Vec<u8> {
        v.to_vec()
    }

    fn to_dense(&self, v: &[u8]) -> Vec<f64> {
        v.iter().map(|&x| if x > 0 { 1.0 } else { 0.0 }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type SparseVSA = SparseBinaryVSA<4096, 32>;

    #[test]
    fn test_random_creation() {
        let v = SparseVSA::random(42);
        assert!(v.0.len() <= 32, "should have at most K indices");
        assert!(!v.0.is_empty(), "should have some indices");
        for i in 1..v.0.len() {
            assert!(v.0[i - 1] < v.0[i], "indices must be sorted and unique");
        }
    }

    #[test]
    fn test_indices_within_dim() {
        let v = SparseVSA::random(42);
        for &idx in &v.0 {
            assert!((idx as usize) < 4096, "index must be within DIM");
        }
    }

    #[test]
    fn test_similarity_of_same_is_one() {
        let v = SparseVSA::random(42);
        let sim = SparseVSA::similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_similarity_of_different_vectors() {
        let a = SparseVSA::random(42);
        let b = SparseVSA::random(123);
        let sim = SparseVSA::similarity(&a, &b);
        // Expected Jaccard ≈ K²/DIM / (2K - K²/DIM) ≈ 0.0039 for K=32, DIM=4096
        assert!(
            sim < 0.2,
            "different sparse vectors should have low similarity"
        );
    }

    #[test]
    fn test_bind_roundtrip() {
        let a = SparseVSA::random(42);
        let b = SparseVSA::random(123);
        let bound = SparseVSA::bind(&a, &b);
        let recovered = SparseVSA::unbind(&bound, &a);
        let sim = SparseVSA::similarity(&recovered, &b);
        // Density maintenance may lose some elements, but should approximate recovery
        assert!(sim > 0.5, "unbind(bind(a,b),a) ≈ b: sim={}", sim);
    }

    #[test]
    fn test_bind_self_returns_random() {
        let a = SparseVSA::random(42);
        let bound = SparseVSA::bind(&a, &a);
        // XOR self = empty set; after density maintenance gets random fill
        assert_eq!(bound.0.len(), 32);
        let sim = SparseVSA::similarity(&bound, &a);
        assert!(sim < 0.2, "self-bind should be dissimilar");
    }

    #[test]
    fn test_bundle_threshold() {
        let a = SparseVSA::random(42);
        let b = SparseVSA::random(123);
        let c = SparseVSA::random(456);
        let bundled = SparseVSA::bundle(&[&a, &b, &c]);
        assert_eq!(bundled.0.len(), 32);
    }

    #[test]
    fn test_permute_roundtrip() {
        let v = SparseVSA::random(42);
        let shifted = SparseVSA::permute(&v, 100);
        let back = SparseVSA::permute(&shifted, -100);
        let sim = SparseVSA::similarity(&back, &v);
        assert!(sim > 0.5, "permute roundtrip should be close: sim={}", sim);
    }

    #[test]
    fn test_permute_magnitude_mod_dim() {
        let v = SparseVSA::random(42);
        let shifted = SparseVSA::permute(&v, 4096); // full wrap = no change modulo 4096
        let sim = SparseVSA::similarity(&shifted, &v);
        assert!(sim > 0.5, "permute by DIM should approximately roundtrip",);
    }

    #[test]
    fn test_negate_produces_disjoint_set() {
        let v = SparseVSA::random(42);
        let neg = SparseVSA::negate(&v);
        let intersection = SparseBinaryVSA::<4096, 32>::intersection_count(&v.0, &neg.0);
        assert_eq!(intersection, 0, "negate should produce disjoint sets");
        assert_eq!(neg.0.len(), 32);
    }

    #[test]
    fn test_to_from_dense_roundtrip() {
        let v = SparseVSA::random(42);
        let dense = v.to_dense();
        assert_eq!(dense.len(), 4096);
        let recovered = SparseVSA::from_dense(&dense);
        assert_eq!(v, recovered, "to/from dense should roundtrip exactly");
    }

    #[test]
    fn test_density_constraint() {
        for seed in 0..10 {
            let v = SparseVSA::random(seed);
            assert!(
                v.0.len() <= 32,
                "seed={} len={} exceeds K=32",
                seed,
                v.0.len()
            );
        }
    }

    #[test]
    fn test_intersection_same_seed() {
        let a = SparseVSA::random(42);
        let b = SparseVSA::random(42);
        let inter = SparseVSA::intersection_size(&a, &b);
        assert_eq!(inter, a.0.len() as u32);
    }

    #[test]
    fn test_intersection_different_seed() {
        let a = SparseVSA::random(42);
        let b = SparseVSA::random(9999);
        let inter = SparseVSA::intersection_size(&a, &b);
        // Should be very small intersection for random vectors
        assert!(inter < 5);
    }

    #[test]
    fn test_different_seeds_different_vectors() {
        let a = SparseVSA::random(42);
        let b = SparseVSA::random(99);
        assert_ne!(a, b, "different seeds should produce different vectors");
    }

    #[test]
    fn test_from_dense_all_zeros() {
        let dense = vec![0u8; 4096];
        let v = SparseVSA::from_dense(&dense);
        assert_eq!(v.0.len(), 0);
    }

    #[test]
    fn test_from_dense_all_ones() {
        let dense = vec![1u8; 4096];
        let v = SparseVSA::from_dense(&dense);
        assert!(v.0.len() <= 32);
        assert!(v.0.len() > 0);
    }

    #[test]
    fn test_bind_commutative() {
        let a = SparseVSA::random(42);
        let b = SparseVSA::random(123);
        let ab = SparseVSA::bind(&a, &b);
        let ba = SparseVSA::bind(&b, &a);
        assert_eq!(ab, ba, "bind (XOR) should be commutative");
    }

    #[test]
    fn test_bind_double_inverse() {
        let a = SparseVSA::random(42);
        let b = SparseVSA::random(123);
        let ab = SparseVSA::bind(&a, &b);
        let aba = SparseVSA::bind(&ab, &b);
        let sim = SparseVSA::similarity(&aba, &a);
        assert!(sim > 0.5, "double inverse: sim={}", sim);
    }

    #[test]
    fn test_empty_bundle_returns_empty() {
        let bundled = SparseVSA::bundle(&[]);
        assert_eq!(bundled.0.len(), 0);
    }

    #[test]
    fn test_default_is_empty() {
        let v: SparseBinaryVSA<4096, 32> = SparseBinaryVSA::default();
        assert!(v.0.is_empty());
    }

    #[test]
    fn test_from_indices_enforces_invariants() {
        let indices = vec![5, 3, 1, 3, 7];
        let v = SparseBinaryVSA::<4096, 32>::from_indices(indices);
        assert_eq!(v.0, vec![1, 3, 5, 7]);
    }

    #[test]
    fn test_density_formula() {
        let v = SparseVSA::random(42);
        let d = v.density();
        assert!((d - 32.0 / 4096.0).abs() < 1e-6);
    }

    #[test]
    fn test_k0_edge_case() {
        let v = SparseBinaryVSA::<4096, 0>::random(42);
        assert!(v.0.is_empty());
        let v2 = SparseBinaryVSA::<4096, 0>::random(99);
        let sim = SparseBinaryVSA::<4096, 0>::similarity(&v, &v2);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_bind_preserves_density() {
        for seed in 0..5 {
            let a = SparseVSA::random(seed);
            let b = SparseVSA::random(seed + 100);
            let ab = SparseVSA::bind(&a, &b);
            assert_eq!(
                ab.0.len(),
                32,
                "seed={}: bind must preserve K density",
                seed
            );
        }
    }

    #[test]
    fn test_symmetric_difference_merge() {
        let a: Vec<u16> = vec![1, 3, 5, 7];
        let b: Vec<u16> = vec![3, 7, 9];
        let diff = SparseBinaryVSA::<4096, 32>::symmetric_difference(&a, &b);
        assert_eq!(diff, vec![1, 5, 9]);
    }

    #[test]
    fn test_intersection_count_merge() {
        let a: Vec<u16> = vec![1, 3, 5, 7];
        let b: Vec<u16> = vec![3, 7, 9];
        let cnt = SparseBinaryVSA::<4096, 32>::intersection_count(&a, &b);
        assert_eq!(cnt, 2);
    }
}
