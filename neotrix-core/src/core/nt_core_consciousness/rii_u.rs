use std::collections::VecDeque;

const HISTORY_SIZE: usize = 20;
const META_MU_SIZE: usize = 20;
const BROADCAST_BUFFER_SIZE: usize = 10;

/// RIIU-inspired Auto-Φ (automatic integration information) pattern
/// (arXiv:2506.13825). Upgrades fixed-weight meta-cognitive scoring
/// with adaptive weights, causal footprint (meta_mu), and a global
/// workspace broadcast buffer.
///
/// # Fields
/// - `weights`: adaptive [w1, w2, w3] normalized to sum 1.0
/// - `meta_mu`: causal footprint — tracks (meta_accuracy, weights) pairs
///   that produced good outcomes (meta_accuracy > 0.7)
/// - `broadcast_buffer`: global workspace aggregating top-K signal values
/// - `top_k`: number of top signals retained in the broadcast buffer
pub struct RiiuAutoPhi {
    pub weights: [f64; 3],
    history: Vec<(f64, f64)>,
    pub lr: f64,
    /// Causal footprint: which weight configurations led to high meta-accuracy
    pub meta_mu: VecDeque<(f64, [f64; 3])>,
    /// Global workspace broadcast buffer — top-K signal amplitudes
    pub broadcast_buffer: VecDeque<f64>,
    /// Number of top signals to retain in broadcast buffer
    pub top_k: usize,
    /// Last predicted health for meta-accuracy computation
    last_predicted: f64,
}

impl RiiuAutoPhi {
    pub fn new() -> Self {
        Self {
            weights: [0.3, 0.3, 0.4],
            history: Vec::with_capacity(HISTORY_SIZE),
            lr: 0.05,
            meta_mu: VecDeque::with_capacity(META_MU_SIZE),
            broadcast_buffer: VecDeque::with_capacity(BROADCAST_BUFFER_SIZE),
            top_k: 3,
            last_predicted: 0.5,
        }
    }

    pub fn compute_health(&self, inputs: &[f64; 3]) -> f64 {
        (self.weights[0] * inputs[0] + self.weights[1] * inputs[1] + self.weights[2] * inputs[2])
            .clamp(0.0, 1.0)
    }

    pub fn update(&mut self, predicted: f64, actual: f64, inputs: &[f64; 3]) {
        let error = predicted - actual;
        // Meta-accuracy: how well we predicted vs actual (1.0 = perfect)
        let meta_acc = 1.0 - error.abs().clamp(0.0, 1.0);
        for i in 0..3 {
            let gradient = error * inputs[i];
            let lr_adj = self.lr * (1.0 + meta_acc * 0.5);
            self.weights[i] = (self.weights[i] - lr_adj * gradient).clamp(0.05, 0.9);
        }
        let sum: f64 = self.weights.iter().sum();
        if sum > 0.0 {
            for w in &mut self.weights {
                *w /= sum;
            }
        } else {
            self.weights = [1.0 / 3.0; 3];
        }
    }

    pub fn record_and_update(&mut self, inputs: &[f64; 3], actual_health: f64) -> f64 {
        let predicted = self.compute_health(inputs);
        self.history.push((predicted, actual_health));
        if self.history.len() > HISTORY_SIZE {
            self.history.remove(0);
        }
        self.update(predicted, actual_health, inputs);

        let meta_acc = 1.0 - (predicted - actual_health).abs().clamp(0.0, 1.0);
        // Record causal footprint when meta-accuracy is good
        if meta_acc > 0.7 {
            self.meta_mu.push_back((meta_acc, self.weights));
            if self.meta_mu.len() > META_MU_SIZE {
                self.meta_mu.pop_front();
            }
        }

        // Broadcast buffer: push all input signals, retain top-K
        for &v in inputs {
            self.broadcast_buffer.push_back(v);
        }
        while self.broadcast_buffer.len() > BROADCAST_BUFFER_SIZE {
            self.broadcast_buffer.pop_front();
        }
        // Retain only top-K by value
        let mut vec: Vec<f64> = self.broadcast_buffer.iter().copied().collect();
        vec.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        vec.truncate(self.top_k);
        self.broadcast_buffer = vec.into_iter().collect();

        self.last_predicted = predicted;
        predicted
    }

    /// Meta-accuracy: how well the health prediction matched actual.
    /// Returns (mean_abs_error, last_meta_accuracy).
    pub fn meta_accuracy(&self) -> (f64, f64) {
        let mae = self.mean_abs_error();
        let last_meta_acc = if let Some((p, a)) = self.history.last() {
            1.0 - (p - a).abs().clamp(0.0, 1.0)
        } else {
            0.0
        };
        (mae, last_meta_acc)
    }

    /// The current broadcast buffer content (top-K signals).
    pub fn broadcast(&self) -> Vec<f64> {
        self.broadcast_buffer.iter().copied().collect()
    }

    /// Number of causal footprint entries.
    pub fn meta_mu_len(&self) -> usize {
        self.meta_mu.len()
    }

    /// Mean weight from causal footprint — the weight configuration that
    /// historically produced the best meta-accuracy.
    pub fn mu_centroid(&self) -> [f64; 3] {
        let n = self.meta_mu.len();
        if n == 0 {
            return self.weights;
        }
        let mut sum = [0.0f64; 3];
        for (_, w) in &self.meta_mu {
            for i in 0..3 {
                sum[i] += w[i];
            }
        }
        let nf = n as f64;
        [sum[0] / nf, sum[1] / nf, sum[2] / nf]
    }

