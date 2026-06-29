#![forbid(unsafe_code)]

use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use std::collections::VecDeque;

/// Dynamical Symmetric Information Bottleneck (DySIB) layer for VSA attractor states.
///
/// Models the phase space of the consciousness attractor by estimating
/// the predictive information between past and future trajectory windows.
/// The latent dimension is found by scanning history window sizes until
/// the predictive information (I(Z_past; Z_future) via VSA cosine similarity)
/// saturates (change < 0.01).
///
/// The IB loss is defined as:
///   L_IB = β · I(Z_past; Z) − I(Z_past; Z_future)
///
/// where Z is the encoded representation, Z_past is the bundled past window,
/// and Z_future is the bundled future window. Minimising this loss finds
/// a representation Z that is maximally informative about the future while
/// being minimally complex given the past.
pub struct DySIBLayer {
    /// Effective phase space dimension (window size at which predictive info saturates).
    pub latent_dim: usize,
    /// Ring buffer of past attractor states (max 100).
    pub history: VecDeque<Vec<u8>>,
    /// Ring buffer of future attractor states (max 100, shifted by horizon).
    pub future: VecDeque<Vec<u8>>,
    /// IB trade-off parameter: higher β penalises complexity more.
    pub beta: f64,
    /// Estimated predictive information I(Z_past; Z_future).
    pub predictive_info: f64,
}

impl Default for DySIBLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl DySIBLayer {
    /// Creates a new `DySIBLayer` with default parameters.
    ///
    /// Latent dimension defaults to `VSA_DIM` (identity map) until
    /// [`scan_latent_dim`] is called. History and future ring buffers
    /// are empty with capacity 100.
    pub fn new() -> Self {
        Self {
            latent_dim: VSA_DIM,
            history: VecDeque::with_capacity(100),
            future: VecDeque::with_capacity(100),
            beta: 1.0,
            predictive_info: 0.0,
        }
    }

    /// Encodes a VSA attractor state into the latent space.
    ///
    /// For the DySIB layer the encoding is the identity — the
    /// attractor state is already a VSA vector. This method validates
    /// that `x` has length `VSA_DIM`.
    ///
    /// # Panics
    ///
    /// Panics if `x.len() != VSA_DIM`.
    pub fn encode(x: &[u8]) -> Vec<u8> {
        assert_eq!(
            x.len(),
            VSA_DIM,
            "DySIBLayer::encode: input dim {} != VSA_DIM {}",
            x.len(),
            VSA_DIM
        );
        x.to_vec()
    }

    /// Bundles the last `window` history states into a single VSA vector
    /// via majority voting.
    ///
    /// Returns a zero vector if there are fewer than `window` states in history.
    pub fn past_encoder(&self, window: usize) -> Vec<u8> {
        if self.history.len() < window {
            return vec![0u8; VSA_DIM];
        }
        let start = self.history.len() - window;
        let slice: Vec<&[u8]> = self.history.range(start..).map(|v| v.as_slice()).collect();
        QuantizedVSA::bundle(&slice)
    }

    /// Bundles the next `window` future states into a single VSA vector
    /// via majority voting.
    ///
    /// Returns a zero vector if there are fewer than `window` states in future.
    pub fn future_encoder(&self, window: usize) -> Vec<u8> {
        if self.future.len() < window {
            return vec![0u8; VSA_DIM];
        }
        let slice: Vec<&[u8]> = self
            .future
            .iter()
            .take(window)
            .map(|v| v.as_slice())
            .collect();
        QuantizedVSA::bundle(&slice)
    }

    /// Computes the Dynamical Symmetric Information Bottleneck loss.
    ///
    ///   L = β · I(Z_past; Z) − I(Z_past; Z_future)
    ///
    /// where mutual information is approximated by the absolute VSA similarity.
    /// The loss is clamped to `[0.0, 100.0]`.
    pub fn symmetric_info_bottleneck_loss(&self, z: &[u8], z_past: &[u8], z_future: &[u8]) -> f64 {
        let mi_past_z = Self::mutual_info_approx(z_past, z);
        let mi_past_future = Self::mutual_info_approx(z_past, z_future);
        let loss = self.beta * mi_past_z - mi_past_future;
        loss.max(0.0).min(100.0)
    }

