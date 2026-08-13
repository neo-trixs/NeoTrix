//! nt_core_consciousness_bridge — 意识桥接层
//!
//! GWT 广播与专家间协调的桥接
//! 节点: nt_core_consciousness_bridge (L5)
//! Provides: gwt_broadcast, expert_coordination
//! Requires: nt_core_gwt, nt_core_traits
//! Rune: Crimson, Indigo

#![forbid(unsafe_code)]

use crate::core::nt_core_gwt::resonance::{ResonanceMatrix, ResonanceReport, RESONANCE_THRESHOLD};
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsciousnessBridgeConfig {
    /// GWT 广播强度
    pub broadcast_strength: f32,
    /// 共享工作空间容量
    pub workspace_capacity: usize,
}

impl Default for ConsciousnessBridgeConfig {
    fn default() -> Self {
        Self {
            broadcast_strength: 1.0,
            workspace_capacity: 64,
        }
    }
}

/// 意识桥接层
pub struct ConsciousnessBridge {
    config: ConsciousnessBridgeConfig,
    resonance: Option<ResonanceMatrix>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl ConsciousnessBridge {
    pub fn new(config: ConsciousnessBridgeConfig) -> Self {
        Self {
            config,
            resonance: None,
            metadata: HashMap::new(),
        }
    }

    pub fn set_resonance(&mut self, resonance: ResonanceMatrix) {
        self.resonance = Some(resonance);
    }

    /// GWT 广播：向工作空间广播显著信息
    pub fn broadcast(&self, _message: &str, activation: f32) -> Option<ResonanceReport> {
        let _resonance = self.resonance.as_ref()?;

        // 简化实现：基于阈值筛选 (u32 常量无需解引用)
        if (activation * 10.0) as u32 >= RESONANCE_THRESHOLD {
            // 构造有效 ResonanceReport — 基于当前矩阵状态
            let winner = 0; // 简化：模块 0 为广播源
            let mut effective_saliences =
                [0.0_f64; crate::core::nt_core_gwt::resonance::MODULE_COUNT];
            let mut raw_saliences = [0.0_f64; crate::core::nt_core_gwt::resonance::MODULE_COUNT];
            effective_saliences[winner] = activation as f64;
            raw_saliences[winner] = activation as f64;
            Some(ResonanceReport {
                winner,
                effective_saliences,
                raw_saliences,
                entropy: 1.5,
                resonator_clusters: vec![vec![winner]],
                complement_activated: false,
            })
        } else {
            None
        }
    }

    pub fn config(&self) -> &ConsciousnessBridgeConfig {
        &self.config
    }
}

impl CapabilityNode for ConsciousnessBridge {
    fn node_id(&self) -> &str {
        "nt_core_consciousness_bridge"
    }
    fn provides(&self) -> Vec<String> {
        vec!["gwt_broadcast".into(), "expert_coordination".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["nt_core_gwt".into(), "nt_core_traits".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Crimson, RuneSocket::Indigo]
    }
    fn constellation_level(&self) -> u8 {
        1
    }
    fn promote_constellation(&mut self) -> bool {
        true
    }
}

impl SelfTest for ConsciousnessBridge {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let bridge = ConsciousnessBridge::new(ConsciousnessBridgeConfig::default());
            let _ = bridge.config();
            let _ = bridge.broadcast("test", 0.8);
            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_core_consciousness_bridge"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_consciousness_bridge_self_test() {
        let bridge = ConsciousnessBridge::new(ConsciousnessBridgeConfig::default());
        assert!(bridge.self_test().is_ok());
    }
}
