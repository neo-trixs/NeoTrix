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
#[derive(Debug)]
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

/// Trait for binary/quantized VSA backends.
/// Parallel to `VsaBackend` but operates on `&[u8]` (bit-packed or byte-per-bit).
pub trait BinaryVsaBackend {
    fn bind(&self, a: &[u8], b: &[u8]) -> Vec<u8>;
    fn unbind(&self, c: &[u8], a: &[u8]) -> Vec<u8>;
    fn bundle(&self, vectors: &[&[u8]]) -> Vec<u8>;
    fn permute(&self, v: &[u8], shift: isize) -> Vec<u8>;
    fn similarity(&self, a: &[u8], b: &[u8]) -> f64;
    fn dimensions(&self) -> usize;
    fn name(&self) -> &str;
    fn to_bits(&self, v: &[u8]) -> Vec<u8>;
    fn to_dense(&self, v: &[u8]) -> Vec<f64>;
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

/// Quantum-inspired VSA backend.
/// Uses superposition states (complex amplitude vectors) instead of classical vectors.
/// Behind `#[cfg(feature = "quantum")]` gate — experimental only.
#[cfg(feature = "quantum")]
pub struct QubitVsaBackend {
    dim: usize,
    /// Threshold for measurement collapse
    measurement_threshold: f64,
}

#[cfg(feature = "quantum")]
impl QubitVsaBackend {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            measurement_threshold: 0.1,
        }
    }

    /// Create a superposition state where all basis states are equally probable
    pub fn superposition(dim: usize) -> Vec<f64> {
        let norm = 1.0 / (dim as f64).sqrt();
        vec![norm; dim]
    }

    /// Quantum bind: tensor product-like operation (for VSA)
    /// For quantum VSA: bind(a, b) = normalize(hadamard_product(a, b))
    pub fn quantum_bind(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        let prod: Vec<f64> = a.iter().zip(b.iter()).map(|(x, y)| x * y).collect();
        let norm_sq: f64 = prod.iter().map(|x| x * x).sum();
        if norm_sq < 1e-12 {
            return prod;
        }
        let inv_norm = 1.0 / norm_sq.sqrt();
        prod.into_iter().map(|x| x * inv_norm).collect()
    }

    /// Quantum measure: collapse superposition to classical vector
    /// Born rule: probability = |amplitude|²
    pub fn measure(&self, state: &[f64]) -> Vec<f64> {
        let norm_sq: f64 = state.iter().map(|x| x * x).sum();
        if norm_sq < 1e-12 {
            return vec![0.0; state.len()];
        }
        let probs: Vec<f64> = state.iter().map(|x| (x * x) / norm_sq).collect();
        let mut result = vec![0.0; state.len()];
        if let Some((max_idx, _)) = probs
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        {
            if probs[max_idx] > self.measurement_threshold {
                result[max_idx] = 1.0;
            }
        }
        result
    }

    /// Amplitude amplification: boost high-probability outcomes
    /// One Grover iteration: flip target sign (oracle) then reflect about average (diffusion)
    pub fn amplify(&self, state: &[f64], target_idx: usize) -> Vec<f64> {
        let n = state.len() as f64;
        // Oracle: flip sign of target
        let mut flipped: Vec<f64> = state.to_vec();
        flipped[target_idx] = -flipped[target_idx];
        // Diffusion: reflect about average
        let avg: f64 = flipped.iter().sum::<f64>() / n;
        flipped.into_iter().map(|x| 2.0 * avg - x).collect()
    }
}

#[cfg(feature = "quantum")]
impl VsaBackend for QubitVsaBackend {
    fn bind(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        self.quantum_bind(a, b)
    }

    fn bundle(&self, vectors: &[&[f64]]) -> Vec<f64> {
        let dim = vectors[0].len();
        let mut sum = vec![0.0; dim];
        for v in vectors {
            for (s, x) in sum.iter_mut().zip(v.iter()) {
                *s += x;
            }
        }
        let norm = sum.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm < 1e-12 {
            return sum;
        }
        for x in sum.iter_mut() {
            *x /= norm;
        }
        sum
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
        dot * dot
    }

    fn dimensions(&self) -> usize {
        self.dim
    }

    fn name(&self) -> &str {
        "qubit-vsa"
    }
}

#[cfg(all(feature = "quantum", test))]
mod quantum_tests {
    use super::*;

    fn backend() -> QubitVsaBackend {
        QubitVsaBackend::new(4096)
    }

    #[test]
    fn test_superposition_norm() {
        let dim = 4096;
        let s = QubitVsaBackend::superposition(dim);
        let norm: f64 = s.iter().map(|x| x * x).sum();
        assert!((norm - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_quantum_bind_different() {
        let b = backend();
        let a: Vec<f64> = (0..4096).map(|i| (i as f64).sin()).collect();
        let c = b.quantum_bind(&a, &a);
        let sim = b.similarity(&c, &a);
        assert!(sim.abs() < 0.1);
    }

    #[test]
    fn test_quantum_measure_collapse() {
        let b = backend();
        let mut state = vec![0.0; 4096];
        state[42] = 1.0;
        let result = b.measure(&state);
        assert_eq!(result[42], 1.0);
    }

    #[test]
    fn test_amplitude_amplification() {
        let dim = 16;
        let b = QubitVsaBackend::new(dim);
        let mut state = vec![0.1; dim];
        state[0] = 0.5;
        let amplified = b.amplify(&state, 0);
        // After Grover iteration target amplitude should be amplified
        assert!(amplified[0] > 0.5);
    }

    #[test]
    fn test_quantum_bundle_normalized() {
        let b = backend();
        let a: Vec<f64> = (0..4096).map(|i| (i as f64).sin()).collect();
        let bundled = b.bundle(&[&a, &a]);
        let norm: f64 = bundled.iter().map(|x| x * x).sum();
        assert!((norm - 1.0).abs() < 1e-10);
    }
}
