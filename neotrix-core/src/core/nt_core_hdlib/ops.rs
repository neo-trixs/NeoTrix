//! Core HD computing operations.
//!
//! All operations work on `&[f64]` (the `HdVector` representation) and
//! follow MAP (Multiply-Add-Permute) VSA semantics:
//!
//! - **Bind** — element-wise multiplication (self-inverse for bipolar values)
//! - **Bundle** — element-wise summation (superposition)
//! - **Permute** — circular shift (role-filler binding)
//! - **Similarity** — cosine similarity

use crate::core::nt_core_hcube::VSAEngine;

use super::HD_DIM;

/// Cosine similarity between two HD vectors in [-1, 1].
///
/// 1.0 = identical direction, 0.0 = orthogonal, -1.0 = opposite.
pub fn similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f64 = a.iter().map(|x| x * x).sum();
    let nb: f64 = b.iter().map(|x| x * x).sum();
    let norm = na.sqrt() * nb.sqrt();
    if norm < 1e-12 {
        0.0
    } else {
        (dot / norm).clamp(-1.0, 1.0)
    }
}

/// Alias for `similarity`.
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    similarity(a, b)
}

/// Dot product of two HD vectors.
pub fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Bundle (superpose) multiple HD vectors via element-wise summation.
///
/// The result is **not normalized** — repeated bundling grows magnitude.
/// Use `normalize` on the result if a unit vector is required.
pub fn bundle(vectors: &[&[f64]]) -> Vec<f64> {
    if vectors.is_empty() {
        return vec![0.0; HD_DIM];
    }
    let dim = vectors[0].len();
    let mut result = vec![0.0; dim];
    for v in vectors {
        for (r, x) in result.iter_mut().zip(v.iter()) {
            *r += x;
        }
    }
    result
}

/// Bind two HD vectors via element-wise multiplication.
///
/// For bipolar values this is self-inverse: `bind(bind(a, b), b) ≈ a`.
pub fn bind(a: &[f64], b: &[f64]) -> Vec<f64> {
    let n = a.len().min(b.len());
    let mut result = Vec::with_capacity(n);
    for i in 0..n {
        result.push(a[i] * b[i]);
    }
    result
}

/// Unbind (decode) a bound pair.
///
/// For MAP VSA unbinding is the same as binding (element-wise multiply),
/// since `a × b × b = a` for bipolar {-1,1}. For real-valued vectors
/// this is approximate.
pub fn unbind(c: &[f64], a: &[f64]) -> Vec<f64> {
    bind(c, a)
}

/// Permute (rotate) an HD vector by `shift` positions.
///
/// Positive shift moves elements forward (right), negative backward (left).
/// Used for role-filler binding in structured representations.
pub fn permute(v: &[f64], shift: isize) -> Vec<f64> {
    let len = v.len();
    let mut result = vec![0.0; len];
    for (i, item) in result.iter_mut().enumerate() {
        let src = ((i as isize - shift).rem_euclid(len as isize)) as usize;
        *item = v[src];
    }
    result
}

/// L2-normalize a vector to unit length.
pub fn l2_normalize(v: &[f64]) -> Vec<f64> {
    let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm < 1e-12 {
        return v.to_vec();
    }
    v.iter().map(|x| x / norm).collect()
}

/// Generate a random HD vector with values in [-1, 1].
pub fn random(dim: usize) -> Vec<f64> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..dim).map(|_| rng.gen::<f64>() * 2.0 - 1.0).collect()
}

/// Generate a seeded deterministic HD vector.
pub fn seeded_random(seed: u64, dim: usize) -> Vec<f64> {
    use rand::Rng;
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    (0..dim).map(|_| rng.gen::<f64>() * 2.0 - 1.0).collect()
}

/// Create a default `VSAEngine` for `HD_DIM` (4096).
pub fn default_engine() -> VSAEngine {
    VSAEngine::new(HD_DIM)
}
