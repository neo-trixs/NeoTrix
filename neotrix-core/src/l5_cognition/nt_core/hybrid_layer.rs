//! Hybrid Layer — Transformer + SSM (Mamba-style) hybrid compute
//!
//! Absorbs the hybrid architecture pattern from Jamba / Zamba / MiCRo:
//! alternating Transformer attention layers with SSM (State-Space Model) layers
//! for O(n) inference cost on long sequences while retaining attention quality
//! on short sequences.
//!
//! Key insight: not every layer needs full quadratic attention. SSM layers
//! handle long-range dependencies cheaply, while attention layers handle
//! precise local retrieval. The hybrid ratio is tunable per task.

use serde::{Deserialize, Serialize};

/// Compute mode for a hybrid layer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum _ComputeMode {
    /// Full self-attention (quadratic cost, precise retrieval)
    Attention,
    /// State-space model (linear cost, long-range pattern)
    Ssm,
}

/// Hybrid layer — weighted combination of Transformer + SSM pathways
///
/// When `transformer_weight` is high, the layer behaves like standard attention.
/// When `ssm_weight` is high, it behaves like Mamba/Jamba SSM layers.
/// The ratio can shift dynamically based on sequence length or task type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _HybridLayer {
    pub transformer_weight: f64,
    pub ssm_weight: f64,
    /// Sequence length threshold: below this, prefer attention; above, prefer SSM
    pub ss_threshold: usize,
}

impl _HybridLayer {
    pub fn new(transformer_weight: f64, ssm_weight: f64) -> Self {
        let total = transformer_weight + ssm_weight;
        Self {
            transformer_weight: transformer_weight / total,
            ssm_weight: ssm_weight / total,
            ss_threshold: 2048,
        }
    }

    pub fn with_threshold(mut self, threshold: usize) -> Self {
        self.ss_threshold = threshold;
        self
    }

    /// Select the optimal compute mode for a given sequence length
    pub fn select_mode(&self, seq_len: usize) -> _ComputeMode {
        if seq_len <= self.ss_threshold {
            _ComputeMode::Attention
        } else {
            _ComputeMode::Ssm
        }
    }

    /// Adaptive weight blend: short sequences favor attention, long favor SSM
    pub(crate) fn _adaptive_weights(&self, seq_len: usize) -> (f64, f64) {
        let ratio = (seq_len as f64 / self.ss_threshold as f64).clamp(0.0, 2.0);
        // At threshold: 50/50; below: attention-heavy; above: ssm-heavy
        let attn_w = self.transformer_weight * (1.0 - ratio * 0.5);
        let ssm_w = self.ssm_weight * (0.5 + ratio * 0.5);
        let total = attn_w + ssm_w;
        (attn_w / total, ssm_w / total)
    }

    /// Compute hybrid output: weighted sum of attention and SSM pathways
    ///
    /// In production, this dispatches to real attention/SSM kernels.
    /// Here we model the interface and gating logic.
    pub fn compute(&self, input_len: usize) -> _HybridOutput {
        let (attn_w, ssm_w) = self._adaptive_weights(input_len);
        let mode = self.select_mode(input_len);

        _HybridOutput {
            mode,
            transformer_weight: attn_w,
            ssm_weight: ssm_w,
            estimated_cost_linear: ssm_w * input_len as f64,
            estimated_cost_quadratic: attn_w * (input_len as f64).powi(2),
        }
    }
}

impl Default for _HybridLayer {
    fn default() -> Self {
        Self::new(0.5, 0.5)
    }
}

/// Result of hybrid layer computation
#[derive(Debug, Clone)]
pub(crate) struct _HybridOutput {
    pub mode: _ComputeMode,
    pub transformer_weight: f64,
    pub ssm_weight: f64,
    /// Estimated cost of the SSM path (linear in seq_len)
    pub estimated_cost_linear: f64,
    /// Estimated cost of the attention path (quadratic in seq_len)
    pub estimated_cost_quadratic: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_seq_uses_attention() {
        let layer = _HybridLayer::new(0.5, 0.5);
        assert_eq!(layer.select_mode(512), _ComputeMode::Attention);
    }

    #[test]
    fn test_long_seq_uses_ssm() {
        let layer = _HybridLayer::new(0.5, 0.5);
        assert_eq!(layer.select_mode(4096), _ComputeMode::Ssm);
    }

    #[test]
    fn test_adaptive_weights_short_favors_attention() {
        let layer = _HybridLayer::new(0.5, 0.5).with_threshold(2048);
        let (attn, ssm) = layer._adaptive_weights(512);
        assert!(attn > ssm, "short seq should favor attention");
    }

    #[test]
    fn test_adaptive_weights_long_favors_ssm() {
        let layer = _HybridLayer::new(0.5, 0.5).with_threshold(2048);
        let (attn, ssm) = layer._adaptive_weights(8192);
        assert!(ssm > attn, "long seq should favor SSM");
    }

    #[test]
    fn test_compute_at_threshold_balanced() {
        let layer = _HybridLayer::new(0.5, 0.5).with_threshold(2048);
        let out = layer.compute(2048);
        assert_eq!(out.mode, _ComputeMode::Attention);
        // At threshold, weights should be roughly balanced
        let diff = (out.transformer_weight - out.ssm_weight).abs();
        assert!(diff < 0.15, "should be balanced at threshold: diff={diff}");
    }

    #[test]
    fn test_compute_long_seq_ssm_dominates() {
        let layer = _HybridLayer::new(0.5, 0.5).with_threshold(2048);
        let out = layer.compute(16384);
        assert!(out.ssm_weight > out.transformer_weight);
        assert!(out.estimated_cost_linear < out.estimated_cost_quadratic);
    }

    #[test]
    fn test_weights_normalized() {
        let layer = _HybridLayer::new(0.3, 0.7);
        let (attn, ssm) = layer._adaptive_weights(1024);
        let sum = attn + ssm;
        assert!((sum - 1.0).abs() < 1e-6, "weights should sum to 1.0");
    }

    #[test]
    fn test_default_hybrid_layer() {
        let layer = _HybridLayer::default();
        assert!((layer.transformer_weight - 0.5).abs() < 1e-6);
        assert!((layer.ssm_weight - 0.5).abs() < 1e-6);
    }
}
