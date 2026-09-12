//! # Value SelfModel (价值函数自我模型)
//!
//! **用途**: 价值评估模型 — "我重视什么？"
//!
//! **注意**: 这是 neotrix-core 中三个 SelfModel 之一:
//! - `nt_core_meta::SelfModel` — 静态结构身份 (模块/文件/依赖)
//! - `nt_core_self::SelfModel` — 动态性能模型 (能力/不确定性/疲劳)
//! - `nt_core_self_model::SelfModel` (本文件) — 价值函数模型
//!
//! NT-CORE SELF-MODEL (L5/L6 自我模型)
//!
//! 最小可编译骨架 (C0→C1)：持有持久化自我状态（身份、目标、价值权重），
//! 提供 `value_function` 评估某个动作/状态相对价值权重的契合度，并暴露
//! `update` 钩子供 SEAL 自改进闭环在产出候选行为变更时调用。
//!
//! 真实价值启发式（如 FEP 自由能最小化 / IIT Φ 一致性）留待后续迭代，
//! 本文件仅落地数据结构与接线点 (TODO 标记)。
//!
//! 铁律: `#![forbid(unsafe_code)]` (R-P1) — 零 unsafe。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::core::nt_core_error::{NeoTrixError, NeoTrixResult};

/// 单一价值维度的权重（自我模型据此评估动作/状态）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValueWeight {
    /// 维度名（如 "coherence" / "safety" / "growth"）。
    pub dimension: String,
    /// 权重 [0,1]，所有权重之和约定为 1.0（归一化在 `value_function` 中保证）。
    pub weight: f64,
}

/// 持久化自我状态 (NT-CORE L5/L6)。
///
/// 当前为最小骨架：身份标识、目标集合、价值权重表。
/// 真实字段（叙事、一致性约束、长期偏好）后续扩展。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModel {
    /// 系统身份标识（与 `nt_core_self::system_identity::SystemIdentity` 对齐的薄引用）。
    pub identity: String,
    /// 当前自我目标集合（候选行为变更会相对这些目标被评估）。
    pub goals: Vec<String>,
    /// 价值权重表，驱动 `value_function` 的线性启发式。
    pub value_weights: Vec<ValueWeight>,
    /// 迭代计数（SEAL `update` 调用次数，用于单调性/衰减）。
    pub revision: u64,
}

impl Default for SelfModel {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfModel {
    /// 构造默认自我模型：中性身份、空目标、三项等权价值维度。
    pub fn new() -> Self {
        Self {
            identity: "neotrix-core".to_string(),
            goals: Vec::new(),
            value_weights: vec![
                ValueWeight {
                    dimension: "coherence".to_string(),
                    weight: 1.0 / 3.0,
                },
                ValueWeight {
                    dimension: "safety".to_string(),
                    weight: 1.0 / 3.0,
                },
                ValueWeight {
                    dimension: "growth".to_string(),
                    weight: 1.0 / 3.0,
                },
            ],
            revision: 0,
        }
    }

    /// 将自我模型序列化为 JSON 串（持久化到 KB / 配置）。
    pub fn to_json(&self) -> NeoTrixResult<String> {
        serde_json::to_string(self).map_err(|e| NeoTrixError::Serde(e.to_string()))
    }

    /// 从 JSON 串恢复自我模型。
    pub fn from_json(s: &str) -> NeoTrixResult<Self> {
        serde_json::from_str(s).map_err(|e| NeoTrixError::Serde(e.to_string()))
    }

    /// 价值函数：评估某个动作/状态相对自我价值权重的契合度。
    ///
    /// 将 action 文本投影到各价值维度得分，按 value_weights 加权求和。
    /// 当前实现使用关键词匹配做简单投影；未来可结合 FEP/IIT 信号。
    pub fn value_function(&self, action: &str) -> f64 {
        if action.is_empty() {
            return 0.0;
        }
        let action_lower = action.to_lowercase();

        // 关键词投影：每个价值维度用关键词匹配计算得分
        let dimension_scores: Vec<f64> = self.value_weights.iter().map(|vw| {
            let keywords = match vw.dimension.as_str() {
                "coherence" => vec!["consistent", "aligned", "unified", "coherent", "integrate"],
                "safety" => vec!["safe", "secure", "protect", "guard", "defend", "harden"],
                "growth" => vec!["learn", "evolve", "improve", "grow", "adapt", "optimize"],
                "efficiency" => vec!["fast", "efficient", "optimize", "cache", "batch", "parallel"],
                "autonomy" => vec!["autonomous", "self", "independent", "decide", "choose"],
                _ => vec![], // 未知维度得 0 分
            };

            // 匹配关键词数量 / 总关键词数 = 维度得分
            let matches = keywords.iter()
                .filter(|kw| action_lower.contains(*kw))
                .count();
            if keywords.is_empty() {
                0.5 // 未知维度给中性分
            } else {
                (matches as f64) / (keywords.len() as f64)
            }
        }).collect();

        // 加权求和
        let total_weight: f64 = self.value_weights.iter().map(|w| w.weight).sum();
        if total_weight <= 0.0 {
            return 0.5; // 无权重时返回中性分
        }

        let weighted_sum: f64 = dimension_scores.iter()
            .zip(self.value_weights.iter())
            .map(|(score, vw)| score * vw.weight)
            .sum();

        (weighted_sum / total_weight).clamp(0.0, 1.0)
    }

    /// SEAL 钩子：候选行为变更产出后更新自我模型。
    ///
    /// 当前仅递增 `revision` 并记录目标占位；真实实现应据候选回写
    /// 价值权重 / 长期偏好（受 `nt_core_self_constitution` 治理约束）。
    pub fn update(&mut self, candidate: &str) -> NeoTrixResult<()> {
        if candidate.is_empty() {
            return Err(NeoTrixError::InvalidInput(
                "self_model.update: empty candidate".to_string(),
            ));
        }
        self.revision += 1;
        Ok(())
    }
}

/// 价值函数模型类型别名，消除三个 SelfModel 之间的歧义
pub type ValueFunctionModel = SelfModel;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_model_value_function_bounded() {
        let m = SelfModel::new();
        assert!((0.0..=1.0).contains(&m.value_function("do something")));
        assert_eq!(m.value_function(""), 0.0);
    }

    #[test]
    fn self_model_update_increments_revision() {
        let mut m = SelfModel::new();
        let before = m.revision;
        m.update("candidate edit").unwrap();
        assert_eq!(m.revision, before + 1);
        assert!(m.update("").is_err());
    }

    #[test]
    fn self_model_json_roundtrip() {
        let m = SelfModel::new();
        let s = m.to_json().unwrap();
        let m2 = SelfModel::from_json(&s).unwrap();
        assert_eq!(m.identity, m2.identity);
    }
}
