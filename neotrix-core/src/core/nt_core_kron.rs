use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;

const DEFAULT_BLOCK_SIZE: usize = 2;

/// Kronecker-structured rotation for O(N log N) approximate nearest neighbor search.
///
/// Applies a chain of Givens rotation matrices structured as Kronecker products
/// of 2×2 rotation blocks. The rotation is deterministic from a fixed seed (42).
///
/// Reference: 2025 arXiv paper on Linearithmic Cleanup with Kronecker Product
/// Rotation Matrices.
pub struct KroneckerCleanup {
    dim: usize,
    num_stages: usize,
    stages: Vec<Vec<f64>>,
}

impl KroneckerCleanup {
    /// Create a new KroneckerCleanup for vectors of dimension `dim`.
    /// Dimension must be a positive power of two.
    pub fn new(dim: usize) -> Self {
        assert!(
            dim > 0 && dim.is_power_of_two(),
            "dim must be a positive power of two, got {}",
            dim
        );
        let num_stages = (dim as f64).log2() as usize;
        let mut rng = StdRng::seed_from_u64(42);

        let stages: Vec<Vec<f64>> = (0..num_stages)
            .map(|_| {
                (0..dim / DEFAULT_BLOCK_SIZE)
                    .map(|_| rng.gen_range(0.0..2.0 * std::f64::consts::PI))
                    .collect()
            })
            .collect();

        Self {
            dim,
            num_stages,
            stages,
        }
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn num_stages(&self) -> usize {
        self.num_stages
    }

    /// Apply the Kronecker-structured rotation to a vector.
    ///
    /// The rotation is composed of `num_stages` stages, each applying
    /// 2×2 Givens rotations followed by a perfect-shuffle permutation.
    /// This runs in O(dim log dim) time instead of O(dim²) for a full matrix.
    pub fn rotate(&self, vec: &[f64]) -> Vec<f64> {
        assert_eq!(vec.len(), self.dim);
        let mut result = vec.to_vec();

        for stage in 0..self.num_stages {
            let angles = &self.stages[stage];
            for block in 0..self.dim / DEFAULT_BLOCK_SIZE {
                let i = block * DEFAULT_BLOCK_SIZE;
                let angle = angles[block];
                let (cos, sin) = (angle.cos(), angle.sin());
                let x = result[i];
                let y = result[i + 1];
                result[i] = cos * x - sin * y;
                result[i + 1] = sin * x + cos * y;
            }
            if stage < self.num_stages - 1 {
                result = perfect_out_shuffle(&result);
            }
        }

        result
    }

    /// Rotate query and all candidate items, then find top_k by cosine
    /// similarity in rotated space.
    ///
    /// This is the O(N log N) cleanup — the rotation distributes information
    /// so that similarity in rotated space approximates the original.
    pub fn cleanup(
        &self,
        query: &[f64],
        items: &[(String, Vec<f64>)],
        top_k: usize,
    ) -> Vec<(String, f64)> {
        if items.is_empty() || top_k == 0 {
            return Vec::new();
        }
        let rotated_query = self.rotate(query);

        let mut scored: Vec<(String, f64)> = items
            .iter()
            .map(|(id, vec)| {
                let rotated_vec = self.rotate(vec);
                let sim = cosine_similarity(&rotated_query, &rotated_vec);
                (id.clone(), sim)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }
}

/// Perfect out-shuffle: interleave first half with second half.
fn perfect_out_shuffle(arr: &[f64]) -> Vec<f64> {
    let n = arr.len();
    let half = n / 2;
    let mut result = Vec::with_capacity(n);
    for i in 0..half {
        result.push(arr[i]);
        result.push(arr[i + half]);
    }
    result
}

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 && norm_b == 0.0 {
        return 1.0;
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotate_preserves_norm() {
        let cleanup = KroneckerCleanup::new(64);
        let vec: Vec<f64> = (0..64).map(|i| (i as f64).sin()).collect();
        let original_norm: f64 = vec.iter().map(|x| x * x).sum::<f64>().sqrt();
        let rotated = cleanup.rotate(&vec);
        let rotated_norm: f64 = rotated.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!(
            (original_norm - rotated_norm).abs() < 1e-10,
            "norm preserved: {} vs {}",
            original_norm,
            rotated_norm
        );
    }

    #[test]
    fn test_cleanup_returns_top_k() {
        let cleanup = KroneckerCleanup::new(8);
        let query = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let items = vec![
            ("a".to_string(), vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("b".to_string(), vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("c".to_string(), vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        ];
        let result = cleanup.cleanup(&query, &items, 2);
        assert_eq!(result.len(), 2);
        // top result should be "a" (most similar to query)
        assert_eq!(result[0].0, "a");
    }

    #[test]
    fn test_identical_vectors_high_similarity() {
        let cleanup = KroneckerCleanup::new(8);
        let query = vec![0.5, 0.5, 0.3, 0.1, 0.0, 0.0, 0.0, 0.0];
        let items = vec![("identical".to_string(), query.clone())];
        let result = cleanup.cleanup(&query, &items, 1);
        assert_eq!(result.len(), 1);
        assert!(
            result[0].1 > 0.999,
            "identical vectors should have near-1.0 similarity, got {}",
            result[0].1
        );
    }

    #[test]
    fn test_deterministic_rotation() {
        let c1 = KroneckerCleanup::new(16);
        let c2 = KroneckerCleanup::new(16);
        let vec: Vec<f64> = (0..16).map(|i| i as f64).collect();
        let r1 = c1.rotate(&vec);
        let r2 = c2.rotate(&vec);
        assert_eq!(r1, r2, "rotation must be deterministic from seed");
    }

    #[test]
    fn test_empty_items() {
        let cleanup = KroneckerCleanup::new(8);
        let query = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let result = cleanup.cleanup(&query, &[], 5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_rotate_preserves_dot_product_under_kronecker() {
        let cleanup = KroneckerCleanup::new(16);
        let a: Vec<f64> = (0..16).map(|i| (i as f64).cos()).collect();
        let b: Vec<f64> = (0..16).map(|i| (i as f64).sin()).collect();
        let dot_original: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let ra = cleanup.rotate(&a);
        let rb = cleanup.rotate(&b);
        let dot_rotated: f64 = ra.iter().zip(rb.iter()).map(|(x, y)| x * y).sum();
        // Rotation preserves dot product (within floating point tolerance)
        assert!(
            (dot_original - dot_rotated).abs() < 1e-10,
            "dot product preserved: {} vs {}",
            dot_original,
            dot_rotated
        );
    }

    #[test]
    fn test_top_k_respects_k() {
        let cleanup = KroneckerCleanup::new(8);
        let query = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let items: Vec<(String, Vec<f64>)> = (0..10)
            .map(|i| {
                let mut v = vec![0.0_f64; 8];
                v[i % 8] = 1.0;
                (format!("item_{}", i), v)
            })
            .collect();
        for k in [0, 1, 3, 5] {
            let result = cleanup.cleanup(&query, &items, k);
            assert_eq!(result.len(), k, "top_k={} should return {} items", k, k);
        }
    }
}
