pub mod omniscient_view;
pub mod visual_cortex;
pub mod auditory_cortex;
pub mod nt_world_sense_hub;
pub mod world_consciousness;
pub mod real_sensors;
pub mod types;
pub mod encoder;
pub mod predictor;
pub mod loss;
pub mod sigreg;
pub mod td_jepa;
pub mod rgm_jepa;
pub mod vit;
pub mod masking;
pub mod action_predictor;
pub mod world_model;
pub mod world_model_v2;
pub mod nt_world_model_types;
pub mod nt_world_model_predict;
pub mod nt_world_model_td_jepa;
pub mod nt_world_model_rgm_jepa;
pub mod nt_world_model_object_partition;

pub use omniscient_view::*;
pub use visual_cortex::*;
pub use auditory_cortex::*;
pub use nt_world_sense_hub::*;
pub use world_consciousness::*;

pub use types::{
    JEPA_LATENT_DIM, JEPA_HIDDEN_DIM, JEPA_EMA_MOMENTUM, JEPA_VARIANCE_TARGET,
    JEPA_VAR_WEIGHT, JEPA_COV_WEIGHT, JEPA_INV_WEIGHT, JEPA_LEARNING_RATE,
    JEPA_GAUSS_STD_TARGET, JEPA_GAUSS_WEIGHT,
    MultiScalePrediction, LatentState, TDExperience, WorldModelState,
};
pub use encoder::JepaEncoder;
pub use predictor::JepaPredictor;
pub use loss::{EnergyModel, VicRegLoss, NonContrastiveLoss, levl_jepa_loss};
pub use sigreg::SIGReg;
pub use td_jepa::{TDTarget, TDDynamics};
pub use rgm_jepa::{CGBlock, MultiScaleJEPA, RGMLatent};
pub use world_model::JepaWorldModel;
pub use vit::JepaViTEncoder;
pub use masking::{MaskingStrategy, MaskInfo};
pub use action_predictor::{ActionConditionedPredictor, FusionMode};
pub use world_model_v2::JepaWorldModelV2;

