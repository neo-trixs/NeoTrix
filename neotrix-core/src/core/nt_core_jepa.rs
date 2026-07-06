/// VL-JEPA style multimodal embedding bridge.
///
/// Inspired by Meta FAIR's VL-JEPA (arXiv 2512.10942, Dec 2025):
/// A vision-language model using JEPA that predicts continuous text embeddings
/// from visual context instead of autoregressive token generation.
///
/// This bridge provides:
/// - Learned projection from vision space to language space
/// - Gradient-based update for the projection matrix
/// - Cross-modal retrieval via cosine similarity
/// - Multimodal fusion (average, weighted, VSA-based)
use std::f32::consts::PI;

/// Input modality type.
#[derive(Debug, Clone, PartialEq)]
pub enum Modality {
    Vision,
    Language,
    Audio,
    Code,
    Custom(String),
}

/// A single modal embedding with its modality and timestamp.
#[derive(Debug, Clone)]
pub struct ModalEmbedding {
    pub data: Vec<f32>,
    pub modality: Modality,
    pub timestamp: f64,
}

impl ModalEmbedding {
    pub fn new(data: Vec<f32>, modality: Modality, timestamp: f64) -> Self {
        Self { data, modality, timestamp }
    }
}

/// Simple xorshift64 PRNG for deterministic initialization.
struct SimpleRng(u64);

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    /// Box-Muller transform for approximate standard normal.
    fn normal(&mut self) -> f32 {
        let u1 = (self.next_u64() as f64) / (u64::MAX as f64);
        let u2 = (self.next_u64() as f64) / (u64::MAX as f64);
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * PI as f64 * u2).cos();
        z as f32
    }
}

/// VL-JEPA style bridge that projects vision embeddings into language space
/// using a learned linear transformation.
#[derive(Debug, Clone)]
pub struct VlJepaBridge {
    pub vision_dim: usize,
    pub language_dim: usize,
    pub projection: Vec<Vec<f32>>,
    pub loss_history: Vec<f32>,
}

impl VlJepaBridge {
    /// Creates a new bridge with orthogonal-initialized projection matrix.
    ///
    /// The projection is a `language_dim × vision_dim` matrix initialized
    /// with random normal values and Gram-Schmidt orthogonalized columns.
    pub fn new(vision_dim: usize, language_dim: usize, seed: u64) -> Self {
        let projection = Self::orthogonal_init(vision_dim, language_dim, seed);
        Self {
            vision_dim,
            language_dim,
            projection,
            loss_history: Vec::new(),
        }
    }

    fn orthogonal_init(cols: usize, rows: usize, seed: u64) -> Vec<Vec<f32>> {
        let mut rng = SimpleRng::new(seed);
        let mut mat = vec![vec![0.0f32; cols]; rows];

        for row in mat.iter_mut() {
            for val in row.iter_mut() {
                *val = rng.normal();
            }
        }

        let n = cols.min(rows);
        for j in 0..n {
            for k in 0..j {
                let dot: f32 = (0..rows).map(|i| mat[i][j] * mat[i][k]).sum();
                let norm_k: f32 = (0..rows).map(|i| mat[i][k] * mat[i][k]).sum();
                if norm_k > 1e-12 {
                    let coeff = dot / norm_k;
                    for i in 0..rows {
                        mat[i][j] -= coeff * mat[i][k];
                    }
                }
            }
            let norm: f32 = (0..rows).map(|i| mat[i][j] * mat[i][j]).sum();
            if norm > 1e-12 {
                let inv = 1.0 / norm.sqrt();
                for i in 0..rows {
                    mat[i][j] *= inv;
                }
            }
        }

        mat
    }

    /// Resize an embedding to `target_dim` by padding or truncating.
    fn resize(embedding: &[f32], target_dim: usize) -> Vec<f32> {
        if embedding.len() == target_dim {
            embedding.to_vec()
        } else {
            let mut res = vec![0.0f32; target_dim];
            let n = embedding.len().min(target_dim);
            res[..n].copy_from_slice(&embedding[..n]);
            res
        }
    }

    /// Projects a vision embedding into language space: `W * vision_embedding`.
    ///
    /// Handles dimension mismatch by padding or truncating the input.
    pub fn predict_language_embedding(&self, vision_embedding: &[f32]) -> Vec<f32> {
        let v = Self::resize(vision_embedding, self.vision_dim);
        let mut result = vec![0.0f32; self.language_dim];
        for i in 0..self.language_dim {
            for j in 0..self.vision_dim {
                result[i] += self.projection[i][j] * v[j];
            }
        }
        result
    }

