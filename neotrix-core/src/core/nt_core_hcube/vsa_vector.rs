// REVIVED Task 2 — dead_code removed
//! Unified VSA vector type system for NeoTrix.
//!
//! Provides a const-generic `VsaVector<DIM>` newtype, a `VsaBackend` trait
//! parameterized over dimension, a MAP VSA implementation, and conversion
//! helpers bridging existing VSA representations.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// VsaError
// ---------------------------------------------------------------------------

/// Errors for VSA vector operations.
#[derive(Debug, Clone, PartialEq)]
pub enum VsaError {
    /// Vector length does not match the declared dimension.
    DimensionMismatch { expected: usize, got: usize },
    /// Bundle operation received an empty slice.
    EmptyBundle,
    /// Cleanup operation received an empty codebook.
    EmptyCodebook,
}

impl std::fmt::Display for VsaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VsaError::DimensionMismatch { expected, got } => {
                write!(f, "dimension mismatch: expected {expected}, got {got}")
            }
            VsaError::EmptyBundle => write!(f, "cannot bundle empty vector list"),
            VsaError::EmptyCodebook => write!(f, "codebook is empty"),
        }
    }
}

impl std::error::Error for VsaError {}

// ---------------------------------------------------------------------------
// VsaVector
// ---------------------------------------------------------------------------

/// A fixed-dimensional VSA vector backed by `Vec<u8>`.
///
/// Each byte stores a single binary value (0 or 1) for MAP VSA semantics.
/// The const generic `DIM` defaults to 4096, matching the standard
/// NeoTrix VSA dimension.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VsaVector<const DIM: usize = 4096>(Vec<u8>);

impl<const DIM: usize> VsaVector<DIM> {
    /// Create a new zero vector (all bytes 0).
    pub fn new() -> Self {
        Self(vec![0u8; DIM])
    }

    /// Create a `VsaVector` from raw bytes, validating length matches `DIM`.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, VsaError> {
        if bytes.len() != DIM {
            return Err(VsaError::DimensionMismatch {
                expected: DIM,
                got: bytes.len(),
            });
        }
        Ok(Self(bytes))
    }

    /// Create a deterministic seeded random binary VSA vector.
    ///
    /// Each element is independently 0 or 1 with equal probability.
    /// Same seed always produces the same vector.
    pub fn random(seed: u64) -> Self {
        use rand::Rng;
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let bytes: Vec<u8> = (0..DIM)
            .map(|_| if rng.gen_bool(0.5) { 1 } else { 0 })
            .collect();
        Self(bytes)
    }

    /// View the underlying byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Consume the vector and return the inner `Vec<u8>`.
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }

    /// Convert from a dense `Vec<f64>` representation (as used by `VSAEngine`).
    ///
    /// Values ≥ 0.0 map to 1, values < 0.0 map to 0.
    pub fn from_f64_dense(v: &[f64]) -> Self {
        let bytes: Vec<u8> = v.iter().map(|&x| if x >= 0.0 { 1 } else { 0 }).collect();
        let mut result = vec![0u8; DIM];
        let n = bytes.len().min(DIM);
        result[..n].copy_from_slice(&bytes[..n]);
        Self(result)
    }

    /// Convert to a dense `Vec<f64>` compatible with `VSAEngine` operations.
    ///
    /// 0 → -1.0, 1 → 1.0 (bipolar representation).
    pub fn to_f64_dense(&self) -> Vec<f64> {
        self.0
            .iter()
            .map(|&b| if b == 0 { -1.0 } else { 1.0 })
            .collect()
    }

    /// Count how many bits (non-zero bytes) are set.
    pub fn popcount(&self) -> u32 {
        self.0.iter().map(|&b| (b & 1) as u32).sum()
    }

    /// Dimensionality of this vector.
    pub const fn dim() -> usize {
        DIM
    }

    /// Create a deterministic binary VSA vector from text using fold-hash → seeded random.
    /// Uses the same seed derivation as `CapabilitySynthesizer::encode` for cross-system
    /// deterministic mapping: same text → same seed. Output is binary (0/1), not FFT-HRR.
    /// Use `to_f64_dense()` / `from_f64_dense()` to bridge to real-valued VSA systems.
    pub fn from_text(text: &str) -> Self {
        use rand::Rng;
        use rand::SeedableRng;
        let seed: u64 = text
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let bytes: Vec<u8> = (0..DIM).map(|_| {
            let v: u8 = rng.gen();
            if v >= 128 { 1 } else { 0 }
        }).collect();
        Self(bytes)
    }
}

