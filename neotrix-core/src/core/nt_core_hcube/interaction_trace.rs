//! P1.28 InteractionTracePredictor — 3D interaction trace prediction via
//! VSA trajectory encoding (μ0 model inspired).
//!
//! Encodes 3D keypoint sequences into VSA vectors for:
//! - Future trajectory prediction via linear extrapolation in VSA space
//! - Trace consistency evaluation via smooth L1 loss
//! - Cross-embodiment skill transfer detection via VSA similarity

#![forbid(unsafe_code)]

use std::collections::VecDeque;

use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;
use crate::core::nt_core_hcube::QuantizedVSA;

const CONTACT_SEED: u64 = 0x4E455F54_52495800;

/// A labeled 3D keypoint, optionally indicating contact.
#[derive(Debug, Clone)]
pub struct Keypoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub label: String,
    pub contact: bool,
}

/// A snapshot containing a set of keypoints at a given timestamp.
#[derive(Debug, Clone)]
pub struct TracePoint {
    pub keypoints: Vec<Keypoint>,
    pub timestamp: u64,
}

/// Predicts future interaction traces by encoding 3D keypoint trajectories
/// into VSA vectors and extrapolating centroid motion.
///
/// Maintains a rolling history of the last 200 trace snapshots.
pub struct InteractionTracePredictor {
    trace_history: VecDeque<TracePoint>,
    prediction_horizon: usize,
    trace_dim: usize,
}

impl InteractionTracePredictor {
    /// Creates a new predictor with default horizon (5) and VSA_DIM (4096).
    pub fn new() -> Self {
        Self {
            trace_history: VecDeque::with_capacity(200),
            prediction_horizon: 5,
            trace_dim: VSA_DIM,
        }
    }

    /// Creates a predictor with custom horizon and dimension.
    pub fn with_params(prediction_horizon: usize, trace_dim: usize) -> Self {
        Self {
            trace_history: VecDeque::with_capacity(200),
            prediction_horizon,
            trace_dim,
        }
    }

    /// Returns a reference to the trace history.
    pub fn history(&self) -> &VecDeque<TracePoint> {
        &self.trace_history
    }

    /// Returns the current prediction horizon.
    pub fn prediction_horizon(&self) -> usize {
        self.prediction_horizon
    }

    /// Returns the trace dimension.
    pub fn trace_dim(&self) -> usize {
        self.trace_dim
    }

    /// Encodes a single 3D keypoint into a VSA vector.
    ///
    /// Position is encoded via `seeded_random` with a hash of the
    /// quantised coordinates. The label is XOR-bound as a mask.
    /// If `contact` is true, an additional contact seed is XOR-bound.
    pub fn encode_keypoint(kp: &Keypoint) -> Vec<u8> {
        let hash_pos =
            ((kp.x * 1000.0) as u64) ^ ((kp.y * 1000.0) as u64) ^ ((kp.z * 1000.0) as u64);
        let mut vec = QuantizedVSA::seeded_random(hash_pos, VSA_DIM);

        // Bind label mask
        let label_hash = hash_str(&kp.label);
        let label_mask = QuantizedVSA::seeded_random(label_hash, VSA_DIM);
        vec = QuantizedVSA::xor_bind(&vec, &label_mask);

        // Bind contact marker
        if kp.contact {
            let contact_mask = QuantizedVSA::seeded_random(CONTACT_SEED, VSA_DIM);
            vec = QuantizedVSA::xor_bind(&vec, &contact_mask);
        }

        vec
    }