    /// Cosine distance loss: `1.0 - cosine_similarity(predicted, target)`.
    pub fn compute_loss(&self, predicted: &[f32], target: &[f32]) -> f32 {
        let len = predicted.len().min(target.len());
        let dot: f32 = predicted[..len].iter().zip(target[..len].iter()).map(|(a, b)| a * b).sum();
        let np: f32 = predicted[..len].iter().map(|x| x * x).sum();
        let nt: f32 = target[..len].iter().map(|x| x * x).sum();
        let denom = (np * nt).sqrt();
        if denom < 1e-12 {
            1.0
        } else {
            1.0 - (dot / denom)
        }
    }

    /// Gradient descent update on the projection matrix.
    ///
    /// `W += lr * Σ (target - W * vision) * visionᵀ`
    ///
    /// Zero learning rate or empty pairs results in no change.
    pub fn update_projection(&mut self, embedding_pairs: &[(&[f32], &[f32])], lr: f32) {
        if lr.abs() < 1e-12 || embedding_pairs.is_empty() {
            return;
        }

        let rows = self.language_dim;
        let cols = self.vision_dim;
        let mut grad = vec![vec![0.0f32; cols]; rows];

        for (vision, target) in embedding_pairs {
            let v = Self::resize(vision, cols);
            let t = Self::resize(target, rows);
            let pred = self.predict_language_embedding(&v);

            let error: Vec<f32> = t.iter().zip(pred.iter()).map(|(a, b)| a - b).collect();

            for i in 0..rows {
                for j in 0..cols {
                    grad[i][j] += error[i] * v[j];
                }
            }
        }

        for i in 0..rows {
            for j in 0..cols {
                self.projection[i][j] += lr * grad[i][j];
            }
        }
    }

    /// Finds the `top_k` candidate indices with highest cosine similarity to the query.
    ///
    /// Handles dimension mismatch by padding/truncating candidates to the query length.
    pub fn cross_modal_retrieve(&self, query: &[f32], candidates: &[Vec<f32>], top_k: usize) -> Vec<usize> {
        if candidates.is_empty() || top_k == 0 {
            return Vec::new();
        }

        let q_norm: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
        if q_norm < 1e-12 {
            return (0..top_k.min(candidates.len())).collect();
        }

        let mut scored: Vec<(usize, f32)> = candidates
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let len = query.len().min(c.len());
                let dot: f32 = query[..len].iter().zip(c[..len].iter()).map(|(a, b)| a * b).sum();
                let c_norm: f32 = c.iter().map(|x| x * x).sum::<f32>().sqrt();
                let sim = if c_norm < 1e-12 { 0.0 } else { dot / (q_norm * c_norm) };
                (i, sim)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(top_k).map(|(i, _)| i).collect()
    }

    /// Returns a reference to the loss history.
    pub fn loss_history(&self) -> &[f32] {
        &self.loss_history
    }
}

/// Combines multiple modal embeddings into a single fused representation.
#[derive(Debug, Clone)]
pub struct MultimodalFusion {
    pub embeddings: Vec<ModalEmbedding>,
    pub fused: Vec<f32>,
}

impl MultimodalFusion {
    /// Creates a new fusion, initializing `fused` as the element-wise average.
    pub fn new(embeddings: Vec<ModalEmbedding>) -> Self {
        let mut fusion = Self {
            embeddings,
            fused: Vec::new(),
        };
        fusion.fuse_average();
        fusion
    }

    /// Returns the fused representation as a dense vector.
    pub fn to_dense_vector(&self) -> Vec<f32> {
        self.fused.clone()
    }

    /// Simple average across all modalities.
    ///
    /// Uses the largest embedding dimension; shorter embeddings are zero-padded
    /// before averaging.
    pub fn fuse_average(&mut self) {
        if self.embeddings.is_empty() {
            self.fused = Vec::new();
            return;
        }

        let dim = self.embeddings.iter().map(|e| e.data.len()).max().unwrap_or(0);
        if dim == 0 {
            self.fused = Vec::new();
            return;
        }

        let mut fused = vec![0.0f32; dim];
        for emb in &self.embeddings {
            let n = emb.data.len().min(dim);
            for i in 0..n {
                fused[i] += emb.data[i];
            }
        }

        let n = self.embeddings.len() as f32;
        for f in fused.iter_mut() {
            *f /= n;
        }

        self.fused = fused;
    }

