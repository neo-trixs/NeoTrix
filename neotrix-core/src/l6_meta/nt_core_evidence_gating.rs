//! T2.9: 证据门控模块 (Hermes C1)
//!
//! EvidenceGating: 基于历史结果的代理信任评分 + 门控

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Outcome
// ============================================================================

/// 代理执行结果记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub task_id: String,
    pub success: bool,
    pub quality_score: f64,
    pub timestamp: i64, // epoch seconds
    pub cost: f64,
}

// ============================================================================
// OutcomeTracker
// ============================================================================

/// 结果追踪器
pub struct OutcomeTracker {
    outcomes: HashMap<String, Vec<Outcome>>,
    max_history: usize,
}

impl OutcomeTracker {
    pub fn new(max_history: usize) -> Self {
        Self {
            outcomes: HashMap::new(),
            max_history,
        }
    }

    pub fn record(&mut self, agent_id: &str, outcome: Outcome) {
        let history = self.outcomes.entry(agent_id.to_string()).or_default();
        history.push(outcome);
        if history.len() > self.max_history {
            history.remove(0);
        }
    }

    pub fn history(&self, agent_id: &str) -> &[Outcome] {
        self.outcomes.get(agent_id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn agents(&self) -> Vec<&str> {
        self.outcomes.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for OutcomeTracker {
    fn default() -> Self {
        Self::new(100)
    }
}

// ============================================================================
// TrustCalibrator
// ============================================================================

/// 信任校准器
#[derive(Debug, Clone)]
pub struct TrustCalibrator {
    /// 基础信任分数
    pub base_trust: f64,
    /// 衰减因子 (失败时乘以)
    pub decay_factor: f64,
    /// 恢复速率 (成功时增加)
    pub recovery_rate: f64,
    /// 门控阈值
    pub gate_threshold: f64,
    /// 代理当前信任分
    trust_scores: HashMap<String, f64>,
}

impl TrustCalibrator {
    pub fn new() -> Self {
        Self {
            base_trust: 0.5,
            decay_factor: 0.8,
            recovery_rate: 0.1,
            gate_threshold: 0.3,
            trust_scores: HashMap::new(),
        }
    }

    /// 根据结果更新信任分数
    pub fn update(&mut self, agent_id: &str, outcome: &Outcome) -> f64 {
        let current = self
            .trust_scores
            .get(agent_id)
            .copied()
            .unwrap_or(self.base_trust);

        let new_trust = if outcome.success {
            (current + self.recovery_rate * outcome.quality_score).min(1.0)
        } else {
            (current * self.decay_factor).max(0.0)
        };

        self.trust_scores.insert(agent_id.to_string(), new_trust);
        new_trust
    }

    /// 获取当前信任分
    pub fn trust(&self, agent_id: &str) -> f64 {
        self.trust_scores
            .get(agent_id)
            .copied()
            .unwrap_or(self.base_trust)
    }

    /// 门控: 信任分数低于阈值时阻止
    pub fn gate(&self, agent_id: &str) -> bool {
        self.trust(agent_id) > self.gate_threshold
    }

    /// 所有代理信任分
    pub fn all_trusts(&self) -> &HashMap<String, f64> {
        &self.trust_scores
    }
}

impl Default for TrustCalibrator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// EvidenceGating
// ============================================================================

/// 证据门控器 — 组合 OutcomeTracker + TrustCalibrator
pub struct EvidenceGating {
    pub outcome_tracker: OutcomeTracker,
    pub trust_calibrator: TrustCalibrator,
}

impl EvidenceGating {
    pub fn new() -> Self {
        Self {
            outcome_tracker: OutcomeTracker::default(),
            trust_calibrator: TrustCalibrator::default(),
        }
    }

    /// 记录结果并更新信任
    pub fn record_and_update(&mut self, agent_id: &str, outcome: Outcome) -> f64 {
        self.outcome_tracker.record(agent_id, outcome.clone());
        self.trust_calibrator.update(agent_id, &outcome)
    }

    /// 门控检查
    pub fn can_execute(&self, agent_id: &str) -> bool {
        self.trust_calibrator.gate(agent_id)
    }

    /// 获取信任分
    pub fn trust(&self, agent_id: &str) -> f64 {
        self.trust_calibrator.trust(agent_id)
    }
}

impl Default for EvidenceGating {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_outcome(success: bool, quality: f64) -> Outcome {
        Outcome {
            task_id: "t1".into(),
            success,
            quality_score: quality,
            timestamp: 0,
            cost: 0.01,
        }
    }

    #[test]
    fn test_trust_update_success() {
        let mut cal = TrustCalibrator::new();
        let trust = cal.update("a", &make_outcome(true, 0.8));
        assert!(trust > cal.base_trust);
    }

    #[test]
    fn test_trust_update_failure() {
        let mut cal = TrustCalibrator::new();
        let trust = cal.update("a", &make_outcome(false, 0.0));
        assert!(trust < cal.base_trust);
    }

    #[test]
    fn test_gate_blocks() {
        let mut cal = TrustCalibrator::new();
        cal.gate_threshold = 0.3;
        // 持续失败 → 信任降低
        for _ in 0..20 {
            cal.update("a", &make_outcome(false, 0.0));
        }
        assert!(!cal.gate("a"));
    }

    #[test]
    fn test_gate_passes() {
        let mut cal = TrustCalibrator::new();
        cal.update("a", &make_outcome(true, 0.9));
        assert!(cal.gate("a"));
    }

    #[test]
    fn test_evidence_gating_integration() {
        let mut eg = EvidenceGating::new();
        let trust = eg.record_and_update("agent1", make_outcome(true, 0.7));
        assert!(trust > 0.5);
        assert!(eg.can_execute("agent1"));

        // 10 failures
        for _ in 0..10 {
            eg.record_and_update("agent1", make_outcome(false, 0.0));
        }
        assert!(!eg.can_execute("agent1"));
    }

    #[test]
    fn test_outcome_tracker_history() {
        let mut tracker = OutcomeTracker::new(3);
        tracker.record("a", make_outcome(true, 0.5));
        tracker.record("a", make_outcome(false, 0.0));
        tracker.record("a", make_outcome(true, 0.8));
        tracker.record("a", make_outcome(true, 0.9)); // 应淘汰第 1 条
        assert_eq!(tracker.history("a").len(), 3);
    }
}
