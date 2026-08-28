//! L6 / NT-ACT — agentiker-plan-follow (github.com/IVRZ-da/agentiker-plan-follow)
//! 吸收节点 (C1)。
//!
//! 源: agentiker-plan-follow — Agent 规划管线贯彻: 把高层计划逐层落实为可执行
//! 步骤, 并持久化步骤状态以保证 follow-through (贯彻)。NeoTrix 视角: 规划步骤
//! 管线 + 完成跟踪 trait。

use crate::core::nt_core_self_test::SelfTest;
use std::collections::HashMap;

/// 规划步骤状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Done,
}

/// 一个规划步骤。
#[derive(Debug, Clone)]
pub struct PlanStep {
    pub id: String,
    pub status: StepStatus,
}

/// 规划贯彻器 — 推进步骤直到全部完成。
pub trait PlanFollower {
    fn advance(&self, steps: &mut HashMap<String, PlanStep>, id: &str);
    fn completion(&self, steps: &HashMap<String, PlanStep>) -> f32;
}

pub struct AgentikerPlanFollower;

impl PlanFollower for AgentikerPlanFollower {
    fn advance(&self, steps: &mut HashMap<String, PlanStep>, id: &str) {
        if let Some(s) = steps.get_mut(id) {
            s.status = StepStatus::Done;
        }
    }

    fn completion(&self, steps: &HashMap<String, PlanStep>) -> f32 {
        if steps.is_empty() {
            return 0.0;
        }
        let done = steps.values().filter(|s| s.status == StepStatus::Done).count();
        done as f32 / steps.len() as f32
    }
}

#[derive(Default)]
pub struct AgentikerPlanFollowerSelfTest;

impl SelfTest for AgentikerPlanFollowerSelfTest {
    fn name(&self) -> &str {
        "nt_act_agentiker_plan"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let f = AgentikerPlanFollower;
        let mut steps = HashMap::new();
        steps.insert(
            "a".into(),
            PlanStep {
                id: "a".into(),
                status: StepStatus::Pending,
            },
        );
        steps.insert(
            "b".into(),
            PlanStep {
                id: "b".into(),
                status: StepStatus::Pending,
            },
        );
        let mut errs = Vec::new();
        if (f.completion(&steps) - 0.0).abs() > 1e-6 {
            errs.push("plan_follow: empty completion must be 0".into());
        }
        f.advance(&mut steps, "a");
        if (f.completion(&steps) - 0.5).abs() > 1e-6 {
            errs.push("plan_follow: 1/2 completion must be 0.5".into());
        }
        f.advance(&mut steps, "b");
        if (f.completion(&steps) - 1.0).abs() > 1e-6 {
            errs.push("plan_follow: full completion must be 1.0".into());
        }
        f.advance(&mut steps, "ghost");
        if (f.completion(&steps) - 1.0).abs() > 1e-6 {
            errs.push("plan_follow: unknown step must not change completion".into());
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk() -> HashMap<String, PlanStep> {
        let mut m = HashMap::new();
        m.insert("a".into(), PlanStep { id: "a".into(), status: StepStatus::Pending });
        m.insert("b".into(), PlanStep { id: "b".into(), status: StepStatus::Pending });
        m
    }

    #[test]
    fn advance_marks_done() {
        let f = AgentikerPlanFollower;
        let mut s = mk();
        f.advance(&mut s, "a");
        assert_eq!(s.get("a").unwrap().status, StepStatus::Done);
    }

    #[test]
    fn completion_ratio() {
        let f = AgentikerPlanFollower;
        let mut s = mk();
        f.advance(&mut s, "a");
        assert_eq!(f.completion(&s), 0.5);
    }

    #[test]
    fn unknown_step_noop() {
        let f = AgentikerPlanFollower;
        let mut s = mk();
        f.advance(&mut s, "x");
        assert_eq!(f.completion(&s), 0.0);
    }
}