    /// VSA bundle fusion using the hypercube VSA engine.
    ///
    /// Converts all embeddings to f64, bundles them via `VSAEngine::bundle()`,
    /// then converts back to f32. Requires the hypercube VSA module.
    pub fn fuse_vsa(&mut self) {
        use crate::core::nt_core_hcube::{VSAEngine, VsaBackend};

        if self.embeddings.is_empty() {
            self.fused = Vec::new();
            return;
        }

        let max_dim = self.embeddings.iter().map(|e| e.data.len()).max().unwrap_or(0);
        if max_dim == 0 {
            self.fused = Vec::new();
            return;
        }

        let f64_vecs: Vec<Vec<f64>> = self
            .embeddings
            .iter()
            .map(|e| {
                let mut v = vec![0.0f64; max_dim];
                for (i, val) in e.data.iter().enumerate() {
                    v[i] = *val as f64;
                }
                v
            })
            .collect();

        let engine = VSAEngine::new(max_dim);
        let refs: Vec<&[f64]> = f64_vecs.iter().map(|v| v.as_slice()).collect();
        let bundled = engine.bundle(&refs);

        self.fused = bundled.iter().map(|x| *x as f32).collect();
    }

    /// Weighted sum fusion: `fused = Σ w_i * emb_i`.
    ///
    /// If `weights` is shorter than `embeddings`, the remaining embeddings
    /// receive zero weight.
    pub fn fuse_weighted(&mut self, weights: &[f32]) {
        if self.embeddings.is_empty() || weights.is_empty() {
            self.fused = Vec::new();
            return;
        }

        let max_dim = self.embeddings.iter().map(|e| e.data.len()).max().unwrap_or(0);
        if max_dim == 0 {
            self.fused = Vec::new();
            return;
        }

        let mut fused = vec![0.0f32; max_dim];
        let n = self.embeddings.len().min(weights.len());

        for idx in 0..n {
            let w = weights[idx];
            let emb = &self.embeddings[idx];
            let len = emb.data.len().min(max_dim);
            for i in 0..len {
                fused[i] += w * emb.data[i];
            }
        }

        self.fused = fused;
    }
}

// ── Next-State Prediction (Orca-inspired) ──────────────────────────────

/// Prediction mode: unconscious (dense trajectory) vs conscious (sparse events).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PredictionMode {
    Unconscious,
    Conscious,
    Hybrid,
}

/// Orca-style Next-State Predictor.
///
/// Unifies next-state prediction across modalities. Maintains a frozen
/// backbone latent space (via HyperCube VSA) and supports modality-specific
/// readout decoders.
///
/// Inspired by Orca (arXiv 2606.30534): "Next-State-Prediction modeling
/// offers a unified state-transition modeling route toward understanding,
/// predicting, and acting upon the world."
#[derive(Debug, Clone)]
pub struct NextStatePredictor {
    pub state_dim: usize,
    pub action_dim: usize,
    pub unconscious_horizon: usize,
    pub conscious_events: usize,
    pub backbone: Vec<f64>,
    pub prediction_log: Vec<PredictionRecord>,
}

#[derive(Debug, Clone)]
pub struct PredictionRecord {
    pub input_latent: Vec<f64>,
    pub action: Vec<f64>,
    pub predicted_latent: Vec<f64>,
    pub actual_latent: Vec<f64>,
    pub mode: PredictionMode,
    pub energy: f64,
}

impl NextStatePredictor {
    pub fn new(state_dim: usize, action_dim: usize) -> Self {
        Self {
            state_dim,
            action_dim,
            unconscious_horizon: 8,
            conscious_events: 3,
            backbone: Vec::new(),
            prediction_log: Vec::new(),
        }
    }

    /// Unconscious mode: dense Markovian prediction.
    /// Uses a linear transition model: s_{t+1} = W * s_t + B * a_t
    /// The transition kernel is learned from observed state-action trajectories.
    pub fn predict_unconscious(
        &self,
        current_state: &[f64],
        action: &[f64],
        transition_kernel: &[Vec<f64>],
        action_kernel: &[Vec<f64>],
    ) -> Vec<f64> {
        let dim = current_state.len().min(self.state_dim);
        let mut next = vec![0.0f64; self.state_dim];
        for i in 0..self.state_dim {
            for j in 0..dim {
                next[i] += transition_kernel[i][j] * current_state[j];
            }
            for j in 0..action.len().min(self.action_dim) {
                next[i] += action_kernel[i][j] * action[j];
            }
        }
        next
    }

