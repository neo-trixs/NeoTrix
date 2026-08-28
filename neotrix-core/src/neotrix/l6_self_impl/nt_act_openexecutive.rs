//! L6 / NT-ACT — openexecutive (github.com/SenteLabsAI/OpenExecutive) 吸收节点 (C1)。
//!
//! 源: OpenExecutive — 执行智能体, 接收高层目标后把任务分解为可委派子任务,
//! 派发给执行单元并汇总结果。NeoTrix 视角: 执行智能体 = 任务委派/执行协调器
//! (delegator), 负责拆分 (decompose) → 委派 (delegate) → 汇总 (reconcile)。
//! 本节点提供无副作用的委派 stub。

use crate::core::nt_core_self_test::SelfTest;

/// 一个已委派 (或待委派) 的子任务。
#[derive(Debug, Clone)]
pub struct DelegatedTask {
    pub id: String,
    pub owner: String,
    pub done: bool,
}

/// 执行智能体委派运行时 (stub)。
pub trait ExecutiveDelegator {
    /// 把高层目标拆为 `n` 个并列子任务。
    fn decompose(&self, goal: &str, n: usize) -> Vec<DelegatedTask>;
    /// 汇总子任务结果: 完成的占比 (0.0..=1.0)。
    fn reconcile(&self, tasks: &[DelegatedTask]) -> f32;
}

/// 纯计算委派器 (无副作用)。
pub struct OpenExecutiveAgent;

impl ExecutiveDelegator for OpenExecutiveAgent {
    fn decompose(&self, goal: &str, n: usize) -> Vec<DelegatedTask> {
        if goal.is_empty() || n == 0 {
            return Vec::new();
        }
        (0..n)
            .map(|i| DelegatedTask {
                id: format!("{goal}#{i}"),
                owner: format!("worker-{i}"),
                done: false,
            })
            .collect()
    }

    fn reconcile(&self, tasks: &[DelegatedTask]) -> f32 {
        if tasks.is_empty() {
            return 0.0;
        }
        let done = tasks.iter().filter(|t| t.done).count();
        done as f32 / tasks.len() as f32
    }
}

/// 纯函数: 汇总完成率。
pub fn reconcile_progress(tasks: &[DelegatedTask]) -> f32 {
    OpenExecutiveAgent.reconcile(tasks)
}

#[derive(Default)]
pub struct OpenExecutiveSelfTest;

impl SelfTest for OpenExecutiveSelfTest {
    fn name(&self) -> &str {
        "nt_act_openexecutive"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let ag = OpenExecutiveAgent;
        let mut errs = Vec::new();
        let tasks = ag.decompose("ship", 3);
        if tasks.len() != 3 {
            errs.push("openexecutive: decompose must yield 3 tasks".into());
        }
        if !tasks.iter().all(|t| t.owner.starts_with("worker-")) {
            errs.push("openexecutive: each task must have an owner".into());
        }
        if (ag.reconcile(&tasks) - 0.0).abs() > 1e-6 {
            errs.push("openexecutive: fresh tasks must be 0% done".into());
        }
        let mut done = tasks.clone();
        done[0].done = true;
        done[1].done = true;
        if (ag.reconcile(&done) - 2.0 / 3.0).abs() > 1e-6 {
            errs.push("openexecutive: 2/3 done must reconcile to 0.667".into());
        }
        if ag.decompose("", 3).len() != 0 {
            errs.push("openexecutive: empty goal must decompose to nothing".into());
        }
        if (reconcile_progress(&[]) - 0.0).abs() > 1e-6 {
            errs.push("openexecutive: empty reconcile must be 0.0".into());
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

    #[test]
    fn decompose_assigns_owners() {
        let ag = OpenExecutiveAgent;
        let tasks = ag.decompose("g", 2);
        assert_eq!(tasks.len(), 2);
        assert!(tasks[0].owner.starts_with("worker-"));
    }

    #[test]
    fn reconcile_ratio() {
        let ag = OpenExecutiveAgent;
        let mut tasks = ag.decompose("g", 4);
        tasks[0].done = true;
        tasks[2].done = true;
        assert_eq!(ag.reconcile(&tasks), 0.5);
    }

    #[test]
    fn empty_goal_noop() {
        let ag = OpenExecutiveAgent;
        assert!(ag.decompose("", 4).is_empty());
        assert_eq!(ag.reconcile(&[]), 0.0);
    }
}
