//! nt_act::harness::orchestrator — 回合预算 + 编排控制
//!
//! 节点: nt_act::harness::orchestrator (L2)
//! Provides: turn_budgeting, orchestration_control

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};

/// 回合编排器 — 限制循环/失控 agent (防跑飞), 对齐 Dark Forest 生存公理
#[derive(Debug, Clone)]
pub struct TurnOrchestrator {
    max_turns: u64,
    used: u64,
}

impl TurnOrchestrator {
    pub fn new(max_turns: u64) -> Self {
        Self { max_turns, used: 0 }
    }

    /// 消耗一个回合配额
    pub fn consume(&mut self) -> Result<u64, NeoTrixError> {
        if self.used >= self.max_turns {
            return Err(NeoTrixError::InvalidState(format!(
                "回合预算耗尽 (max {})",
                self.max_turns
            )));
        }
        self.used += 1;
        Ok(self.used)
    }

    pub fn remaining(&self) -> u64 {
        self.max_turns.saturating_sub(self.used)
    }

    pub fn used(&self) -> u64 {
        self.used
    }

    pub fn reset(&mut self) {
        self.used = 0;
    }
}

impl CapabilityNode for TurnOrchestrator {
    fn node_id(&self) -> &str {
        "nt_act::harness::orchestrator"
    }
    fn provides(&self) -> Vec<String> {
        vec!["turn_budgeting".into(), "orchestration_control".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["agent_harness".into(), "tool_delegation".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Golden, RuneSocket::Alabaster]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for TurnOrchestrator {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut o = TurnOrchestrator::new(3);
        for i in 1..=3 {
            assert_eq!(o.consume().map_err(|e| e.to_string()).unwrap(), i);
        }
        assert!(o.consume().is_err(), "第 4 次应被预算拒绝");
        assert_eq!(o.remaining(), 0);
        o.reset();
        assert_eq!(o.remaining(), 3);
        Ok(())
    }

    fn name(&self) -> &str {
        "nt_act_harness_orchestrator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_consumption() {
        let mut o = TurnOrchestrator::new(2);
        assert_eq!(o.remaining(), 2);
        o.consume().unwrap();
        o.consume().unwrap();
        assert_eq!(o.remaining(), 0);
        assert!(o.consume().is_err());
    }

    #[test]
    fn test_reset() {
        let mut o = TurnOrchestrator::new(5);
        for _ in 0..3 {
            o.consume().unwrap();
        }
        assert_eq!(o.used(), 3);
        o.reset();
        assert_eq!(o.used(), 0);
        assert_eq!(o.remaining(), 5);
    }

    #[test]
    fn test_zero_budget() {
        let mut o = TurnOrchestrator::new(0);
        assert!(o.consume().is_err());
    }
}
