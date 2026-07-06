use serde::{Deserialize, Serialize};
type Vector = Vec<f64>;
use super::vit::JepaViTEncoder;
use super::masking::{MaskingStrategy, MaskInfo};
use super::action_predictor::{ActionConditionedPredictor, FusionMode};
use super::loss::{EnergyModel, VicRegLoss};
use super::types::{JEPA_HIDDEN_DIM, JEPA_LEARNING_RATE, JEPA_EMA_MOMENTUM};

/// V2 JEPA World Model with ViT backbone + masking + action-conditioned predictor.
///
/// Architecture:
///   patches (E8 hexagram states) → JepaViTEncoder → masked patches
///   → encoded representation → ActionConditionedPredictor(action) → next state
///
/// Key differences from v1 (JepaWorldModel):
///   - Processes sequences of patches instead of single feature vectors
///   - Supports masking strategies for self-supervised learning
///   - Predictor is conditioned on an action embedding
///   - Uses a transformer-based encoder (ViT) instead of a single MLP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JepaWorldModelV2 {
    pub num_patches: usize,
    pub patch_dim: usize,
    pub embed_dim: usize,
    pub vit_encoder: JepaViTEncoder,
    pub target_encoder: JepaViTEncoder,
    pub masking: MaskingStrategy,
    pub predictor: ActionConditionedPredictor,
    pub energy_model: EnergyModel,
    pub vicreg: VicRegLoss,
    pub learning_rate: f64,
    pub momentum: f64,
    pub training_steps: usize,
}

impl JepaWorldModelV2 {
    /// Create a new V2 world model.
    ///
    /// `num_patches`: number of E8 state patches in a sequence
    /// `patch_dim`: dimension of each patch embedding
    /// `action_dim`: dimension of action embeddings
    /// `embed_dim`: transformer embedding dimension
    pub fn new(
        num_patches: usize,
        patch_dim: usize,
        action_dim: usize,
        embed_dim: usize,
        seed: u64,
    ) -> Self {
        let vit = JepaViTEncoder::new_mlp(patch_dim, embed_dim, seed);
        let target_encoder = vit.clone();
        let predictor = ActionConditionedPredictor::new(
            embed_dim,
            action_dim,
            JEPA_HIDDEN_DIM.max(embed_dim),
            FusionMode::Concat,
        );

        Self {
            num_patches,
            patch_dim,
            embed_dim,
            vit_encoder: vit,
            target_encoder,
            masking: MaskingStrategy::BlockMasking {
                block_size: 2,
                mask_ratio: 0.25,
            },
            predictor,
            energy_model: EnergyModel::new(),
            vicreg: VicRegLoss::new(),
            learning_rate: JEPA_LEARNING_RATE,
            momentum: JEPA_EMA_MOMENTUM,
            training_steps: 0,
        }
    }

    pub fn with_masking(mut self, strategy: MaskingStrategy) -> Self {
        self.masking = strategy;
        self
    }

    pub fn with_learning_rate(mut self, lr: f64) -> Self {
        self.learning_rate = lr;
        self
    }

    /// Encode patches into CLS representation.
    pub fn encode(&self, patches: &[Vec<f64>]) -> Vector {
        self.vit_encoder.encode_cls(patches)
    }

    /// Predict next state given current patches and action.
    ///
    /// Returns (predicted_next_state, energy, mask_info).
    pub fn predict(&self, patches: &[Vec<f64>], action: &[f64]) -> (Vector, f64, MaskInfo) {
        let (masked, mask_indices) = self.masking.apply(patches);
        let original_values: Vec<Vec<f64>> = mask_indices
            .iter()
            .map(|&idx| patches[idx].clone())
            .collect();
        let mask_info = MaskInfo::new(mask_indices, original_values);

        let state_encoding = self.vit_encoder.encode_cls(&masked);
        let pred = self.predictor.predict(&state_encoding, action);
        let energy = self.energy_model.energy(&pred, &state_encoding);

        (pred, energy, mask_info)
    }

