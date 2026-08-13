//! nt_act::rcm::counterfactual — 反事实验证 (根因确认)
//!
//! 节点: nt_act::rcm::counterfactual (L1)
//! Provides: counterfactual_check, root_cause_confirmation

#![forbid(unsafe_code)]

use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Verdict {
    Confirmed,
    Refuted,
}

/// 反事实验证 — 检验"若修复根因, 症状是否消失"
#[derive(Debug, Clone, Default)]
pub struct CounterfactualCheck;

impl CounterfactualCheck {
    pub fn new() -> Self {
        Self
    }

    /// simulate(fixed_root): 传入修复后是否无复现; true → 确认根因
    pub fn evaluate(&self, fixed_root: bool) -> Verdict {
        if fixed_root {
            Verdict::Confirmed
        } else {
            Verdict::Refuted
        }
    }

    /// 组合判定: 根因候选 + 修复后症状消失 → 确认; 否则需回溯新候选
    pub fn judge(&self, root_cause_present: bool, symptom_gone_after_fix: bool) -> Verdict {
        if root_cause_present && symptom_gone_after_fix {
            Verdict::Confirmed
        } else {
            Verdict::Refuted
        }
    }
}

impl CapabilityNode for CounterfactualCheck {
    fn node_id(&self) -> &str {
        "nt_act::rcm::counterfactual"
    }
    fn provides(&self) -> Vec<String> {
        vec![
            "counterfactual_check".into(),
            "root_cause_confirmation".into(),
        ]
    }
    fn requires(&self) -> Vec<String> {
        vec!["root_cause_method".into()]
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

impl SelfTest for CounterfactualCheck {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let c = CounterfactualCheck::new();
        assert_eq!(c.evaluate(true), Verdict::Confirmed, "修复后无复现 → 确认");
        assert_eq!(c.evaluate(false), Verdict::Refuted, "修复后仍复现 → 推翻");
        assert_eq!(c.judge(true, true), Verdict::Confirmed);
        assert_eq!(c.judge(true, false), Verdict::Refuted);
        Ok(())
    }

    fn name(&self) -> &str {
        "nt_act_rcm_counterfactual"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_when_fixed() {
        let c = CounterfactualCheck::new();
        assert_eq!(c.evaluate(true), Verdict::Confirmed);
    }

    #[test]
    fn test_refute_when_not_fixed() {
        let c = CounterfactualCheck::new();
        assert_eq!(c.evaluate(false), Verdict::Refuted);
    }

    #[test]
    fn test_judge_requires_both() {
        let c = CounterfactualCheck::new();
        assert_eq!(c.judge(true, true), Verdict::Confirmed);
        assert_eq!(c.judge(false, true), Verdict::Refuted);
        assert_eq!(c.judge(true, false), Verdict::Refuted);
    }

    #[test]
    fn test_verdict_eq() {
        assert_eq!(Verdict::Confirmed, Verdict::Confirmed);
        assert_ne!(Verdict::Confirmed, Verdict::Refuted);
    }
}
