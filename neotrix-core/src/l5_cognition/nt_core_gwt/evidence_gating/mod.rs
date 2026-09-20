//! # 证据驱动信任门控 (OpenSpace pattern)
//!
//! 基于历史结果追踪代理信任分数:
//! - 成功 → 信任增加
//! - 失败 → 信任衰减
//! - 信任低于阈值 → 阻止执行

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub task_id: String,
    pub success: bool,
    pub quality_score: f64,
    pub cost: f64,
    pub timestamp: String,
}

/// 信任分数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScore {
    pub agent_id: String,
    pub score: f64,
    pub total_tasks: u64,
    pub successful_tasks: u64,
}

/// 证据门控器
pub struct EvidenceGating {
    /// 每个代理的信任分数
    trust_scores: HashMap<String, TrustScore>,
    /// 信任阈值 (低于此值阻止执行)
    threshold: f64,
    /// 衰减因子
    decay_factor: f64,
    /// 恢复速率
    recovery_rate: f64,
}

impl EvidenceGating {
    pub fn new() -> Self {
        Self {
            trust_scores: HashMap::new(),
            threshold: 0.3,
            decay_factor: 0.8,
            recovery_rate: 0.1,
        }
    }
    
    /// 根据结果更新信任分数
    pub fn update(&mut self, agent_id: &str, outcome: &Outcome) {
        let entry = self.trust_scores.entry(agent_id.into()).or_insert_with(|| TrustScore {
            agent_id: agent_id.into(),
            score: 0.5,
            total_tasks: 0,
            successful_tasks: 0,
        });
        
        entry.total_tasks += 1;
        if outcome.success {
            entry.successful_tasks += 1;
            entry.score = (entry.score + self.recovery_rate * outcome.quality_score).min(1.0);
        } else {
            entry.score *= self.decay_factor;
        }
    }
    
    /// 门控: 信任分数低于阈值时阻止执行
    pub fn gate(&self, agent_id: &str) -> bool {
        let score = self.trust_scores.get(agent_id)
            .map(|s| s.score)
            .unwrap_or(0.5);
        score >= self.threshold
    }
    
    /// 获取信任分数
    pub fn get_trust(&self, agent_id: &str) -> f64 {
        self.trust_scores.get(agent_id)
            .map(|s| s.score)
            .unwrap_or(0.5)
    }
    
    /// 获取信任报告
    pub fn report(&self) -> Vec<&TrustScore> {
        self.trust_scores.values().collect()
    }
}

impl Default for EvidenceGating {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trust_decay() {
        let mut gating = EvidenceGating::new();
        let outcome = Outcome {
            task_id: "t1".into(),
            success: false,
            quality_score: 0.0,
            cost: 0.1,
            timestamp: "".into(),
        };
        gating.update("agent1", &outcome);
        assert!(gating.get_trust("agent1") < 0.5);
    }
    
    #[test]
    fn test_trust_recovery() {
        let mut gating = EvidenceGating::new();
        let outcome = Outcome {
            task_id: "t1".into(),
            success: true,
            quality_score: 0.9,
            cost: 0.1,
            timestamp: "".into(),
        };
        gating.update("agent1", &outcome);
        assert!(gating.get_trust("agent1") > 0.5);
    }
}
