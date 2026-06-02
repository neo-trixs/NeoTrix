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
}
