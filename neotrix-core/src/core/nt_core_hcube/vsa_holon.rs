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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_holon_backend_new() {
        let b = HolonBackend::new(256);
        assert_eq!(b.dimensions(), 256);
    }

    #[test]
    fn test_holon_backend_name() {
        let b = HolonBackend::new(64);
        assert_eq!(b.name(), "holon-simd-vsa");
    }

    #[test]
    fn test_bind_returns_same_length() {
        let b = HolonBackend::new(64);
        let a = vec![1.0; 64];
        let c = vec![0.5; 64];
        let result = b.bind(&a, &c);
        assert_eq!(result.len(), 64);
    }

    #[test]
    fn test_permute_preserves_length() {
        let b = HolonBackend::new(32);
        let v = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0,
                     1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0,
                     2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 2.9, 3.0,
                     3.1, 3.2];
        let result = b.permute(&v, 3);
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn test_similarity_self() {
        let b = HolonBackend::new(128);
        let v = vec![0.5; 128];
        let sim = b.similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_similarity_opposite() {
        let b = HolonBackend::new(128);
        let v1 = vec![1.0; 128];
        let v2 = vec![-1.0; 128];
        let sim = b.similarity(&v1, &v2);
        assert!((sim - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_similarity_orthogonal() {
        let b = HolonBackend::new(128);
        let v1 = vec![1.0; 128];
        let v2 = vec![0.0; 128];
        let sim = b.similarity(&v1, &v2);
        assert!(sim.abs() < 1e-6);
    }
}