pub use nt_world_model_types::*;
pub use nt_world_model_predict::*;
pub use nt_world_model_td_jepa::*;
pub use nt_world_model_rgm_jepa::*;
pub use nt_world_model_object_partition::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::nt_core_hcube::cube::KnowledgeHyperCube;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WifiStatus {
    pub enabled: bool,
    pub occupant_count: usize,
    pub zone_count: usize,
    pub heatmap: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEncoder {
    pub projection: Matrix,
}

impl Default for ContextEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextEncoder {
    pub fn new() -> Self {
        let mut projection = vec![vec![0.0; 64]; LATENT_DIM];

        for row in projection.iter_mut() {
            for val in row.iter_mut() {
                *val = (rand::random::<f64>() - 0.5) * 0.1;
            }
        }

        Self { projection }
    }

    pub fn encode(&self, context: &Context) -> Vector {
        let features = context.to_features();
        let mut z = vec![0.0; LATENT_DIM];

        for (i, item) in z.iter_mut().enumerate() {
            for (j, &feat) in features.iter().enumerate().take(64) {
                if j < self.projection[i].len() {
                    *item += self.projection[i][j] * feat;
                }
            }
            *item = item.tanh();
        }

        z
    }

    pub fn update(&mut self, context: &Context, target_z: &[f64], performance: f64, lr: f64) {
        let features = context.to_features();
        let encoded = self.encode(context);

        let error_sign = if performance > 0.5 { 1.0 } else { -1.0 };

        for i in 0..LATENT_DIM.min(target_z.len()) {
            let error = error_sign * (target_z[i] - encoded[i]);

            for (j, &feat) in features.iter().enumerate().take(features.len().min(64)) {
                if j < self.projection[i].len() {
                    self.projection[i][j] += lr * error * feat;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldModel {
    pub latent_dim: usize,
    pub transition_model: LatentTransition,
    pub expert_predictor: ExpertPredictor,
    pub context_encoder: ContextEncoder,
    pub num_experts: usize,
    pub occupant_count: usize,
    pub zone_heatmap: HashMap<String, f64>,
    pub wifi_enabled: bool,
}

impl WorldModel {
    pub fn new(num_experts: usize) -> Self {
        let num_experts = num_experts.max(1);
        Self {
            latent_dim: LATENT_DIM,
            transition_model: LatentTransition::new(num_experts),
            expert_predictor: ExpertPredictor::new(),
            context_encoder: ContextEncoder::new(),
            num_experts,
            occupant_count: 0,
            zone_heatmap: HashMap::new(),
            wifi_enabled: false,
        }
    }

    pub fn predict_expert_performance(
        &self,
        _x: &Vector,
        context: &Context,
        expert_ids: &[usize],
    ) -> Vector {
        let z = self.context_encoder.encode(context);
        let latent = LatentState { value: z.clone(), delta: vec![0.0; z.len()] };

        self.expert_predictor.predict_all(&latent, expert_ids)
    }

    pub fn plan(
        &self,
        initial_state: &LatentState,
        horizon: usize,
        num_candidates: usize,
    ) -> Vec<usize> {
        let mut candidates = Vec::new();

        for _ in 0..num_candidates {
            let mut sequence = Vec::new();
            let mut z = initial_state.value.clone();

            for _ in 0..horizon {
                let expert_id = if self.num_experts > 0 {
                    (rand::random::<f64>() * self.num_experts as f64) as usize % self.num_experts
                } else {
                    0
                };
                sequence.push(expert_id);

                let action = vec![0.0; self.num_experts];
                z = self.transition_model.predict(&z, &action);
            }

            candidates.push(sequence);
        }

        let mut scored: Vec<(Vec<usize>, f64)> = candidates
            .into_iter()
            .map(|seq| {
                let score = self.evaluate_sequence(&initial_state.value, &seq);
                (seq, score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((best_seq, _)) = scored.first() {
            best_seq.clone()
        } else {
            vec![0]
        }
    }

    fn evaluate_sequence(&self, initial_z: &[f64], sequence: &[usize]) -> f64 {
        if sequence.is_empty() {
            return 0.0;
        }
        let mut z = initial_z.to_vec();
        let mut total_score = 0.0;

        for &expert_id in sequence {
            let action = vec![0.0; self.num_experts];
            z = self.transition_model.predict(&z, &action);
            total_score += self.expert_predictor.predict(expert_id, &z);
        }

        total_score / sequence.len() as f64
    }

    pub fn update(
        &mut self,
        x: &Vector,
        context: &Context,
        selected_experts: &[usize],
        performance: f64,
    ) {
        let mut z = self.context_encoder.encode(context);

        if !x.is_empty() {
            let input_factor = x.iter().take(LATENT_DIM).cloned().collect::<Vector>();
            for i in 0..LATENT_DIM.min(x.len()) {
                z[i] = 0.7 * z[i] + 0.3 * input_factor[i].tanh();
            }
        }

        for &expert_id in selected_experts {
            self.expert_predictor.update(expert_id, &z, performance, 0.01);
        }

        if performance > 0.5 && !selected_experts.is_empty() {
            let action = vec![0.0; self.num_experts];
            let z_next = self.transition_model.predict(&z, &action);

            let target = if performance > 0.7 {
                z_next.clone()
            } else {
                z_next.iter().map(|v| -v).collect()
            };

            self.transition_model.update(&z, &action, &target, 0.005);
        }

        self.context_encoder.update(context, &z, performance, 0.005);
    }

    pub fn update_from_wifi(&mut self, status: &WifiStatus) {
        self.occupant_count = status.occupant_count;
        self.zone_heatmap = status.heatmap.clone();
        self.wifi_enabled = status.enabled;
    }

    pub fn reward_from_knowledge_quality(&self, cube: &KnowledgeHyperCube) -> f64 {
        let mut reward = 0.0;

        let coverage = if cube.is_empty() {
            0.0
        } else {
            let density_sum: f64 = (0..8).map(|d| cube.coord_density(d)).sum();
            (density_sum / 8.0).min(1.0)
        };
        reward += coverage * 0.4;

        let recency: f64 = if cube.is_empty() {
            0.0
        } else {
            let total: f64 = cube.entries().map(|e| e.value).sum();
            (total / cube.len() as f64).min(1.0)
        };
        reward += recency * 0.3;

        let consistency: f64 = if cube.is_empty() {
            1.0
        } else {
            let sparse_count = (0..8)
                .filter(|&d| cube.coord_density(d) < 0.1)
                .count();
            if sparse_count > 4 {
                0.3
            } else if sparse_count > 2 {
                0.6
            } else {
                1.0
            }
        };
        reward += consistency * 0.3;

        reward.clamp(0.0, 1.0)
    }
}

pub use crate::neotrix::nt_world_model_v2::WorldModelV2;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod model_tests {
    use super::*;

    #[test]
    fn test_latent_state_similarity() {
        let s1 = LatentState::new();
        let s2 = LatentState::new();
        let sim = s1.similarity(&s2);
        assert!(sim > 0.9);
    }

    #[test]
    fn test_context_from_description() {
        let ctx = Context::from_task_description("设计一个iOS原型");
        assert_eq!(ctx.task_type as usize, TaskType::Design as usize);

        let ctx2 = Context::from_task_description("分析这段代码的性能");
        assert_eq!(ctx2.task_type as usize, TaskType::CodeAnalysis as usize);
    }

    #[test]
    fn test_wifi_integration() {
        let mut wm = WorldModel::new(4);
        assert_eq!(wm.occupant_count, 0);
        assert!(wm.zone_heatmap.is_empty());
        assert!(!wm.wifi_enabled);

        let mut heatmap = std::collections::HashMap::new();
        heatmap.insert("zone_1".to_string(), 0.85);
        let status = WifiStatus {
            enabled: true,
            occupant_count: 2,
            zone_count: 3,
            heatmap,
        };
        wm.update_from_wifi(&status);
        assert_eq!(wm.occupant_count, 2);
        assert!((wm.zone_heatmap.get("zone_1").expect("value should be ok in test") - 0.85).abs() < 1e-6);
        assert!(wm.wifi_enabled);
    }

    #[test]
    fn test_reward_from_knowledge_quality_empty_cube() {
        let wm = WorldModel::new(4);
        let cube = KnowledgeHyperCube::new();
        let reward = wm.reward_from_knowledge_quality(&cube);
        assert!(reward >= 0.0 && reward <= 1.0);
    }

    #[test]
    fn test_reward_from_knowledge_quality_partially_filled() {
        let wm = WorldModel::new(4);
        let mut cube = KnowledgeHyperCube::new();
        let coord = crate::core::nt_core_hcube::coord::HyperCoord::new();
        cube.insert(&coord, "test-source", "test-entry");
        let reward = wm.reward_from_knowledge_quality(&cube);
        assert!(reward >= 0.0 && reward <= 1.0);
    }
}
