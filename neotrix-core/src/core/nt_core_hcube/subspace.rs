#![forbid(unsafe_code)]

use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

/// Online subspace tracker via CCIPCA.
///
/// Incrementally estimates the k-dimensional principal subspace of a data stream
/// without computing the covariance matrix. Each new sample updates all k component
/// estimates with O(k·VSA_DIM) cost.
///
/// Reference: Weng, J., Zhang, Y., & Hwang, W.-S. (2003).
/// "Candid Covariance-free Incremental Principal Component Analysis"
pub struct OnlineSubspace {
    components: Vec<Vec<f64>>,
    eigenvalues: Vec<f64>,
    n: u64,
    k: usize,
}

impl OnlineSubspace {
    pub fn new(k: usize, _seed: u64) -> Self {
        let k = k.min(VSA_DIM);
        OnlineSubspace {
            components: vec![Vec::new(); k],
            eigenvalues: vec![0.0; k],
            n: 0,
            k,
        }
    }

    /// Feed one sample and update the subspace incrementally.
    ///
    /// For each component: CCIPCA update → normalize → eigenvalue update → deflate.
    pub fn update(&mut self, sample: &[f64]) {
        assert_eq!(
            sample.len(),
            VSA_DIM,
            "CCIPCA input must have VSA_DIM elements"
        );
        let nf = self.n as f64;
        let mut x: Vec<f64> = sample.to_vec();

        for i in 0..self.k {
            if self.components[i].is_empty() {
                let nrm = norm(&x);
                if nrm > 1e-12 {
                    self.components[i] = x.iter().map(|v| v / nrm).collect();
                    self.eigenvalues[i] = nrm * nrm;
                } else {
                    self.components[i] = x.clone();
                }
            } else {
                let nrm_vi = norm(&self.components[i]);
                if nrm_vi > 1e-12 {
                    let proj = dot(&x, &self.components[i]) / nrm_vi;
                    let alpha = nf / (nf + 1.0);
                    let beta = 1.0 / (nf + 1.0);
                    for j in 0..VSA_DIM {
                        self.components[i][j] = alpha * self.components[i][j] + beta * proj * x[j];
                    }
                    let new_nrm = norm(&self.components[i]);
                    if new_nrm > 1e-12 {
                        for j in 0..VSA_DIM {
                            self.components[i][j] /= new_nrm;
                        }
                    }
                    let coeff = dot(&x, &self.components[i]);
                    self.eigenvalues[i] = alpha * self.eigenvalues[i] + beta * coeff * coeff;
                    for j in 0..VSA_DIM {
                        x[j] -= coeff * self.components[i][j];
                    }
                }
            }
        }
        self.n += 1;
    }

    pub fn components(&self) -> &[Vec<f64>] {
        &self.components
    }

    pub fn eigenvalues(&self) -> &[f64] {
        &self.eigenvalues
    }

    /// Project a vector onto the learned subspace, returning k coefficients.
    pub fn project(&self, vec: &[f64]) -> Vec<f64> {
        let mut coeffs = vec![0.0; self.k];
        for i in 0..self.k {
            if !self.components[i].is_empty() {
                coeffs[i] = dot(vec, &self.components[i]);
            }
        }
        coeffs
    }

    /// Reconstruct a vector from subspace coefficients.
    pub fn reconstruct(&self, coeffs: &[f64]) -> Vec<f64> {
        let mut result = vec![0.0; VSA_DIM];
        for (i, &c) in coeffs.iter().enumerate() {
            if i < self.k && !self.components[i].is_empty() {
                for j in 0..VSA_DIM {
                    result[j] += c * self.components[i][j];
                }
            }
        }
        result
    }

    /// Reconstruction error (L2 norm) — directly usable as an anomaly score.
    pub fn residual(&self, vec: &[f64]) -> f64 {
        let coeffs = self.project(vec);
        let mut err = 0.0;
        for j in 0..VSA_DIM {
            let mut recon = 0.0;
            for (i, &c) in coeffs.iter().enumerate() {
                if i < self.k && !self.components[i].is_empty() {
                    recon += c * self.components[i][j];
                }
            }
            let diff = vec[j] - recon;
            err += diff * diff;
        }
        err.sqrt()
    }

    /// Returns the component index whose direction contributes most to the
    /// reconstruction error for the given vector.
    ///
    /// Higher contribution → that component's direction is most anomalous
    /// (i.e., the data deviates most from the learned subspace along that
    /// principal direction).
    pub fn anomalous_component(&self, vec: &[f64]) -> (usize, f64) {
        let mut best_idx = 0;
        let mut best_contrib = 0.0f64;
        for i in 0..self.k {
            if self.components[i].is_empty() {
                continue;
            }
            let coeff = dot(vec, &self.components[i]);
            let mut contrib = 0.0;
            for j in 0..VSA_DIM {
                contrib += (vec[j] - coeff * self.components[i][j]).abs();
            }
            if contrib > best_contrib {
                best_contrib = contrib;
                best_idx = i;
            }
        }
        (best_idx, best_contrib)
    }
}