    /// Conscious mode: event-conditioned prediction.
    /// Predicts next state conditioned on sparse event features.
    /// Events are encoded as a weighted sum of event embeddings.
    pub fn predict_conscious(
        &self,
        current_state: &[f64],
        event_embeddings: &[Vec<f64>],
        event_weights: &[f64],
        event_kernel: &[Vec<f64>],
    ) -> Vec<f64> {
        let dim = current_state.len().min(self.state_dim);
        let mut next = current_state.to_vec();
        if next.len() < self.state_dim {
            next.resize(self.state_dim, 0.0);
        }
        let n = event_embeddings.len().min(event_weights.len());
        for k in 0..n {
            let w = event_weights[k];
            for i in 0..self.state_dim.min(event_embeddings[k].len()) {
                for j in 0..dim.min(event_embeddings[k].len()) {
                    next[i] += w * event_kernel[i][j] * event_embeddings[k][j];
                }
            }
        }
        next
    }

    /// Hybrid mode: blend unconscious + conscious predictions.
    /// alpha ∈ [0,1] controls blend (0 = pure unconscious, 1 = pure conscious).
    pub fn predict_hybrid(
        &self,
        unconscious_pred: &[f64],
        conscious_pred: &[f64],
        alpha: f64,
    ) -> Vec<f64> {
        let alpha = alpha.clamp(0.0, 1.0);
        unconscious_pred
            .iter()
            .zip(conscious_pred.iter())
            .map(|(u, c)| u * (1.0 - alpha) + c * alpha)
            .collect()
    }

    /// Record a prediction for later analysis.
    pub fn record_prediction(
        &mut self,
        input_latent: Vec<f64>,
        action: Vec<f64>,
        predicted_latent: Vec<f64>,
        actual_latent: Vec<f64>,
        mode: PredictionMode,
    ) {
        let energy = self.compute_energy(&predicted_latent, &actual_latent);
        self.prediction_log.push(PredictionRecord {
            input_latent,
            action,
            predicted_latent,
            actual_latent,
            mode,
            energy,
        });
        if self.prediction_log.len() > 1000 {
            self.prediction_log.remove(0);
        }
    }

    /// Prediction energy = MSE between predicted and actual latent.
    pub fn compute_energy(&self, predicted: &[f64], actual: &[f64]) -> f64 {
        let len = predicted.len().min(actual.len());
        if len == 0 {
            return 1.0;
        }
        predicted[..len]
            .iter()
            .zip(actual[..len].iter())
            .map(|(p, a)| (p - a).powi(2))
            .sum::<f64>() / len as f64
    }

    /// Freeze the current backbone as the reference state.
    pub fn freeze_backbone(&mut self, state: &[f64]) {
        self.backbone = state.to_vec();
    }

    /// Historical prediction accuracy (lower is better).
    pub fn average_energy(&self) -> f64 {
        let n = self.prediction_log.len();
        if n == 0 {
            return 0.0;
        }
        self.prediction_log.iter().map(|r| r.energy).sum::<f64>() / n as f64
    }

    /// Energy per prediction mode for diagnostic comparison.
    pub fn mode_energy(&self, mode: PredictionMode) -> f64 {
        let records: Vec<&PredictionRecord> =
            self.prediction_log.iter().filter(|r| r.mode == mode).collect();
        let n = records.len();
        if n == 0 {
            return 0.0;
        }
        records.iter().map(|r| r.energy).sum::<f64>() / n as f64
    }
}

// ── Orca-style lightweight decoder readout ─────────────────────────────

/// Lightweight modality-specific decoder readout.
///
/// Orca principle: backbone is frozen, only lightweight decoders are trained.
/// Each decoder maps from the shared latent space to a specific output modality.
#[derive(Debug, Clone)]
pub struct DecoderReadout {
    pub output_dim: usize,
    pub latent_dim: usize,
    pub weights: Vec<Vec<f64>>,
    pub bias: Vec<f64>,
}

impl DecoderReadout {
    pub fn new(output_dim: usize, latent_dim: usize, seed: u64) -> Self {
        let mut rng = SimpleRng(seed);
        let weights = (0..output_dim)
            .map(|_| (0..latent_dim).map(|_| rng.normal() as f64 * 0.1).collect())
            .collect();
        let bias = vec![0.0f64; output_dim];
        Self { output_dim, latent_dim, weights, bias }
    }

