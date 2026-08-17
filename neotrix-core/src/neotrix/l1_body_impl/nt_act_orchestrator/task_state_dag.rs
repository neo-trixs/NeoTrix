//! task_state_dag — 依赖感知任务 DAG + 严格状态机 + 领取门禁 + attempt 能力令牌
//!
//! dsh-agent-teams 吸收 (evidence: notes/absorption-20260817-dsh-agent-teams.md):
//!   - P2 (L34-38): 任务带显式 `dependencies`; 仅当全部依赖 reach completed 才可
//!     claim; 严格状态迁移表 `pending → claimed → in_progress → completed`
//!     (终态不可变)。
//!   - P4 (L50-54): 每次执行代际 = 单调 attempt + 唯一 attemptId 能力令牌; 更新必须
//!     携带当前 attempt_id, 过期即拒绝 ("stop work"); 交接 = 静默所有权转移
//!     (不重新认领, 迟到结果无法覆盖新属主)。
//!
//! R-P42: 复用 state_graph::ArtifactNode / ArtifactState / ArtifactType / DagEdge,
//!        不建平行任务模型; ArtifactState 已扩展出 Claimed 状态 (state_graph.rs)。

use super::state_graph::{ArtifactNode, ArtifactState, ArtifactType, DagEdge};
use std::collections::HashMap;

/// 每次认领铸造的能力令牌。attempt_id 单调递增; 属主持令牌才可推进/提交。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptToken {
    pub task_id: String,
    pub attempt_id: String,
    pub attempt_seq: u64,
    pub owner: String,
}

impl AttemptToken {
    pub fn new(task_id: impl Into<String>, attempt_id: impl Into<String>, attempt_seq: u64, owner: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            attempt_id: attempt_id.into(),
            attempt_seq,
            owner: owner.into(),
        }
    }
}

/// 严格状态迁移表。`Done` 为唯一终态 (不可变); 反向 revert 仅限 attempt
/// 失效/回收路径 (release), 不属于正向生命周期。
fn legal_transition(from: &ArtifactState, to: &ArtifactState) -> bool {
    use ArtifactState::*;
    match (from, to) {
        (Pending, Ready) | (Pending, Blocked(_)) => true,
        (Blocked(_), Ready) => true,
        (Ready, Claimed) => true,
        (Claimed, InProgress) => true,
        (InProgress, Done) => true,
        // attempt 失效/回收: 认领撤销后任务回到可认领池
        (Claimed, Ready) | (InProgress, Ready) => true,
        // 幂等 no-op
        (a, b) if a == b => true,
        _ => false,
    }
}

/// 依赖感知任务 DAG: 逐任务状态机 + 依赖门禁领取 + attempt 能力令牌。
#[derive(Debug, Clone)]
pub struct TaskStateDag {
    nodes: HashMap<String, ArtifactNode>,
    edges: Vec<DagEdge>,
    attempts: HashMap<String, AttemptToken>,
    next_seq: u64,
}

