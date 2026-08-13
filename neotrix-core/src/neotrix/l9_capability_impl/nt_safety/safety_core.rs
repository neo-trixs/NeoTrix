//! nt_mind::safety::safety_core — 核心安全层
//!
//! 安全约束、边界检测、风险评估
//! 节点: nt_mind::safety::safety_core (L2)
//! Provides: safety_check, risk_assessment, constraint_verification
//! Requires: peft_suite, config
//! Rune: Obsidian, Alabaster

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SafetyConfig {
    /// 最大权重变化幅度
    pub max_weight_change: f32,
    /// 风险阈值
    pub risk_threshold: f32,
    /// 是否启用约束检查
    pub enable_constraints: bool,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            max_weight_change: 0.1,
            risk_threshold: 0.5,
            enable_constraints: true,
        }
    }
}

/// 核心安全层
pub struct SafetyCore {
    config: SafetyConfig,
    risk_history: Vec<f32>,
    violation_count: usize,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl SafetyCore {
    pub fn new(config: SafetyConfig) -> Self {
        Self {
            config,
            risk_history: Vec::new(),
            violation_count: 0,
            metadata: HashMap::new(),
        }
    }

    /// 检查权重变化是否超出安全边界
    pub fn check_weight_change(
        &self,
        old_weights: &HashMap<String, Vec<f32>>,
        new_weights: &HashMap<String, Vec<f32>>,
    ) -> Result<(), NeoTrixError> {
        for (name, new_w) in new_weights {
            let old_w = old_weights.get(name);
            if let Some(ow) = old_w {
                if new_w.len() != ow.len() {
                    return Err(NeoTrixError::InvalidInput(format!(
                        "Shape mismatch for {}",
                        name
                    )));
                }

                let mut max_diff = 0.0f32;
                for (_i, (on, nw)) in ow.iter().zip(new_w.iter()).enumerate() {
                    let diff = (nw - on).abs();
                    if diff > max_diff {
                        max_diff = diff;
                    }
                }

                if max_diff > self.config.max_weight_change {
                    return Err(NeoTrixError::SafetyViolation(format!(
                        "Weight change for {} exceeds threshold: {} > {}",
                        name, max_diff, self.config.max_weight_change
                    )));
                }
            }
        }
        Ok(())
    }

    /// 风险评估
    pub fn assess_risk(&self, weights: &HashMap<String, Vec<f32>>) -> f32 {
        // 简化的风险评估：基于权重范数的综合评分
        let mut total_risk = 0.0f32;
        for w in weights.values() {
            let norm: f32 = w.iter().map(|x| x * x).sum::<f32>().sqrt();
            total_risk += norm;
        }
        if !weights.is_empty() {
            total_risk / weights.len() as f32
        } else {
            0.0
        }
    }

    pub fn config(&self) -> &SafetyConfig {
        &self.config
    }
}

impl CapabilityNode for SafetyCore {
    fn node_id(&self) -> &str {
        "nt_mind::safety::safety_core"
    }
    fn provides(&self) -> Vec<String> {
        vec![
            "safety_check".into(),
            "risk_assessment".into(),
            "constraint_verification".into(),
        ]
    }
    fn requires(&self) -> Vec<String> {
        vec!["peft_suite".into(), "config".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Obsidian, RuneSocket::Alabaster]
    }
    fn constellation_level(&self) -> u8 {
        1
    }
    fn promote_constellation(&mut self) -> bool {
        true
    }
}

impl SelfTest for SafetyCore {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let safety = SafetyCore::new(SafetyConfig::default());
            let mut old = HashMap::new();
            old.insert("layer1".into(), vec![1.0, 2.0, 3.0]);
            let mut new = HashMap::new();
            new.insert("layer1".into(), vec![1.05, 2.0, 3.0]);

            safety.check_weight_change(&old, &new)?;
            let risk = safety.assess_risk(&new);
            assert!(risk >= 0.0);
            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_mind_safety_core"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_safety_core_self_test() {
        let safety = SafetyCore::new(SafetyConfig::default());
        assert!(safety.self_test().is_ok());
    }
}