    /// Decode from latent space to output modality.
    pub fn decode(&self, latent: &[f64]) -> Vec<f64> {
        let n = latent.len().min(self.latent_dim);
        let mut output = self.bias.clone();
        for i in 0..self.output_dim {
            for j in 0..n {
                output[i] += self.weights[i][j] * latent[j];
            }
        }
        output
    }

    /// Gradient update: MSE between decoded and target.
    pub fn update(&mut self, latent: &[f64], target: &[f64], lr: f64) -> f64 {
        let predicted = self.decode(latent);
        let len = predicted.len().min(target.len());
        let mut loss = 0.0f64;
        let mut grad = vec![vec![0.0f64; self.latent_dim]; self.output_dim];
        let mut grad_bias = vec![0.0f64; self.output_dim];

        let n = latent.len().min(self.latent_dim);
        for i in 0..len {
            let error = predicted[i] - target[i];
            loss += error * error;
            grad_bias[i] = error;
            for j in 0..n {
                grad[i][j] = error * latent[j];
            }
        }
        loss /= len.max(1) as f64;

        for i in 0..self.output_dim {
            self.bias[i] -= lr * grad_bias[i];
            for j in 0..n {
                self.weights[i][j] -= lr * grad[i][j];
            }
        }
        loss
    }
}

