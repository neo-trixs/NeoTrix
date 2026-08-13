//! nt_mind::peft::dora — DoRA: Weight-Decomposed Low-Rank Adaptation
//!
//! 论文: DoRA (2402.09353) — 权重分解 W = |W| * Ŵ，幅度+方向分离
//! 节点: nt_mind::peft::dora (L1)
//! Provides: weight_decomposed_lora
//! Requires: lora_core, svd_decomp
//! Rune: Indigo, Golden

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DoRAConfig {
    pub rank: usize,
    pub alpha: f32,
    pub dropout: f32,
    pub target_modules: Vec<String>,
    /// 幅度向量学习率缩放
    pub magnitude_lr_scale: f32,
    /// 方向 LoRA 秩
    pub direction_rank: usize,
}

impl Default for DoRAConfig {
    fn default() -> Self {
        Self {
            rank: 8,
            alpha: 16.0,
            dropout: 0.0,
            target_modules: vec![
                "q_proj".into(),
                "v_proj".into(),
                "k_proj".into(),
                "o_proj".into(),
                "gate_proj".into(),
                "up_proj".into(),
                "down_proj".into(),
            ],
            magnitude_lr_scale: 0.1,
            direction_rank: 8,
        }
    }
}

/// DoRA 权重: 幅度 + 方向
#[derive(Debug, Clone)]
pub struct DoRAWeights {
    /// 幅度向量: |W| (out_features,)
    pub magnitude: Vec<f32>,
    /// 方向 LoRA: A (rank, in_features), B (out_features, rank)
    pub direction_a: Vec<Vec<f32>>,
    pub direction_b: Vec<Vec<f32>>,
    /// 缩放因子
    pub scaling: f32,
}

/// DoRA 实现
pub struct DoRA {
    config: DoRAConfig,
    adapters: std::collections::HashMap<String, DoRAWeights>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl DoRA {
    pub fn new(config: DoRAConfig) -> Self {
        Self {
            config,
            adapters: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn init_adapter(
        &mut self,
        module_name: &str,
        in_features: usize,
        out_features: usize,
    ) -> Result<(), NeoTrixError> {
        let rank = self
            .config
            .direction_rank
            .min(in_features)
            .min(out_features);
        let _scaling = self.config.alpha / rank as f32;

        let std = (2.0 / (in_features + rank) as f32).sqrt();
        let direction_a = (0..rank)
            .map(|_| {
                (0..in_features)
                    .map(|_| rand::random::<f32>() * std)
                    .collect()
            })
            .collect();
        let direction_b = vec![vec![0.0; rank]; out_features];

        // 幅度初始化为 1.0 (不改变原始模型幅度)
        let magnitude = vec![1.0; out_features];

        self.adapters.insert(
            module_name.to_string(),
            DoRAWeights {
                magnitude,
                direction_a,
                direction_b,
                scaling: self.config.alpha / rank as f32,
            },
        );
        Ok(())
    }

    /// 合并 DoRA 到基础权重: W_new = (|W| + Δ|W|) * (Ŵ + ΔŴ)
    pub fn merge_weights(
        &self,
        module_name: &str,
        _base_weight: &[f32],
        out_features: usize,
        in_features: usize,
    ) -> Result<Vec<f32>, NeoTrixError> {
        let weights = self.adapters.get(module_name).ok_or_else(|| {
            NeoTrixError::NotFound(format!("DoRA adapter not found: {}", module_name))
        })?;

        // 简化实现：返回方向 LoRA 的合并增量
        let rank = self.config.direction_rank;
        let mut delta = vec![0.0; out_features * in_features];
        for o in 0..out_features {
            for i in 0..in_features {
                let mut sum = 0.0;
                for r in 0..rank {
                    sum += weights.direction_b[o][r] * weights.direction_a[r][i];
                }
                delta[o * in_features + i] = sum * weights.scaling;
            }
        }
        Ok(delta)
    }

    pub fn config(&self) -> &DoRAConfig {
        &self.config
    }
}

impl CapabilityNode for DoRA {
    fn node_id(&self) -> &str {
        "nt_mind::peft::dora"
    }
    fn provides(&self) -> Vec<String> {
        vec!["weight_decomposed_lora".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["lora_core".into(), "svd_decomp".into()]
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

impl SelfTest for DoRA {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let mut dora = DoRA::new(DoRAConfig::default());
            dora.init_adapter("test_proj", 32, 64)?;

            let base = vec![0.5; 64 * 32];
            let merged = dora.merge_weights("test_proj", &base, 64, 32)?;
            assert_eq!(merged.len(), 64 * 32);
            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_mind_peft_dora"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dora_self_test() {
        let dora = DoRA::new(DoRAConfig::default());
        assert!(dora.self_test().is_ok());
    }
}