    pub fn with_lr(learning_rate: f64) -> Self {
        Self {
            weights: [0.3, 0.3, 0.4],
            history: Vec::with_capacity(HISTORY_SIZE),
            lr: learning_rate,
            meta_mu: VecDeque::with_capacity(META_MU_SIZE),
            broadcast_buffer: VecDeque::with_capacity(BROADCAST_BUFFER_SIZE),
            top_k: 3,
            last_predicted: 0.5,
        }
    }

    pub fn with_weights(weights: [f64; 3]) -> Self {
        Self {
            weights,
            history: Vec::with_capacity(HISTORY_SIZE),
            lr: 0.05,
            meta_mu: VecDeque::with_capacity(META_MU_SIZE),
            broadcast_buffer: VecDeque::with_capacity(BROADCAST_BUFFER_SIZE),
            top_k: 3,
            last_predicted: 0.5,
        }
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn mean_abs_error(&self) -> f64 {
        let n = self.history.len();
        if n == 0 {
            return 0.0;
        }
        self.history.iter().map(|(p, a)| (p - a).abs()).sum::<f64>() / n as f64
    }
}

impl Default for RiiuAutoPhi {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_weights() {
        let r = RiiuAutoPhi::new();
        assert_eq!(r.weights, [0.3, 0.3, 0.4]);
    }

    #[test]
    fn test_compute_health_clamped() {
        let r = RiiuAutoPhi::new();
        let h = r.compute_health(&[0.8, 0.7, 0.9]);
        assert!((0.0..=1.0).contains(&h));
        assert!((h - (0.3 * 0.8 + 0.3 * 0.7 + 0.4 * 0.9)).abs() < 1e-12);
    }

    #[test]
    fn test_record_and_update_returns_health() {
        let mut r = RiiuAutoPhi::new();
        let h = r.record_and_update(&[0.8, 0.7, 0.9], 0.75);
        assert!((0.0..=1.0).contains(&h));
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_weights_normalized_after_update() {
        let mut r = RiiuAutoPhi::new();
        r.update(0.8, 0.6, &[1.0, 1.0, 1.0]);
        let sum: f64 = r.weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-12);
        for &w in &r.weights {
            assert!(w >= 0.05 && w <= 0.9);
        }
    }

    #[test]
    fn test_weights_adapt_after_error() {
        let mut r = RiiuAutoPhi::new();
        let initial = r.weights;
        r.update(1.0, 0.0, &[1.0, 1.0, 1.0]);
        assert_ne!(initial, r.weights);
    }

    #[test]
    fn test_history_capped_at_20() {
        let mut r = RiiuAutoPhi::new();
        for _ in 0..30 {
            r.record_and_update(&[0.5, 0.5, 0.5], 0.5);
        }
        assert!(r.len() <= HISTORY_SIZE);
    }

    #[test]
    fn test_mean_abs_error() {
        let mut r = RiiuAutoPhi::new();
        r.record_and_update(&[1.0, 1.0, 1.0], 0.8);
        let mae = r.mean_abs_error();
        assert!((mae - 0.2).abs() < 1e-12);
    }

    #[test]
    fn test_weights_clamped_lower() {
        let mut r = RiiuAutoPhi::with_weights([0.05, 0.05, 0.9]);
        r.update(1.0, -10.0, &[1.0, 1.0, 1.0]);
        for &w in &r.weights {
            assert!(w >= 0.05, "weight {} below 0.05", w);
        }
    }

    #[test]
    fn test_all_weights_zero_recovery() {
        let mut r = RiiuAutoPhi::with_weights([0.0, 0.0, 0.0]);
        let h = r.compute_health(&[0.5, 0.5, 0.5]);
        assert!((h - 0.0).abs() < 1e-12);
        r.update(0.0, 0.5, &[0.5, 0.5, 0.5]);
        let sum: f64 = r.weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_with_lr() {
        let r = RiiuAutoPhi::with_lr(0.1);
        assert!((r.lr - 0.1).abs() < 1e-12);
    }

    #[test]
    fn test_meta_mu_tracks_good_outcomes() {
        let mut r = RiiuAutoPhi::new();
        // Feed accurate predictions to build causal footprint
        r.record_and_update(&[0.9, 0.9, 0.9], 0.85);
        assert_eq!(r.meta_mu_len(), 1);
        let centroid = r.mu_centroid();
        assert!((centroid[0] + centroid[1] + centroid[2] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_broadcast_buffer_top_k() {
        let mut r = RiiuAutoPhi::new();
        r.record_and_update(&[0.9, 0.5, 0.3], 0.7);
        let buf = r.broadcast();
        assert!(buf.len() <= r.top_k);
        assert!(buf.contains(&0.9));
    }

    #[test]
    fn test_meta_accuracy_returns_metrics() {
        let mut r = RiiuAutoPhi::new();
        r.record_and_update(&[0.8, 0.7, 0.9], 0.75);
        let (mae, last) = r.meta_accuracy();
        assert!(mae > 0.0);
        assert!(last > 0.0);
    }
}
