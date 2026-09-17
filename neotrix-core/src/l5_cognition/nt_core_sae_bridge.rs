//! Bridge: SAE ↔ E8 ↔ Observer
//!
//! During E8 inference:
//! 1. Extract E8 hexagram state → encode through SAE → interpretable features
//! 2. Feed feature activations to Observer (PRM) for interpretability scoring
//! 3. Enable feature steering for behavioral intervention
//! 4. Provide causal attribution labels for each active feature

use std::sync::{Arc, RwLock};

use crate::core::nt_core_error::{NeoTrixError, NeoTrixResult};
use crate::core::{SaeFeature, SparseAutoencoder, SAE_INPUT_DIM};

/// Bridge connecting SAE feature extraction to the E8 reasoning path.
///
/// Wraps the SparseAutoencoder behind an Arc<RwLock> so it can be shared
/// across the ReasoningEngine, Observer, and SEAL pipeline.
pub struct SAEBridge {
    sae: Arc<RwLock<SparseAutoencoder>>,
    /// Cached features from the most recent `extract_features` call.
    last_active_features: Vec<SaeFeature>,
}

impl SAEBridge {
    pub fn new(sae: Arc<RwLock<SparseAutoencoder>>) -> Self {
        Self {
            sae,
            last_active_features: Vec::new(),
        }
    }

    /// Extract SAE features from the current E8 reasoning state.
    ///
    /// Converts the E8 hexagram state + meta bits into a one-hot SAE input,
    /// runs the forward pass through the sparse autoencoder, and returns
    /// the active SaeFeatures with their activation values.
    pub fn extract_features(
        &mut self,
        e8_state: u8,
        meta_bits: u8,
        _task_embedding: &[f64],
    ) -> Vec<SaeFeature> {
        let input = SparseAutoencoder::e8_to_input(e8_state, meta_bits);
        let mut sae = self.sae.write().unwrap_or_else(|e| e.into_inner());
        let output = sae.forward(&input);
        let features = output.active_features.clone();
        self.last_active_features = features.clone();
        features
    }

    /// Return human-readable labels for the top-K active features.
    ///
    /// Uses discovered `MonosemanticFeature` labels if available, otherwise
    /// falls back to `feature_{index}`.
    pub fn get_active_feature_names(&self, top_k: usize) -> Vec<String> {
        if self.last_active_features.is_empty() {
            return Vec::new();
        }
        let mut sorted = self.last_active_features.clone();
        sorted.sort_by(|a, b| {
            b.activation
                .partial_cmp(&a.activation)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.truncate(top_k);
        sorted
            .iter()
            .map(|f| {
                f.label
                    .clone()
                    .unwrap_or_else(|| format!("feature_{}", f.index))
            })
            .collect()
    }

    /// Steer the SAE toward a target feature by clamping the latent activation.
    ///
    /// Performs a forward pass on the given E8 state, then overwrites the
    /// specified feature's latent activation with `strength`. Returns the
    /// steered reconstruction (what the E8 state would look like after
    /// intervention).
    ///
    /// This enables causal intervention — "what happens if feature X is on?"
    pub fn steer_toward_feature(
        &self,
        e8_state: u8,
        meta_bits: u8,
        feature_idx: usize,
        strength: f64,
    ) -> NeoTrixResult<Vec<f64>> {
        let input = SparseAutoencoder::e8_to_input(e8_state, meta_bits);
        let sae = self.sae.read().unwrap_or_else(|e| e.into_inner());

        let mut latent = sae.encoder.encode(&input);
        sae.encoder.enforce_sparsity(&mut latent);
        sae.steer(&mut latent, feature_idx, strength);
        let reconstruction = sae.decoder.decode(&latent);

        if reconstruction.len() != SAE_INPUT_DIM {
            return Err(NeoTrixError::Brain(format!(
                "SAE steer: reconstruction length {} != input dim {}",
                reconstruction.len(),
                SAE_INPUT_DIM,
            )));
        }
        Ok(reconstruction)
    }

    /// Access the underlying SAE for training or inspection.
    pub fn sae(&self) -> &Arc<RwLock<SparseAutoencoder>> {
        &self.sae
    }

    /// Number of active features in the last forward pass.
    pub fn active_feature_count(&self) -> usize {
        self.last_active_features.len()
    }
}
