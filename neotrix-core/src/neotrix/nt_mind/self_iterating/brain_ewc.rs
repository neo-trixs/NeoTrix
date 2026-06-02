use serde::{Serialize, Deserialize};
use super::super::core::CapabilityVector;
use super::brain_seal::CapabilityDelta;

/// SIA weight update 记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightUpdateRecord {
    pub generation: u64,
    pub reward: f64,
    pub algorithm: Option<RLAlgorithm>,
    pub timestamp: u64,
}

/// RL 算法选择枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RLAlgorithm {
    PPOGAE,
    GRPO,
    EntropicAdvantage,
    REINFORCEKL,
    BestOfNBC,
    DPO,
}

/// 评估记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationRecord {
    pub iteration: u64,
    pub task_type: super::super::super::nt_world_model::TaskType,
    pub score_before: f64,
    pub score_after: f64,
    pub improved: bool,
}

/// Fisher 信息矩阵 — 防止灾难性遗忘
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FisherMatrix {
    pub values: Vec<f64>,
    pub total_samples: u64,
}

impl FisherMatrix {
    pub fn new(dim: usize) -> Self {
        Self { values: vec![0.0; dim], total_samples: 0 }
    }

    pub fn update_raw(&mut self, deltas: &[f64]) {
        self.total_samples += 1;
        let n = self.total_samples as f64;
        for (i, &delta) in deltas.iter().enumerate() {
            if i < self.values.len() {
                self.values[i] = self.values[i] * ((n - 1.0) / n) + delta.abs() * (1.0 / n);
            }
        }
    }

    pub fn update(&mut self, deltas: &[CapabilityDelta], lr: f64) {
        self.total_samples += 1;
        let n = self.total_samples as f64;
        for d in deltas {
            if let Some(idx) = CapabilityVector::index_from_name(&d.dimension) {
                if idx < self.values.len() {
                    let importance = d.delta.abs() * lr;
                    self.values[idx] = self.values[idx] * ((n - 1.0) / n) + importance * (1.0 / n);
                }
            }
        }
    }

    pub fn ewc_penalty(&self, current: &[f64], proposed: &[f64]) -> f64 {
        current.iter().zip(proposed.iter())
            .enumerate()
            .filter(|(i, _)| *i < self.values.len())
            .map(|(i, (c, p))| self.values[i] * (p - c) * (p - c))
            .sum()
    }
}

/// EWC persistence methods for ReasoningBrain
mod ewc_persistence {
    use super::super::brain_core::ReasoningBrain;
    use super::FisherMatrix;

    impl ReasoningBrain {
        pub fn save_ewc(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
            let data = serde_json::to_string(&self.ewc_lambda)?;
            let fisher_data = serde_json::to_string(&self.fisher)?;
            let combined = format!("{{\"ewc_lambda\":{},\"fisher\":{}}}", data, fisher_data);
            std::fs::write(path, combined)?;
            Ok(())
        }

        pub fn load_ewc(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
            let content = std::fs::read_to_string(path)?;
            let parsed: serde_json::Value = serde_json::from_str(&content)?;
            if let Some(lambda) = parsed["ewc_lambda"].as_f64() {
                self.ewc_lambda = lambda;
            }
            if let Some(fisher) = parsed["fisher"].as_object() {
                let values: Vec<f64> = fisher.get("values")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|x| x.as_f64()).collect())
                    .unwrap_or_default();
                let total_samples = fisher.get("total_samples")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                if !values.is_empty() {
                    self.fisher = Some(FisherMatrix { values, total_samples });
                }
            }
            Ok(())
        }
    }
}
