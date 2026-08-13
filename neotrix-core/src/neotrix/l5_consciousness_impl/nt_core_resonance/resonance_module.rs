//! nt_core_resonance — 共振模块
//!
//! 意识共振网络和驻频机制
//! 节点: nt_core_resonance (L5)
//! Provides: resonance_sync, specialist_routing
//! Requires: nt_core_traits, serde
//! Rune: Indigo, Alabaster

#![forbid(unsafe_code)]

use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResonanceConfig {
    /// 共振阈值
    pub threshold: f32,
    /// 衰减率
    pub decay_rate: f32,
    /// 专家模块数量
    pub specialist_count: usize,
}

impl Default for ResonanceConfig {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            decay_rate: 0.1,
            specialist_count: 8,
        }
    }
}

/// 共振状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResonanceState {
    pub active_specialists: Vec<usize>,
    pub activation_strength: f32,
    pub coherence: f32,
}

/// 共振模块
pub struct ResonanceModule {
    config: ResonanceConfig,
    state: ResonanceState,
    specialist_activations: HashMap<usize, f32>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl ResonanceModule {
    pub fn new(config: ResonanceConfig) -> Self {
        Self {
            config,
            state: ResonanceState {
                active_specialists: Vec::new(),
                activation_strength: 0.0,
                coherence: 0.0,
            },
            specialist_activations: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn update_specialist(&mut self, specialist_id: usize, activation: f32) {
        self.specialist_activations
            .insert(specialist_id, activation);

        // 重新计算活跃专家
        let mut active: Vec<usize> = self
            .specialist_activations
            .iter()
            .filter(|(_, &a)| a > self.config.threshold)
            .map(|(id, _)| *id)
            .collect();
        active.sort_by(|a, b| {
            self.specialist_activations
                .get(b)
                .unwrap_or(&0.0)
                .partial_cmp(self.specialist_activations.get(a).unwrap_or(&0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        self.state.active_specialists = active;

        // 计算激活强度
        let total: f32 = self.specialist_activations.values().sum();
        self.state.activation_strength = if !self.specialist_activations.is_empty() {
            total / self.specialist_activations.len() as f32
        } else {
            0.0
        };

        // 计算一致性/共识
        if !self.state.active_specialists.is_empty() {
            let avg_activation: f32 = self.specialist_activations.values().sum::<f32>()
                / self.state.active_specialists.len() as f32;
            self.state.coherence = avg_activation.min(1.0);
        } else {
            self.state.coherence = 0.0;
        }
    }

    pub fn get_state(&self) -> &ResonanceState {
        &self.state
    }

    pub fn config(&self) -> &ResonanceConfig {
        &self.config
    }
}

impl CapabilityNode for ResonanceModule {
    fn node_id(&self) -> &str {
        "nt_core_resonance"
    }
    fn provides(&self) -> Vec<String> {
        vec!["resonance_sync".into(), "specialist_routing".into()]
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

impl SelfTest for ResonanceModule {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let mut rm = ResonanceModule::new(ResonanceConfig::default());

            // 更新几个专家的激活度
            rm.update_specialist(0, 0.8);
            rm.update_specialist(1, 0.6);
            rm.update_specialist(2, 0.3); // 低于阈值

            let state = rm.get_state();
            assert!(!state.active_specialists.is_empty());
            assert!(state.coherence > 0.0);
            assert!(state.activation_strength > 0.0);

            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_core_resonance_module"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_resonance_module_self_test() {
        let rm = ResonanceModule::new(ResonanceConfig::default());
        assert!(rm.self_test().is_ok());
    }
}
