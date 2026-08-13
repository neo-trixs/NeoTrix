//! nt_mind::merge::mergekit_adapter — MergeKit 适配器接口
//!
//! 支持多种合并算法的统一适配器
//! 节点: nt_mind::merge::mergekit_adapter (L1)
//! Provides: mergekit_merging, adapter
//! Requires: mergekit, peft_suite
//! Rune: Crimson, Golden

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MergeKitConfig {
    /// 合并算法
    pub algorithm: MergeAlgorithm,
    /// 基础模型路径
    pub base_model: String,
    /// LoRA 路径
    pub lora_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum MergeAlgorithm {
    /// 简单平均
    Average,
    /// TIES 合并
    Ties,
    /// DARE
    Dare,
    /// LoRA MergeKit
    LoraMerge,
}

impl Default for MergeKitConfig {
    fn default() -> Self {
        Self {
            algorithm: MergeAlgorithm::Average,
            base_model: String::new(),
            lora_paths: Vec::new(),
        }
    }
}

/// MergeKit 适配器
pub struct MergeKitAdapter {
    config: MergeKitConfig,
    merged_weights: Option<HashMap<String, Vec<f32>>>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl MergeKitAdapter {
    pub fn new(config: MergeKitConfig) -> Self {
        Self {
            config,
            merged_weights: None,
            metadata: HashMap::new(),
        }
    }

    /// 执行 MergeKit 合并
    pub fn merge(&mut self) -> Result<HashMap<String, Vec<f32>>, NeoTrixError> {
        match self.config.algorithm {
            MergeAlgorithm::Average => self.average_merge(),
            MergeAlgorithm::Ties => self.ties_merge(),
            MergeAlgorithm::Dare => self.dare_merge(),
            MergeAlgorithm::LoraMerge => self.lora_merge(),
        }
    }

    fn average_merge(&mut self) -> Result<HashMap<String, Vec<f32>>, NeoTrixError> {
        // 平均合并: 简单平均所有权重
        // 实现简化版
        Err(NeoTrixError::NotImplemented(
            "MergeKit average merge not fully implemented".into(),
        ))
    }

    fn ties_merge(&mut self) -> Result<HashMap<String, Vec<f32>>, NeoTrixError> {
        // TIES merging
        Err(NeoTrixError::NotImplemented(
            "MergeKit ties merge not fully implemented".into(),
        ))
    }

    fn dare_merge(&mut self) -> Result<HashMap<String, Vec<f32>>, NeoTrixError> {
        // DARE merging
        Err(NeoTrixError::NotImplemented(
            "MergeKit dare merge not fully implemented".into(),
        ))
    }

    fn lora_merge(&mut self) -> Result<HashMap<String, Vec<f32>>, NeoTrixError> {
        // LoRA merge via MergeKit
        Err(NeoTrixError::NotImplemented(
            "MergeKit lora merge not fully implemented".into(),
        ))
    }

    pub fn get_merged(&self) -> Option<&HashMap<String, Vec<f32>>> {
        self.merged_weights.as_ref()
    }

    pub fn config(&self) -> &MergeKitConfig {
        &self.config
    }
}

impl CapabilityNode for MergeKitAdapter {
    fn node_id(&self) -> &str {
        "nt_mind::merge::mergekit_adapter"
    }
    fn provides(&self) -> Vec<String> {
        vec!["mergekit_merging".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["mergekit".into(), "peft_suite".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Crimson, RuneSocket::Golden]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for MergeKitAdapter {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let adapter = MergeKitAdapter::new(MergeKitConfig::default());
            // 基础检查: 配置验证
            let _ = adapter.config().algorithm;
            assert_eq!(adapter.config().algorithm, MergeAlgorithm::Average);
            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_mind_merge_mergekit"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mergekit_adapter_self_test() {
        let adapter = MergeKitAdapter::new(MergeKitConfig::default());
        assert!(adapter.self_test().is_ok());
    }
}
