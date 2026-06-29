#![forbid(unsafe_code)]

//! Koopman Operator for VSA state evolution modeling.
//! Transform (FWHT), learns a linear transition matrix `K` that approximates the
//! non-linear dynamics of the VSA state space, and provides prediction error signals
//! for negentropy calibration.
//!
//! ## Theory
//!
//! Given a sequence of VSA states `s₁, s₂, ..., sₘ`, the Koopman operator posits a
//! linear operator `K` acting on observation functions `g` such that:
//!
//! ```text
//! g(s_{t+1}) ≈ K · g(s_t)
//! ```
//!
//! Here `g` is the FWHT, mapping VSA byte states into a frequency-domain observation
//! space where linear dynamics are more plausible. `K` is learned via least-squares
//! from observed state sequences.

use std::f64;

use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

// ── Fast Walsh-Hadamard Transform helpers ──────────────────────────────

/// In-place Fast Walsh-Hadamard Transform.
///
/// Applies the unnormalized FWHT to `data`, which must have power-of-two length.
/// After calling this, apply [`normalize`] to make the transform self-inverse.
fn fwht_inplace(data: &mut [f64]) {
    let n = data.len();
    debug_assert!(n.is_power_of_two(), "FWHT requires power-of-two length");
    let mut len = 1;
    while len < n {
        let step = len * 2;
        for i in (0..n).step_by(step) {
            for j in 0..len {
                let u = data[i + j];
                let v = data[i + j + len];
                data[i + j] = u + v;
                data[i + j + len] = u - v;
            }
        }
        len = step;
    }
}

/// Divide every element by `sqrt(N)` to make FWHT self-inverse.
fn normalize(data: &mut [f64]) {
    let norm = (data.len() as f64).sqrt();
    for v in data.iter_mut() {
        *v /= norm;
    }
}

// ── Matrix helpers (private) ──────────────────────────────────────────

/// Multiply two matrices: `a * b`.
///
/// Panics if dimensions are incompatible.
fn mat_mul(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let m = a.len();
    let n = b[0].len();
    let p = a[0].len();
    assert_eq!(p, b.len(), "mat_mul: inner dimensions must match");
    let mut result = vec![vec![0.0; n]; m];
    for i in 0..m {
        for k in 0..p {
            let aik = a[i][k];
            if aik == 0.0 {
                continue;
            }
            for j in 0..n {
                result[i][j] += aik * b[k][j];
            }
        }
    }
    result
}

/// Multiply matrix by column vector: `a * x`.
fn mat_vec_mul(a: &[Vec<f64>], x: &[f64]) -> Vec<f64> {
    let m = a.len();
    let n = a[0].len();
    assert_eq!(n, x.len(), "mat_vec_mul: dimensions must match");
    let mut result = vec![0.0; m];
    for i in 0..m {
        let mut sum = 0.0;
        let row = &a[i];
        for j in 0..n {
            sum += row[j] * x[j];
        }
        result[i] = sum;
    }
    result
}

/// Compute the inverse of a square matrix via Gauss-Jordan elimination.
///
/// Returns the inverse, or an approximation (regularized identity) if the matrix
/// is numerically singular.
fn mat_inv(a: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = a.len();
    let mut aug = vec![vec![0.0; 2 * n]; n];
    for i in 0..n {
        for j in 0..n {
            aug[i][j] = a[i][j];
        }
        aug[i][n + i] = 1.0;
    }

    for i in 0..n {
        let mut max_row = i;
        for k in i + 1..n {
            if aug[k][i].abs() > aug[max_row][i].abs() {
                max_row = k;
            }
        }
        aug.swap(i, max_row);

        if aug[i][i].abs() < 1e-14 {
            let mut identity = vec![vec![0.0; n]; n];
            for r in 0..n {
                identity[r][r] = 1.0;
            }
            return identity;
        }

        let pivot = aug[i][i];
        for j in 0..2 * n {
            aug[i][j] /= pivot;
        }

        for k in 0..n {
            if k == i {
                continue;
            }
            let factor = aug[k][i];
            if factor == 0.0 {
                continue;
            }
            for j in 0..2 * n {
                aug[k][j] -= factor * aug[i][j];
            }
        }
    }

    let mut inv = vec![vec![0.0; n]; n];
    for i in 0..n {
        inv[i].copy_from_slice(&aug[i][n..]);
    }
    inv
}

