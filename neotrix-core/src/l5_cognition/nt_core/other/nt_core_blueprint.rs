//! NT-CORE — blueprint 吸收 (github.com/haskaomni/blueprint).
//!
//! blueprint: 蓝图/规划 (blueprint/planning) — 从高层目标生成可执行的分层
//! 执行蓝图。本模块实现 `_BlueprintPlanner` trait (C1: trait 存在 + 基础逻辑
//! + SelfTest T1 + 3 测试), 建模目标 → 阶段 → 步骤的蓝图展开。

use crate::core::nt_core_self_test::SelfTest;
use std::collections::VecDeque;

/// 蓝图中的单个执行步骤。
#[derive(Debug, Clone, PartialEq)]
pub struct _BlueprintStep {
    pub phase: usize,
    pub action: String,
}

/// blueprint 规划实现。
pub struct _BlueprintPlannerImpl {
    goal: String,
    steps: VecDeque<_BlueprintStep>,
}

impl _BlueprintPlannerImpl {
    pub fn new() -> Self {
        Self {
            goal: String::new(),
            steps: VecDeque::new(),
        }
    }

    pub fn goal(&self) -> &str {
        &self.goal
    }

    pub fn plan(&mut self, goal: &str, phases: usize) {
        self.goal = goal.to_string();
        self.steps.clear();
        if goal.is_empty() {
            return;
        }
        for i in 0..phases {
            self.steps.push_back(_BlueprintStep {
                phase: i,
                action: format!("phase_{}: execute toward '{}'", i, goal),
            });
        }
    }

    pub fn steps(&self) -> Vec<_BlueprintStep> {
        self.steps.iter().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for _BlueprintPlannerImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfTest for _BlueprintPlannerImpl {
    fn name(&self) -> &'static str {
        "_BlueprintPlanner"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        if !self.goal.is_empty() && self.steps.is_empty() {
            return Err(vec!["goal set but blueprint has no steps".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_generates_phased_steps() {
        let mut p = _BlueprintPlannerImpl::new();
        p.plan("ship feature", 3);
        assert_eq!(p.len(), 3);
        assert_eq!(p.goal(), "ship feature");
        let steps = p.steps();
        assert_eq!(steps[0].phase, 0);
        assert_eq!(steps[2].phase, 2);
    }

    #[test]
    fn test_empty_goal_yields_empty_blueprint() {
        let mut p = _BlueprintPlannerImpl::new();
        p.plan("", 2);
        assert!(p.is_empty());
        assert!(p.self_test().is_ok());
    }

    #[test]
    fn test_self_test_flags_goal_without_steps() {
        let mut p = _BlueprintPlannerImpl::new();
        p.plan("orphan goal", 0);
        assert!(p.self_test().is_err());
    }
}
