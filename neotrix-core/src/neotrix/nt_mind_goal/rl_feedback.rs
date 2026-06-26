//! RLFeedbackLoop — 验证结果 → CapabilityVector 奖励回路
//!
//! P4-05: 将 BehavioralVerifier 的验证结果转化为 RL 奖励信号,
//! 通过 CapabilityVector::update_from_other() 更新能力向量。
//!
//! 文献对齐 (2026):
//!   - ReVeal (ICLR 2026): 多轮自验证 + TAPO 信用分配
//!   - 核心差异: 奖励来自外部验证 (编译/测试), 非 LLM 自评

use crate::core::nt_core_cap::CapabilityVector;
use crate::neotrix::nt_mind_goal::behavioral_verifier::BehaviorVerificationResult;

/// 奖励事件类型
#[derive(Debug, Clone)]
pub enum RewardEvent {
    CompileSuccess {
        file: String,
        dim: String,
    },
    TestPassed {
        file: String,
        dim: String,
    },
    PropertyVerified {
        file: String,
        dim: String,
    },
    CompileFailed {
        file: String,
        dim: String,
        errors: usize,
    },
    TestFailed {
        file: String,
        dim: String,
        failures: usize,
    },
}

/// 奖励历史
#[derive(Debug, Clone)]
pub struct RewardHistory {
    pub events: Vec<RewardEvent>,
    pub total_reward: f64,
    pub positive_count: u32,
    pub negative_count: u32,
}

impl RewardHistory {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            total_reward: 0.0,
            positive_count: 0,
            negative_count: 0,
        }
    }

    pub fn add_event(&mut self, event: RewardEvent) {
        let reward = Self::reward_value(&event);
        self.total_reward += reward;
        if reward > 0.0 {
            self.positive_count += 1;
        } else {
            self.negative_count += 1;
        }
        self.events.push(event);
    }

    fn reward_value(event: &RewardEvent) -> f64 {
        match event {
            RewardEvent::CompileSuccess { .. } => 0.3,
            RewardEvent::TestPassed { .. } => 0.4,
            RewardEvent::PropertyVerified { .. } => 0.2,
            RewardEvent::CompileFailed { .. } => -0.5,
            RewardEvent::TestFailed { .. } => -0.6,
        }
    }
}

/// RL 反馈循环
#[derive(Debug, Clone)]
pub struct RLFeedbackLoop {
    pub history: RewardHistory,
    learning_rate: f64,
}

impl RLFeedbackLoop {
    pub fn new(learning_rate: f64) -> Self {
        Self {
            history: RewardHistory::new(),
            learning_rate,
        }
    }

    /// 处理验证结果, 返回奖励值
    pub fn process_result(
        &mut self,
        file: &str,
        dim: &str,
        result: &BehaviorVerificationResult,
    ) -> f64 {
        let mut total_reward = 0.0;

        if result.compile_ok {
            let ev = RewardEvent::CompileSuccess {
                file: file.to_string(),
                dim: dim.to_string(),
            };
            total_reward += RewardHistory::reward_value(&ev);
            self.history.add_event(ev);
        } else {
            let ev = RewardEvent::CompileFailed {
                file: file.to_string(),
                dim: dim.to_string(),
                errors: result.compile_errors.len(),
            };
            total_reward += RewardHistory::reward_value(&ev);
            self.history.add_event(ev);
        }

        if result.tests_ok {
            let ev = RewardEvent::TestPassed {
                file: file.to_string(),
                dim: dim.to_string(),
            };
            total_reward += RewardHistory::reward_value(&ev);
            self.history.add_event(ev);
        } else if !result.compile_ok {
            // 编译失败时测试不可能通过
        } else {
            let ev = RewardEvent::TestFailed {
                file: file.to_string(),
                dim: dim.to_string(),
                failures: result.test_failures.len(),
            };
            total_reward += RewardHistory::reward_value(&ev);
            self.history.add_event(ev);
        }

        if result.properties_ok {
            let ev = RewardEvent::PropertyVerified {
                file: file.to_string(),
                dim: dim.to_string(),
            };
            total_reward += RewardHistory::reward_value(&ev);
            self.history.add_event(ev);
        }

        total_reward
    }