/// Compute the dot product of two vectors.
fn dot_product(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Compute the L2 norm of a vector.
fn l2_norm(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}

// ── KoopmanStats ──────────────────────────────────────────────────────

/// Statistical summary of the Koopman Operator's current state.
#[derive(Debug, Clone)]
pub struct KoopmanStats {
    /// Number of observations fed to `learn()`.
    pub observation_count: usize,
    /// Whether `K_matrix` has been learned and is ready for prediction.
    pub is_initialized: bool,
    /// Most recent prediction error (cosine distance).
    pub last_prediction_error: f64,
    /// EMA-smoothed prediction error.
    pub prediction_error_ema: f64,
}

// ── KoopmanOperator ───────────────────────────────────────────────────

/// Koopman Operator for VSA dynamics modeling.
///
/// Lifts byte-encoded VSA states into an FWHT frequency-domain observation space,
/// learns a linear transition matrix via least squares, and predicts future states.
pub struct KoopmanOperator {
    /// Dimension of the observation space (must be power of two).
    dim: usize,
    /// Learned linear transition matrix (dim × dim).
    k_matrix: Vec<Vec<f64>>,
    /// Ring buffer of lifted observations for batch learning.
    observation_buffer: Vec<Vec<f64>>,
    /// History of prediction errors for diagnostics.
    prediction_history: Vec<f64>,
    /// Running statistics.
    stats: KoopmanStats,
    /// EMA smoothing factor (0.0–1.0).
    ema_alpha: f64,
    /// Maximum observations to retain in buffer.
    max_buffer: usize,
}

impl Default for KoopmanOperator {
    fn default() -> Self {
        Self::new(VSA_DIM)
    }
}

impl KoopmanOperator {
    /// Create a new Koopman operator with the given observation dimension.
    ///
    /// `dim` must be a power of two and equals the byte-length of VSA states
    /// passed to `lift()` and `predict()`.
    ///
    /// # Panics
    ///
    /// Panics if `dim` is not a power of two.
    pub fn new(dim: usize) -> Self {
        assert!(
            dim.is_power_of_two(),
            "KoopmanOperator: dim must be a power of two, got {dim}"
        );
        Self {
            dim,
            k_matrix: vec![vec![0.0; dim]; dim],
            observation_buffer: Vec::with_capacity(256),
            prediction_history: Vec::with_capacity(1024),
            stats: KoopmanStats {
                observation_count: 0,
                is_initialized: false,
                last_prediction_error: 0.0,
                prediction_error_ema: 0.0,
            },
            ema_alpha: 0.3,
            max_buffer: 256,
        }
    }

    /// Lift a VSA state (byte slice) into the Koopman observation space.
    ///
    /// The lift consists of:
    /// 1. Converting bytes to f64 values
    /// 2. Applying the Fast Walsh-Hadamard Transform
    /// 3. Normalizing by sqrt(dim)
    ///
    /// # Panics
    ///
    /// Panics if `state.len() != self.dim`.
    pub fn lift(&self, state: &[u8]) -> Vec<f64> {
        assert_eq!(
            state.len(),
            self.dim,
            "KoopmanOperator::lift: state length {} != dim {}",
            state.len(),
            self.dim
        );
        let mut data: Vec<f64> = state.iter().map(|&b| b as f64).collect();
        fwht_inplace(&mut data);
        normalize(&mut data);
        data
    }

    /// Invert a lifted observation vector back to VSA byte state.
    ///
    /// Since the normalized FWHT is self-inverse, this applies the same
    /// transform as `lift()`.
    ///
    /// # Panics
    ///
    /// Panics if `observation.len() != self.dim`.
    pub fn inv_lift(&self, observation: &[f64]) -> Vec<u8> {
        assert_eq!(
            observation.len(),
            self.dim,
            "KoopmanOperator::inv_lift: observation length {} != dim {}",
            observation.len(),
            self.dim
        );
        let mut data = observation.to_vec();
        fwht_inplace(&mut data);
        normalize(&mut data);
        data.iter().map(|&v| (v.clamp(0.0, 255.0)) as u8).collect()
    }

    /// Batch-learn the K matrix from a sequence of VSA states.
    ///
    /// Constructs a linear transition matrix `K` such that `lift(s_{t+1}) ≈
    /// K · lift(s_t)` using least squares (normal equations with Tikhonov
    /// regularization).
    ///
    /// After calling this, `is_initialized()` returns `true` and `predict()`
    /// is usable.
    pub fn learn(&mut self, observations: &[Vec<u8>]) {
        if observations.len() < 2 {
            return;
        }

        // Lift all observations
        let lifted: Vec<Vec<f64>> = observations
            .iter()
            .take(self.max_buffer)
            .map(|obs| self.lift(obs))
            .collect();

        self.observation_buffer = lifted;
        let d = self.dim;
        let n = self.observation_buffer.len() - 1;

        // Compute XXt = X * X^T  (d × d)
        let mut xxt = vec![vec![0.0; d]; d];
        for k in 0..n {
            let xk = &self.observation_buffer[k];
            for i in 0..d {
                let xki = xk[i];
                if xki == 0.0 {
                    continue;
                }
                let row = &mut xxt[i];
                for j in 0..d {
                    row[j] += xki * xk[j];
                }
            }
        }

        // Compute YXt = Y * X^T  (d × d)
        let mut yxt = vec![vec![0.0; d]; d];
        for k in 0..n {
            let xk = &self.observation_buffer[k];
            let yk = &self.observation_buffer[k + 1];
            for i in 0..d {
                let yki = yk[i];
                if yki == 0.0 {
                    continue;
                }
                let row = &mut yxt[i];
                for j in 0..d {
                    row[j] += yki * xk[j];
                }
            }
        }

        // Regularize: λI to handle near-singular matrices
        let lambda = 1e-6;
        for i in 0..d {
            xxt[i][i] += lambda;
        }

        // K = YXt * inv(XXt)
        let xxt_inv = mat_inv(&xxt);
        self.k_matrix = mat_mul(&yxt, &xxt_inv);
        self.stats.is_initialized = true;
        self.stats.observation_count = self.observation_buffer.len();
    }

    /// Predict the next VSA state given the current one.
    ///
    /// Returns a byte vector of length `dim` representing the predicted
    /// next state.
    ///
    /// # Panics
    ///
    /// Panics if not initialized (call `learn()` first) or if
    /// `current_state.len() != self.dim`.
    pub fn predict(&mut self, current_state: &[u8]) -> Vec<u8> {
        assert!(
            self.stats.is_initialized,
            "KoopmanOperator::predict: K matrix not learned yet; call learn() first"
        );
        let lifted = self.lift(current_state);
        let predicted_lifted = mat_vec_mul(&self.k_matrix, &lifted);
        self.inv_lift(&predicted_lifted)
    }

    /// Compute the cosine distance between a predicted and actual VSA state.
    ///
    /// Returns a value in [0, 2] where 0 = identical direction and 1+ =
    /// increasingly divergent.
    pub fn prediction_error(&self, predicted: &[u8], actual: &[u8]) -> f64 {
        let p: Vec<f64> = predicted.iter().map(|&b| b as f64).collect();
        let a: Vec<f64> = actual.iter().map(|&b| b as f64).collect();
        let dot = dot_product(&p, &a);
        let norm_p = l2_norm(&p);
        let norm_a = l2_norm(&a);
        let cosine = dot / (norm_p * norm_a + 1e-12);
        (1.0 - cosine).max(0.0)
    }

    /// Record a prediction error to the history and update the EMA.
    pub fn record_error(&mut self, err: f64) {
        self.prediction_history.push(err);
        if self.prediction_history.len() > 1024 {
            self.prediction_history.remove(0);
        }
        self.stats.last_prediction_error = err;
        self.stats.prediction_error_ema =
            self.ema_alpha * err + (1.0 - self.ema_alpha) * self.stats.prediction_error_ema;
    }

    /// Compare Koopman prediction error with JEPA prediction error.
    ///
    /// Returns a qualitative comparison string.
    pub fn koopman_vs_jepa_error(&self, koopman_err: f64, jepa_err: f64) -> String {
        let diff = koopman_err - jepa_err;
        if diff < -0.1 {
            format!(
                "Koopman ({:.4}) < JEPA ({:.4}): linear model captures dynamics",
                koopman_err, jepa_err
            )
        } else if diff > 0.1 {
            format!(
                "Koopman ({:.4}) > JEPA ({:.4}): non-linear dynamics dominate",
                koopman_err, jepa_err
            )
        } else {
            format!(
                "Koopman ({:.4}) ≈ JEPA ({:.4}): comparable predictive performance",
                koopman_err, jepa_err
            )
        }
    }

    /// Reset the operator, clearing all learned state.
    pub fn reset(&mut self) {
        let dim = self.dim;
        self.k_matrix = vec![vec![0.0; dim]; dim];
        self.observation_buffer.clear();
        self.prediction_history.clear();
        self.stats = KoopmanStats {
            observation_count: 0,
            is_initialized: false,
            last_prediction_error: 0.0,
            prediction_error_ema: 0.0,
        };
    }

    /// Whether the K matrix has been learned.
    pub fn is_initialized(&self) -> bool {
        self.stats.is_initialized
    }

    /// Number of observations in the buffer.
    pub fn observation_count(&self) -> usize {
        self.stats.observation_count
    }

    /// Return a snapshot of current statistics.
    pub fn stats(&self) -> KoopmanStats {
        self.stats.clone()
    }

    /// Dimension of the observation space.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Return a reference to the K matrix (for inspection / debugging).
    pub fn k_matrix(&self) -> &[Vec<f64>] {
        &self.k_matrix
    }

    /// Return a reference to the prediction error history.
    pub fn prediction_history(&self) -> &[f64] {
        &self.prediction_history
    }
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DIM: usize = 64;

    fn make_koopman() -> KoopmanOperator {
        KoopmanOperator::new(TEST_DIM)
    }

    fn make_sine_wave_states(n: usize, dim: usize, freq: f64) -> Vec<Vec<u8>> {
        (0..n)
            .map(|t| {
                (0..dim)
                    .map(|i| {
                        let phase = 2.0 * f64::consts::PI * freq * (t as f64) + 0.1 * (i as f64);
                        ((phase.sin() * 0.5 + 0.5) * 255.0) as u8
                    })
                    .collect()
            })
            .collect()
    }

    // ── 1. Constructor ─────────────────────────────────────────────────

    #[test]
    fn test_new() {
        let kp = make_koopman();
        assert_eq!(kp.dim(), TEST_DIM);
        assert!(!kp.is_initialized());
        assert_eq!(kp.observation_count(), 0);
    }

    #[test]
    fn test_new_power_of_two() {
        KoopmanOperator::new(2);
        KoopmanOperator::new(4);
        KoopmanOperator::new(8);
        KoopmanOperator::new(16);
        KoopmanOperator::new(32);
        KoopmanOperator::new(64);
        KoopmanOperator::new(128);
        KoopmanOperator::new(256);
        KoopmanOperator::new(512);
    }

    #[test]
    #[should_panic(expected = "power of two")]
    fn test_new_not_power_of_two() {
        KoopmanOperator::new(10);
    }

    // ── 2. FWHT roundtrip ─────────────────────────────────────────────

    #[test]
    fn test_lift_inv_lift_roundtrip() {
        let kp = make_koopman();
        let original: Vec<u8> = (0..TEST_DIM).map(|i| (i * 7 % 256) as u8).collect();
        let lifted = kp.lift(&original);
        assert_eq!(lifted.len(), TEST_DIM);
        let recovered = kp.inv_lift(&lifted);
        assert_eq!(
            recovered, original,
            "roundtrip should recover original bytes"
        );
    }

    #[test]
    fn test_lift_uniform_state() {
        let kp = make_koopman();
        let state = vec![128u8; TEST_DIM];
        let lifted = kp.lift(&state);
        let sum: f64 = lifted.iter().map(|v| v * v).sum();
        let rms = (sum / TEST_DIM as f64).sqrt();
        assert!(
            rms > 0.0,
            "uniform state should have non-zero energy in FWHT space"
        );
        let recovered = kp.inv_lift(&lifted);
        assert_eq!(recovered, state);
    }

    // ── 3. K matrix learning ──────────────────────────────────────────

    #[test]
    fn test_learn_from_sine_wave() {
        let mut kp = make_koopman();
        let states = make_sine_wave_states(20, TEST_DIM, 0.05);
        kp.learn(&states);
        assert!(kp.is_initialized());
        assert!(kp.observation_count() >= 19);
    }

    #[test]
    fn test_learn_too_few() {
        let mut kp = make_koopman();
        let states = make_sine_wave_states(1, TEST_DIM, 0.05);
        kp.learn(&states);
        assert!(!kp.is_initialized(), "need at least 2 observations");
    }

    #[test]
    fn test_learn_empty() {
        let mut kp = make_koopman();
        kp.learn(&[]);
        assert!(!kp.is_initialized());
    }

    // ── 4. Prediction ─────────────────────────────────────────────────

    #[test]
    fn test_predict_after_learning() {
        let mut kp = make_koopman();
        let states = make_sine_wave_states(30, TEST_DIM, 0.03);
        kp.learn(&states);
        let prediction = kp.predict(&states[15]);
        assert_eq!(prediction.len(), TEST_DIM);
        let err = kp.prediction_error(&prediction, &states[16]);
        kp.record_error(err);
        assert!(err >= 0.0);
    }

    #[test]
    #[should_panic(expected = "not initialized")]
    fn test_predict_before_learn() {
        let mut kp = make_koopman();
        kp.predict(&vec![0u8; TEST_DIM]);
    }

    // ── 5. Prediction convergence ─────────────────────────────────────

    #[test]
    fn test_error_decreases_with_more_data() {
        let mut kp_small = make_koopman();
        let mut kp_large = make_koopman();

        let states = make_sine_wave_states(60, TEST_DIM, 0.02);

        kp_small.learn(&states[..10]);
        let pred_small = kp_small.predict(&states[9]);
        let err_small = kp_small.prediction_error(&pred_small, &states[10]);
        kp_small.record_error(err_small);

        kp_large.learn(&states[..50]);
        let pred_large = kp_large.predict(&states[49]);
        let err_large = kp_large.prediction_error(&pred_large, &states[50]);
        kp_large.record_error(err_large);

        assert!(
            err_large <= err_small + 0.05,
            "more data should not substantially increase error (small={:.4}, large={:.4})",
            err_small,
            err_large
        );
    }

    // ── 6. Error computation ──────────────────────────────────────────

    #[test]
    fn test_prediction_error_identical() {
        let kp = make_koopman();
        let state = vec![42u8; TEST_DIM];
        let err = kp.prediction_error(&state, &state);
        assert!(
            err.abs() < 1e-10,
            "identical states should have near-zero error, got {err}"
        );
    }

    #[test]
    fn test_prediction_error_opposite() {
        let kp = make_koopman();
        let a = vec![0u8; TEST_DIM];
        let b = vec![255u8; TEST_DIM];
        let err = kp.prediction_error(&a, &b);
        assert!(
            err > 0.0,
            "opposite states should have positive error, got {err}"
        );
    }

    // ── 7. Koopman vs JEPA comparison ─────────────────────────────────

    #[test]
    fn test_koopman_vs_jepa_comparison() {
        let kp = make_koopman();
        let s = kp.koopman_vs_jepa_error(0.2, 0.5);
        assert!(s.contains("Koopman"));
        assert!(s.contains("JEPA"));
        assert!(
            s.contains("captures dynamics") || s.contains("dominate") || s.contains("comparable")
        );
    }

    // ── 8. Reset ───────────────────────────────────────────────────────

    #[test]
    fn test_reset() {
        let mut kp = make_koopman();
        let states = make_sine_wave_states(10, TEST_DIM, 0.05);
        kp.learn(&states);
        assert!(kp.is_initialized());
        assert!(kp.observation_count() > 0);
        kp.reset();
        assert!(!kp.is_initialized());
        assert_eq!(kp.observation_count(), 0);
        assert_eq!(kp.prediction_history().len(), 0);
    }

    // ── 9. Statistics ─────────────────────────────────────────────────

    #[test]
    fn test_stats_snapshot() {
        let mut kp = make_koopman();
        let stats_before = kp.stats();
        assert_eq!(stats_before.observation_count, 0);
        assert!(!stats_before.is_initialized);

        let states = make_sine_wave_states(15, TEST_DIM, 0.05);
        kp.learn(&states);
        kp.record_error(0.123);

        let stats_after = kp.stats();
        assert!(stats_after.is_initialized);
        assert!((stats_after.last_prediction_error - 0.123).abs() < 1e-10);
    }

    // ── 10. Error history ─────────────────────────────────────────────

    #[test]
    fn test_prediction_history() {
        let mut kp = make_koopman();
        assert_eq!(kp.prediction_history().len(), 0);
        kp.record_error(0.1);
        kp.record_error(0.2);
        kp.record_error(0.3);
        assert_eq!(kp.prediction_history().len(), 3);
        assert!((kp.prediction_history()[2] - 0.3).abs() < 1e-10);
    }

    // ── 11. K matrix accessor ─────────────────────────────────────────

    #[test]
    fn test_k_matrix_after_learn() {
        let mut kp = make_koopman();
        let states = make_sine_wave_states(20, TEST_DIM, 0.04);
        kp.learn(&states);
        let km = kp.k_matrix();
        assert_eq!(km.len(), TEST_DIM);
        assert_eq!(km[0].len(), TEST_DIM);
        let has_nonzero = km.iter().flatten().any(|&v| v.abs() > 1e-10);
        assert!(
            has_nonzero,
            "K matrix should have non-zero entries after learning"
        );
    }

    // ── 12. Predict with noise tolerance ──────────────────────────────

    #[test]
    fn test_predict_noisy_input() {
        let mut kp = make_koopman();
        let states = make_sine_wave_states(30, TEST_DIM, 0.03);
        kp.learn(&states);

        let mut noisy = states[15].clone();
        noisy[0] = noisy[0].wrapping_add(1);
        let pred_clean = kp.predict(&states[15]);
        kp.reset();

        let mut kp2 = make_koopman();
        kp2.learn(&states);
        let pred_noisy = kp2.predict(&noisy);

        let err = kp2.prediction_error(&pred_clean, &pred_noisy);
        assert!(
            err < 0.5,
            "small input noise should not catastrophically change prediction, err={err}"
        );
    }
}