impl<const DIM: usize> Default for VsaVector<DIM> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// VsaBackend trait
// ---------------------------------------------------------------------------

/// Trait abstracting VSA operations over a unified binary vector type.
pub trait VsaBackend<const DIM: usize = 4096> {
    /// Bind two vectors (XOR in MAP VSA).
    fn bind(&self, a: &VsaVector<DIM>, b: &VsaVector<DIM>) -> VsaVector<DIM>;

    /// Bundle multiple vectors via majority sum.
    fn bundle(&self, vectors: &[&VsaVector<DIM>]) -> VsaVector<DIM>;

    /// Compute similarity in [0, 1] between two vectors.
    fn similarity(&self, a: &VsaVector<DIM>, b: &VsaVector<DIM>) -> f64;

    /// Find the nearest neighbor index in a codebook by similarity.
    /// Returns `None` if the codebook is empty.
    fn cleanup(&self, query: &VsaVector<DIM>, codebook: &[VsaVector<DIM>]) -> Option<usize>;

    /// Measure how well `target` is bound into `probe`.
    ///
    /// For MAP VSA: if `probe = bind(target, x)` for some x, then `probe` and
    /// `target` are orthogonal. Returns `1.0 - similarity(probe, target)`.
    fn is_bound(&self, probe: &VsaVector<DIM>, target: &VsaVector<DIM>) -> f64;

    /// Measure how strongly `item` is present in a `bundle`.
    ///
    /// For MAP VSA majority-sum bundling, component similarity is preserved:
    /// returns `similarity(bundle, item)`.
    fn contains(&self, bundle: &VsaVector<DIM>, item: &VsaVector<DIM>) -> f64;
}

// ---------------------------------------------------------------------------
// MapVsaBackend
// ---------------------------------------------------------------------------

/// MAP (Multiply-Add-Permute) VSA backend operating on binary `VsaVector`.
///
/// Operations:
/// - **bind**: element-wise XOR
/// - **bundle**: element-wise majority sum (threshold at `DIM/2`)
/// - **similarity**: normalized hamming distance → `1 - hamming/DIM`
/// - **cleanup**: nearest-neighbor lookup by similarity
/// - **is_bound**: orthogonality proxy — `1 - sim(probe, target)`
/// - **contains**: bundle membership — `sim(bundle, item)`
#[derive(Debug, Clone, Copy, Default)]
pub struct MapVsaBackend;

impl<const DIM: usize> VsaBackend<DIM> for MapVsaBackend {
    fn bind(&self, a: &VsaVector<DIM>, b: &VsaVector<DIM>) -> VsaVector<DIM> {
        let bytes: Vec<u8> = a
            .as_bytes()
            .iter()
            .zip(b.as_bytes().iter())
            .map(|(x, y)| x ^ y)
            .collect();
        VsaVector(bytes)
    }

    fn bundle(&self, vectors: &[&VsaVector<DIM>]) -> VsaVector<DIM> {
        if vectors.is_empty() {
            return VsaVector::new();
        }
        let n = vectors.len();
        let mut counts = vec![0i32; DIM];
        for v in vectors {
            for (c, &b) in counts.iter_mut().zip(v.as_bytes().iter()) {
                if b != 0 {
                    *c += 1;
                }
            }
        }
        let threshold = (n as i32) / 2;
        let bytes: Vec<u8> = counts
            .iter()
            .map(|&c| if c > threshold { 1 } else { 0 })
            .collect();
        VsaVector(bytes)
    }

