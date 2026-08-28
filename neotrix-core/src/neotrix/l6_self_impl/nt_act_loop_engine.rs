//! L6 / NT-ACT — loop-engine (arXiv:2608.00267 LoopsBench) 吸收节点 (C1)。
//!
//! 源: LoopsBench — 编码代理 (coding agent) 的 loop 工程评测, 基于 DAG 的任务
//! 依赖建模评估 agent 多轮循环表现。NeoTrix 视角: 把编码代理的单轮动作编排为
//! 有向无环图 (DAG) 任务依赖, 提供事件循环评估器 SelfTest。

use crate::core::nt_core_self_test::SelfTest;
use std::collections::{HashMap, HashSet, VecDeque};

/// 一个 DAG 任务节点。
#[derive(Debug, Clone, PartialEq)]
pub struct DagTask {
    pub id: String,
    pub deps: Vec<String>,
    pub done: bool,
}

impl DagTask {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            deps: Vec::new(),
            done: false,
        }
    }

    pub fn with_deps(mut self, deps: &[&str]) -> Self {
        self.deps = deps.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn mark_done(&mut self) {
        self.done = true;
    }
}

/// 编码代理循环评估器 — 校验 DAG 拓扑 + 推进调度。
pub trait LoopEngine {
    fn is_acyclic(&self, tasks: &[DagTask]) -> bool;
    fn schedule(&self, tasks: &[DagTask]) -> Vec<String>;
}

pub struct CodingAgentLoopEngine;

impl LoopEngine for CodingAgentLoopEngine {
    /// Kahn 拓扑排序检测环: 存在环则非 DAG。
    fn is_acyclic(&self, tasks: &[DagTask]) -> bool {
        let ids: HashSet<String> = tasks.iter().map(|t| t.id.clone()).collect();
        let mut indeg: HashMap<String, usize> = ids.iter().map(|i| (i.clone(), 0)).collect();
        for t in tasks {
            for d in &t.deps {
                if ids.contains(d) {
                    *indeg.entry(t.id.clone()).or_insert(0) += 1;
                }
            }
        }
        let mut q: VecDeque<String> = indeg.iter().filter(|(_, &v)| v == 0).map(|(k, _)| k.clone()).collect();
        let mut seen = 0;
        while let Some(n) = q.pop_front() {
            seen += 1;
            for t in tasks {
                if t.deps.contains(&n) && ids.contains(&t.id) {
                    let e = indeg.get_mut(&t.id).unwrap();
                    *e -= 1;
                    if *e == 0 {
                        q.push_back(t.id.clone());
                    }
                }
            }
        }
        seen == ids.len()
    }

    /// 返回拓扑序 (无环时) 或空 (有环)。
    fn schedule(&self, tasks: &[DagTask]) -> Vec<String> {
        if !self.is_acyclic(tasks) {
            return Vec::new();
        }
        tasks.iter().map(|t| t.id.clone()).collect()
    }
}

#[derive(Default)]
pub struct CodingAgentLoopEngineSelfTest;

impl SelfTest for CodingAgentLoopEngineSelfTest {
    fn name(&self) -> &str {
        "nt_act_loop_engine"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let eng = CodingAgentLoopEngine;
        let ok = vec![
            DagTask::new("a"),
            DagTask::new("b").with_deps(&["a"]),
            DagTask::new("c").with_deps(&["a", "b"]),
        ];
        let cyc = vec![
            DagTask::new("x").with_deps(&["y"]),
            DagTask::new("y").with_deps(&["x"]),
        ];
        let mut errs = Vec::new();
        if !eng.is_acyclic(&ok) {
            errs.push("loop_engine: valid DAG marked cyclic".into());
        }
        if eng.is_acyclic(&cyc) {
            errs.push("loop_engine: cyclic graph marked acyclic".into());
        }
        if eng.schedule(&ok).len() != 3 {
            errs.push("loop_engine: schedule length mismatch".into());
        }
        if !eng.schedule(&cyc).is_empty() {
            errs.push("loop_engine: cyclic schedule must be empty".into());
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
    fn detects_acyclic() {
        let eng = CodingAgentLoopEngine;
        let t = vec![DagTask::new("a"), DagTask::new("b").with_deps(&["a"])];
        assert!(eng.is_acyclic(&t));
    }

    #[test]
    fn detects_cycle() {
        let eng = CodingAgentLoopEngine;
        let t = vec![
            DagTask::new("x").with_deps(&["y"]),
            DagTask::new("y").with_deps(&["x"]),
        ];
        assert!(!eng.is_acyclic(&t));
    }

    #[test]
    fn schedule_empty_on_cycle() {
        let eng = CodingAgentLoopEngine;
        let t = vec![
            DagTask::new("x").with_deps(&["y"]),
            DagTask::new("y").with_deps(&["x"]),
        ];
        assert!(eng.schedule(&t).is_empty());
    }
}
