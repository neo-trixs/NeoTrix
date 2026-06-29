//! 世界模型 (World Model) - 预测专家表现与任务适配
//!
//! 基于 Nemotron 3 Super 的 LatentMoE 思想 + JEPA 联合嵌入预测架构:
//! - 在压缩的 latent 空间进行 routing
//! - 使用 CEM (Cross-Entropy Method) 进行规划
//! - 学习专家在不同场景下的表现模式
//!
//! Core: z_{t+1} = predict(z_t, a_t) where a_t is an expert selection
//! JEPA: S_y = Predictor(Encoder(x)) — latent space prediction

pub mod bridge;
pub mod intel_profile;
pub mod moe_to_dense;
pub mod moment_feed;
pub mod nt_world_model_object_partition;
pub mod nt_world_model_predict;
pub mod nt_world_model_rgm_jepa;
pub mod nt_world_model_td_jepa;
pub mod nt_world_model_types;
pub mod workspace;

pub use bridge::{WorldModelBridge, WorldModelReport};
pub use nt_world_model_predict::*;
pub use nt_world_model_types::*;

pub use moe_to_dense::*;
pub use nt_world_model_object_partition::*;

use crate::core::nt_core_hcube::cube::KnowledgeHyperCube;
use crate::neotrix::nt_mind::wifi_sensing::WifiStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// Context Encoder - 将上下文编码到 latent 空间
// ============================================================

/// Encodes task context into the latent space via a learned projection matrix.
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

    /// 更新投影矩阵（在线学习）
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

// ============================================================
// World Model - 主结构
// ============================================================

/// World Model — predicts expert performance in latent space using CEM planning.
///
/// Core equation: z_{t+1} = predict(z_t, a_t) where a_t is an expert selection.
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

    /// 预测专家表现: 给定输入和上下文，预测每个专家的表现
    pub fn predict_expert_performance(
        &self,
        _x: &Vector,
        context: &Context,
        expert_ids: &[usize],
    ) -> Vector {
        let z = self.context_encoder.encode(context);
        let latent = LatentState {
            vector: z,
            timestamp: 0,
        };

        self.expert_predictor.predict_all(&latent, expert_ids)
    }

    /// 规划: 使用 CEM 找到最优专家组合
    pub fn plan(
        &self,
        initial_state: &LatentState,
        horizon: usize,
        num_candidates: usize,
    ) -> Vec<usize> {
        let mut candidates = Vec::new();

        for _ in 0..num_candidates {
            let mut sequence = Vec::new();
            let mut z = initial_state.vector.clone();

            for _ in 0..horizon {
                let expert_id =
                    (rand::random::<f64>() * self.num_experts as f64) as usize % self.num_experts;
                sequence.push(expert_id);

                let action = vec![0.0; self.num_experts];
                z = self.transition_model.predict(&z, &action);
            }

            candidates.push(sequence);
        }

        let mut scored: Vec<(Vec<usize>, f64)> = candidates
            .into_iter()
            .map(|seq| {
                let score = self.evaluate_sequence(&initial_state.vector, &seq);
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
        let mut z = initial_z.to_vec();
        let mut total_score = 0.0;

        for &expert_id in sequence {
            let action = vec![0.0; self.num_experts];
            z = self.transition_model.predict(&z, &action);
            total_score += self.expert_predictor.predict(expert_id, &z);
        }

        total_score / sequence.len() as f64
    }

    /// 更新世界模型
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
            self.expert_predictor
                .update(expert_id, &z, performance, 0.01);
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

    /// 从 WiFi 感知引擎注入空间数据
    pub fn update_from_wifi(&mut self, status: &WifiStatus) {
        self.occupant_count = status.occupant_count;
        self.zone_heatmap = status.heatmap.clone();
        self.wifi_enabled = status.enabled;
    }

    /// Compute an RL reward from KnowledgeHyperCube quality metrics.
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
            let sparse_count = (0..8).filter(|&d| cube.coord_density(d) < 0.1).count();
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

// Re-export WorldModelV2 from its own module (split to reduce file size)
pub use crate::neotrix::nt_world_model_v2::WorldModelV2;

// ============================================================
// 测试（统一推理大脑架构）
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_mind::wifi_sensing::WifiStatus;
    use crate::neotrix::nt_mind::ReasoningBrain;

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
    fn test_nt_world_model_with_nt_mind() {
        let mut brain = ReasoningBrain::new();
        brain.absorb(crate::neotrix::nt_mind::KnowledgeSource::HeroUI);

        let ctx = Context::from_task_description("设计一个极简风格的页面");
        let score = brain.evaluate_capability(ctx.task_type);

        assert!(score >= 0.0 && score <= 1.0);
        assert!(score > 0.0);
    }

    #[test]
    fn test_brain_stats() {
        let mut brain = ReasoningBrain::new();
        brain.absorb(crate::neotrix::nt_mind::KnowledgeSource::BaseUI);

        let stats = brain.get_statistics();
        assert!(stats.total_absorbed > 0);
        assert!(!stats.unique_sources.is_empty());
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
        assert!(
            (wm.zone_heatmap
                .get("zone_1")
                .expect("value should be ok in test")
                - 0.85)
                .abs()
                < 1e-6
        );
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