    /// Encodes an ordered trace of keypoints into a single VSA vector.
    ///
    /// Each keypoint is first individually encoded, then permuted by its
    /// index (shift = position) and finally bundled via majority summation.
    pub fn encode_trace(&self, trace: &[Keypoint]) -> Vec<u8> {
        if trace.is_empty() {
            return vec![0; self.trace_dim];
        }

        let encoded: Vec<Vec<u8>> = trace.iter().map(|kp| Self::encode_keypoint(kp)).collect();

        let bundled: Vec<Vec<u8>> = encoded
            .into_iter()
            .enumerate()
            .map(|(i, e)| QuantizedVSA::permute(&e, i as isize))
            .collect();

        let refs: Vec<&[u8]> = bundled.iter().map(|v| v.as_slice()).collect();
        QuantizedVSA::bundle(&refs)
    }

    /// Predicts the next `n_steps` future traces by linearly extrapolating
    /// the centroid movement observed in the last 3 history entries.
    ///
    /// Each predicted step offsets the current keypoints by the average
    /// per-step centroid delta, then VSA-encodes the result.
    pub fn predict_horizon(&self, n_steps: usize, current_trace: &[Keypoint]) -> Vec<Vec<u8>> {
        if current_trace.is_empty() || n_steps == 0 {
            return Vec::new();
        }

        let current_centroid = centroid_of(current_trace);

        // Gather up to 3 history centroids
        let history_count = self.trace_history.len();
        let n_history = history_count.min(3);
        let mut centroids: Vec<(f32, f32, f32)> = Vec::with_capacity(n_history);
        for i in (history_count.saturating_sub(n_history))..history_count {
            if let Some(tp) = self.trace_history.get(i) {
                centroids.push(centroid_of(&tp.keypoints));
            }
        }
        centroids.push(current_centroid);

        // Compute average per-step delta
        let (dx, dy, dz) = if centroids.len() >= 2 {
            let mut sum_dx = 0.0f32;
            let mut sum_dy = 0.0f32;
            let mut sum_dz = 0.0f32;
            for i in 1..centroids.len() {
                sum_dx += centroids[i].0 - centroids[i - 1].0;
                sum_dy += centroids[i].1 - centroids[i - 1].1;
                sum_dz += centroids[i].2 - centroids[i - 1].2;
            }
            let count = (centroids.len() - 1) as f32;
            if count == 0.0 {
                (0.0, 0.0, 0.0)
            } else {
                (sum_dx / count, sum_dy / count, sum_dz / count)
            }
        } else {
            (0.0, 0.0, 0.0)
        };

        let mut predictions = Vec::with_capacity(n_steps);
        for step in 1..=n_steps {
            let s = step as f32;
            let predicted: Vec<Keypoint> = current_trace
                .iter()
                .map(|kp| Keypoint {
                    x: kp.x + dx * s,
                    y: kp.y + dy * s,
                    z: kp.z + dz * s,
                    label: kp.label.clone(),
                    contact: kp.contact,
                })
                .collect();
            predictions.push(self.encode_trace(&predicted));
        }

        predictions
    }

    /// Records a trace snapshot into the rolling history (max 200).
    pub fn record_trace(&mut self, trace: &[Keypoint]) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let tp = TracePoint {
            keypoints: trace.to_vec(),
            timestamp,
        };

        if self.trace_history.len() >= 200 {
            self.trace_history.pop_front();
        }
        self.trace_history.push_back(tp);
    }

    /// Computes the mean smooth L1 loss between predicted and actual
    /// trace encodings. If the lengths differ, the extra elements are
    /// ignored.
    pub fn trace_consistency_loss(predicted: &[Vec<u8>], actual: &[Vec<u8>]) -> f64 {
        let n = predicted.len().min(actual.len());
        if n == 0 {
            return 0.0;
        }
        let mut total = 0.0;
        for i in 0..n {
            total += smooth_l1_loss(&predicted[i], &actual[i], 1.0);
        }
        total / n as f64
    }

    /// Computes VSA cosine similarity between a human trace and a robot
    /// trace encoding. High similarity indicates embodiment-agnostic skill
    /// transfer is feasible.
    pub fn cross_embodiment_transfer(
        &self,
        human_trace: &[Keypoint],
        robot_trace: &[Keypoint],
    ) -> f64 {
        let human_vec = self.encode_trace(human_trace);
        let robot_vec = self.encode_trace(robot_trace);
        QuantizedVSA::similarity(&human_vec, &robot_vec)
    }
}

