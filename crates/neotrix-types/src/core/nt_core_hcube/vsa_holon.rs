use holon::{Primitives, Similarity, Vector};

use super::vsa::VsaBackend;

/// SIMD-accelerated VSA backend backed by holon-rs (bipolar vectors).
pub struct HolonBackend {
    dim: usize,
}

impl HolonBackend {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
}

impl VsaBackend for HolonBackend {
    fn bind(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        let va = Vector::from_f64(a);
        let vb = Vector::from_f64(b);
        let bound = Primitives::bind(&va, &vb);
        bound.to_f64()
    }

    fn bundle(&self, vectors: &[&[f64]]) -> Vec<f64> {
        let holon_vecs: Vec<Vector> = vectors.iter().map(|v| Vector::from_f64(v)).collect();
        let refs: Vec<&Vector> = holon_vecs.iter().collect();
        let bundled = Primitives::bundle(&refs);
        bundled.to_f64()
    }

    fn permute(&self, v: &[f64], shift: isize) -> Vec<f64> {
        let vv = Vector::from_f64(v);
        let permuted = Primitives::permute(&vv, shift as i32);
        permuted.to_f64()
    }

    fn similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        let va = Vector::from_f64(a);
        let vb = Vector::from_f64(b);
        Similarity::cosine(&va, &vb)
    }

    fn dimensions(&self) -> usize {
        self.dim
    }

    fn name(&self) -> &str {
        "holon-simd-vsa"
    }
}
