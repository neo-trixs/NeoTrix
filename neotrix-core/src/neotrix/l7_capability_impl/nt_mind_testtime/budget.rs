//! nt_mind::testtime::budget — 计算预算分配 (防无限思考)
//!
//! 节点: nt_mind::testtime::budget (L1)
//! Provides: budget_allocation, reasoning_guard

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};

/// 推理预算 — 限制 test-time 计算的思考轮数/令牌, 防 OOM 与无限循环
#[derive(Debug, Clone)]
pub struct ReasonBudget {
    max_rounds: u64,
    used_rounds: u64,
}

impl ReasonBudget {
    pub fn new(max_rounds: u64) -> Self {
        Self {
            max_rounds,
            used_rounds: 0,
        }
    }

    /// 申请一轮思考预算
    pub fn spend_round(&mut self) -> Result<u64, NeoTrixError> {
        if self.used_rounds >= self.max_rounds {
            return Err(NeoTrixError::InvalidState(format!(
                "推理预算耗尽 (max {})",
                self.max_rounds
            )));
        }
        self.used_rounds += 1;
        Ok(self.used_rounds)
    }

    pub fn remaining(&self) -> u64 {
        self.max_rounds.saturating_sub(self.used_rounds)
    }

    pub fn used(&self) -> u64 {
        self.used_rounds
    }

    pub fn reset(&mut self) {
        self.used_rounds = 0;
    }
}

impl CapabilityNode for ReasonBudget {
    fn node_id(&self) -> &str {
        "nt_mind::testtime::budget"
    }
    fn provides(&self) -> Vec<String> {
        vec!["budget_allocation".into(), "reasoning_guard".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["testtime_reasoning".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Obsidian, RuneSocket::Golden]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for ReasonBudget {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut b = ReasonBudget::new(3);
        assert_eq!(b.spend_round().map_err(|e| e.to_string()).unwrap(), 1);
        assert_eq!(b.spend_round().map_err(|e| e.to_string()).unwrap(), 2);
        assert_eq!(b.spend_round().map_err(|e| e.to_string()).unwrap(), 3);
        assert!(b.spend_round().is_err(), "第 4 轮应被拒绝");
        assert_eq!(b.remaining(), 0);
        b.reset();
        assert_eq!(b.remaining(), 3);
        Ok(())
    }

    fn name(&self) -> &str {
        "nt_mind_testtime_budget"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_flow() {
        let mut b = ReasonBudget::new(2);
        assert_eq!(b.used(), 0);
        b.spend_round().unwrap();
        b.spend_round().unwrap();
        assert_eq!(b.used(), 2);
        assert!(b.spend_round().is_err());
    }

    #[test]
    fn test_reset_budget() {
        let mut b = ReasonBudget::new(4);
        b.spend_round().unwrap();
        b.spend_round().unwrap();
        assert_eq!(b.remaining(), 2);
        b.reset();
        assert_eq!(b.remaining(), 4);
        assert_eq!(b.used(), 0);
    }

    #[test]
    fn test_zero_budget() {
        let mut b = ReasonBudget::new(0);
        assert!(b.spend_round().is_err());
    }

    #[test]
    fn test_remaining_reports_correctly() {
        let mut b = ReasonBudget::new(5);
        for _ in 0..3 {
            b.spend_round().unwrap();
        }
        assert_eq!(b.remaining(), 2);
    }
}