    fn similarity(&self, a: &VsaVector<DIM>, b: &VsaVector<DIM>) -> f64 {
        let dist: u32 = a
            .as_bytes()
            .iter()
            .zip(b.as_bytes().iter())
            .map(|(x, y)| ((x ^ y) & 1) as u32)
            .sum();
        1.0 - (dist as f64 / DIM as f64)
    }

    fn cleanup(&self, query: &VsaVector<DIM>, codebook: &[VsaVector<DIM>]) -> Option<usize> {
        if codebook.is_empty() {
            return None;
        }
        codebook
            .iter()
            .enumerate()
            .map(|(i, c)| (i, self.similarity(query, c)))
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
    }

    fn is_bound(&self, probe: &VsaVector<DIM>, target: &VsaVector<DIM>) -> f64 {
        1.0 - self.similarity(probe, target)
    }

    fn contains(&self, bundle: &VsaVector<DIM>, item: &VsaVector<DIM>) -> f64 {
        self.similarity(bundle, item)
    }
}

// ---------------------------------------------------------------------------
// Conversion helpers (DIM = 4096, the standard NeoTrix dimension)
// ---------------------------------------------------------------------------

/// Convert a `VsaVector<4096>` to the `Vec<f64>` format used by `VSAEngine`.
pub fn vsa_vector_to_f64(v: &VsaVector<4096>) -> Vec<f64> {
    v.to_f64_dense()
}

/// Convert a `Vec<f64>` (e.g., from `VSAEngine::bind`) to a `VsaVector<4096>`.
pub fn f64_to_vsa_vector(v: &[f64]) -> VsaVector<4096> {
    VsaVector::from_f64_dense(v)
}

/// Convert raw `Vec<u8>` (as used by `QuantizedVSA`) to `VsaVector<4096>`.
pub fn bytes_to_vsa_vector(bytes: Vec<u8>) -> Result<VsaVector<4096>, VsaError> {
    VsaVector::from_bytes(bytes)
}

