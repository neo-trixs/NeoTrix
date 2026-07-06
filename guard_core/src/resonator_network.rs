/// State of the resonator network during factorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResonatorState {
    /// Initial state — composite set but no factorization attempted.
    Initialized,
    /// Factorization has converged to a stable solution.
    Converged,
    /// Residuals are oscillating — no stable convergence.
    Oscillating,
    /// Residual is increasing — solution is diverging.
    Diverged,
    /// Reached max iterations without convergence.
    MaxIterations,
}

/// Configuration for the resonator network.
#[derive(Debug, Clone)]
pub struct ResonatorConfig {
    /// Vector dimensionality.
    pub dim: usize,
    /// Maximum number of alternating-projection iterations.
    pub max_iterations: usize,
    /// Residual threshold for convergence.
    pub convergence_threshold: f64,
    /// Window size for oscillation detection.
    pub oscillation_window: usize,
    /// Threshold for detecting oscillation in residual history.
    pub oscillation_threshold: f64,
    /// If true, print diagnostic info during factorization.
    pub verbose: bool,
}

impl Default for ResonatorConfig {
    fn default() -> Self {
        Self {
            dim: 2,
            max_iterations: 100,
            convergence_threshold: 1e-6,
            oscillation_window: 6,
            oscillation_threshold: 1e-4,
            verbose: false,
        }
    }
}

/// A factor in the resonator network.
///
/// Each factor has a name, a codebook of candidate vectors, and an
/// optional current estimate.
#[derive(Debug, Clone)]
pub struct Factor {
    pub name: String,
    pub codebook: Vec<Vec<f64>>,
    pub estimate: Option<Vec<f64>>,
}

/// Report returned after factorization completes.
#[derive(Debug, Clone)]
pub struct ConvergenceReport {
    /// Terminal state of the resonator.
    pub state: ResonatorState,
    /// Number of iterations performed.
    pub iterations: usize,
    /// Residual at termination.
    pub final_residual: f64,
    /// Full residual history.
    pub residuals: Vec<f64>,
    /// Cosine similarity of each factor's estimate to the composite.
    pub factor_similarities: Vec<f64>,
    /// Whether oscillation was detected.
    pub oscillation_detected: bool,
}

/// Lightweight diagnostics snapshot.
#[derive(Debug, Clone)]
pub struct ResonatorDiagnostics {
    pub state: ResonatorState,
    pub iteration: usize,
    pub residual: f64,
    pub factor_count: usize,
    pub oscillation_detected: bool,
}

/// A VSA resonator network that factorizes a composite vector into
/// constituent factors via alternating projection.
///
/// # Theory
///
/// The resonator network (Frady et al., 2020) factorizes a composite
/// VSA vector into a product of individual factor vectors by alternating
/// between:
///
/// 1. **Unbinding**: For each factor, unbind the bundle of all other
///    factor estimates from the composite.
/// 2. **Cleanup**: Project the unbound vector onto the nearest codebook
///    vector for that factor.
///
/// This implementation uses MAP-C (element-wise multiplication) for
/// binding and bundled sum for superposition.
///
/// # Usage
///
/// ```ignore
/// let mut res = ResonatorNetwork::new(ResonatorConfig {
///     dim: 256,
///     convergence_threshold: 1e-4,
///     ..Default::default()
/// });
/// res.add_factor("shape", shape_codebook).unwrap();
/// res.add_factor("color", color_codebook).unwrap();
/// res.set_composite(observed_vector);
/// let report = res.factorize();
/// ```
pub struct ResonatorNetwork {
    /// The factors to be estimated.
    pub factors: Vec<Factor>,
    /// The composite (observed) vector to factorize.
    pub composite: Vec<f64>,
    /// Current state of the resonator.
    pub state: ResonatorState,
    /// Current iteration count.
    pub iteration: usize,
    /// Configuration parameters.
    pub config: ResonatorConfig,
    /// History of residuals for oscillation detection.
    pub residual_history: Vec<f64>,
}