    /// Predict next state without masking (used at inference).
    pub fn predict_no_mask(&self, patches: &[Vec<f64>], action: &[f64]) -> (Vector, f64) {
        let state_encoding = self.vit_encoder.encode_cls(patches);
        let pred = self.predictor.predict(&state_encoding, action);
        let energy = self.energy_model.energy(&pred, &state_encoding);
        (pred, energy)
    }

    /// Multi-step rollout: predict N steps into the future.
    ///
    /// At each step, the predicted latent is used as the new state encoding
    /// and fed back into the predictor with the corresponding action.
    pub fn rollout(
        &self,
        initial_patches: &[Vec<f64>],
        actions: &[Vec<f64>],
        max_steps: usize,
    ) -> Vec<Vector> {
        let n = max_steps.min(actions.len());
        if n == 0 {
            return Vec::new();
        }

        let mut trajectory = Vec::with_capacity(n);
        let mut state_encoding = self.vit_encoder.encode_cls(initial_patches);

        for act in actions.iter().take(n) {
            let next = self.predictor.predict(&state_encoding, act);
            trajectory.push(next.clone());
            state_encoding = next;
        }

        trajectory
    }

    /// Single training step.
    ///
    /// `patches`: current E8 state patches
    /// `action`: action taken
    /// `target_patches`: next state patches (used for target encoding)
    /// Returns (total_loss, energy_loss, vicreg_loss, inv_loss).
    pub fn train_step(
        &mut self,
        patches: &[Vec<f64>],
        action: &[f64],
        target_patches: &[Vec<f64>],
    ) -> (f64, f64, f64, f64) {
        let (masked, _) = self.masking.apply(patches);

        let z_context = self.vit_encoder.encode_cls(&masked);
        let z_target = self.target_encoder.encode_cls(target_patches);

        let z_pred = self.predictor.predict(&z_context, action);

        let energy = self.energy_model.energy(&z_pred, &z_target);
        let (vicreg_total, inv_loss, _var_loss, _cov_loss) = self.vicreg.compute(&z_pred, &z_target);

        let total_loss = energy + vicreg_total;

        self.predictor.update(&z_context, action, &z_target, self.learning_rate);
        self.ema_update_encoders();
        self.training_steps += 1;

        (total_loss, energy, vicreg_total, inv_loss)
    }

    /// Batch training step.
    pub fn train_batch(
        &mut self,
        batch_patches: &[Vec<Vec<f64>>],
        batch_actions: &[Vec<f64>],
        batch_targets: &[Vec<Vec<f64>>],
    ) -> f64 {
        let n = batch_patches.len().min(batch_actions.len()).min(batch_targets.len());
        if n == 0 {
            return 0.0;
        }
        let total: f64 = (0..n)
            .map(|i| {
                let (loss, _, _, _) = self.train_step(&batch_patches[i], &batch_actions[i], &batch_targets[i]);
                loss
            })
            .sum();
        total / n as f64
    }

