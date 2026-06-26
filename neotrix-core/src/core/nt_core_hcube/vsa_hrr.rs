//! HRR (Holographic Reduced Representation) binding backend.
//! Uses circular convolution in frequency domain via FFT (rustfft).
//! All vectors are f64; all operations produce unit-length normalized vectors.

use num_complex::Complex64;
use rand::Rng;
use rustfft::FftPlanner;

/// HRR VSA backend operating on f64 vectors with FFT-based binding.
#[derive(Debug, Clone)]
pub struct HrrBackend {
    dimension: usize,
}

impl HrrBackend {
    pub fn new(dimension: usize) -> Self {
        assert!(
            dimension.is_power_of_two(),
            "HrrBackend dimension must be a power of two, got {}",
            dimension
        );
        Self { dimension }
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Bind two vectors via circular convolution (FFT-based).
    /// FFT(a) * FFT(b) pointwise → IFFT → normalize.
    pub fn bind(a: &[f64], b: &[f64]) -> Vec<f64> {
        let n = a.len().min(b.len());
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(n);
        let ifft = planner.plan_fft_inverse(n);

        let mut fa: Vec<Complex64> = a.iter().map(|&x| Complex64::new(x, 0.0)).collect();
        let mut fb: Vec<Complex64> = b.iter().map(|&x| Complex64::new(x, 0.0)).collect();

        fft.process(&mut fa);
        fft.process(&mut fb);

        for i in 0..n {
            fa[i] = fa[i] * fb[i];
        }

        ifft.process(&mut fa);

        let result: Vec<f64> = fa.iter().take(n).map(|c| c.re).collect();
        Self::normalize(&result)
    }

    /// Unbind: approximate inverse via circular cross-correlation.
    /// IFFT(FFT(bound) * conj(FFT(key))) → normalize.
    pub fn unbind(bound: &[f64], key: &[f64]) -> Vec<f64> {
        let n = bound.len().min(key.len());
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(n);
        let ifft = planner.plan_fft_inverse(n);

        let mut f_bound: Vec<Complex64> = bound.iter().map(|&x| Complex64::new(x, 0.0)).collect();
        let mut f_key: Vec<Complex64> = key.iter().map(|&x| Complex64::new(x, 0.0)).collect();

        fft.process(&mut f_bound);
        fft.process(&mut f_key);

        for i in 0..n {
            f_bound[i] = f_bound[i] * f_key[i].conj();
        }

        ifft.process(&mut f_bound);

        let result: Vec<f64> = f_bound.iter().take(n).map(|c| c.re).collect();
        Self::normalize(&result)
    }

    /// Circular cross-correlation (same as unbind).
    pub fn correlate(a: &[f64], b: &[f64]) -> Vec<f64> {
        Self::unbind(a, b)
    }

    /// Cosine similarity between two vectors (clamped to [-1, 1]).
    pub fn similarity(a: &[f64], b: &[f64]) -> f64 {
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_a < f64::EPSILON || norm_b < f64::EPSILON {
            return 0.0;
        }
        (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
    }

    /// Generate a random VSA vector with uniform distribution in [-1, 1],
    /// then normalize to unit length.
    pub fn random_vector(&self, rng: &mut impl Rng) -> Vec<f64> {
        let v: Vec<f64> = (0..self.dimension)
            .map(|_| rng.gen::<f64>() * 2.0 - 1.0)
            .collect();
        Self::normalize(&v)
    }

    /// Bundle (superimpose) multiple vectors via normalized sum.
    pub fn bundle(vectors: &[&[f64]]) -> Vec<f64> {
        if vectors.is_empty() {
            return Vec::new();
        }
        let dim = vectors[0].len();
        let n = vectors.len() as f64;
        let mut sum = vec![0.0; dim];
        for v in vectors {
            for (s, &x) in sum.iter_mut().zip(v.iter()) {
                *s += x;
            }
        }
        for s in sum.iter_mut() {
            *s /= n;
        }
        Self::normalize(&sum)
    }

    /// Cleanup: find the candidate vector with highest similarity to `noisy`.
    /// Returns the index into `candidates`.
    pub fn cleanup(noisy: &[f64], candidates: &[Vec<f64>]) -> usize {
        let mut best_idx = 0;
        let mut best_sim = f64::NEG_INFINITY;
        for (i, c) in candidates.iter().enumerate() {
            let sim = Self::similarity(noisy, c);
            if sim > best_sim {
                best_sim = sim;
                best_idx = i;
            }
        }
        best_idx
    }

    // ─── internal helpers ───

    pub fn normalize(v: &[f64]) -> Vec<f64> {
        let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm < f64::EPSILON {
            return v.to_vec();
        }
        v.iter().map(|x| x / norm).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn backend() -> HrrBackend {
        HrrBackend::new(64) // small power-of-2 for tests
    }

    #[test]
    fn test_bind_unbind_roundtrip() {
        let backend = backend();
        let mut rng = StdRng::seed_from_u64(42);
        let a = backend.random_vector(&mut rng);
        let b = backend.random_vector(&mut rng);

        let bound = HrrBackend::bind(&a, &b);
        let rebound = HrrBackend::unbind(&bound, &b);

        let sim = HrrBackend::similarity(&a, &rebound);
        assert!(sim > 0.5, "bind/unbind roundtrip similarity too low: {sim}");
    }

    #[test]
    fn test_similarity_identical() {
        let backend = backend();
        let mut rng = StdRng::seed_from_u64(7);
        let v = backend.random_vector(&mut rng);
        let sim = HrrBackend::similarity(&v, &v);
        assert!(
            (sim - 1.0).abs() < 1e-10,
            "self-similarity should be 1.0, got {sim}"
        );
    }

    #[test]
    fn test_similarity_orthogonal() {
        let n = 64;
        let a = vec![1.0_f64 / (n as f64).sqrt(); n];
        let mut b = vec![0.0_f64; n];
        b[0] = 1.0;
        let sim = HrrBackend::similarity(&a, &b);
        assert!(
            (sim - (1.0 / (n as f64).sqrt())).abs() < 1e-10,
            "expected ~{:.4}, got {sim}",
            1.0 / (n as f64).sqrt()
        );
    }

    #[test]
    fn test_bundle_noise_reduction() {
        let backend = backend();
        let mut rng = StdRng::seed_from_u64(99);
        let prototype = backend.random_vector(&mut rng);

        let mut noisy_copies = Vec::new();
        for _ in 0..10 {
            let mut copy = prototype.clone();
            for x in copy.iter_mut() {
                *x += rng.gen::<f64>() * 0.5 - 0.25;
            }
            let copy = HrrBackend::normalize(&copy);
            noisy_copies.push(copy);
        }

        let refs: Vec<&[f64]> = noisy_copies.iter().map(|v| v.as_slice()).collect();
        let bundled = HrrBackend::bundle(&refs);

        let noisy_sim = HrrBackend::similarity(&prototype, &noisy_copies[0]);
        let bundle_sim = HrrBackend::similarity(&prototype, &bundled);
        assert!(
            bundle_sim >= noisy_sim - 0.05,
            "bundling should not reduce similarity: noise={noisy_sim:.4}, bundle={bundle_sim:.4}"
        );
    }

    #[test]
    fn test_cleanup_selects_best() {
        let backend = backend();
        let mut rng = StdRng::seed_from_u64(123);
        let target = backend.random_vector(&mut rng);
        let mut candidates = vec![target.clone()];
        for _ in 0..5 {
            candidates.push(backend.random_vector(&mut rng));
        }

        // Noise-corrupted version of target
        let mut noisy = target.clone();
        for x in noisy.iter_mut() {
            *x += rng.gen::<f64>() * 0.3;
        }
        let noisy = HrrBackend::normalize(&noisy);

        let best = HrrBackend::cleanup(&noisy, &candidates);
        assert_eq!(best, 0, "cleanup should select the original target vector");
    }

    #[test]
    fn test_normalize_unit_length() {
        let v = vec![3.0, 4.0];
        let n = HrrBackend::normalize(&v);
        let len: f64 = n.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!(
            (len - 1.0).abs() < 1e-10,
            "normalized length should be 1.0, got {len}"
        );
    }

    #[test]
    fn test_non_power_of_two_panics() {
        let result = std::panic::catch_unwind(|| HrrBackend::new(100));
        assert!(result.is_err(), "non-power-of-2 dimension should panic");
    }
}
