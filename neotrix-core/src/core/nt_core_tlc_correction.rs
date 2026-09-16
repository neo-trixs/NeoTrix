//! # TLCM Layer Correction Module
//!
//! Layer correction mechanism from arXiv 2609.07876.
//! Provides real-time layer-level correction for neural representation alignment.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Layer correction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TLCMConfig {
    pub max_deviation: f64,
    pub correction_lr: f64,
    pub correction_steps: usize,
    pub validate_on_apply: bool,
}

impl Default for TLCMConfig {
    fn default() -> Self {
        Self {
            max_deviation: 0.1,
            correction_lr: 0.01,
            correction_steps: 3,
            validate_on_apply: true,
        }
    }
}

/// Layer correction state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerCorrectionState {
    pub layer_id: usize,
    pub deviation: f64,
    pub alignment_score: f64,
    pub correction_applied: bool,
    pub iteration_count: u64,
}

/// TLCM Layer Correction — arXiv 2609.07876
pub struct TLCMLayerCorrection {
    config: TLCMConfig,
    state: Arc<AtomicU64>,
    correction_history: Arc<std::sync::Mutex<Vec<LayerCorrectionState>>>,
}

impl TLCMLayerCorrection {
    pub fn new(config: TLCMConfig) -> Self {
        Self {
            config,
            state: Arc::new(AtomicU64::new(0)),
            correction_history: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn correct_layer(&self, layer_id: usize, layer_output: &[f64], target: &[f64]) -> (Vec<f64>, f64) {
        let deviation = self.compute_deviation(layer_output, target);
        let correction_magnitude = deviation.min(self.config.max_deviation);
        let corrected: Vec<f64> = layer_output
            .iter()
            .zip(target.iter())
            .map(|(o, t)| {
                if deviation > self.config.max_deviation {
                    o + self.config.correction_lr * (t - o)
                } else {
                    *o
                }
            })
            .collect();
        let state = LayerCorrectionState {
            layer_id,
            deviation,
            alignment_score: 1.0 - (deviation / (deviation + 1.0)),
            correction_applied: deviation > self.config.max_deviation * 0.1,
            iteration_count: self.state.fetch_add(1, Ordering::Relaxed),
        };
        self.correction_history.lock().unwrap().push(state);
        (corrected, correction_magnitude)
    }

    pub fn validate_alignment(&self, layer_output: &[f64], target: &[f64]) -> bool {
        let deviation = self.compute_deviation(layer_output, target);
        let score = 1.0 - (deviation / (deviation + 1.0));
        score >= (1.0 - self.config.max_deviation)
    }

    pub fn apply_correction(&self, layer_id: usize, layer_output: &[f64], target: &[f64]) -> (Vec<f64>, f64, bool) {
        if self.config.validate_on_apply && !self.validate_alignment(layer_output, target) {
            let (corrected, _magnitude) = self.correct_layer(layer_id, layer_output, target);
            let score = self.compute_alignment_score(&corrected, target);
            (corrected, score, true)
        } else {
            let score = self.compute_alignment_score(layer_output, target);
            (layer_output.to_vec(), score, false)
        }
    }

    fn compute_deviation(&self, output: &[f64], target: &[f64]) -> f64 {
        if output.is_empty() || target.is_empty() { return f64::MAX; }
        let min_len = output.len().min(target.len());
        let sum_sq: f64 = output.iter().zip(target.iter().take(min_len)).map(|(o, t)| (o - t).powi(2)).sum();
        (sum_sq / min_len as f64).sqrt()
    }

    fn compute_alignment_score(&self, output: &[f64], target: &[f64]) -> f64 {
        let deviation = self.compute_deviation(output, target);
        1.0 / (1.0 + deviation)
    }

    pub fn get_history(&self) -> Vec<LayerCorrectionState> {
        self.correction_history.lock().unwrap().clone()
    }
}

impl Default for TLCMLayerCorrection {
    fn default() -> Self { Self::new(TLCMConfig::default()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_correct_layer() { let tlc = TLCMLayerCorrection::default(); let (c, m) = tlc.correct_layer(0, &[1.0,2.0,3.0], &[1.1,2.1,3.1]); assert_eq!(c.len(), 3); assert!(m >= 0.0); }
    #[test] fn test_validate_alignment() { let tlc = TLCMLayerCorrection::default(); assert!(tlc.validate_alignment(&[1.0,2.0], &[1.0,2.0])); }
    #[test] fn test_apply_correction() { let tlc = TLCMLayerCorrection::default(); let (c, s, _) = tlc.apply_correction(0, &[1.0,2.0], &[1.1,2.1]); assert_eq!(c.len(), 2); assert!(s >= 0.0 && s <= 1.0); }
}