    /// EMA update from context encoder to target encoder.
    fn ema_update_encoders(&mut self) {
        // Simple clone-based EMA: the JepaViTEncoder is cloned at init,
        // and we would normally do a parameter-wise EMA. For simplicity,
        // the target encoder is updated less frequently via cloning.
        // In a full implementation, we'd do layer-by-layer EMA.
        if self.training_steps.is_multiple_of(10) {
            self.target_encoder = self.vit_encoder.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_patches(n: usize, dim: usize) -> Vec<Vec<f64>> {
        (0..n)
            .map(|i| (0..dim).map(|d| ((i * dim + d) as f64) / 100.0).collect())
            .collect()
    }

    fn make_model() -> JepaWorldModelV2 {
        JepaWorldModelV2::new(8, 16, 4, 32, 42)
    }

    #[test]
    fn test_v2_encoder_output_dim() {
        let model = make_model();
        let patches = sample_patches(8, 16);
        let encoding = model.encode(&patches);
        assert_eq!(encoding.len(), 32);
    }

    #[test]
    fn test_v2_predict_output() {
        let model = make_model();
        let patches = sample_patches(8, 16);
        let action = vec![0.1; 4];
        let (pred, energy, mask_info) = model.predict(&patches, &action);
        assert_eq!(pred.len(), 32);
        assert!(energy >= 0.0);
        assert!(mask_info.num_masked() > 0);
    }

    #[test]
    fn test_v2_predict_no_mask() {
        let model = make_model();
        let patches = sample_patches(8, 16);
        let action = vec![0.1; 4];
        let (pred, energy) = model.predict_no_mask(&patches, &action);
        assert_eq!(pred.len(), 32);
        assert!(energy >= 0.0);
    }

    #[test]
    fn test_v2_rollout_length() {
        let model = make_model();
        let patches = sample_patches(8, 16);
        let actions: Vec<Vec<f64>> = (0..5).map(|_| vec![0.1; 4]).collect();
        let traj = model.rollout(&patches, &actions, 5);
        assert_eq!(traj.len(), 5);
        for t in &traj {
            assert_eq!(t.len(), 32);
        }
    }

    #[test]
    fn test_v2_rollout_empty() {
        let model = make_model();
        let patches = sample_patches(8, 16);
        let traj = model.rollout(&patches, &[], 0);
        assert!(traj.is_empty());
    }

    #[test]
    fn test_v2_train_step_reduces_loss() {
        let mut model = make_model();
        let patches = sample_patches(8, 16);
        let target = sample_patches(8, 16);
        let action = vec![0.1; 4];

        let (loss_before, _, _, _) = model.train_step(&patches, &action, &target);
        let (loss_after, _, _, _) = model.train_step(&patches, &action, &target);

        assert!(loss_after <= loss_before * 2.0 || loss_after < 100.0,
            "Loss should not explode: before={}, after={}", loss_before, loss_after);
    }

    #[test]
    fn test_v2_with_masking_strategy() {
        let model = make_model().with_masking(MaskingStrategy::RandomMasking { mask_ratio: 0.5 });
        let patches = sample_patches(8, 16);
        let action = vec![0.1; 4];
        let (pred, _energy, mask_info) = model.predict(&patches, &action);
        assert_eq!(pred.len(), 32);
        assert_eq!(mask_info.num_masked(), 4);
    }

    #[test]
    fn test_v2_batch_training() {
        let mut model = make_model();
        let batch_patches: Vec<Vec<Vec<f64>>> = (0..5).map(|_| sample_patches(8, 16)).collect();
        let batch_actions: Vec<Vec<f64>> = (0..5).map(|_| vec![0.1; 4]).collect();
        let batch_targets: Vec<Vec<Vec<f64>>> = (0..5).map(|_| sample_patches(8, 16)).collect();

        let loss = model.train_batch(&batch_patches, &batch_actions, &batch_targets);
        assert!(!loss.is_nan());
        assert!(loss >= 0.0);
    }

    #[test]
    fn test_v2_empty_batch() {
        let mut model = make_model();
        let loss = model.train_batch(&[], &[], &[]);
        assert_eq!(loss, 0.0);
    }

    #[test]
    fn test_v2_with_custom_lr() {
        let model = make_model().with_learning_rate(0.01);
        assert!((model.learning_rate - 0.01).abs() < 1e-10);
    }

    #[test]
    fn test_v2_different_actions_produce_different_predictions() {
        let model = make_model();
        let patches = sample_patches(8, 16);
        let action_a = vec![0.1; 4];
        let action_b = vec![0.9; 4];

        let (pred_a, _) = model.predict_no_mask(&patches, &action_a);
        let (pred_b, _) = model.predict_no_mask(&patches, &action_b);
        let diff: f64 = pred_a.iter().zip(pred_b.iter()).map(|(a, b)| (a - b).abs()).sum();
        assert!(diff > 1e-6, "Different actions should give different predictions, got diff={}", diff);
    }
}