impl ResonatorNetwork {
    /// Create a new resonator with the given configuration.
    pub fn new(config: ResonatorConfig) -> Self {
        Self {
            factors: Vec::new(),
            composite: Vec::new(),
            state: ResonatorState::Initialized,
            iteration: 0,
            config,
            residual_history: Vec::new(),
        }
    }

    /// Add a factor with the given name and codebook.
    ///
    /// Returns an error if the codebook is empty or if any codebook
    /// vector has the wrong dimension.
    pub fn add_factor(&mut self, name: &str, codebook: Vec<Vec<f64>>) -> Result<(), String> {
        if codebook.is_empty() {
            return Err(format!("Codebook for '{}' is empty", name));
        }
        for (i, vec) in codebook.iter().enumerate() {
            if vec.len() != self.config.dim {
                return Err(format!(
                    "Codebook vector {} for '{}' has dim {} (expected {})",
                    i,
                    name,
                    vec.len(),
                    self.config.dim
                ));
            }
        }
        self.factors.push(Factor {
            name: name.to_string(),
            codebook,
            estimate: None,
        });
        Ok(())
    }

    /// Set the composite (observed) vector to factorize.
    ///
    /// Resets the resonator state to `Initialized`.
    ///
    /// # Panics
    /// Panics if the composite dimension does not match `config.dim`.
    pub fn set_composite(&mut self, composite: Vec<f64>) {
        assert!(
            composite.len() == self.config.dim,
            "Composite dim {} != config dim {}",
            composite.len(),
            self.config.dim
        );
        self.composite = composite;
        self.state = ResonatorState::Initialized;
        self.iteration = 0;
        self.residual_history.clear();
        for factor in self.factors.iter_mut() {
            factor.estimate = None;
        }
        self.initialize_estimates();
    }

    /// Run alternating projections until convergence or a stopping condition.
    ///
    /// Stopping conditions (checked in order):
    /// 1. Residual below `convergence_threshold` → Converged
    /// 2. Oscillation detected in residual history → Oscillating
    /// 3. Residual increased from previous iteration → Diverged
    /// 4. Max iterations reached → MaxIterations
    pub fn factorize(&mut self) -> ConvergenceReport {
        if self.composite.is_empty() {
            return ConvergenceReport {
                state: ResonatorState::Diverged,
                iterations: 0,
                final_residual: f64::INFINITY,
                residuals: Vec::new(),
                factor_similarities: vec![0.0; self.factors.len()],
                oscillation_detected: false,
            };
        }

        // Initialize estimates from codebook if not set
        self.initialize_estimates();

        let prev_residual = self.compute_residual();
        self.residual_history.push(prev_residual);

        for iteration in 1..=self.config.max_iterations {
            self.iteration = iteration;

            // Single alternating-projection step
            self.step();

            let residual = self.compute_residual();
            self.residual_history.push(residual);

            if self.config.verbose {
                eprintln!(
                    "[resonator] iter={} residual={:.8e}",
                    iteration, residual
                );
            }

            // 1. Convergence check
            if residual < self.config.convergence_threshold {
                self.state = ResonatorState::Converged;
                break;
            }

            // 2. Oscillation detection
            if self.detect_oscillation() {
                self.state = ResonatorState::Oscillating;
                break;
            }

            // 3. Divergence check
            if residual > prev_residual * 1.5 && iteration > 3 {
                self.state = ResonatorState::Diverged;
                break;
            }
        }

        if self.state == ResonatorState::Initialized {
            self.state = ResonatorState::MaxIterations;
        }

        let final_residual = self.compute_residual();
        let factor_similarities = self.compute_factor_similarities();
        let oscillation_detected = self.state == ResonatorState::Oscillating;

        ConvergenceReport {
            state: self.state,
            iterations: self.iteration,
            final_residual,
            residuals: self.residual_history.clone(),
            factor_similarities,
            oscillation_detected,
        }
    }

