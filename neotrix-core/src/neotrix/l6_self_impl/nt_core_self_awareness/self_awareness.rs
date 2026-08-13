//! nt_core_self_awareness — 自我意识模型
//!
//! 自我模型追踪和元认知监控
//! 节点: nt_core_self_awareness (L6)
//! Provides: self_model, meta_cognition
//! Requires: nt_core_traits, serde
//! Rune: Indigo, Alabaster

#![forbid(unsafe_code)]

use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SelfAwarenessConfig {
    /// 自我模型更新率
    pub update_rate: f32,
    /// 元认知检查间隔
    pub meta_check_interval: usize,
}

impl Default for SelfAwarenessConfig {
    fn default() -> Self {
        Self {
            update_rate: 0.1,
            meta_check_interval: 100,
        }
    }
}

/// 自我模型
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SelfModel {
    pub identity: String,
    pub capabilities: Vec<String>,
    pub confidence: f32,
    pub last_updated: usize,
}

/// 元认知状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetaCognitionState {
    pub awareness_level: f32,
    pub coherence: f32,
    pub last_inspection: usize,
}

/// 自我意识模块
pub struct SelfAwarenessModule {
    config: SelfAwarenessConfig,
    self_model: SelfModel,
    meta_state: MetaCognitionState,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl SelfAwarenessModule {
    pub fn new(config: SelfAwarenessConfig) -> Self {
        Self {
            config,
            self_model: SelfModel {
                identity: "default".into(),
                capabilities: Vec::new(),
                confidence: 1.0,
                last_updated: 0,
            },
            meta_state: MetaCognitionState {
                awareness_level: 1.0,
                coherence: 1.0,
                last_inspection: 0,
            },
            metadata: HashMap::new(),
        }
    }

    pub fn update_self_model(&mut self, new_capabilities: Vec<String>) {
        self.self_model.capabilities = new_capabilities;
        self.self_model.last_updated += 1;
        self.self_model.confidence = (self.self_model.confidence * (1.0 - self.config.update_rate)
            + self.config.update_rate)
            .min(1.0);
    }

    pub fn assess_meta_cognition(&mut self) -> f32 {
        let coherence = self.meta_state.coherence.min(self.self_model.confidence);
        self.meta_state.awareness_level = coherence;
        coherence
    }

    pub fn config(&self) -> &SelfAwarenessConfig {
        &self.config
    }
}

impl CapabilityNode for SelfAwarenessModule {
    fn node_id(&self) -> &str {
        "nt_core_self_awareness"
    }
    fn provides(&self) -> Vec<String> {
        vec!["self_model".into(), "meta_cognition".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["nt_core_traits".into(), "serde".into()]
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

impl SelfTest for SelfAwarenessModule {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let mut sa = SelfAwarenessModule::new(SelfAwarenessConfig::default());

            sa.update_self_model(vec!["reasoning".into(), "memory".into()]);
            let coherence = sa.assess_meta_cognition();

            assert!(coherence >= 0.0 && coherence <= 1.0);
            assert!(sa.self_model.capabilities.contains(&"reasoning".into()));

            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_core_self_awareness_module"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_self_awareness_self_test() {
        let sa = SelfAwarenessModule::new(SelfAwarenessConfig::default());
        assert!(sa.self_test().is_ok());
    }
}
