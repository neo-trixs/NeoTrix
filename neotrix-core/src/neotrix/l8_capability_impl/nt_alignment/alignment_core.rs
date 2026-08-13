//! nt_mind::alignment::alignment_core — 核心对齐层
//!
//! 实现统一对齐目标：行为对齐、奖励对齐、风格对齐
//! 节点: nt_mind::alignment::alignment_core (L2)
//! Provides: alignment, style_alignment, behavior_alignment
//! Requires: peft_suite, tokenizer, reward_model
//! Rune: Indigo, Alabaster

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AlignmentConfig {
    /// 对齐权重
    pub alignment_weight: f32,
    /// 风格保留率
    pub style_retention: f32,
    /// 温度系数
    pub temperature: f32,
}

impl Default for AlignmentConfig {
    fn default() -> Self {
        Self {
            alignment_weight: 0.1,
            style_retention: 0.8,
            temperature: 0.1,
        }
    }
}

/// 核心对齐层
pub struct AlignmentCore {
    config: AlignmentConfig,
    aligned_model: Option<HashMap<String, Vec<f32>>>,
    history: Vec<AlignmentStep>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// 单步对齐记录
#[derive(Debug, Clone)]
pub struct AlignmentStep {
    pub step_id: usize,
    pub delta: HashMap<String, Vec<f32>>,
    pub reward_change: f32,
    pub style_distance: f32,
}

impl AlignmentCore {
    pub fn new(config: AlignmentConfig) -> Self {
        Self {
            config,
            aligned_model: None,
            history: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// 执行对齐更新
    pub fn align(
        &mut self,
        base_weights: &HashMap<String, Vec<f32>>,
        gradient: &HashMap<String, Vec<f32>>,
        rewards: &[f32],
    ) -> Result<(), NeoTrixError> {
        let mut delta = HashMap::new();

        for (name, g) in gradient {
            let fallback = vec![0.0; g.len()];
            let base = match base_weights.get(name) {
                Some(b) => b,
                None => &fallback,
            };
            let reward_scale = if !rewards.is_empty() {
                rewards.last().unwrap_or(&0.0)
            } else {
                &0.0
            };

            let mut d = vec![0.0; g.len()];
            for (i, g_val) in g.iter().enumerate() {
                // 对齐更新: W <- W - lr * (grad - reward * base)
                d[i] = g_val - reward_scale * base[i];
            }
            delta.insert(name.clone(), d);
        }

        self.aligned_model = Some(delta.clone());
        self.history.push(AlignmentStep {
            step_id: self.history.len(),
            delta,
            reward_change: 0.0,
            style_distance: 0.0,
        });
        Ok(())
    }

    pub fn get_aligned(&self) -> Option<&HashMap<String, Vec<f32>>> {
        self.aligned_model.as_ref()
    }

    pub fn config(&self) -> &AlignmentConfig {
        &self.config
    }
}

impl CapabilityNode for AlignmentCore {
    fn node_id(&self) -> &str {
        "nt_mind::alignment::alignment_core"
    }
    fn provides(&self) -> Vec<String> {
        vec![
            "alignment".into(),
            "style_alignment".into(),
            "behavior_alignment".into(),
        ]
    }
    fn requires(&self) -> Vec<String> {
        vec![
            "peft_suite".into(),
            "tokenizer".into(),
            "reward_model".into(),
        ]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Indigo, RuneSocket::Alabaster]
    }
    fn constellation_level(&self) -> u8 {
        1
    }
    fn promote_constellation(&mut self) -> bool {
        true
    }
}

impl SelfTest for AlignmentCore {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let mut align = AlignmentCore::new(AlignmentConfig::default());
            let mut base = HashMap::new();
            base.insert("embed".into(), vec![1.0, 2.0, 3.0]);
            let mut grad = HashMap::new();
            grad.insert("embed".into(), vec![0.1, 0.2, 0.3]);
            let rewards = vec![1.0];

            align.align(&base, &grad, &rewards)?;
            assert!(align.get_aligned().is_some());
            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_mind_alignment_core"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_alignment_core_self_test() {
        let align = AlignmentCore::new(AlignmentConfig::default());
        assert!(align.self_test().is_ok());
    }
}