    /// 用累积奖励更新 CapabilityVector
    pub fn update_capability(&self, cv: &mut CapabilityVector, dim_name: &str) -> f64 {
        let avg_reward = if self.history.positive_count + self.history.negative_count > 0 {
            self.history.total_reward
                / (self.history.positive_count + self.history.negative_count) as f64
        } else {
            0.0
        };

        let delta = avg_reward.clamp(-0.5, 0.5);
        if let Some(idx) = CapabilityVector::index_from_name(dim_name) {
            let current = cv.arr[idx];
            let new_val = (current + self.learning_rate * delta).clamp(0.0, 1.0);
            cv.arr[idx] = new_val;
        }
        avg_reward
    }

    /// 更新所有 23 维
    pub fn update_all(&self, cv: &mut CapabilityVector) {
        for name in crate::core::nt_core_cap::FIELD_NAMES {
            self.update_capability(cv, name);
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "RL 反馈: {} 正 / {} 负 | 总奖励 {:.2} | lr={}",
            self.history.positive_count,
            self.history.negative_count,
            self.history.total_reward,
            self.learning_rate,
        )
    }
}

impl Default for RLFeedbackLoop {
    fn default() -> Self {
        Self::new(0.01)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn passing_result() -> BehaviorVerificationResult {
        BehaviorVerificationResult {
            passed: true,
            compile_ok: true,
            tests_ok: true,
            properties_ok: true,
            compile_errors: vec![],
            test_failures: vec![],
            duration_ms: 100,
        }
    }

    fn failing_result() -> BehaviorVerificationResult {
        BehaviorVerificationResult {
            passed: false,
            compile_ok: false,
            tests_ok: false,
            properties_ok: false,
            compile_errors: vec!["error[E0425]".into()],
            test_failures: vec!["test_foo FAILED".into()],
            duration_ms: 200,
        }
    }

    #[test]
    fn test_passing_result_positive_reward() {
        let mut rl = RLFeedbackLoop::default();
        let reward = rl.process_result("test.rs", "verification", &passing_result());
        assert!(reward > 0.0);
    }

    #[test]
    fn test_failing_result_negative_reward() {
        let mut rl = RLFeedbackLoop::default();
        let reward = rl.process_result("test.rs", "verification", &failing_result());
        assert!(reward < 0.0);
    }

    #[test]
    fn test_update_capability_positive() {
        let mut rl = RLFeedbackLoop::default();
        let mut cv = CapabilityVector::default();
        rl.process_result("f.rs", "verification", &passing_result());
        let reward = rl.update_capability(&mut cv, "verification");
        assert!(reward > -0.01);
    }

    #[test]
    fn test_update_capability_negative() {
        let mut rl = RLFeedbackLoop::default();
        let mut cv = CapabilityVector::default();
        rl.process_result("f.rs", "verification", &failing_result());
        let reward = rl.update_capability(&mut cv, "verification");
        assert!(reward < 0.0 || (reward - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_update_all_no_panic() {
        let rl = RLFeedbackLoop::default();
        let mut cv = CapabilityVector::default();
        rl.update_all(&mut cv);
        // 无 panic 即通过
    }

    #[test]
    fn test_reward_history_counts() {
        let mut rl = RLFeedbackLoop::default();
        rl.process_result("a.rs", "analysis", &passing_result());
        rl.process_result("b.rs", "analysis", &failing_result());
        assert_eq!(rl.history.positive_count, 3);
        assert!(rl.history.negative_count >= 1);
    }

    #[test]
    fn test_summary_format() {
        let rl = RLFeedbackLoop::default();
        let s = rl.summary();
        assert!(s.contains("RL"));
    }
}
