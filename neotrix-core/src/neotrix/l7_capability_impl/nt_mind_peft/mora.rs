//! nt_mind::peft::mora — MoRA: 高秩更新 LoRA
//!
//! 论文: MoRA (2405.12130) — 方阵实现高秩更新，保持参数量不变
//! 节点: nt_mind::peft::mora (L1)
//! Provides: high_rank_update
//! Requires: lora_core, svd_decomp
//! Rune: Indigo, Alabaster

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MoRAConfig {
    pub rank: usize, // 方阵边长
    pub alpha: f32,
    pub dropout: f32,
    pub target_modules: Vec<String>,
    /// 输入压缩维度
    pub input_dim: usize,
    /// 输出扩展维度
    pub output_dim: usize,
}

impl Default for MoRAConfig {
    fn default() -> Self {
        Self {
            rank: 8, // 方阵 8x8 = 64 参数，等价 LoRA rank=8
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
            input_dim: 512,
            output_dim: 512,
        }
    }
}

/// MoRA 方阵权重
#[derive(Debug, Clone)]
pub struct MoRAWeights {
    /// 方阵 M (rank x rank)
    pub matrix: Vec<Vec<f32>>,
    /// 输入压缩投影 (input_dim x rank)
    pub input_proj: Vec<Vec<f32>>,
    /// 输出扩展投影 (rank x output_dim)
    pub output_proj: Vec<Vec<f32>>,
    pub scaling: f32,
}

/// MoRA 实现
pub struct MoRA {
    config: MoRAConfig,
    adapters: HashMap<String, MoRAWeights>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl MoRA {
    pub fn new(config: MoRAConfig) -> Self {
        Self {
            config,
            adapters: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn init_adapter(
        &mut self,
        module_name: &str,
        _in_features: usize,
        _out_features: usize,
    ) -> Result<(), NeoTrixError> {
        let rank = self.config.rank;
        let _scaling = self.config.alpha / rank as f32;

        // 方阵初始化
        let std = (2.0 / (rank * 2) as f32).sqrt();
        let matrix = (0..rank)
            .map(|_| (0..rank).map(|_| rand::random::<f32>() * std).collect())
            .collect();

        // 输入压缩
        let input_proj = (0..rank)
            .map(|_| {
                (0..self.config.input_dim)
                    .map(|_| rand::random::<f32>() * std)
                    .collect()
            })
            .collect();

        // 输出扩展
        let output_proj = (0..self.config.output_dim)
            .map(|_| (0..rank).map(|_| rand::random::<f32>() * std).collect())
            .collect();

        self.adapters.insert(
            module_name.to_string(),
            MoRAWeights {
                matrix,
                input_proj,
                output_proj,
                scaling: self.config.alpha / rank as f32,
            },
        );
        Ok(())
    }

    /// 前向: x -> input_proj -> M -> output_proj -> output
    pub fn forward(&self, module_name: &str, input: &[f32]) -> Result<Vec<f32>, NeoTrixError> {
        let weights = self.adapters.get(module_name).ok_or_else(|| {
            NeoTrixError::NotFound(format!("MoRA adapter not found: {}", module_name))
        })?;

        let input_dim = self.config.input_dim;
        let output_dim = self.config.output_dim;
        let rank = self.config.rank;

        if input.len() % input_dim != 0 {
            return Err(NeoTrixError::InvalidInput("Input dim mismatch".into()));
        }
        let batch = input.len() / input_dim;
        let mut output = vec![0.0; batch * output_dim];

        for b in 0..batch {
            let x = &input[b * input_dim..(b + 1) * input_dim];

            // 1. input_proj @ x -> [rank]
            let mut hidden = vec![0.0; rank];
            for r in 0..rank {
                let mut sum = 0.0;
                for i in 0..input_dim {
                    sum += x[i] * weights.input_proj[r][i];
                }
                hidden[r] = sum;
            }

            // 2. M @ hidden -> [rank]
            let mut transformed = vec![0.0; rank];
            for r in 0..rank {
                let mut sum = 0.0;
                for c in 0..rank {
                    sum += weights.matrix[r][c] * hidden[c];
                }
                transformed[r] = sum;
            }

            // 3. output_proj @ transformed -> [output_dim]
            for o in 0..output_dim {
                let mut sum = 0.0;
                for r in 0..rank {
                    sum += transformed[r] * weights.output_proj[o][r];
                }
                output[b * output_dim + o] = sum * weights.scaling;
            }
        }
        Ok(output)
    }

    pub fn config(&self) -> &MoRAConfig {
        &self.config
    }
}

impl CapabilityNode for MoRA {
    fn node_id(&self) -> &str {
        "nt_mind::peft::mora"
    }
    fn provides(&self) -> Vec<String> {
        vec!["high_rank_update".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["lora_core".into(), "svd_decomp".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Indigo, RuneSocket::Alabaster]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for MoRA {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let mut mora = MoRA::new(MoRAConfig::default());
            mora.init_adapter("test_proj", 512, 512)?;

            let input = vec![0.5; 512];
            let output = mora.forward("test_proj", &input)?;
            assert_eq!(output.len(), 512);
            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_mind_peft_mora"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mora_self_test() {
        let mora = MoRA::new(MoRAConfig::default());
        assert!(mora.self_test().is_ok());
    }
}