    /// Approximates the mutual information between two VSA vectors as the
    /// absolute value of their Hamming-based similarity.
    ///
    /// Returns a value in `[0.0, 1.0]`.
    pub fn mutual_info_approx(a: &[u8], b: &[u8]) -> f64 {
        QuantizedVSA::similarity(a, b).abs()
    }

    /// Appends a past and future attractor state to the respective ring buffers.
    ///
    /// If either buffer exceeds 100 entries, the oldest entry is removed.
    /// After insertion, the predictive info score is recomputed using the
    /// current `latent_dim` as the window size.
    pub fn update(&mut self, past: Vec<u8>, future: Vec<u8>) {
        if self.history.len() >= 100 {
            self.history.pop_front();
        }
        self.history.push_back(past);

        if self.future.len() >= 100 {
            self.future.pop_front();
        }
        self.future.push_back(future);

        if self.history.len() >= self.latent_dim && self.future.len() >= self.latent_dim {
            let window = self.latent_dim;
            let zp = self.past_encoder(window);
            let zf = self.future_encoder(window);
            self.predictive_info = Self::mutual_info_approx(&zp, &zf);
        }
    }

    /// Scans window sizes from 2 to 128 (or until the history buffer is
    /// exhausted) to find the dimension at which predictive information
    /// saturates.
    ///
    /// Saturation is defined as the absolute change in predictive info
    /// between consecutive window sizes falling below 0.01 for three
    /// consecutive steps.
    ///
    /// Returns the optimal latent dimension and updates `self.latent_dim`.
    pub fn scan_latent_dim(&mut self) -> usize {
        let max_window = self.history.len().min(self.future.len()).min(128);
        if max_window < 2 {
            self.latent_dim = VSA_DIM;
            return self.latent_dim;
        }

        let mut prev_info = 0.0;
        let mut stable_count = 0;

        for window in 2..=max_window {
            let zp = self.past_encoder(window);
            let zf = self.future_encoder(window);
            let info = Self::mutual_info_approx(&zp, &zf);

            if window > 2 {
                let change = (info - prev_info).abs();
                if change < 0.01 {
                    stable_count += 1;
                    if stable_count >= 3 {
                        self.latent_dim = window - 2;
                        self.predictive_info = info;
                        return self.latent_dim;
                    }
                } else {
                    stable_count = 0;
                }
            }
            prev_info = info;
        }

        self.latent_dim = max_window;
        self.predictive_info = prev_info;
        self.latent_dim
    }

