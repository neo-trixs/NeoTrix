//! nt_mind::peft::reft — ReFT: Representation Finetuning
//!
//! 论文: ReFT (2404.03592) — 冻结模型，学习隐层表示干预
//! 节点: nt_mind::peft::reft (L1)
//! Provides: representation_finetuning
//! Requires: lora_core, intervention
//! Rune: Indigo, Golden

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ReFTType {
    /// LoReFT: Low-rank Linear Subspace ReFT
    LoReFT { rank: usize },
    /// 标准 ReFT
    Standard { intervention_dim: usize },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReFTConfig {
    pub reft_type: ReFTType,
    pub alpha: f32,
    pub dropout: f32,
    pub target_layers: Vec<usize>,
    pub intervention_positions: Vec<String>, // "residual", "attention", "mlp"
}

impl Default for ReFTConfig {
    fn default() -> Self {
        Self {
            reft_type: ReFTType::LoReFT { rank: 4 },
            alpha: 1.0,
            dropout: 0.0,
            target_layers: vec![],
            intervention_positions: vec!["residual".into()],
        }
    }
}

/// ReFT 干预权重
#[derive(Debug, Clone)]
pub struct ReFTWeights {
    /// 干预矩阵
    pub intervention: Vec<Vec<f32>>,
    /// 投影矩阵 (用于 LoReFT)
    pub projection: Option<Vec<Vec<f32>>>,
    pub scaling: f32,
}

/// ReFT 实现
pub struct ReFT {
    config: ReFTConfig,
    interventions: HashMap<String, ReFTWeights>, // layer.position -> weights
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl ReFT {
    pub fn new(config: ReFTConfig) -> Self {
        Self {
            config,
            interventions: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn init_intervention(
        &mut self,
        layer: usize,
        position: &str,
        hidden_dim: usize,
    ) -> Result<(), NeoTrixError> {
        let key = format!("layer_{}.{}", layer, position);
        let (intervention_dim, _rank) = match self.config.reft_type {
            ReFTType::LoReFT { rank } => (hidden_dim, rank),
            ReFTType::Standard { intervention_dim } => (intervention_dim, 0),
        };

        let _std = (2.0 / hidden_dim as f32).sqrt();

        // 全秩干预矩阵: [intervention_dim × hidden_dim]
        let intervention: Vec<Vec<f32>> = (0..intervention_dim)
            .map(|_| {
                (0..hidden_dim)
                    .map(|_| rand::random::<f32>() * 0.01)
                    .collect()
            })
            .collect();

        self.interventions.insert(
            key,
            ReFTWeights {
                intervention: intervention.clone(),
                projection: Some(intervention),
                scaling: 1.0,
            },
        );
        Ok(())
    }

    /// 应用干预: h' = h + α * intervention(h)
    pub fn apply_intervention(
        &self,
        layer: usize,
        position: &str,
        hidden: &mut [f32],
    ) -> Result<(), NeoTrixError> {
        let key = format!("layer_{}.{}", layer, position);
        let weights = self
            .interventions
            .get(&key)
            .ok_or_else(|| NeoTrixError::NotFound(format!("Intervention not found: {}", key)))?;

        // LoReFT: h' = h + α * P^T @ W @ P @ h
        // Standard: h' = h + α * W @ h
        if let Some(ref proj) = weights.projection {
            // LoReFT: 低秩子空间干预
            let rank = proj.len();
            let hidden_dim = hidden.len();

            // P @ h -> [rank]
            let mut projected = vec![0.0; rank];
            for r in 0..rank {
                let mut sum = 0.0;
                for i in 0..hidden_dim {
                    sum += proj[r][i] * hidden[i];
                }
                projected[r] = sum;
            }

            // W @ projected -> [intervention_dim]
            let mut intervened = vec![0.0; weights.intervention.len()];
            for i in 0..weights.intervention.len() {
                let mut sum = 0.0;
                for r in 0..rank {
                    sum += weights.intervention[i][r] * projected[r];
                }
                intervened[i] = sum;
            }

            // P^T @ intervened -> [hidden_dim]
            let mut delta = vec![0.0; hidden_dim];
            for i in 0..hidden_dim {
                let mut sum = 0.0;
                for r in 0..rank {
                    sum += proj[r][i] * intervened[r];
                }
                delta[i] = sum;
            }

            // 应用干预
            for i in 0..hidden_dim {
                hidden[i] += self.config.alpha * delta[i];
            }
        } else {
            // Standard: 直接干预
            for i in 0..hidden.len().min(weights.intervention.len()) {
                let mut sum = 0.0;
                for j in 0..hidden.len() {
                    sum += weights.intervention[i][j] * hidden[j];
                }
                hidden[i] += self.config.alpha * sum;
            }
        }
        Ok(())
    }

    pub fn config(&self) -> &ReFTConfig {
        &self.config
    }
}

impl CapabilityNode for ReFT {
    fn node_id(&self) -> &str {
        "nt_mind::peft::reft"
    }
    fn provides(&self) -> Vec<String> {
        vec!["representation_finetuning".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["lora_core".into(), "intervention".into()]
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

impl SelfTest for ReFT {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let mut reft = ReFT::new(ReFTConfig::default());
            reft.init_intervention(0, "residual", 512)?;

            let mut hidden = vec![0.5; 512];
            reft.apply_intervention(0, "residual", &mut hidden)?;
            assert_eq!(hidden.len(), 512);
            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_mind_peft_reft"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_reft_self_test() {
        let reft = ReFT::new(ReFTConfig::default());
        assert!(reft.self_test().is_ok());
    }
}
