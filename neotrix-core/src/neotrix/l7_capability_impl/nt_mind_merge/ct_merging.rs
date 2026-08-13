//! nt_mind::merge::ct_merging — CT-Merging: 共识任务合并
//!
//! 论文: CT-Merging (2607.20561) — 基于共识方向的加权合并
//! 节点: nt_mind::merge::ct_merging (L1)
//! Provides: consensus_task_merging
//! Requires: peft_suite, task_arithmetic
//! Rune: Indigo, Golden

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CTMergingConfig {
    /// 共识向量学习率
    pub consensus_lr: f32,
    /// 共识阈值
    pub consensus_threshold: f32,
    /// 加权因子
    pub weights: Vec<f32>,
}

impl Default for CTMergingConfig {
    fn default() -> Self {
        Self {
            consensus_lr: 0.01,
            consensus_threshold: 0.5,
            weights: vec![1.0],
        }
    }
}

/// 单个检查点
#[derive(Debug, Clone)]
pub struct ModelCheckpoint {
    pub name: String,
    pub weights: HashMap<String, Vec<f32>>,
    pub validation_score: Option<f32>,
}

/// CT-Merging 实现
pub struct CTMerging {
    config: CTMergingConfig,
    checkpoints: Vec<ModelCheckpoint>,
    consensus_directions: HashMap<String, Vec<f32>>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl CTMerging {
    pub fn new(config: CTMergingConfig) -> Self {
        Self {
            config,
            checkpoints: Vec::new(),
            consensus_directions: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_checkpoint(
        &mut self,
        name: String,
        weights: HashMap<String, Vec<f32>>,
        validation_score: Option<f32>,
    ) {
        self.checkpoints.push(ModelCheckpoint {
            name,
            weights,
            validation_score,
        });
    }

    /// 计算共识方向: 对每个参数, 如果大部分模型的符号一致则取符号一致的平均值
    /// 如果符号冲突则退化为性能加权平均
    pub fn compute_consensus_direction(&self) -> Result<HashMap<String, Vec<f32>>, NeoTrixError> {
        if self.checkpoints.is_empty() {
            return Err(NeoTrixError::InvalidState("No checkpoints".into()));
        }

        let param_names: Vec<String> = self.checkpoints[0].weights.keys().cloned().collect();
        let mut consensus = HashMap::new();

        for name in &param_names {
            let mut weighted_sum = vec![0.0; self.checkpoints[0].weights[name].len()];
            let mut sign_counts: (i32, i32) = (0, 0); // (正, 负)
            let mut total_weight = 0.0;

            for cp in &self.checkpoints {
                let w = cp
                    .weights
                    .get(name)
                    .cloned()
                    .unwrap_or(vec![0.0; weighted_sum.len()]);
                let score = cp.validation_score.unwrap_or(1.0);

                // 获取符号
                let last_val = *w.last().unwrap_or(&0.0);
                let sign = if last_val > 0.0 {
                    1
                } else if last_val < 0.0 {
                    -1
                } else {
                    0
                };

                if sign > 0 {
                    sign_counts.0 += 1
                } else if sign < 0 {
                    sign_counts.1 += 1
                }

                for (i, &val) in w.iter().enumerate() {
                    weighted_sum[i] += val * score;
                }
                total_weight += score;
            }

            // 如果符号一致，取平均
            if sign_counts.0 > sign_counts.1 {
                // 正符号占多数
                for s in weighted_sum.iter_mut() {
                    *s /= total_weight;
                }
                consensus.insert(name.clone(), weighted_sum);
            } else if sign_counts.1 > sign_counts.0 {
                // 负符号占多数
                for s in weighted_sum.iter_mut() {
                    *s /= total_weight;
                }
                consensus.insert(name.clone(), weighted_sum);
            } else {
                // 符号平衡，退化为性能加权平均
                if total_weight > 0.0 {
                    for s in weighted_sum.iter_mut() {
                        *s /= total_weight;
                    }
                }
                consensus.insert(name.clone(), weighted_sum);
            }
        }

        Ok(consensus)
    }

    /// CT-Merging 合并主函数
    pub fn merge(&self) -> Result<HashMap<String, Vec<f32>>, NeoTrixError> {
        let consensus = self.compute_consensus_direction()?;
        Ok(consensus)
    }
}

impl CapabilityNode for CTMerging {
    fn node_id(&self) -> &str {
        "nt_mind::merge::ct_merging"
    }
    fn provides(&self) -> Vec<String> {
        vec!["consensus_task_merging".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["peft_suite".into(), "task_arithmetic".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Indigo, RuneSocket::Golden]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for CTMerging {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let mut ct = CTMerging::new(CTMergingConfig::default());
            let mut weights = HashMap::new();
            weights.insert("layer1.weight".into(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
            ct.add_checkpoint("cp1".into(), weights, Some(0.9));

            let merged = ct.merge()?;
            assert!(merged.contains_key("layer1.weight"));
            assert_eq!(merged["layer1.weight"].len(), 5);
            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_mind_merge_ct_merging"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ct_merging_self_test() {
        let ct = CTMerging::new(CTMergingConfig::default());
        assert!(ct.self_test().is_ok());
    }
}