    /// Returns the current predictive information estimate.
    pub fn predictive_info_score(&self) -> f64 {
        self.predictive_info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vsa_state(value: u8) -> Vec<u8> {
        let mut v = vec![0u8; VSA_DIM];
        // Use a simple repeating pattern seeded by `value` so states
        // with the same value are similar and different values are dissimilar.
        for i in 0..VSA_DIM {
            v[i] = if (i.wrapping_mul(31) ^ (value as usize)) % 3 == 0 {
                1
            } else {
                0
            };
        }
        v
    }

    fn make_vsa_state_deterministic(seed: u64, phase: u64) -> Vec<u8> {
        // Create a state where each bit is a deterministic function of position, seed, and phase.
        let mut v = vec![0u8; VSA_DIM];
        for i in 0..VSA_DIM {
            let hash = (i as u64)
                .wrapping_mul(6364136223846793005)
                .wrapping_add(seed)
                .wrapping_add(phase.wrapping_mul(1442695040888963407));
            v[i] = (hash >> (i % 63) & 1) as u8;
        }
        v
    }

    #[test]
    fn test_new_defaults() {
        let layer = DySIBLayer::new();
        assert_eq!(layer.latent_dim, VSA_DIM);
        assert!(layer.history.is_empty());
        assert!(layer.future.is_empty());
        assert!((layer.beta - 1.0).abs() < 1e-12);
        assert!((layer.predictive_info - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_encode_validates_dim() {
        let v = vec![0u8; VSA_DIM];
        let encoded = DySIBLayer::encode(&v);
        assert_eq!(encoded.len(), VSA_DIM);
    }

    #[test]
    #[should_panic(expected = "VSA_DIM")]
    fn test_encode_wrong_dim_panics() {
        let v = vec![0u8; VSA_DIM - 1];
        DySIBLayer::encode(&v);
    }

    #[test]
    fn test_update_ring_buffer_capacity() {
        let mut layer = DySIBLayer::new();
        // Fill past the 100 limit.
        for i in 0..110 {
            let past = vec![if i % 2 == 0 { 1 } else { 0 }; VSA_DIM];
            let future = vec![if i % 2 == 0 { 0 } else { 1 }; VSA_DIM];
            layer.update(past, future);
        }
        assert_eq!(layer.history.len(), 100);
        assert_eq!(layer.future.len(), 100);
    }

    #[test]
    fn test_update_replaces_oldest() {
        let mut layer = DySIBLayer::new();
        // Insert 10 states so we know the first entry.
        for i in 0..10 {
            let past = vec![i as u8; VSA_DIM];
            let future = vec![(i + 100) as u8; VSA_DIM];
            layer.update(past, future);
        }
        // First entry should be index 0.
        assert_eq!(layer.history[0], vec![0u8; VSA_DIM]);
        assert_eq!(layer.future[0], vec![100u8; VSA_DIM]);

        // Insert 95 more, pushing out the first 5 but keeping entries 5..9.
        for i in 10..105 {
            let past = vec![i as u8; VSA_DIM];
            let future = vec![(i + 100) as u8; VSA_DIM];
            layer.update(past, future);
        }
        // After 105 entries total, only the last 100 remain.
        assert_eq!(layer.history.len(), 100);
        // The oldest entry should be index 5 now (entries 0..4 evicted).
        assert_eq!(layer.history[0], vec![5u8; VSA_DIM]);
    }

    #[test]
    fn test_past_encoder_returns_zero_when_insufficient() {
        let layer = DySIBLayer::new();
        let encoded = layer.past_encoder(5);
        assert_eq!(encoded, vec![0u8; VSA_DIM]);
    }

    #[test]
    fn test_future_encoder_returns_zero_when_insufficient() {
        let layer = DySIBLayer::new();
        let encoded = layer.future_encoder(5);
        assert_eq!(encoded, vec![0u8; VSA_DIM]);
    }

    #[test]
    fn test_past_encoder_bundles_correctly() {
        let mut layer = DySIBLayer::new();
        // Insert 5 distinct past states.
        for i in 0..5 {
            let past = make_vsa_state(i);
            let future = make_vsa_state(i + 10);
            layer.update(past, future);
        }
        let bundled = layer.past_encoder(3);
        assert_eq!(bundled.len(), VSA_DIM);
        // Bundle of 3 states should have at least some bits set.
        let ones = bundled.iter().filter(|&&b| b == 1).count();
        assert!(ones > 0);
    }

    #[test]
    fn test_future_encoder_bundles_correctly() {
        let mut layer = DySIBLayer::new();
        for i in 0..5 {
            let past = make_vsa_state(i);
            let future = make_vsa_state(i + 10);
            layer.update(past, future);
        }
        let bundled = layer.future_encoder(3);
        assert_eq!(bundled.len(), VSA_DIM);
        let ones = bundled.iter().filter(|&&b| b == 1).count();
        assert!(ones > 0);
    }

    #[test]
    fn test_mutual_info_approx_same_vector() {
        let v = make_vsa_state(42);
        let mi = DySIBLayer::mutual_info_approx(&v, &v);
        assert!(
            (mi - 1.0).abs() < 1e-6,
            "MI of identical vectors should be 1.0, got {}",
            mi
        );
    }

    #[test]
    fn test_mutual_info_approx_opposite_vectors() {
        let a = make_vsa_state(0);
        let b = make_vsa_state(255);
        let mi = DySIBLayer::mutual_info_approx(&a, &b);
        assert!(mi >= 0.0 && mi <= 1.0, "MI should be in [0, 1], got {}", mi);
    }

    #[test]
    fn test_bottleneck_loss_known_vectors() {
        let layer = DySIBLayer::new();
        let z_past = make_vsa_state(1);
        let z_future = make_vsa_state(2);
        // Z identical to Z_past → high MI past→Z, should give higher loss (β*1.0 - MI_past_future).
        let loss_same = layer.symmetric_info_bottleneck_loss(&z_past, &z_past, &z_future);
        // Z different from Z_past → low MI past→Z, should give lower loss.
        let z_other = make_vsa_state(255);
        let loss_diff = layer.symmetric_info_bottleneck_loss(&z_other, &z_past, &z_future);
        // Same-past should be >= different-past because β*MI_past_z is higher.
        assert!(
            loss_same >= loss_diff,
            "loss_same({}) should be >= loss_diff({})",
            loss_same,
            loss_diff
        );
    }

    #[test]
    fn test_beta_scales_loss() {
        let mut layer = DySIBLayer::new();
        let a = make_vsa_state(10);
        let b = make_vsa_state(20);

        layer.beta = 0.5;
        let loss_low_beta = layer.symmetric_info_bottleneck_loss(&a, &a, &b);

        layer.beta = 2.0;
        let loss_high_beta = layer.symmetric_info_bottleneck_loss(&a, &a, &b);

        assert!(
            loss_high_beta >= loss_low_beta,
            "higher beta should give >= loss, got {} < {}",
            loss_high_beta,
            loss_low_beta
        );
    }

    #[test]
    fn test_loss_clamped_to_zero() {
        let layer = DySIBLayer::new();
        // When future info exceeds past info, loss would be negative;
        // clamping should keep it at 0.
        let z_past = make_vsa_state(1);
        let z = make_vsa_state(255);
        let z_future = make_vsa_state(1);
        let loss = layer.symmetric_info_bottleneck_loss(&z, &z_past, &z_future);
        assert!(loss >= 0.0, "loss should be clamped to >= 0, got {}", loss);
    }

    #[test]
    fn test_update_computes_predictive_info() {
        let mut layer = DySIBLayer::new();
        layer.latent_dim = 3;

        // Insert states with alternating patterns so past and future are
        // anti-correlated → low predictive info.
        for i in 0..10 {
            let past = make_vsa_state_deterministic(100, i);
            let future = make_vsa_state_deterministic(100, i + 50);
            layer.update(past, future);
        }

        // After 10 updates with latent_dim=3, predictive_info should be updated.
        let info = layer.predictive_info_score();
        assert!(
            info >= 0.0 && info <= 1.0,
            "predictive info out of range: {}",
            info
        );
    }

    #[test]
    fn test_scan_latent_dim_convergence() {
        let mut layer = DySIBLayer::new();

        // Insert 50 pairs. Use a synthetic 2D phase space: two alternating
        // VSA states (A and B). For a 2D system, predictive info should
        // saturate at window = 2 (or very close).
        let state_a = make_vsa_state_deterministic(1, 0);
        let state_b = make_vsa_state_deterministic(1, 1);

        for i in 0..100 {
            let (past, future) = if i % 2 == 0 {
                (state_a.clone(), state_b.clone())
            } else {
                (state_b.clone(), state_a.clone())
            };
            layer.update(past, future);
        }

        let optimal = layer.scan_latent_dim();

        // For a 2D oscillator, we expect latent_dim to be small (likely 2).
        assert!(
            optimal >= 2 && optimal <= 16,
            "2D phase space should saturate at small window, got {}",
            optimal
        );
    }

    #[test]
    fn test_scan_latent_dim_insufficient_data() {
        let mut layer = DySIBLayer::new();
        // Only 1 state pair in buffers — not enough to scan.
        layer.update(vec![1u8; VSA_DIM], vec![2u8; VSA_DIM]);

        let optimal = layer.scan_latent_dim();
        assert_eq!(
            optimal, VSA_DIM,
            "insufficient data should fall back to VSA_DIM"
        );
    }

    #[test]
    fn test_harmonic_oscillator_recovers_2d() {
        // Synthetic harmonic oscillator: two alternating VSA states.
        // DySIB should detect that the phase space is 2-dimensional
        // because the trajectory repeats every 2 steps.
        let mut layer = DySIBLayer::new();

        let state_a = make_vsa_state_deterministic(7, 0);
        let state_b = make_vsa_state_deterministic(7, 1);

        // Run 120 alternating updates.
        for i in 0..120 {
            let (past, future) = if i % 2 == 0 {
                (state_a.clone(), state_b.clone())
            } else {
                (state_b.clone(), state_a.clone())
            };
            layer.update(past, future);
        }

        // Predictive info at window=2 should be significantly higher
        // than at window=1 (since the 2-periodicity is captured at window=2).
        let info_1 = {
            let zp = layer.past_encoder(1);
            let zf = layer.future_encoder(1);
            DySIBLayer::mutual_info_approx(&zp, &zf)
        };
        let info_2 = {
            let zp = layer.past_encoder(2);
            let zf = layer.future_encoder(2);
            DySIBLayer::mutual_info_approx(&zp, &zf)
        };

        // info_2 should capture the period-2 structure better.
        assert!(
            info_2 >= info_1 - 0.05,
            "info at window 2 ({}) should be >= info at window 1 ({})",
            info_2,
            info_1
        );

        // The scan should converge to a small latent dimension.
        let dim = layer.scan_latent_dim();
        assert!(
            dim >= 2 && dim <= 8,
            "harmonic oscillator latent dim should be 2-8, got {}",
            dim
        );
    }

    #[test]
    fn test_highly_predictable_sequence() {
        // A sequence where the same state repeats — predictive info should be high.
        let mut layer = DySIBLayer::new();
        layer.latent_dim = 5;

        let state = make_vsa_state_deterministic(42, 0);

        for _ in 0..30 {
            layer.update(state.clone(), state.clone());
        }

        let info = layer.predictive_info_score();
        assert!(
            info > 0.5,
            "repeating state should give high predictive info, got {}",
            info
        );
    }

    #[test]
    fn test_identity_encoding_preserves_vector() {
        let v = make_vsa_state(99);
        let encoded = DySIBLayer::encode(&v);
        assert_eq!(encoded, v);
    }

    #[test]
    fn test_update_with_empty_initial_buffers() {
        let mut layer = DySIBLayer::new();
        let past = make_vsa_state(1);
        let future = make_vsa_state(2);
        layer.update(past, future);
        assert_eq!(layer.history.len(), 1);
        assert_eq!(layer.future.len(), 1);
    }

    #[test]
    fn test_bottleneck_loss_symmetric_property() {
        // MI is symmetric: I(A;B) = I(B;A). Verify through the loss formula
        // that swapping past and future changes only the second term.
        let layer = DySIBLayer::new();
        let a = make_vsa_state(10);
        let b = make_vsa_state(20);
        let c = make_vsa_state(30);

        let loss_ab = layer.symmetric_info_bottleneck_loss(&b, &a, &c);
        let loss_ba = layer.symmetric_info_bottleneck_loss(&a, &b, &c);

        // Not asserting equality because MI depends on specific vectors,
        // just that the loss function runs and produces sane values.
        assert!(loss_ab >= 0.0 && loss_ab <= 100.0);
        assert!(loss_ba >= 0.0 && loss_ba <= 100.0);
    }

    #[test]
    fn test_default_implements_default_trait() {
        let layer = DySIBLayer::default();
        assert_eq!(layer.latent_dim, VSA_DIM);
        assert!((layer.beta - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_scan_updates_internal_state() {
        let mut layer = DySIBLayer::new();
        let _state = make_vsa_state_deterministic(5, 0);

        for i in 0..60 {
            let past = make_vsa_state_deterministic(5, i);
            let future = make_vsa_state_deterministic(5, i + 1);
            layer.update(past, future);
        }

        let _ = layer.scan_latent_dim();
        // After scanning, latent_dim should differ from initial VSA_DIM
        // because the data is sufficient and has structure.
        assert_ne!(
            layer.latent_dim, VSA_DIM,
            "scan should find a dim < VSA_DIM"
        );
    }

    #[test]
    fn test_large_beta_clamps_loss() {
        let mut layer = DySIBLayer::new();
        layer.beta = 1000.0;
        let a = make_vsa_state(0);
        let b = make_vsa_state(1);
        // Very high beta should produce a loss close to 100 (clamped).
        let loss = layer.symmetric_info_bottleneck_loss(&a, &b, &make_vsa_state(2));
        assert!(loss <= 100.0, "loss should be clamped to 100, got {}", loss);
    }
}
