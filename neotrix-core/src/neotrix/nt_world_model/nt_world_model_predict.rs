use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use crate::neotrix::nt_core_signal::ops::cosine_similarity;
use super::LATENT_DIM;
use super::Vector;

/// Latent transition model: z_{t+1} = f(z_t, a_t) with learned weights and bias.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatentTransition {
    pub weights: super::Matrix,
    pub bias: Vector,
}

impl LatentTransition {
    pub fn new(num_experts: usize) -> Self {
        let input_dim = LATENT_DIM + num_experts;
        let mut weights = vec![vec![0.0; input_dim]; LATENT_DIM];
        let bias = vec![0.0; LATENT_DIM];

        let std = (2.0 / (LATENT_DIM + input_dim) as f64).sqrt();
        for row in weights.iter_mut() {
            for val in row.iter_mut() {
                *val = (rand::random::<f64>() - 0.5) * 2.0 * std;
            }
        }

        Self { weights, bias }
    }

    /// 预测下一个 latent 状态
    pub fn predict(&self, z_t: &[f64], action: &[f64]) -> Vector {
        let input: Vector = z_t.iter().chain(action.iter()).cloned().collect();
        let mut z_next = vec![0.0; LATENT_DIM];

        for (i, item) in z_next.iter_mut().enumerate() {
            let mut sum = self.bias[i];
            for (j, &val) in input.iter().enumerate() {
                if j < self.weights[i].len() {
                    sum += self.weights[i][j] * val;
                }
            }
            *item = sum.tanh();
        }

        z_next
    }

    /// 更新模型 (简单的梯度下降)
    pub fn update(&mut self, z_t: &[f64], action: &[f64], z_target: &[f64], lr: f64) {
        let input: Vector = z_t.iter().chain(action.iter()).cloned().collect();
        let prediction = self.predict(z_t, action);

        for (i, &p) in prediction.iter().enumerate().take(LATENT_DIM) {
            let error = z_target[i] - p;
            self.bias[i] += lr * error;

            for (j, &inp) in input.iter().enumerate().take(input.len().min(self.weights[i].len())) {
                self.weights[i][j] += lr * error * inp;
            }
        }
    }
}

/// Predicts expert performance for a given latent state using weighted history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertPredictor {
    pub expert_weights: HashMap<usize, Vector>,
    pub performance_history: VecDeque<PerformanceRecord>,
}

/// A single performance observation for an expert on a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecord {
    pub expert_id: usize,
    pub latent_state: Vector,
    pub performance: f64,
    pub timestamp: i64,
}

impl Default for ExpertPredictor {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpertPredictor {
    pub fn new() -> Self {
        Self {
            expert_weights: HashMap::new(),
            performance_history: VecDeque::with_capacity(1000),
        }
    }

    /// 预测专家在给定 latent 状态下的表现
    /// 结合权重预测和历史相似任务检索
    pub fn predict(&self, expert_id: usize, latent: &[f64]) -> f64 {
        let weight_pred = if let Some(weights) = self.expert_weights.get(&expert_id) {
            weights.iter()
                .zip(latent.iter())
                .map(|(w, z)| w * z)
                .sum::<f64>()
                .tanh()
                .clamp(-1.0, 1.0)
        } else {
            0.0
        };

        let history_factor = self.retrieve_similar_performance(expert_id, latent);

        let combined = 0.7 * (weight_pred * 0.5 + 0.5) + 0.3 * history_factor;

        combined.clamp(0.0, 1.0)
    }

    /// 检索历史相似任务的表现
    fn retrieve_similar_performance(&self, expert_id: usize, latent: &[f64]) -> f64 {
        if self.performance_history.is_empty() {
            return 0.5;
        }

        let expert_history: Vec<_> = self.performance_history
            .iter()
            .filter(|r| r.expert_id == expert_id)
            .collect();

        if expert_history.is_empty() {
            return 0.5;
        }

        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        let current_time = chrono::Utc::now().timestamp();

        for record in &expert_history {
            let similarity = cosine_similarity(latent, &record.latent_state);
            if similarity > 0.5 {
                let time_diff = (current_time - record.timestamp) as f64;
                let time_decay = (-time_diff / 86400.0).exp();

                let weight = similarity * time_decay;
                weighted_sum += weight * record.performance;
                total_weight += weight;
            }
        }

        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.5
        }
    }

    /// 为所有专家生成预测向量 (兼容旧接口)
    pub fn predict_all(&self, latent: &super::LatentState, _experts: &[usize]) -> Vector {
        vec![self.predict(0, &latent.vector)]
    }

    /// 更新专家权重 (在线学习)
    pub fn update(&mut self, expert_id: usize, latent: &[f64], performance: f64, lr: f64) {
        let prediction = self.predict(expert_id, latent);
        let error = performance - prediction;

        let weights = self.expert_weights.entry(expert_id).or_insert_with(|| {
            vec![0.0; LATENT_DIM]
        });

        for (w, &z) in weights.iter_mut().zip(latent.iter()) {
            *w += lr * error * z;
        }

        self.performance_history.push_back(PerformanceRecord {
            expert_id,
            latent_state: latent.to_vec(),
            performance,
            timestamp: chrono::Utc::now().timestamp(),
        });

        if self.performance_history.len() > 1000 {
            self.performance_history.pop_front();
        }
    }
}