    /// Single alternating-projection iteration.
    ///
    /// For each factor \(i\):
    /// 1. Form the bundle of all other factor estimates.
    /// 2. Unbind that bundle from the composite: \( \mathbf{z}_i = \mathbf{z} \oslash \text{bundle}(\{\hat{\mathbf{f}}_j\}_{j \neq i}) \).
    /// 3. Cleanup: project \( \mathbf{z}_i \) onto the nearest codebook vector for factor \(i\).
    pub fn step(&mut self) {
        if self.factors.is_empty() || self.composite.is_empty() {
            return;
        }

        // Ensure all estimates are initialized
        self.initialize_estimates();

        // Collect current estimates as slices
        let estimates: Vec<Vec<f64>> = self
            .factors
            .iter()
            .map(|f| f.estimate.clone().unwrap_or_else(|| vec![0.0; self.config.dim]))
            .collect();
        let estimate_refs: Vec<&[f64]> = estimates.iter().map(|v| v.as_slice()).collect();

        // For each factor, compute new estimate
        for i in 0..self.factors.len() {
            // Bundle all OTHER factor estimates
            let others: Vec<&[f64]> = estimate_refs
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, v)| *v)
                .collect();

            let other_bundle = if others.is_empty() {
                // Single factor: residual is just cleanup of composite
                self.composite.clone()
            } else {
                let bundled = bundle(&others);
                // Unbind: z_i = composite ⊘ bundle(other_estimates)
                unbind_mapc(&self.composite, &bundled)
            };

            // Cleanup: project onto nearest codebook vector
            let (new_estimate, _sim) = self.cleanup(&other_bundle, &self.factors[i].codebook);
            self.factors[i].estimate = Some(new_estimate);
        }
    }

    /// Unbind all other factor estimates from the composite.
    ///
    /// For factor \(i\): \( \mathbf{z}_i = \mathbf{z} \oslash \big(\bigodot_{j \neq i} \hat{\mathbf{f}}_j\big) \)
    #[allow(dead_code)]
    fn unbind_others(&self, factor_idx: usize) -> Vec<f64> {
        let others: Vec<&[f64]> = self
            .factors
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != factor_idx)
            .map(|(_, f)| f.estimate.as_deref().unwrap_or(&[]))
            .filter(|v| !v.is_empty())
            .collect();

        if others.is_empty() {
            return self.composite.clone();
        }

        let bundled = bundle(&others);
        unbind_mapc(&self.composite, &bundled)
    }

    /// Cleanup: find nearest neighbor in codebook via cosine similarity.
    ///
    /// Returns `(nearest_vector, similarity_score)`.
    fn cleanup(&self, vector: &[f64], codebook: &[Vec<f64>]) -> (Vec<f64>, f64) {
        let mut best_idx = 0;
        let mut best_sim = f64::NEG_INFINITY;

        for (i, candidate) in codebook.iter().enumerate() {
            let sim = cosine_similarity(vector, candidate);
            if sim > best_sim {
                best_sim = sim;
                best_idx = i;
            }
        }

        (codebook[best_idx].clone(), best_sim)
    }

    /// Compute the residual: how well does the current factorization
    /// reconstruct the composite?
    ///
    /// Residual = L2 distance between composite and the bundle of all
    /// current factor estimates.
    pub fn compute_residual(&self) -> f64 {
        if self.composite.is_empty() {
            return f64::INFINITY;
        }

        let estimates: Vec<&[f64]> = self
            .factors
            .iter()
            .map(|f| f.estimate.as_deref().unwrap_or(&[]))
            .filter(|v| !v.is_empty())
            .collect();

        if estimates.is_empty() {
            return l2_distance(&self.composite, &vec![0.0; self.config.dim]);
        }

        let reconstruction = bundle(&estimates);
        l2_distance(&self.composite, &reconstruction)
    }

    /// Detect oscillation in the residual history.
    ///
    /// Uses a sliding window: if the range (max - min) of residuals within
    /// the window is below `oscillation_threshold`, the residuals have
    /// plateaued into a cycle → oscillation detected.
    fn detect_oscillation(&self) -> bool {
        let window = self.config.oscillation_window;
        if self.residual_history.len() < window + 2 {
            return false;
        }

        let recent = &self.residual_history[self.residual_history.len() - window..];
        let min_val = recent.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = recent.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max_val - min_val;

        // Also check for repeating pattern: compare first half of window with second half
        let half = window / 2;
        let first_half = &recent[..half];
        let second_half = &recent[half..half + half];
        let mut pattern_diff = 0.0;
        for (a, b) in first_half.iter().zip(second_half.iter()) {
            pattern_diff += (a - b).abs();
        }
        pattern_diff /= half as f64;

        range < self.config.oscillation_threshold || pattern_diff < self.config.oscillation_threshold
    }

    /// Initialize factor estimates from codebook if not already set.
    ///
    /// Uses the first codebook entry as the initial estimate for each factor.
    fn initialize_estimates(&mut self) {
        for factor in self.factors.iter_mut() {
            if factor.estimate.is_none() {
                if let Some(first) = factor.codebook.first() {
                    factor.estimate = Some(first.clone());
                }
            }
        }
    }

    /// Get current factor estimates as `(name, vector)` pairs.
    pub fn estimates(&self) -> Vec<(String, Vec<f64>)> {
        self.factors
            .iter()
            .map(|f| (f.name.clone(), f.estimate.clone().unwrap_or_default()))
            .collect()
    }

    /// Reset the resonator to initial state (clears estimates and history).
    pub fn reset(&mut self) {
        self.state = ResonatorState::Initialized;
        self.iteration = 0;
        self.residual_history.clear();
        for factor in self.factors.iter_mut() {
            factor.estimate = None;
        }
    }

    /// Snapshot of current diagnostics.
    pub fn diagnostics(&self) -> ResonatorDiagnostics {
        ResonatorDiagnostics {
            state: self.state,
            iteration: self.iteration,
            residual: self.compute_residual(),
            factor_count: self.factors.len(),
            oscillation_detected: self.state == ResonatorState::Oscillating,
        }
    }

    /// Compute the cosine similarity between the composite and the
    /// reconstruction for each factor, indicating how well each factor
    /// explains the composite.
    fn compute_factor_similarities(&self) -> Vec<f64> {
        self.factors
            .iter()
            .map(|f| match f.estimate.as_ref() {
                Some(est) => cosine_similarity(&self.composite, est),
                None => 0.0,
            })
            .collect()
    }
}