impl Default for InteractionTracePredictor {
    fn default() -> Self {
        Self::new()
    }
}

// --- helper functions ---

/// Simple string-to-u64 hash (djb2-like).
fn hash_str(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u64);
    }
    h
}

/// Computes the centroid (average position) of a set of keypoints.
fn centroid_of(kps: &[Keypoint]) -> (f32, f32, f32) {
    if kps.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let n = kps.len() as f32;
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut cz = 0.0f32;
    for kp in kps {
        cx += kp.x;
        cy += kp.y;
        cz += kp.z;
    }
    (cx / n, cy / n, cz / n)
}

/// Smooth L1 loss between two byte slices.
///
/// For each byte pair with absolute difference `d`:
/// - if `d < delta`: `0.5 * d² / delta`
/// - else: `d - 0.5 * delta`
///
/// Returns the mean across all byte pairs.
pub fn smooth_l1_loss(a: &[u8], b: &[u8], delta: f64) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    let mut total = 0.0;
    for i in 0..n {
        let d = (a[i] as f64 - b[i] as f64).abs();
        if d < delta {
            total += 0.5 * d * d / delta;
        } else {
            total += d - 0.5 * delta;
        }
    }
    total / n as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trace(origin_x: f32, step: f32, length: usize) -> Vec<Keypoint> {
        (0..length)
            .map(|i| Keypoint {
                x: origin_x + i as f32 * step,
                y: 0.0,
                z: 0.0,
                label: "test".into(),
                contact: false,
            })
            .collect()
    }

    fn single_kp(x: f32, y: f32, z: f32) -> Vec<Keypoint> {
        vec![Keypoint {
            x,
            y,
            z,
            label: "p".into(),
            contact: false,
        }]
    }

    // --- smooth_l1_loss ---

    #[test]
    fn test_smooth_l1_identical() {
        let a = vec![100u8; 64];
        let b = vec![100u8; 64];
        let loss = smooth_l1_loss(&a, &b, 1.0);
        assert!((loss - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_smooth_l1_positive() {
        let a = vec![0u8; 64];
        let b = vec![10u8; 64];
        let loss = smooth_l1_loss(&a, &b, 1.0);
        assert!(loss > 0.0);
    }

    #[test]
    fn test_smooth_l1_empty() {
        assert!((smooth_l1_loss(&[], &[], 1.0) - 0.0).abs() < 1e-10);
    }

    // --- encode_keypoint ---

    #[test]
    fn test_encode_keypoint_deterministic() {
        let kp = Keypoint {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            label: "gripper".into(),
            contact: false,
        };
        let a = InteractionTracePredictor::encode_keypoint(&kp);
        let b = InteractionTracePredictor::encode_keypoint(&kp);
        assert_eq!(a, b);
    }

    #[test]
    fn test_encode_keypoint_differs_by_label() {
        let kp_a = Keypoint {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            label: "left".into(),
            contact: false,
        };
        let kp_b = Keypoint {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            label: "right".into(),
            contact: false,
        };
        let a = InteractionTracePredictor::encode_keypoint(&kp_a);
        let b = InteractionTracePredictor::encode_keypoint(&kp_b);
        assert_ne!(a, b);
    }

    #[test]
    fn test_encode_keypoint_contact_differs() {
        let kp_a = Keypoint {
            x: 5.0,
            y: 5.0,
            z: 5.0,
            label: "finger".into(),
            contact: false,
        };
        let kp_b = Keypoint {
            x: 5.0,
            y: 5.0,
            z: 5.0,
            label: "finger".into(),
            contact: true,
        };
        let a = InteractionTracePredictor::encode_keypoint(&kp_a);
        let b = InteractionTracePredictor::encode_keypoint(&kp_b);
        assert_ne!(a, b);
    }

    // --- encode_trace ---

    #[test]
    fn test_encode_trace_empty() {
        let p = InteractionTracePredictor::new();
        let v = p.encode_trace(&[]);
        assert_eq!(v.len(), VSA_DIM);
        assert!(v.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_encode_trace_similar_traces_similar() {
        let p = InteractionTracePredictor::new();
        let t1 = make_trace(0.0, 1.0, 5);
        let t2 = make_trace(0.1, 1.0, 5);
        let v1 = p.encode_trace(&t1);
        let v2 = p.encode_trace(&t2);
        let sim = QuantizedVSA::similarity(&v1, &v2);
        // VSA-encoded similar trajectories should have non-trivial similarity
        assert!(sim > 0.3, "similarity {} too low", sim);
    }

    #[test]
    fn test_encode_trace_distinct_traces_distinct() {
        let p = InteractionTracePredictor::new();
        let t1 = make_trace(0.0, 1.0, 5);
        let t2 = make_trace(100.0, 1.0, 5);
        let v1 = p.encode_trace(&t1);
        let v2 = p.encode_trace(&t2);
        let sim = QuantizedVSA::similarity(&v1, &v2);
        assert!(sim < 0.6, "similarity {} too high", sim);
    }

    // --- predict_horizon ---

    #[test]
    fn test_predict_horizon_output_count() {
        let mut p = InteractionTracePredictor::new();
        // Seed history with some traces for velocity estimation
        let trace = single_kp(0.0, 0.0, 0.0);
        p.record_trace(&make_trace(0.0, 1.0, 3));
        p.record_trace(&make_trace(0.0, 2.0, 3));
        p.record_trace(&make_trace(0.0, 3.0, 3));

        let preds = p.predict_horizon(5, &trace);
        assert_eq!(preds.len(), 5);
    }

    #[test]
    fn test_predict_horizon_empty_trace() {
        let p = InteractionTracePredictor::new();
        let preds = p.predict_horizon(3, &[]);
        assert!(preds.is_empty());
    }

    #[test]
    fn test_predict_horizon_zero_steps() {
        let p = InteractionTracePredictor::new();
        let trace = single_kp(1.0, 2.0, 3.0);
        let preds = p.predict_horizon(0, &trace);
        assert!(preds.is_empty());
    }

    #[test]
    fn test_predict_horizon_no_history() {
        let p = InteractionTracePredictor::new();
        let trace = single_kp(1.0, 2.0, 3.0);
        let preds = p.predict_horizon(3, &trace);
        assert_eq!(preds.len(), 3);
        // Without history, each prediction should be the same (velocity = 0)
        assert_eq!(preds[0], preds[1]);
        assert_eq!(preds[1], preds[2]);
    }

    // --- trace_consistency_loss ---

    #[test]
    fn test_loss_identical() {
        let p = InteractionTracePredictor::new();
        let trace = make_trace(0.0, 1.0, 5);
        let enc = p.encode_trace(&trace);
        let pred = vec![enc.clone(); 3];
        let act = vec![enc; 3];
        let loss = InteractionTracePredictor::trace_consistency_loss(&pred, &act);
        assert!((loss - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_loss_different() {
        let p = InteractionTracePredictor::new();
        let trace_a = make_trace(0.0, 1.0, 5);
        let trace_b = make_trace(100.0, 1.0, 5);
        let enc_a = p.encode_trace(&trace_a);
        let enc_b = p.encode_trace(&trace_b);
        let loss = InteractionTracePredictor::trace_consistency_loss(&[enc_a], &[enc_b]);
        assert!(loss > 0.0);
    }

    #[test]
    fn test_loss_ignores_extra() {
        let a = vec![vec![0u8; VSA_DIM]; 2];
        let b = vec![vec![0u8; VSA_DIM]; 5];
        let loss = InteractionTracePredictor::trace_consistency_loss(&a, &b);
        assert!((loss - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_loss_empty() {
        let loss = InteractionTracePredictor::trace_consistency_loss(&[], &[]);
        assert!((loss - 0.0).abs() < 1e-10);
    }

    // --- cross_embodiment_transfer ---

    #[test]
    fn test_cross_embodiment_similar_pattern() {
        let p = InteractionTracePredictor::new();
        // Same parabola pattern at different scales
        let human: Vec<Keypoint> = (0..10)
            .map(|i| {
                let t = i as f32;
                Keypoint {
                    x: t,
                    y: t * t,
                    z: 0.0,
                    label: "end-effector".into(),
                    contact: false,
                }
            })
            .collect();

        let robot: Vec<Keypoint> = (0..10)
            .map(|i| {
                let t = i as f32 * 0.5;
                Keypoint {
                    x: t,
                    y: t * t * 4.0,
                    z: 0.0,
                    label: "end-effector".into(),
                    contact: false,
                }
            })
            .collect();

        let sim = p.cross_embodiment_transfer(&human, &robot);
        assert!(sim > 0.2, "cross-embodiment similarity {} too low", sim);
    }

    #[test]
    fn test_cross_embodiment_different_pattern() {
        let p = InteractionTracePredictor::new();
        let upward: Vec<Keypoint> = (0..10)
            .map(|i| Keypoint {
                x: 0.0,
                y: i as f32,
                z: 0.0,
                label: "tip".into(),
                contact: false,
            })
            .collect();
        let downward: Vec<Keypoint> = (0..10)
            .map(|i| Keypoint {
                x: 0.0,
                y: 10.0 - i as f32,
                z: 0.0,
                label: "tip".into(),
                contact: false,
            })
            .collect();
        let sim = p.cross_embodiment_transfer(&upward, &downward);
        // Different trajectories should have lower similarity than identical patterns
        let same_sim = p.cross_embodiment_transfer(&upward, &upward);
        assert!(
            sim < same_sim,
            "different patterns ({}) should be less similar than identical ({})",
            sim,
            same_sim
        );
    }

    // --- record_trace ---

    #[test]
    fn test_record_trace_increases_history() {
        let mut p = InteractionTracePredictor::new();
        assert_eq!(p.history().len(), 0);
        p.record_trace(&make_trace(0.0, 1.0, 3));
        assert_eq!(p.history().len(), 1);
        p.record_trace(&make_trace(0.0, 1.0, 3));
        assert_eq!(p.history().len(), 2);
    }

    #[test]
    fn test_record_trace_caps_at_200() {
        let mut p = InteractionTracePredictor::new();
        for i in 0..250 {
            p.record_trace(&make_trace(i as f32, 0.0, 1));
        }
        assert_eq!(p.history().len(), 200);
    }

    // --- single keypoint ---

    #[test]
    fn test_single_keypoint_roundtrip() {
        let p = InteractionTracePredictor::new();
        let kps = single_kp(42.0, -3.0, 7.5);
        let enc = p.encode_trace(&kps);
        assert_eq!(enc.len(), VSA_DIM);
        // Should not be all zeros
        assert!(enc.iter().any(|&b| b != 0));
    }

    // --- label discrimination ---

    #[test]
    fn test_different_labels_different_encodings() {
        let p = InteractionTracePredictor::new();
        let a: Vec<Keypoint> = (0..3)
            .map(|i| Keypoint {
                x: i as f32,
                y: 0.0,
                z: 0.0,
                label: "A".into(),
                contact: false,
            })
            .collect();
        let b: Vec<Keypoint> = (0..3)
            .map(|i| Keypoint {
                x: i as f32,
                y: 0.0,
                z: 0.0,
                label: "B".into(),
                contact: false,
            })
            .collect();
        let va = p.encode_trace(&a);
        let vb = p.encode_trace(&b);
        assert_ne!(va, vb);
    }
}