#[cfg(test)]
fn is_approx_orthogonal(mat: &[Vec<f32>], tol: f32) -> bool {
    let rows = mat.len();
    let cols = if rows == 0 { 0 } else { mat[0].len() };
    let n = cols.min(rows);

    for j in 0..n {
        let norm_j = (0..rows).map(|i| mat[i][j] * mat[i][j]).sum::<f32>().sqrt();
        if (norm_j - 1.0).abs() > tol {
            return false;
        }
        for k in (j + 1)..n {
            let dot: f32 = (0..rows).map(|i| mat[i][j] * mat[i][k]).sum();
            if dot.abs() > tol {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bridge() -> VlJepaBridge {
        VlJepaBridge::new(256, 256, 42)
    }

    fn random_vec(rng: &mut SimpleRng, n: usize) -> Vec<f32> {
        (0..n).map(|_| rng.normal()).collect()
    }

    // ——— 1. Language prediction from vision embedding ———
    #[test]
    fn test_predict_language_embedding() {
        let bridge = make_bridge();
        let vision = vec![0.5f32; 256];
        let predicted = bridge.predict_language_embedding(&vision);
        assert_eq!(predicted.len(), 256);
        // prediction should be non-zero for non-zero input
        let norm: f32 = predicted.iter().map(|x| x * x).sum();
        assert!(norm > 0.0);
    }

    // ——— 2. Loss decreases after update ———
    #[test]
    fn test_loss_decreases_after_update() {
        let mut bridge = make_bridge();
        let mut rng = SimpleRng::new(123);

        let pairs: Vec<(Vec<f32>, Vec<f32>)> = (0..50)
            .map(|_| (random_vec(&mut rng, 256), random_vec(&mut rng, 256)))
            .collect();

        let refs: Vec<(&[f32], &[f32])> = pairs.iter().map(|(v, t)| (v.as_slice(), t.as_slice())).collect();

        let first_pair = &refs[0];
        let loss_before = bridge.compute_loss(
            &bridge.predict_language_embedding(first_pair.0),
            first_pair.1,
        );

        bridge.update_projection(&refs, 0.01);

        let loss_after = bridge.compute_loss(
            &bridge.predict_language_embedding(first_pair.0),
            first_pair.1,
        );

        assert!(
            loss_after <= loss_before + 1e-6,
            "loss should not increase after gradient update: before={} after={}",
            loss_before,
            loss_after
        );
    }

    // ——— 3. Cross-modal retrieval returns correct item ———
    #[test]
    fn test_cross_modal_retrieve() {
        let bridge = make_bridge();
        let mut rng = SimpleRng::new(77);

        let query = vec![1.0f32; 256];
        let mut candidates: Vec<Vec<f32>> = (0..10).map(|_| random_vec(&mut rng, 256)).collect();
        // insert the exact match
        candidates.push(query.clone());

        let result = bridge.cross_modal_retrieve(&query, &candidates, 1);
        assert!(!result.is_empty());
        assert_eq!(result[0], 10);
    }

    // ——— 4. Fusion average with 2+ modalities ———
    #[test]
    fn test_fusion_average() {
        let emb1 = ModalEmbedding::new(vec![1.0, 2.0, 3.0], Modality::Vision, 0.0);
        let emb2 = ModalEmbedding::new(vec![3.0, 4.0, 5.0], Modality::Language, 0.0);

        let fusion = MultimodalFusion::new(vec![emb1, emb2]);
        assert_eq!(fusion.fused.len(), 3);
        assert!((fusion.fused[0] - 2.0).abs() < 1e-6);
        assert!((fusion.fused[1] - 3.0).abs() < 1e-6);
        assert!((fusion.fused[2] - 4.0).abs() < 1e-6);
    }

    // ——— 5. VSA fusion ———
    #[test]
    fn test_fusion_vsa() {
        let emb1 = ModalEmbedding::new(vec![1.0, 0.0, 0.0], Modality::Vision, 0.0);
        let emb2 = ModalEmbedding::new(vec![0.0, 1.0, 0.0], Modality::Language, 0.0);

        let mut fusion = MultimodalFusion::new(vec![emb1, emb2]);
        fusion.fuse_vsa();
        assert_eq!(fusion.fused.len(), 3);
        // VSA bundling is element-wise sum (no normalization)
        assert!((fusion.fused[0] - 1.0).abs() < 1e-6);
        assert!((fusion.fused[1] - 1.0).abs() < 1e-6);
        assert!((fusion.fused[2] - 0.0).abs() < 1e-6);
    }

    // ——— 6. Dimension mismatch handling ———
    #[test]
    fn test_dimension_mismatch() {
        let bridge = VlJepaBridge::new(128, 64, 99);
        // vision embedding with wrong dimension (should be 128, passing 64)
        let short = vec![0.5f32; 64];
        let predicted = bridge.predict_language_embedding(&short);
        assert_eq!(predicted.len(), 64);
        let norm: f32 = predicted.iter().map(|x| x * x).sum();
        assert!(norm > 0.0);

        // oversized
        let long = vec![0.5f32; 256];
        let predicted2 = bridge.predict_language_embedding(&long);
        assert_eq!(predicted2.len(), 64);
    }

    // ——— 7. Empty embedding list ———
    #[test]
    fn test_empty_embedding_list() {
        let fusion = MultimodalFusion::new(vec![]);
        assert!(fusion.fused.is_empty());
        assert!(fusion.to_dense_vector().is_empty());
    }

    // ——— 8. Projection matrix orthogonal initialization ———
    #[test]
    fn test_orthogonal_init() {
        let bridge = VlJepaBridge::new(256, 256, 42);
        // Check first min(dim) columns are approximately orthonormal
        let ok = is_approx_orthogonal(&bridge.projection, 1e-4);
        assert!(ok, "projection matrix columns should be near-orthonormal");
    }

    #[test]
    fn test_orthogonal_init_non_square() {
        let bridge = VlJepaBridge::new(128, 64, 7);
        let ok = is_approx_orthogonal(&bridge.projection, 1e-4);
        assert!(ok, "non-square projection columns should be orthonormal");
    }

    // ——— 9. Fusion weighted ———
    #[test]
    fn test_fusion_weighted() {
        let emb1 = ModalEmbedding::new(vec![1.0, 0.0], Modality::Vision, 0.0);
        let emb2 = ModalEmbedding::new(vec![0.0, 2.0], Modality::Language, 0.0);

        let mut fusion = MultimodalFusion::new(vec![emb1, emb2]);
        fusion.fuse_weighted(&[2.0, 3.0]);
        assert!((fusion.fused[0] - 2.0).abs() < 1e-6);
        assert!((fusion.fused[1] - 6.0).abs() < 1e-6);
    }

    // ——— 10. Single modality fusion (identity) ———
    #[test]
    fn test_single_modality_fusion() {
        let emb = ModalEmbedding::new(vec![4.0, 5.0, 6.0], Modality::Code, 0.0);
        let fusion = MultimodalFusion::new(vec![emb]);
        assert!((fusion.fused[0] - 4.0).abs() < 1e-6);
        assert!((fusion.fused[1] - 5.0).abs() < 1e-6);
        assert!((fusion.fused[2] - 6.0).abs() < 1e-6);
    }

    // ——— 11. Zero learning rate → no update ———
    #[test]
    fn test_zero_learning_rate() {
        let mut bridge = make_bridge();
        let original = bridge.projection.clone();

        let pairs = [(&[1.0f32; 256][..], &[2.0f32; 256][..])];
        bridge.update_projection(&pairs, 0.0);

        for (orig_row, new_row) in original.iter().zip(bridge.projection.iter()) {
            for (o, n) in orig_row.iter().zip(new_row.iter()) {
                assert!((o - n).abs() < 1e-6);
            }
        }
    }

    // ——— 12. Compute loss with zero vectors ———
    #[test]
    fn test_loss_zero_vectors() {
        let bridge = make_bridge();
        let loss = bridge.compute_loss(&[0.0f32; 256], &[0.0f32; 256]);
        assert!((loss - 1.0).abs() < 1e-6);
    }

    // ——— 13. Cross-modal retrieval empty candidates ———
    #[test]
    fn test_cross_modal_retrieve_empty() {
        let bridge = make_bridge();
        let query = vec![1.0f32; 256];
        let result = bridge.cross_modal_retrieve(&query, &[], 5);
        assert!(result.is_empty());
    }

    // ——— 14. ModalEmbedding constructor ———
    #[test]
    fn test_modal_embedding_new() {
        let emb = ModalEmbedding::new(vec![0.1, 0.2], Modality::Audio, 100.0);
        assert_eq!(emb.modality, Modality::Audio);
        assert!((emb.timestamp - 100.0).abs() < 1e-6);
    }

    // ——— NextStatePredictor tests ———
    #[test]
    fn test_next_state_predictor_new() {
        let predictor = NextStatePredictor::new(4, 2);
        assert_eq!(predictor.state_dim, 4);
        assert_eq!(predictor.action_dim, 2);
        assert_eq!(predictor.unconscious_horizon, 8);
        assert_eq!(predictor.conscious_events, 3);
        assert!(predictor.backbone.is_empty());
        assert!(predictor.prediction_log.is_empty());
    }

    #[test]
    fn test_predict_unconscious_basic() {
        let predictor = NextStatePredictor::new(4, 2);
        let state = vec![1.0, 2.0, 3.0, 4.0];
        let action = vec![0.5, 1.0];
        let kernel: Vec<Vec<f64>> = (0..4)
            .map(|i| (0..4).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
            .collect();
        let action_kernel: Vec<Vec<f64>> = (0..4).map(|_| vec![0.0; 2]).collect();
        let predicted = predictor.predict_unconscious(&state, &action, &kernel, &action_kernel);
        assert_eq!(predicted.len(), 4);
        let norm: f64 = predicted.iter().map(|x| x * x).sum();
        assert!(norm > 0.0);
    }

    #[test]
    fn test_predict_conscious_basic() {
        let predictor = NextStatePredictor::new(4, 2);
        let state = vec![1.0; 4];
        let events = vec![vec![1.0, 0.0, 0.0, 0.0], vec![0.0, 1.0, 0.0, 0.0]];
        let weights = vec![2.0, 3.0];
        let kernel: Vec<Vec<f64>> = (0..4)
            .map(|i| (0..4).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
            .collect();
        let predicted = predictor.predict_conscious(&state, &events, &weights, &kernel);
        assert_eq!(predicted.len(), 4);
        assert!((predicted[0] - 3.0).abs() < 1e-6);
        assert!((predicted[1] - 4.0).abs() < 1e-6);
        assert!((predicted[2] - 1.0).abs() < 1e-6);
        assert!((predicted[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_predict_hybrid_blend() {
        let predictor = NextStatePredictor::new(4, 2);
        let u = vec![1.0, 2.0, 3.0, 4.0];
        let c = vec![5.0, 6.0, 7.0, 8.0];
        let blended = predictor.predict_hybrid(&u, &c, 0.3);
        assert_eq!(blended.len(), 4);
        assert!((blended[0] - 2.2).abs() < 1e-6);
        assert!((blended[1] - 3.2).abs() < 1e-6);
        assert!((blended[2] - 4.2).abs() < 1e-6);
        assert!((blended[3] - 5.2).abs() < 1e-6);
    }

    #[test]
    fn test_predict_hybrid_alpha_0_pure_unconscious() {
        let predictor = NextStatePredictor::new(4, 2);
        let u = vec![1.0, 2.0, 3.0, 4.0];
        let c = vec![5.0, 6.0, 7.0, 8.0];
        let result = predictor.predict_hybrid(&u, &c, 0.0);
        for (r, ue) in result.iter().zip(u.iter()) {
            assert!((r - ue).abs() < 1e-6);
        }
    }

    #[test]
    fn test_predict_hybrid_alpha_1_pure_conscious() {
        let predictor = NextStatePredictor::new(4, 2);
        let u = vec![1.0, 2.0, 3.0, 4.0];
        let c = vec![5.0, 6.0, 7.0, 8.0];
        let result = predictor.predict_hybrid(&u, &c, 1.0);
        for (r, ce) in result.iter().zip(c.iter()) {
            assert!((r - ce).abs() < 1e-6);
        }
    }

    #[test]
    fn test_compute_energy_identical() {
        let predictor = NextStatePredictor::new(4, 2);
        let v = vec![1.0, 2.0, 3.0, 4.0];
        let energy = predictor.compute_energy(&v, &v);
        assert!((energy - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_energy_different() {
        let predictor = NextStatePredictor::new(4, 2);
        let a = vec![1.0, 0.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0, 0.0];
        let energy = predictor.compute_energy(&a, &b);
        assert!(energy > 0.0);
    }

    #[test]
    fn test_record_prediction_energy_tracking() {
        let mut predictor = NextStatePredictor::new(4, 2);
        let input = vec![1.0; 4];
        let action = vec![0.0; 2];
        let predicted = vec![2.0; 4];
        let actual = vec![2.0; 4];
        predictor.record_prediction(
            input, action, predicted, actual, PredictionMode::Unconscious,
        );
        assert_eq!(predictor.prediction_log.len(), 1);
        assert!((predictor.average_energy() - 0.0).abs() < 1e-6);
        predictor.record_prediction(
            vec![1.0; 4],
            vec![0.0; 2],
            vec![10.0; 4],
            vec![0.0; 4],
            PredictionMode::Conscious,
        );
        assert_eq!(predictor.prediction_log.len(), 2);
        assert!(predictor.average_energy() > 0.0);
    }

    #[test]
    fn test_decoder_readout_decode() {
        let decoder = DecoderReadout::new(3, 4, 42);
        let latent = vec![1.0, 0.5, -0.5, 0.0];
        let output = decoder.decode(&latent);
        assert_eq!(output.len(), 3);
        let output2 = decoder.decode(&latent);
        for (o1, o2) in output.iter().zip(output2.iter()) {
            assert!((o1 - o2).abs() < 1e-6);
        }
    }

    #[test]
    fn test_decoder_readout_update_reduces_loss() {
        let mut decoder = DecoderReadout::new(4, 4, 42);
        let latent = vec![1.0, 0.5, -0.5, 0.0];
        let target = vec![0.8, 0.6, -0.4, 0.1];
        let pred_before = decoder.decode(&latent);
        let loss_before: f64 = pred_before
            .iter()
            .zip(target.iter())
            .map(|(p, t)| (p - t).powi(2))
            .sum::<f64>()
            / pred_before.len() as f64;
        let returned = decoder.update(&latent, &target, 0.1);
        assert!((returned - loss_before).abs() < 1e-6);
        let pred_after = decoder.decode(&latent);
        let loss_after: f64 = pred_after
            .iter()
            .zip(target.iter())
            .map(|(p, t)| (p - t).powi(2))
            .sum::<f64>()
            / pred_after.len() as f64;
        assert!(
            loss_after < loss_before,
            "loss should decrease after update: before={} after={}",
            loss_before,
            loss_after
        );
    }

    #[test]
    fn test_mode_energy_tracking() {
        let mut predictor = NextStatePredictor::new(4, 2);
        assert!((predictor.mode_energy(PredictionMode::Unconscious) - 0.0).abs() < 1e-6);
        for _ in 0..3 {
            predictor.record_prediction(
                vec![0.0; 4],
                vec![0.0; 2],
                vec![1.0; 4],
                vec![1.0; 4],
                PredictionMode::Unconscious,
            );
        }
        assert!((predictor.mode_energy(PredictionMode::Unconscious) - 0.0).abs() < 1e-6);
        for _ in 0..2 {
            predictor.record_prediction(
                vec![0.0; 4],
                vec![0.0; 2],
                vec![5.0; 4],
                vec![0.0; 4],
                PredictionMode::Conscious,
            );
        }
        let conscious_e = predictor.mode_energy(PredictionMode::Conscious);
        assert!(conscious_e > 0.0);
        assert!((predictor.mode_energy(PredictionMode::Unconscious) - 0.0).abs() < 1e-6);
    }
}