impl Default for TaskStateDag {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskStateDag {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            attempts: HashMap::new(),
            next_seq: 0,
        }
    }

    pub fn add_task(&mut self, id: &str, description: &str) {
        self.nodes.entry(id.to_string()).or_insert_with(|| {
            ArtifactNode::new(id, ArtifactType::Task, description)
        });
    }

    pub fn add_task_node(&mut self, node: ArtifactNode) {
        self.nodes.entry(node.id.clone()).or_insert(node);
    }

    /// 声明 task -> dep 依赖边 (task 需等 dep 完成后才可认领)。
    pub fn add_dependency(&mut self, task: &str, dep: &str) {
        if self.nodes.contains_key(task) && self.nodes.contains_key(dep) && task != dep {
            self.edges.push(DagEdge { from: dep.to_string(), to: task.to_string() });
        }
    }

    pub fn node(&self, id: &str) -> Option<&ArtifactNode> {
        self.nodes.get(id)
    }

    pub fn state(&self, id: &str) -> Option<&ArtifactState> {
        self.nodes.get(id).map(|n| &n.state)
    }

    /// 直接前置依赖 (incoming edges)。
    pub fn dependencies(&self, id: &str) -> Vec<String> {
        self.edges.iter().filter(|e| e.to == id).map(|e| e.from.clone()).collect()
    }

    /// 直接后继 (outgoing edges)。
    pub fn dependents(&self, id: &str) -> Vec<String> {
        self.edges.iter().filter(|e| e.from == id).map(|e| e.to.clone()).collect()
    }

    /// 依赖门禁: 所有直接依赖 reach Done。
    pub fn deps_satisfied(&self, id: &str) -> bool {
        self.dependencies(id).iter().all(|d| {
            self.state(d).map(|s| matches!(s, ArtifactState::Done)).unwrap_or(false)
        })
    }

    /// 严格状态迁移; 非法跳转返回 Err (证据 P2: 严格状态迁移表)。
    pub fn transition(&mut self, id: &str, to: ArtifactState) -> Result<(), String> {
        let node = self.nodes.get_mut(id)
            .ok_or_else(|| format!("task '{}' not found", id))?;
        if !legal_transition(&node.state, &to) {
            return Err(format!(
                "illegal transition for '{}': {:?} -> {:?}",
                id, node.state, to
            ));
        }
        node.state = to;
        Ok(())
    }

    /// 重算资格: Pending/Blocked 任务按依赖满足度置 Ready / Blocked。
    /// 进行中 (Claimed/InProgress) 与终态 (Done) 任务跳过。
    pub fn recompute_eligibility(&mut self) {
        let ids: Vec<String> = self.nodes.keys().cloned().collect();
        for id in ids {
            let state = self.state(&id).cloned().unwrap_or(ArtifactState::Pending);
            if matches!(state, ArtifactState::Done | ArtifactState::Claimed | ArtifactState::InProgress) {
                continue;
            }
            let target = if self.deps_satisfied(&id) {
                ArtifactState::Ready
            } else {
                let unmet: Vec<String> = self.dependencies(&id).into_iter()
                    .filter(|d| self.state(d).map(|s| !matches!(s, ArtifactState::Done)).unwrap_or(true))
                    .collect();
                ArtifactState::Blocked(format!("unmet deps: {}", unmet.join(", ")))
            };
            let same_kind = match (&state, &target) {
                (ArtifactState::Blocked(_), ArtifactState::Blocked(_)) => true,
                (a, b) => a == b,
            };
            if !same_kind {
                let _ = self.transition(&id, target);
            }
        }
    }

    /// 当前可认领任务 (Ready 且无活跃 attempt), 确定性排序。
    pub fn eligible_for_claim(&self) -> Vec<String> {
        let mut out: Vec<String> = self.nodes.values()
            .filter(|n| matches!(n.state, ArtifactState::Ready))
            .map(|n| n.id.clone())
            .filter(|id| !self.attempts.contains_key(id))
            .collect();
        out.sort();
        out
    }

    /// 依赖门禁认领: 仅当全部依赖 Done; 认领铸造新 attempt 能力令牌
    /// (证据 P4: 单调 attempt + 唯一 attemptId)。
    pub fn claim(&mut self, id: &str, owner: &str) -> Result<AttemptToken, String> {
        if !self.nodes.contains_key(id) {
            return Err(format!("task '{}' not found", id));
        }
        if self.attempts.contains_key(id) {
            return Err(format!("task '{}' already claimed", id));
        }
        if !self.deps_satisfied(id) {
            let unmet: Vec<String> = self.dependencies(id).into_iter()
                .filter(|d| self.state(d).map(|s| !matches!(s, ArtifactState::Done)).unwrap_or(true))
                .collect();
            return Err(format!("claim blocked for '{}': unmet dependencies {:?}", id, unmet));
        }
        let state = self.state(id).cloned().unwrap_or(ArtifactState::Pending);
        if matches!(state, ArtifactState::Done) {
            return Err(format!("task '{}' already done", id));
        }
        // 依赖已满足但仍在 Pending/Blocked → 先合法推进到 Ready
        if matches!(state, ArtifactState::Pending | ArtifactState::Blocked(_)) {
            self.transition(id, ArtifactState::Ready)?;
        }
        self.transition(id, ArtifactState::Claimed)?;
        self.next_seq += 1;
        let token = AttemptToken::new(
            id,
            format!("{}#{}", id, self.next_seq),
            self.next_seq,
            owner,
        );
        self.attempts.insert(id.to_string(), token.clone());
        Ok(token)
    }

    fn verify_token(&self, id: &str, token: &AttemptToken) -> Result<(), String> {
        let current = self.attempts.get(id)
            .ok_or_else(|| format!("no active attempt for '{}'", id))?;
        if current.attempt_id != token.attempt_id || current.attempt_seq != token.attempt_seq {
            return Err(format!(
                "stale attempt token rejected for '{}' (current {}, got {})",
                id, current.attempt_id, token.attempt_id
            ));
        }
        if current.owner != token.owner {
            return Err(format!(
                "attempt '{}' owned by '{}', token presented by '{}'",
                id, current.owner, token.owner
            ));
        }
        Ok(())
    }

    /// 开始执行: Claimed -> InProgress (需当前令牌; 对应 P2 的 claimed → in_progress)。
    pub fn start(&mut self, id: &str, token: &AttemptToken) -> Result<(), String> {
        self.verify_token(id, token)?;
        self.transition(id, ArtifactState::InProgress)?;
        Ok(())
    }

    /// 提交结果: InProgress -> Done (终态); 迟到/过期令牌被拒 (证据 P4 L51)。
    /// 完成后解锁后继任务。
    pub fn submit_result(&mut self, id: &str, token: &AttemptToken) -> Result<(), String> {
        self.verify_token(id, token)?;
        self.transition(id, ArtifactState::Done)?;
        self.attempts.remove(id);
        self.unlock_dependents(id);
        Ok(())
    }

    /// 撤销 attempt (失效/转派/冷恢复): 清能力令牌, Claimed/InProgress 回退 Ready。
    pub fn release(&mut self, id: &str, token: &AttemptToken) -> Result<(), String> {
        self.verify_token(id, token)?;
        self.attempts.remove(id);
        let state = self.state(id).cloned().unwrap_or(ArtifactState::Pending);
        if matches!(state, ArtifactState::Claimed | ArtifactState::InProgress) {
            self.transition(id, ArtifactState::Ready)?;
        }
        Ok(())
    }

    /// 静默交接: 所有权转移, 不重新认领、不换代际 (证据 P4 L51 "交接协议")。
    /// 返回携带新属主的能力令牌; 旧令牌即刻失效。
    pub fn handoff(&mut self, id: &str, current: &AttemptToken, new_owner: &str) -> Result<AttemptToken, String> {
        self.verify_token(id, current)?;
        let state = self.state(id).cloned().unwrap_or(ArtifactState::Pending);
        if matches!(state, ArtifactState::Done) {
            return Err(format!("cannot handoff completed task '{}'", id));
        }
        let new_token = AttemptToken::new(
            id,
            current.attempt_id.clone(),
            current.attempt_seq,
            new_owner,
        );
        self.attempts.insert(id.to_string(), new_token.clone());
        Ok(new_token)
    }

    pub fn active_attempt(&self, id: &str) -> Option<&AttemptToken> {
        self.attempts.get(id)
    }

    /// 任务完成后, 下游 Pending/Blocked 依赖满足即置 Ready (证据 P2: 图变更解锁)。
    pub fn unlock_dependents(&mut self, id: &str) {
        for dep_id in self.dependents(id) {
            let state = self.state(&dep_id).cloned().unwrap_or(ArtifactState::Pending);
            if matches!(state, ArtifactState::Pending | ArtifactState::Blocked(_))
                && self.deps_satisfied(&dep_id)
            {
                let _ = self.transition(&dep_id, ArtifactState::Ready);
            }
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn done_count(&self) -> usize {
        self.nodes.values().filter(|n| n.state == ArtifactState::Done).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// (a) 依赖门禁: 未满足依赖时 claim 被阻断, 依赖全部 Done 后方可认领。
    #[test]
    fn dep_gate_blocks_claim_until_deps_done() {
        let mut dag = TaskStateDag::new();
        dag.add_task("a", "task a");
        dag.add_task("b", "task b");
        dag.add_task("c", "task c");
        dag.add_dependency("b", "a");
        dag.add_dependency("c", "a");
        dag.add_dependency("c", "b");
        dag.recompute_eligibility();

        assert!(dag.claim("c", "m1").is_err(), "c must be gated by unmet deps");
        assert!(dag.claim("b", "m1").is_err(), "b must be gated by unmet dep a");

        let ta = dag.claim("a", "m1").expect("a has no deps");
        dag.start("a", &ta).unwrap();
        dag.submit_result("a", &ta).unwrap();

        assert!(dag.claim("c", "m1").is_err(), "c still gated by b");
        let tb = dag.claim("b", "m2").expect("b unlocked after a done");
        dag.start("b", &tb).unwrap();
        dag.submit_result("b", &tb).unwrap();

        let tc = dag.claim("c", "m3").expect("c unlocked after a and b done");
        assert!(tc.owner == "m3");
    }

    /// (b) 严格状态迁移: 非法跳转一律拒绝; Done 为不可变终态。
    #[test]
    fn strict_transition_rejects_illegal_jumps() {
        let mut dag = TaskStateDag::new();
        dag.add_task("t", "task t");

        assert!(dag.transition("t", ArtifactState::Done).is_err(), "Pending -> Done illegal");
        assert!(dag.transition("t", ArtifactState::InProgress).is_err(), "Pending -> InProgress illegal");

        dag.transition("t", ArtifactState::Ready).unwrap();
        assert!(dag.transition("t", ArtifactState::InProgress).is_err(), "Ready -> InProgress illegal");

        dag.transition("t", ArtifactState::Claimed).unwrap();
        assert!(dag.transition("t", ArtifactState::Done).is_err(), "Claimed -> Done illegal");

        dag.transition("t", ArtifactState::InProgress).unwrap();
        dag.transition("t", ArtifactState::Done).unwrap();

        assert!(dag.transition("t", ArtifactState::Pending).is_err(), "Done is immutable");
        assert!(dag.transition("t", ArtifactState::InProgress).is_err(), "Done is immutable");
    }

    /// (c) 迟到/过期 attempt 令牌被拒 (release 后新一代令牌接管)。
    #[test]
    fn stale_attempt_token_rejected() {
        let mut dag = TaskStateDag::new();
        dag.add_task("t", "task t");

        let tok1 = dag.claim("t", "m1").unwrap();
        dag.release("t", &tok1).unwrap();

        let tok2 = dag.claim("t", "m2").unwrap();
        assert!(tok2.attempt_seq > tok1.attempt_seq, "attempt must be monotonic");

        assert!(dag.start("t", &tok1).is_err(), "stale token must be rejected on start");
        assert!(dag.submit_result("t", &tok1).is_err(), "stale token must be rejected on submit");

        dag.start("t", &tok2).unwrap();
        dag.submit_result("t", &tok2).unwrap();
        assert!(dag.submit_result("t", &tok2).is_err(), "completed task has no active attempt");
    }

    /// (d) 静默交接: 所有权转移不换代际, 旧令牌失效, 新属主持令牌完成。
    #[test]
    fn handoff_transfers_ownership_silently() {
        let mut dag = TaskStateDag::new();
        dag.add_task("t", "task t");

        let tok1 = dag.claim("t", "m1").unwrap();
        dag.start("t", &tok1).unwrap();

        let tok2 = dag.handoff("t", &tok1, "m2").unwrap();
        assert_eq!(tok2.owner, "m2");
        assert_eq!(tok2.attempt_id, tok1.attempt_id, "handoff keeps attempt identity");
        assert_eq!(tok2.attempt_seq, tok1.attempt_seq, "handoff is not a re-claim");
        assert_eq!(dag.active_attempt("t").unwrap().owner, "m2");

        assert!(dag.submit_result("t", &tok1).is_err(), "old owner token is stale after handoff");
        dag.submit_result("t", &tok2).unwrap();
        assert_eq!(dag.state("t").unwrap(), &ArtifactState::Done);
    }

    #[test]
    fn completion_unlocks_dependents() {
        let mut dag = TaskStateDag::new();
        dag.add_task("a", "a");
        dag.add_task("b", "b");
        dag.add_dependency("b", "a");
        dag.recompute_eligibility();
        assert!(matches!(dag.state("b").unwrap(), ArtifactState::Blocked(_)));

        let ta = dag.claim("a", "m1").unwrap();
        dag.start("a", &ta).unwrap();
        dag.submit_result("a", &ta).unwrap();

        assert!(matches!(dag.state("b").unwrap(), ArtifactState::Ready));
        assert_eq!(dag.done_count(), 1);
    }

    #[test]
    fn eligible_list_excludes_claimed_and_done() {
        let mut dag = TaskStateDag::new();
        dag.add_task("a", "a");
        dag.add_task("b", "b");
        dag.recompute_eligibility();

        let el = dag.eligible_for_claim();
        assert_eq!(el, vec!["a".to_string(), "b".to_string()]);

        let ta = dag.claim("a", "m1").unwrap();
        dag.start("a", &ta).unwrap();
        dag.submit_result("a", &ta).unwrap();

        assert_eq!(dag.eligible_for_claim(), vec!["b".to_string()]);
        let _ = dag.claim("b", "m1").unwrap();
        assert!(dag.eligible_for_claim().is_empty());
    }
}
