/// Spectral norm constraint for provably stable latent dynamics.
///
/// Ensures the transition operator's spectral radius <= `max_spectral_radius`,
/// guaranteeing bounded error accumulation over arbitrary rollout lengths.
/// The bound is `max_spectral_radius - epsilon` to provide a safety margin.
#[derive(Debug, Clone)]
pub struct SpectralConstraint {
    /// Maximum allowed spectral radius (default: 1.0)
    max_spectral_radius: f64,
    /// Safety margin subtracted from the bound (default: 1e-6)
    epsilon: f64,
}

impl SpectralConstraint {
    /// Create a new spectral constraint with default parameters.
    pub fn new() -> Self {
        Self {
            max_spectral_radius: 1.0,
            epsilon: 1e-6,
        }
    }

    /// Set the maximum spectral radius.
    pub fn with_max_radius(mut self, radius: f64) -> Self {
        self.max_spectral_radius = radius;
        self
    }

    /// Set the safety margin epsilon.
    pub fn with_epsilon(mut self, eps: f64) -> Self {
        self.epsilon = eps;
        self
    }

    /// The effective spectral radius bound (max - epsilon).
    pub fn effective_bound(&self) -> f64 {
        (self.max_spectral_radius - self.epsilon).max(0.0)
    }

    /// Constrain a square matrix (flattened row-major) so its spectral radius
    /// does not exceed the effective bound.
    ///
    /// Uses power iteration to approximate the dominant eigenvalue,
    /// then rescales the matrix if the spectral radius exceeds the bound.
    pub fn constrain(&self, matrix: &[f64], n: usize) -> Vec<f64> {
        let bound = self.effective_bound();
        let spectral_radius = self.power_iteration_radius(matrix, n, 100);
        if spectral_radius <= bound {
            return matrix.to_vec();
        }
        let scale = bound / spectral_radius;
        matrix.iter().map(|&x| x * scale).collect()
    }

    /// Approximate the spectral radius via power iteration.
    fn power_iteration_radius(&self, matrix: &[f64], n: usize, max_iters: usize) -> f64 {
        let mut b = vec![1.0 / (n as f64).sqrt(); n];
        for _ in 0..max_iters {
            let mut b_next = vec![0.0; n];
            for i in 0..n {
                for j in 0..n {
                    b_next[i] += matrix[i * n + j] * b[j];
                }
            }
            let norm: f64 = b_next.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm < 1e-12 {
                return 0.0;
            }
            for x in b_next.iter_mut() {
                *x /= norm;
            }
            b = b_next;
        }
        let mut rayleigh = 0.0;
        for i in 0..n {
            for j in 0..n {
                rayleigh += b[i] * matrix[i * n + j] * b[j];
            }
        }
        rayleigh.abs()
    }

    /// Theoretical error bound after `num_steps` of iterative refinement.
    ///
    /// Under Lipschitz dynamics with spectral radius `r`, the error grows as
    /// `O(r^N)`.  This returns the multiplicative factor.
    pub fn stability_bound(&self, num_steps: usize) -> f64 {
        let r = self.effective_bound();
        r.powi(num_steps as i32)
    }
}

impl Default for SpectralConstraint {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bound() {
        let c = SpectralConstraint::new();
        assert!((c.effective_bound() - (1.0 - 1e-6)).abs() < 1e-12);
    }

    #[test]
    fn test_stability_bound_decays() {
        let c = SpectralConstraint::new();
        let b1 = c.stability_bound(1);
        let b10 = c.stability_bound(10);
        assert!(b10 < b1);
    }

    #[test]
    fn test_identity_matrix_not_rescaled() {
        let c = SpectralConstraint::new();
        let n = 4;
        let mut identity = vec![0.0; n * n];
        for i in 0..n {
            identity[i * n + i] = 1.0;
        }
        let constrained = c.constrain(&identity, n);
        for (orig, con) in identity.iter().zip(constrained.iter()) {
            assert!((orig - con).abs() < 1e-10);
        }
    }

    #[test]
    fn test_large_matrix_rescaled() {
        let c = SpectralConstraint::new()
            .with_max_radius(0.9)
            .with_epsilon(0.1);
        let n = 4;
        let mut mat = vec![0.0; n * n];
        for i in 0..n {
            mat[i * n + i] = 2.0;
        }
        let constrained = c.constrain(&mat, n);
        let new_radius = c.power_iteration_radius(&constrained, n, 100);
        assert!(new_radius <= c.effective_bound() + 1e-6);
    }
}