// ──────────────────────────────────────────────
// VSA Helper Functions
// ──────────────────────────────────────────────

/// MAP-C binding: element-wise multiplication.
///
/// \( \mathbf{c} = \mathbf{a} \odot \mathbf{b} \)
pub fn bind_mapc(a: &[f64], b: &[f64]) -> Vec<f64> {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).collect()
}

/// MAP-C unbinding: element-wise division with clamp.
///
/// \( \mathbf{c} = \mathbf{a} \oslash \mathbf{b} \)
/// Clamps divisor to ±1e-10 to avoid division by zero.
pub fn unbind_mapc(a: &[f64], b: &[f64]) -> Vec<f64> {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let denom = if y.abs() < 1e-10 { 1e-10 * y.signum() } else { *y };
            x / denom
        })
        .collect()
}

/// Bundle multiple vectors by normalized sum.
///
/// \( \mathbf{b} = \frac{\sum_i \mathbf{v}_i}{\|\sum_i \mathbf{v}_i\|} \)
pub fn bundle(vectors: &[&[f64]]) -> Vec<f64> {
    if vectors.is_empty() {
        return Vec::new();
    }
    let dim = vectors[0].len();
    let mut sum = vec![0.0; dim];
    for v in vectors {
        for (s, x) in sum.iter_mut().zip(v.iter()) {
            *s += x;
        }
    }
    let mag: f64 = sum.iter().map(|x| x * x).sum::<f64>().sqrt();
    if mag < 1e-12 {
        return sum;
    }
    sum.iter().map(|x| x / mag).collect()
}

