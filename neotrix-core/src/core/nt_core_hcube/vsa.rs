/// Trait abstracting VSA operations over different backends.
pub trait VsaBackend {
    fn bind(&self, a: &[f64], b: &[f64]) -> Vec<f64>;
    fn bundle(&self, vectors: &[&[f64]]) -> Vec<f64>;
    fn permute(&self, v: &[f64], shift: isize) -> Vec<f64>;
    fn similarity(&self, a: &[f64], b: &[f64]) -> f64;
    fn dimensions(&self) -> usize;
    fn name(&self) -> &str;
}

/// Default MAP-based VSA engine on real-valued vectors.
pub struct VSAEngine {
    dim: usize,
}

impl VSAEngine {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
}

impl VsaBackend for VSAEngine {
    fn bind(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).collect()
    }

    fn bundle(&self, vectors: &[&[f64]]) -> Vec<f64> {
        let dim = vectors[0].len();
        let mut result = vec![0.0; dim];
        for v in vectors {
            for (r, x) in result.iter_mut().zip(v.iter()) {
                *r += x;
            }
        }
        result
    }

    fn permute(&self, v: &[f64], shift: isize) -> Vec<f64> {
        let len = v.len();
        let mut result = vec![0.0; len];
        for (i, item) in result.iter_mut().enumerate() {
            let src = ((i as isize - shift).rem_euclid(len as isize)) as usize;
            *item = v[src];
        }
        result
    }

    fn similarity(&self, a: &[f64], b: &[f64]) -> f64 {
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

    fn dimensions(&self) -> usize {
        self.dim
    }

    fn name(&self) -> &str {
        "map-vsa"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> VSAEngine {
        VSAEngine::new(4096)
    }

    #[test]
    fn test_bind_different_from_inputs() {
        let e = engine();
        let a: Vec<f64> = (0..4096).map(|i| (i as f64).sin()).collect();
        let b: Vec<f64> = (0..4096).map(|i| (i as f64).cos()).collect();
        let c = e.bind(&a, &b);
        let sim_a = e.similarity(&c, &a);
        let sim_b = e.similarity(&c, &b);
        assert!(sim_a.abs() < 0.1);
        assert!(sim_b.abs() < 0.1);
    }

    #[test]
    fn test_bundle_similar_to_all_components() {
        let e = engine();
        let a: Vec<f64> = (0..4096).map(|i| (i as f64).sin()).collect();
        let b: Vec<f64> = (0..4096).map(|i| (i as f64).cos()).collect();
        let c = e.bundle(&[&a, &b]);
        assert!(e.similarity(&c, &a) > 0.5);
        assert!(e.similarity(&c, &b) > 0.5);
    }

    #[test]
    fn test_permute_reversible() {
        let e = engine();
        let v: Vec<f64> = (0..4096).map(|i| (i as f64).sin()).collect();
        let p = e.permute(&v, 100);
        let r = e.permute(&p, -100);
        let sim = e.similarity(&r, &v);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_self_similarity_one() {
        let e = engine();
        let v: Vec<f64> = (0..4096).map(|i| (i as f64).sin()).collect();
        let sim = e.similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_dimensions() {
        let e = VSAEngine::new(1024);
        assert_eq!(e.dimensions(), 1024);
    }
}