/// Convert `VsaVector<4096>` back to raw `Vec<u8>`.
pub fn vsa_vector_to_bytes(v: VsaVector<4096>) -> Vec<u8> {
    v.into_inner()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    type V4096 = VsaVector<4096>;

    fn backend() -> MapVsaBackend {
        MapVsaBackend
    }

    #[test]
    fn test_new_zero_vector() {
        let v = V4096::new();
        assert_eq!(v.as_bytes().len(), 4096);
        assert!(v.as_bytes().iter().all(|&b| b == 0));
        assert_eq!(v.popcount(), 0);
    }

    #[test]
    fn test_random_deterministic() {
        let a = V4096::random(42);
        let b = V4096::random(42);
        assert_eq!(a, b, "same seed must produce identical vectors");

        let c = V4096::random(99);
        assert_ne!(a, c, "different seeds should produce different vectors");
    }

    #[test]
    fn test_random_distribution_near_balanced() {
        let v = V4096::random(123);
        let pc = v.popcount();
        assert!(
            pc > 1800 && pc < 2200,
            "popcount {pc} should be near 2048 for DIM=4096"
        );
    }

    #[test]
    fn test_from_bytes_validates_length() {
        let ok = vec![0u8; 4096];
        assert!(V4096::from_bytes(ok).is_ok());

        let short = vec![0u8; 128];
        let err = V4096::from_bytes(short).unwrap_err();
        assert_eq!(
            err,
            VsaError::DimensionMismatch {
                expected: 4096,
                got: 128
            }
        );
    }

    #[test]
    fn test_bind_xor() {
        let b = backend();
        let a = V4096::random(42);
        let c = V4096::random(99);

        let bound = b.bind(&a, &c);
        assert_ne!(bound, a);
        assert_ne!(bound, c);

        let self_bound = b.bind(&a, &a);
        assert_eq!(self_bound, V4096::new(), "bind(a,a) must be zero");
    }

    #[test]
    fn test_bind_self_inverse() {
        let b = backend();
        let a = V4096::random(42);
        let c = V4096::random(99);
        let bound = b.bind(&a, &c);
        let roundtrip = b.bind(&bound, &c);
        assert_eq!(roundtrip, a, "bind(bind(a,b),b) should recover a");
    }

    #[test]
    fn test_bundle_majority() {
        let b = backend();
        let a = V4096::random(10);
        let c = V4096::random(20);
        let d = V4096::random(30);

        let bundled = b.bundle(&[&a, &c, &d]);
        assert_eq!(bundled.as_bytes().len(), 4096);

        let sim_a = b.similarity(&bundled, &a);
        let sim_c = b.similarity(&bundled, &c);
        let sim_d = b.similarity(&bundled, &d);
        assert!(
            sim_a > 0.4 || sim_c > 0.4 || sim_d > 0.4,
            "bundled should be similar to at least one component"
        );
    }

    #[test]
    fn test_bundle_empty_returns_zero() {
        let b = backend();
        let bundled = b.bundle(&[]);
        assert_eq!(bundled, V4096::new());
    }

    #[test]
    fn test_bundle_identical_vectors() {
        let b = backend();
        let a = V4096::random(42);
        let bundled = b.bundle(&[&a, &a, &a]);
        assert_eq!(bundled, a, "all-identical bundle should equal the vector");
    }

    #[test]
    fn test_similarity_identical() {
        let b = backend();
        let a = V4096::random(42);
        let sim = b.similarity(&a, &a);
        assert!(
            (sim - 1.0).abs() < 1e-10,
            "self-similarity must be 1.0, got {sim}"
        );
    }

    #[test]
    fn test_similarity_orthogonal() {
        let b = backend();
        let zeros = V4096::new();
        let ones = VsaVector::<4096>(vec![1u8; 4096]);
        let sim = b.similarity(&zeros, &ones);
        assert!(
            (sim - 0.0).abs() < 1e-10,
            "all-zero vs all-one similarity must be 0.0, got {sim}"
        );
    }

    #[test]
    fn test_similarity_range() {
        let b = backend();
        let a = V4096::random(1);
        let c = V4096::random(2);
        let sim = b.similarity(&a, &c);
        assert!(
            sim >= 0.0 && sim <= 1.0,
            "similarity must be in [0,1], got {sim}"
        );
    }

    #[test]
    fn test_cleanup_finds_nearest() {
        let b = backend();
        let query = V4096::random(100);
        let mut codebook: Vec<V4096> = (0..20).map(|s| V4096::random(s)).collect();
        codebook[5] = query.clone();

        let idx = b.cleanup(&query, &codebook);
        assert_eq!(idx, Some(5));
    }

    #[test]
    fn test_cleanup_empty_codebook() {
        let b = backend();
        let query = V4096::random(42);
        assert_eq!(b.cleanup(&query, &[]), None);
    }

    #[test]
    fn test_is_bound_identical_is_zero() {
        let b = backend();
        let a = V4096::random(42);
        let boundness = b.is_bound(&a, &a);
        assert!(
            (boundness - 0.0).abs() < 1e-10,
            "identical vectors have is_bound = 0, got {boundness}"
        );
    }

    #[test]
    fn test_contains_self() {
        let b = backend();
        let a = V4096::random(42);
        let bundle = b.bundle(&[&a]);
        let c = b.contains(&bundle, &a);
        assert!(
            (c - 1.0).abs() < 1e-10,
            "bundle([a]) should fully contain a, got {c}"
        );
    }

    #[test]
    fn test_from_f64_dense_roundtrip() {
        let v = V4096::random(42);
        let dense = v.to_f64_dense();
        let recovered = V4096::from_f64_dense(&dense);
        assert_eq!(v, recovered);
    }

    #[test]
    fn test_conversion_helpers_roundtrip() {
        let v = V4096::random(42);

        let f64v = vsa_vector_to_f64(&v);
        let back = f64_to_vsa_vector(&f64v);
        assert_eq!(v, back);

        let bytes = vsa_vector_to_bytes(v.clone());
        let from_bytes = bytes_to_vsa_vector(bytes).unwrap();
        assert_eq!(v, from_bytes);
    }

    #[test]
    fn test_dim_constant() {
        assert_eq!(V4096::dim(), 4096);
    }
}