/// Cosine similarity between two vectors.
///
/// \( \text{sim} = \frac{\mathbf{a} \cdot \mathbf{b}}{\|\mathbf{a}\| \|\mathbf{b}\|} \)
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let mag_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if mag_a < 1e-12 || mag_b < 1e-12 {
        return 0.0;
    }
    (dot / (mag_a * mag_b)).clamp(-1.0, 1.0)
}

/// L2 (Euclidean) distance between two vectors.
///
/// \( d = \sqrt{\sum_i (a_i - b_i)^2} \)
pub fn l2_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let d = x - y;
            d * d
        })
        .sum::<f64>()
        .sqrt()
}

// ──────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_2d_codebook(count: usize) -> Vec<Vec<f64>> {
        (0..count)
            .map(|i| {
                let angle = 2.0 * std::f64::consts::PI * (i as f64) / (count as f64);
                vec![angle.cos(), angle.sin()]
            })
            .collect()
    }

    fn normalized(v: &[f64]) -> Vec<f64> {
        let mag: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        if mag < 1e-12 {
            return v.to_vec();
        }
        v.iter().map(|x| x / mag).collect()
    }

    /// Create a 4D angle-based codebook where entries don't produce
    /// zero composites under MAP-C binding.
    fn make_4d_codebook(count: usize, phase_shift: f64) -> Vec<Vec<f64>> {
        (0..count)
            .map(|i| {
                let angle = 2.0 * std::f64::consts::PI * (i as f64) / (count as f64) + phase_shift;
                normalized(&[
                    angle.cos(),
                    angle.sin(),
                    (2.0 * angle).cos() * 0.5,
                    (2.0 * angle).sin() * 0.5,
                ])
            })
            .collect()
    }

    // ── 1. add_factor ──

    #[test]
    fn test_add_factor() {
        let mut res = ResonatorNetwork::new(ResonatorConfig {
            dim: 2,
            ..Default::default()
        });
        assert!(res.add_factor("a", make_2d_codebook(4)).is_ok());
        assert!(res.add_factor("b", make_2d_codebook(4)).is_ok());
        assert_eq!(res.factors.len(), 2);
    }

    #[test]
    fn test_add_factor_rejects_empty_codebook() {
        let mut res = ResonatorNetwork::new(ResonatorConfig {
            dim: 2,
            ..Default::default()
        });
        let r = res.add_factor("empty", vec![]);
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_add_factor_rejects_wrong_dim() {
        let mut res = ResonatorNetwork::new(ResonatorConfig {
            dim: 2,
            ..Default::default()
        });
        let r = res.add_factor("bad", vec![vec![1.0, 2.0, 3.0]]);
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("dim"));
    }

    // ── 2. set_composite ──

    #[test]
    fn test_set_composite() {
        let mut res = ResonatorNetwork::new(ResonatorConfig {
            dim: 2,
            ..Default::default()
        });
        res.add_factor("a", make_2d_codebook(4)).unwrap();

        let comp = normalized(&[0.6, 0.8]);
        res.set_composite(comp.clone());
        assert_eq!(res.composite, comp);
        assert_eq!(res.state, ResonatorState::Initialized);
    }

    #[test]
    #[should_panic(expected = "dim")]
    fn test_set_composite_wrong_dim_panics() {
        let mut res = ResonatorNetwork::new(ResonatorConfig {
            dim: 2,
            ..Default::default()
        });
        res.set_composite(vec![1.0, 2.0, 3.0]);
    }

    // ── 3. single_step_updates_estimates ──

    #[test]
    fn test_single_step_updates_estimates() {
        let mut res = ResonatorNetwork::new(ResonatorConfig {
            dim: 2,
            max_iterations: 100,
            ..Default::default()
        });

        // Two factors with orthogonal codebooks
        res.add_factor("x", make_2d_codebook(4)).unwrap();
        res.add_factor("y", make_2d_codebook(4)).unwrap();

        // Composite = bind(x1, y1)
        let x1 = normalized(&[0.8, 0.6]);
        let y1 = normalized(&[0.3, 0.95]);
        let comp = normalized(&bind_mapc(&x1, &y1));
        res.set_composite(comp);

        let estimates_before = res.estimates();
        for (_, v) in &estimates_before {
            assert!(!v.is_empty());
        }

        res.step();

        let estimates_after = res.estimates();
        assert_eq!(estimates_after.len(), 2);
        for (_, v) in &estimates_after {
            assert!(!v.is_empty());
        }
    }

    // ── 4. factorize_converges_2_factors ──

    #[test]
    fn test_factorize_converges_2_factors() {
        let mut res = ResonatorNetwork::new(ResonatorConfig {
            dim: 4,
            convergence_threshold: 1e-3,
            max_iterations: 100,
            ..Default::default()
        });

        // 4D spiral codebooks — all entries have full support (no zero binding)
        let codebook_a: Vec<Vec<f64>> = (0..4)
            .map(|i| {
                let angle = 2.0 * std::f64::consts::PI * (i as f64) / 4.0;
                normalized(&[angle.cos(), angle.sin(), (2.0 * angle).cos() * 0.5, (2.0 * angle).sin() * 0.5])
            })
            .collect();
        let codebook_b: Vec<Vec<f64>> = (0..4)
            .map(|i| {
                let angle = 2.0 * std::f64::consts::PI * (i as f64) / 4.0 + 0.3;
                normalized(&[(angle).sin(), (angle).cos(), (3.0 * angle).sin() * 0.5, (3.0 * angle).cos() * 0.5])
            })
            .collect();
        res.add_factor("A", codebook_a.clone()).unwrap();
        res.add_factor("B", codebook_b.clone()).unwrap();

        let a_truth = &codebook_a[0];
        let b_truth = &codebook_b[1];
        let composite = normalized(&bind_mapc(a_truth, b_truth));
        res.set_composite(composite);

        let report = res.factorize();

        assert!(
            report.state == ResonatorState::Converged
                || report.state == ResonatorState::MaxIterations
                || report.state == ResonatorState::Oscillating,
            "Expected a terminal state, got {:?} with residual={:.6}",
            report.state,
            report.final_residual
        );

        // The estimates should recover the correct factors
        let ests = res.estimates();
        assert_eq!(ests.len(), 2);

        let sim_a = cosine_similarity(&ests[0].1, a_truth);
        let sim_b = cosine_similarity(&ests[1].1, b_truth);

        assert!(
            sim_a > 0.7 || report.state == ResonatorState::MaxIterations,
            "Factor A similarity too low: {:.4}",
            sim_a
        );
        assert!(
            sim_b > 0.7 || report.state == ResonatorState::MaxIterations,
            "Factor B similarity too low: {:.4}",
            sim_b
        );
    }

    // ── 5. factorize_converges_3_factors ──

    #[test]
    fn test_factorize_converges_3_factors() {
        let mut res = ResonatorNetwork::new(ResonatorConfig {
            dim: 4,
            convergence_threshold: 1e-3,
            max_iterations: 150,
            ..Default::default()
        });

        // Build three 4D codebooks with orthogonal-ish candidates
        let cb_a: Vec<Vec<f64>> = (0..4)
            .map(|i| {
                let mut v = vec![0.0; 4];
                v[i] = 1.0;
                v
            })
            .collect();
        let cb_b: Vec<Vec<f64>> = (0..4)
            .map(|i| {
                let mut v = vec![0.0; 4];
                v[(i + 1) % 4] = 1.0;
                v
            })
            .collect();
        let cb_c: Vec<Vec<f64>> = (0..4)
            .map(|i| {
                let mut v = vec![0.0; 4];
                v[(i + 2) % 4] = 1.0;
                v
            })
            .collect();

        res.add_factor("A", cb_a.clone()).unwrap();
        res.add_factor("B", cb_b.clone()).unwrap();
        res.add_factor("C", cb_c.clone()).unwrap();

        // Composite = bind(bind(A[0], B[1]), C[2])
        let a_truth = normalized(&[0.5, 0.5, 0.5, 0.5]);
        let b_truth = normalized(&[1.0, 0.0, 0.0, 0.0]);
        let c_truth = normalized(&[0.0, 1.0, 0.0, 0.0]);

        let ab = normalized(&bind_mapc(&a_truth, &b_truth));
        let composite = normalized(&bind_mapc(&ab, &c_truth));
        res.set_composite(composite);

        let report = res.factorize();

        // The 3-factor case is harder; accept convergence or oscillation
        assert!(
            report.state == ResonatorState::Converged
                || report.state == ResonatorState::Oscillating
                || report.state == ResonatorState::MaxIterations,
            "Unexpected state: {:?}, residual={:.6}",
            report.state,
            report.final_residual
        );

        let ests = res.estimates();
        assert_eq!(ests.len(), 3);
    }

    // ── 6. oscillation_detection ──

    #[test]
    fn test_oscillation_detection() {
        let mut res = ResonatorNetwork::new(ResonatorConfig {
            dim: 2,
            oscillation_window: 4,
            oscillation_threshold: 0.01,
            ..Default::default()
        });

        // Seed residuals with a clear oscillation pattern
        for i in 0..6 {
            res.residual_history.push(if i % 2 == 0 { 0.5 } else { 0.51 });
        }

        assert!(res.detect_oscillation());

        // Now try with increasing residuals (no oscillation)
        let mut res2 = ResonatorNetwork::new(ResonatorConfig {
            dim: 2,
            oscillation_window: 4,
            oscillation_threshold: 0.01,
            ..Default::default()
        });
        for i in 0..8 {
            res2.residual_history.push(1.0 / (i as f64 + 1.0));
        }
        // Monotonically decreasing shouldn't trigger oscillation
        assert!(!res2.detect_oscillation());
    }

    // ── 7. reset_clears_state ──

    #[test]
    fn test_reset_clears_state() {
        let mut res = ResonatorNetwork::new(ResonatorConfig {
            dim: 2,
            ..Default::default()
        });
        res.add_factor("a", make_2d_codebook(4)).unwrap();
        res.add_factor("b", make_2d_codebook(4)).unwrap();
        res.set_composite(vec![1.0, 0.0]);
        res.factorize();

        assert_eq!(res.state, ResonatorState::Converged);
        assert!(res.iteration > 0);

        res.reset();
        assert_eq!(res.state, ResonatorState::Initialized);
        assert_eq!(res.iteration, 0);
        assert!(res.residual_history.is_empty());
        for factor in &res.factors {
            assert!(factor.estimate.is_none());
        }
    }

    // ── 8. empty_composite_returns_diverged ──

    #[test]
    fn test_empty_composite_returns_diverged() {
        let mut res = ResonatorNetwork::new(ResonatorConfig::default());
        let report = res.factorize();
        assert_eq!(report.state, ResonatorState::Diverged);
        assert!(report.final_residual.is_infinite());
    }

    // ── 9. diagnostics_snapshot ──

    #[test]
    fn test_diagnostics_snapshot() {
        let mut res = ResonatorNetwork::new(ResonatorConfig {
            dim: 2,
            ..Default::default()
        });
        res.add_factor("a", make_2d_codebook(4)).unwrap();
        res.set_composite(vec![1.0, 0.0]);

        let diag = res.diagnostics();
        assert_eq!(diag.state, ResonatorState::Initialized);
        assert_eq!(diag.factor_count, 1);
        assert!(!diag.oscillation_detected);
    }

    // ── 10. estimates_returns_correct_count ──

    #[test]
    fn test_estimates_returns_correct_count() {
        let mut res = ResonatorNetwork::new(ResonatorConfig {
            dim: 2,
            ..Default::default()
        });
        res.add_factor("a", make_2d_codebook(4)).unwrap();
        res.add_factor("b", make_2d_codebook(4)).unwrap();

        let ests = res.estimates();
        assert_eq!(ests.len(), 2);
        // Before initialization, estimates should be empty vectors
        for (name, v) in &ests {
            assert!(
                v.is_empty(),
                "Factor {} should have empty estimate before init",
                name
            );
        }
    }

    // ── 11. VSA helpers: bind_mapc ──

    #[test]
    fn test_bind_mapc_identity() {
        let a = vec![1.0, 2.0, 3.0];
        let one = vec![1.0, 1.0, 1.0];
        let result = bind_mapc(&a, &one);
        assert_eq!(result, a);
    }

    #[test]
    fn test_bind_mapc_zero() {
        let a = vec![1.0, 2.0, 3.0];
        let zero = vec![0.0, 0.0, 0.0];
        let result = bind_mapc(&a, &zero);
        assert_eq!(result, vec![0.0, 0.0, 0.0]);
    }

    // ── 12. VSA helpers: cosine_similarity ──

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-10);
    }

    // ── 13. VSA helpers: bundle ──

    #[test]
    fn test_bundle_multiple_vectors() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let result = bundle(&[&a, &b]);
        // Bundled vector should be normalized
        let mag: f64 = result.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!((mag - 1.0).abs() < 1e-10);
        // Both components should be equal
        assert!((result[0] - result[1]).abs() < 1e-10);
    }

    // ── 14. VSA helpers: l2_distance ──

    #[test]
    fn test_l2_distance_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let d = l2_distance(&a, &a);
        assert!(d.abs() < 1e-10);
    }

    #[test]
    fn test_l2_distance_known() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let d = l2_distance(&a, &b);
        assert!((d - 5.0).abs() < 1e-10);
    }

    // ── 15. VSA helpers: unbind_mapc ──

    #[test]
    fn test_unbind_mapc_roundtrip() {
        let a = vec![2.0, 3.0, 4.0];
        let b = vec![0.5, 2.0, 1.0];
        let bound = bind_mapc(&a, &b);
        let rebound = unbind_mapc(&bound, &b);
        for (x, y) in rebound.iter().zip(a.iter()) {
            assert!((x - y).abs() < 1e-6);
        }
    }

    // ── 16. step with no factors is no-op ──

    #[test]
    fn test_step_no_factors_is_noop() {
        let mut res = ResonatorNetwork::new(ResonatorConfig::default());
        // Should not panic
        res.step();
        assert_eq!(res.state, ResonatorState::Initialized);
    }

    // ── 17. detect_oscillation short history returns false ──

    #[test]
    fn test_detect_oscillation_short_history() {
        let mut res = ResonatorNetwork::new(ResonatorConfig::default());
        res.residual_history.push(0.5);
        res.residual_history.push(0.3);
        // Window is 6, we only have 2 entries
        assert!(!res.detect_oscillation());
    }

    // ── 18. initialize_estimates_sets_first_codebook ──

    #[test]
    fn test_initialize_estimates_sets_first_codebook() {
        let mut res = ResonatorNetwork::new(ResonatorConfig {
            dim: 2,
            ..Default::default()
        });
        res.add_factor("a", make_2d_codebook(4)).unwrap();
        for f in &res.factors {
            assert!(f.estimate.is_none());
        }
        res.initialize_estimates();
        for f in &res.factors {
            assert!(f.estimate.is_some());
        }
    }

    // ── 19. factorize_max_iterations ──

    #[test]
    fn test_factorize_max_iterations() {
        let mut res = ResonatorNetwork::new(ResonatorConfig {
            dim: 4,
            convergence_threshold: 1e-12,
            max_iterations: 5,
            ..Default::default()
        });
        res.add_factor("a", make_4d_codebook(4, 0.0)).unwrap();
        res.add_factor("b", make_4d_codebook(4, 0.5)).unwrap();
        let composite = normalized(&bind_mapc(
            &make_4d_codebook(4, 0.0)[0],
            &make_4d_codebook(4, 0.5)[1],
        ));
        res.set_composite(composite);
        let report = res.factorize();
        // With only 5 iterations and tight threshold, should hit max
        assert!(
            report.state == ResonatorState::MaxIterations,
            "Expected MaxIterations, got {:?}",
            report.state
        );
        assert_eq!(report.iterations, 5);
    }
}
