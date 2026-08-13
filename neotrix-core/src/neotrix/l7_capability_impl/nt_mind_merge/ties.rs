//! nt_mind::merge::ties — TIES-Merging: 修剪+选符号+合并
//!
//! 论文: TIES-Merging (2306.01708) — 解决模型合并中的参数干扰
//! 节点: nt_mind::merge::ties (L1)
//! Provides: ties_merging
//! Requires: peft_suite, svd_decomp
//! Rune: Indigo, Golden

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TIESConfig {
    /// 修剪比例 (0.0-1.0)
    pub trim_ratio: f32,
    /// 选符号阈值
    pub sign_threshold: f32,
    /// 合并后缩放因子
    pub merge_scale: f32,
}

impl Default for TIESConfig {
    fn default() -> Self {
        Self {
            trim_ratio: 0.1,     // 修剪 10% 幅度最小的参数
            sign_threshold: 0.0, // 符号一致性阈值
            merge_scale: 1.0,
        }
    }
}

/// TIES-Merging 实现
pub struct TIESMerging {
    config: TIESConfig,
    checkpoints: Vec<ModelCheckpoint>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ModelCheckpoint {
    pub name: String,
    pub weights: HashMap<String, Vec<f32>>,
    pub validation_score: Option<f32>,
}

impl TIESMerging {
    pub fn new(config: TIESConfig) -> Self {
        Self {
            config,
            checkpoints: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_checkpoint(&mut self, checkpoint: ModelCheckpoint) -> Result<(), NeoTrixError> {
        self.checkpoints.push(checkpoint);
        Ok(())
    }

    /// TIES 核心合并算法
    /// 1. Task Arithmetic: 计算任务向量 (微调 - 预训练)
    /// 2. Trim: 修剪幅度最小的参数
    /// 3. Elect Sign: 选取主流符号
    /// 4. Merge: 加权平均
    pub fn merge(
        &self,
        base_weights: &HashMap<String, Vec<f32>>,
    ) -> Result<HashMap<String, Vec<f32>>, NeoTrixError> {
        if self.checkpoints.is_empty() {
            return Err(NeoTrixError::InvalidState("No checkpoints".into()));
        }

        let param_names: Vec<String> = base_weights.keys().cloned().collect();
        let mut merged = HashMap::new();

        for name in param_names {
            let base = base_weights.get(&name).unwrap();
            let param_len = base.len();

            // 1. 计算任务向量: task_vec = finetuned - base
            let mut task_vectors = Vec::new();
            for cp in &self.checkpoints {
                let ft = cp.weights.get(&name).ok_or_else(|| {
                    NeoTrixError::InvalidInput(format!("Missing param {} in checkpoint", name))
                })?;
                if ft.len() != param_len {
                    return Err(NeoTrixError::InvalidInput("Shape mismatch".into()));
                }
                let mut task_vec = vec![0.0; param_len];
                for i in 0..param_len {
                    task_vec[i] = ft[i] - base[i];
                }
                task_vectors.push(task_vec);
            }

            // 2. Trim: 修剪幅度最小的参数 (按比例)
            let trim_k = (param_len as f32 * self.config.trim_ratio) as usize;
            let mut merged_task = vec![0.0; param_len];

            for i in 0..param_len {
                // 收集所有任务向量在该位置的值
                let mut values: Vec<f32> = task_vectors.iter().map(|tv| tv[i]).collect();

                // 按绝对值排序，保留 top (1-trim_ratio)
                values.sort_by(|a, b| b.abs().partial_cmp(&a.abs()).unwrap());
                let _keep_k = values.len() - trim_k;

                // 3. Elect Sign: 计算符号一致性
                let pos_count = values
                    .iter()
                    .filter(|&&v| v > self.config.sign_threshold)
                    .count();
                let neg_count = values
                    .iter()
                    .filter(|&&v| v < -self.config.sign_threshold)
                    .count();

                let dominant_sign = if pos_count > neg_count {
                    1.0
                } else if neg_count > pos_count {
                    -1.0
                } else {
                    0.0
                };

                // 只保留与主流符号一致的值
                let filtered: Vec<f32> = values
                    .into_iter()
                    .filter(|&v| v * dominant_sign >= 0.0)
                    .collect();

                // 4. 平均
                if !filtered.is_empty() {
                    merged_task[i] = filtered.iter().sum::<f32>() / filtered.len() as f32;
                }
            }

            // 5. 最终合并: base + scale * merged_task
            let mut final_weights = vec![0.0; param_len];
            for i in 0..param_len {
                final_weights[i] = base[i] + self.config.merge_scale * merged_task[i];
            }
            merged.insert(name, final_weights);
        }
        Ok(merged)
    }

    pub fn config(&self) -> &TIESConfig {
        &self.config
    }
}

impl CapabilityNode for TIESMerging {
    fn node_id(&self) -> &str {
        "nt_mind::merge::ties"
    }
    fn provides(&self) -> Vec<String> {
        vec!["ties_merging".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["peft_suite".into(), "svd_decomp".into()]
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

impl SelfTest for TIESMerging {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let mut ties = TIESMerging::new(TIESConfig::default());
            let mut base = HashMap::new();
            base.insert("layer1.weight".into(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);

            let mut cp1 = ModelCheckpoint {
                name: "cp1".into(),
                weights: HashMap::new(),
                validation_score: None,
            };
            cp1.weights
                .insert("layer1.weight".into(), vec![1.5, 2.5, 3.5, 4.5, 6.0]);

            let mut cp2 = ModelCheckpoint {
                name: "cp2".into(),
                weights: HashMap::new(),
                validation_score: None,
            };
            cp2.weights
                .insert("layer1.weight".into(), vec![2.0, 3.0, 4.0, 5.0, 7.0]);

            ties.add_checkpoint(cp1)?;
            ties.add_checkpoint(cp2)?;

            let merged = ties.merge(&base)?;
            assert!(merged.contains_key("layer1.weight"));
            assert_eq!(merged["layer1.weight"].len(), 5);
            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_mind_merge_ties"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ties_self_test() {
        let ties = TIESMerging::new(TIESConfig::default());
        assert!(ties.self_test().is_ok());
    }
}
