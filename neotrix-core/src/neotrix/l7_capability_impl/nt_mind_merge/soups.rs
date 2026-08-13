//! nt_mind::merge::soups — Model Soups: 多模型权重平均
//!
//! 论文: Model Soups (2203.05482) — 同一基座不同超参微调模型权重平均
//! 节点: nt_mind::merge::soups (L1)
//! Provides: model_soups_averaging
//! Requires: peft_suite
//! Rune: Indigo, Obsidian

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelSoupsConfig {
    /// 平均策略
    pub averaging_strategy: AveragingStrategy,
    /// 是否归一化
    pub normalize: bool,
    /// 温度参数 (用于加权平均)
    pub temperature: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AveragingStrategy {
    /// 简单均值
    Uniform,
    /// 验证集性能加权
    PerformanceWeighted { metric: String },
    /// 梯度相似度加权
    GradientSimilarityWeighted,
}

impl Default for ModelSoupsConfig {
    fn default() -> Self {
        Self {
            averaging_strategy: AveragingStrategy::Uniform,
            normalize: true,
            temperature: 1.0,
        }
    }
}

/// Model Soups 实现
pub struct ModelSoups {
    config: ModelSoupsConfig,
    checkpoints: Vec<ModelCheckpoint>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ModelCheckpoint {
    pub name: String,
    pub weights: HashMap<String, Vec<f32>>, // param_name -> weight vector
    pub validation_score: Option<f32>,
    pub hyperparams: serde_json::Value,
}

impl ModelSoups {
    pub fn new(config: ModelSoupsConfig) -> Self {
        Self {
            config,
            checkpoints: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// 添加检查点
    pub fn add_checkpoint(&mut self, checkpoint: ModelCheckpoint) -> Result<(), NeoTrixError> {
        if self.checkpoints.is_empty() {
            // 首个检查点作为基准，记录参数结构
        } else {
            // 验证参数结构一致
            let first = &self.checkpoints[0];
            for (name, weights) in &checkpoint.weights {
                if let Some(first_w) = first.weights.get(name) {
                    if first_w.len() != weights.len() {
                        return Err(NeoTrixError::InvalidInput(format!(
                            "Weight shape mismatch for {}: {} vs {}",
                            name,
                            first_w.len(),
                            weights.len()
                        )));
                    }
                } else {
                    return Err(NeoTrixError::InvalidInput(format!(
                        "Missing parameter: {}",
                        name
                    )));
                }
            }
        }
        self.checkpoints.push(checkpoint);
        Ok(())
    }

    /// 执行模型平均
    pub fn merge(&self) -> Result<HashMap<String, Vec<f32>>, NeoTrixError> {
        if self.checkpoints.is_empty() {
            return Err(NeoTrixError::InvalidState("No checkpoints to merge".into()));
        }

        let mut merged = HashMap::new();
        let param_names: Vec<String> = self.checkpoints[0].weights.keys().cloned().collect();

        for name in param_names {
            let mut merged_weights = vec![0.0; self.checkpoints[0].weights[&name].len()];

            let _weights = match &self.config.averaging_strategy {
                AveragingStrategy::Uniform => {
                    let weight = 1.0 / self.checkpoints.len() as f32;
                    for cp in &self.checkpoints {
                        let w = &cp.weights[&name];
                        for (i, &v) in w.iter().enumerate() {
                            merged_weights[i] += v * weight;
                        }
                    }
                }
                AveragingStrategy::PerformanceWeighted { metric: _ } => {
                    // 使用验证分数加权
                    let total_score: f32 = self
                        .checkpoints
                        .iter()
                        .map(|cp| cp.validation_score.unwrap_or(1.0))
                        .sum();
                    for cp in &self.checkpoints {
                        let score = cp.validation_score.unwrap_or(1.0);
                        let weight = (score / total_score).powf(self.config.temperature);
                        let w = &cp.weights[&name];
                        for (i, &v) in w.iter().enumerate() {
                            merged_weights[i] += v * weight;
                        }
                    }
                }
                AveragingStrategy::GradientSimilarityWeighted => {
                    // 简化：退化为均值
                    let weight = 1.0 / self.checkpoints.len() as f32;
                    for cp in &self.checkpoints {
                        let w = &cp.weights[&name];
                        for (i, &v) in w.iter().enumerate() {
                            merged_weights[i] += v * weight;
                        }
                    }
                }
            };

            if self.config.normalize {
                // L2 归一化
                let norm: f32 = merged_weights.iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm > 0.0 {
                    for v in &mut merged_weights {
                        *v /= norm;
                    }
                }
            }

            merged.insert(name, merged_weights);
        }
        Ok(merged)
    }

    pub fn config(&self) -> &ModelSoupsConfig {
        &self.config
    }
    pub fn checkpoint_count(&self) -> usize {
        self.checkpoints.len()
    }
}

impl CapabilityNode for ModelSoups {
    fn node_id(&self) -> &str {
        "nt_mind::merge::soups"
    }
    fn provides(&self) -> Vec<String> {
        vec!["model_soups_averaging".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["peft_suite".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Indigo, RuneSocket::Obsidian]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for ModelSoups {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let mut soups = ModelSoups::new(ModelSoupsConfig {
                averaging_strategy: AveragingStrategy::Uniform,
                normalize: false,
                temperature: 1.0,
            });

            // 创建测试检查点
            let cp1 = ModelCheckpoint {
                name: "cp1".into(),
                weights: {
                    let mut m = HashMap::new();
                    m.insert("layer1.weight".into(), vec![1.0, 2.0, 3.0]);
                    m.insert("layer2.bias".into(), vec![0.1, 0.2]);
                    m
                },
                validation_score: Some(0.9),
                hyperparams: serde_json::json!({"lr": 1e-4}),
            };
            let cp2 = ModelCheckpoint {
                name: "cp2".into(),
                weights: {
                    let mut m = HashMap::new();
                    m.insert("layer1.weight".into(), vec![2.0, 3.0, 4.0]);
                    m.insert("layer2.bias".into(), vec![0.2, 0.3]);
                    m
                },
                validation_score: Some(0.85),
                hyperparams: serde_json::json!({"lr": 5e-5}),
            };

            soups.add_checkpoint(cp1)?;
            soups.add_checkpoint(cp2)?;

            let merged = soups.merge()?;
            assert!(merged.contains_key("layer1.weight"));
            assert_eq!(merged["layer1.weight"].len(), 3);
            // 均值: (1+2)/2=1.5, (2+3)/2=2.5, (3+4)/2=3.5
            assert!((merged["layer1.weight"][0] - 1.5).abs() < 1e-5);

            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_mind_merge_soups"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_soups_self_test() {
        let soups = ModelSoups::new(ModelSoupsConfig::default());
        assert!(soups.self_test().is_ok());
    }
}