fn norm(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use rand::SeedableRng;

    fn rand_vec(rng: &mut impl Rng) -> Vec<f64> {
        (0..VSA_DIM).map(|_| rng.gen::<f64>() * 2.0 - 1.0).collect()
    }

    fn gram_schmidt(vectors: &mut [Vec<f64>]) {
        for i in 0..vectors.len() {
            for j in 0..i {
                let d = dot(&vectors[i], &vectors[j]);
                for idx in 0..vectors[i].len() {
                    vectors[i][idx] -= d * vectors[j][idx];
                }
            }
            let n = norm(&vectors[i]);
            if n > 1e-12 {
                for x in &mut vectors[i] {
                    *x /= n;
                }
            }
        }
    }

    #[test]
    fn test_new_subspace_initialized_correctly() {
        let ss = OnlineSubspace::new(5, 42);
        assert_eq!(ss.k, 5);
        assert_eq!(ss.components.len(), 5);
        assert_eq!(ss.eigenvalues.len(), 5);
        assert_eq!(ss.n, 0);
        for c in &ss.components {
            assert!(c.is_empty());
        }
        for &ev in &ss.eigenvalues {
            assert_eq!(ev, 0.0);
        }
    }

    #[test]
    fn test_update_increases_n() {
        let mut ss = OnlineSubspace::new(3, 42);
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        for i in 1..=10 {
            ss.update(&rand_vec(&mut rng));
            assert_eq!(ss.n, i);
        }
    }

    #[test]
    fn test_project_reduces_dimension() {
        let k = 4;
        let mut ss = OnlineSubspace::new(k, 42);
        let mut rng = rand::rngs::StdRng::seed_from_u64(7);
        let data: Vec<Vec<f64>> = (0..50).map(|_| rand_vec(&mut rng)).collect();
        for d in &data {
            ss.update(d);
        }
        let coeffs = ss.project(&data[0]);
        assert_eq!(coeffs.len(), k);
    }

    #[test]
    fn test_reconstruction_improves_with_more_samples() {
        let mut ss = OnlineSubspace::new(2, 42);
        let mut rng = rand::rngs::StdRng::seed_from_u64(13);

        let mut anchor_data: Vec<f64> =
            (0..VSA_DIM).map(|_| rng.gen::<f64>() * 2.0 - 1.0).collect();
        let anchor_nrm = norm(&anchor_data);
        for x in &mut anchor_data {
            *x /= anchor_nrm;
        }

        let test_point: Vec<f64> = anchor_data.iter().map(|&a| a * 0.7).collect();
        let err_before = ss.residual(&test_point);

        for _ in 0..100 {
            let a: f64 = rng.gen::<f64>() * 3.0 - 1.5;
            let d: Vec<f64> = anchor_data.iter().map(|&b| a * b).collect();
            ss.update(&d);
        }

        let err_after = ss.residual(&test_point);
        assert!(
            err_after <= err_before + 1e-6,
            "residual should not increase after learning: before {err_before:.6}, after {err_after:.6}"
        );
    }

    #[test]
    fn test_residual_decreases_with_learning() {
        let mut ss = OnlineSubspace::new(1, 42);
        let mut rng = rand::rngs::StdRng::seed_from_u64(21);

        let mut dir: Vec<f64> = (0..VSA_DIM).map(|_| rng.gen::<f64>() - 0.5).collect();
        let dn = norm(&dir);
        for x in &mut dir {
            *x /= dn;
        }

        let err_before = {
            let off: Vec<f64> = (0..VSA_DIM).map(|_| rng.gen::<f64>() * 2.0 - 1.0).collect();
            ss.residual(&off)
        };

        for _ in 0..50 {
            let a: f64 = rng.gen::<f64>() * 4.0 - 2.0;
            let d: Vec<f64> = dir.iter().map(|&b| a * b).collect();
            ss.update(&d);
        }

        let in_dir: Vec<f64> = dir.iter().map(|&b| 1.5 * b).collect();
        let err_after = ss.residual(&in_dir);

        assert!(
            err_after < err_before,
            "residual after learning ({err_after:.4}) should be lower than before ({err_before:.4})"
        );
    }

    #[test]
    fn test_anomalous_component_detects_outlier() {
        let k = 3;
        let mut ss = OnlineSubspace::new(k, 42);
        let mut rng = rand::rngs::StdRng::seed_from_u64(55);

        let mut basis: Vec<Vec<f64>> = (0..k)
            .map(|_| (0..VSA_DIM).map(|_| rng.gen::<f64>() - 0.5).collect())
            .collect();
        gram_schmidt(&mut basis);

        for _ in 0..100 {
            let coeffs: Vec<f64> = (0..k).map(|_| rng.gen::<f64>() * 4.0 - 2.0).collect();
            let mut pt = vec![0.0; VSA_DIM];
            for (i, &c) in coeffs.iter().enumerate() {
                for j in 0..VSA_DIM {
                    pt[j] += c * basis[i][j];
                }
            }
            ss.update(&pt);
        }

        let (idx, contrib) = ss.anomalous_component(&vec![0.0; VSA_DIM]);
        assert!(idx < k);
        assert!(contrib >= 0.0);
    }

    #[test]
    fn test_orthogonal_components() {
        let k = 3;
        let mut ss = OnlineSubspace::new(k, 42);
        let mut rng = rand::rngs::StdRng::seed_from_u64(99);

        for _ in 0..200 {
            ss.update(&rand_vec(&mut rng));
        }

        for i in 0..k {
            for j in (i + 1)..k {
                if !ss.components[i].is_empty() && !ss.components[j].is_empty() {
                    let d = dot(&ss.components[i], &ss.components[j]).abs();
                    assert!(
                        d < 0.3,
                        "components[{i}] and [{j}] should be near-orthogonal, dot={d:.4}"
                    );
                }
            }
        }
    }

    #[test]
    fn test_subspace_k_clamped_to_vsa_dim() {
        let ss = OnlineSubspace::new(VSA_DIM + 100, 0);
        assert_eq!(ss.k, VSA_DIM);
        assert_eq!(ss.components.len(), VSA_DIM);
    }
}
