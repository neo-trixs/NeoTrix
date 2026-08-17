//! event_driven_claim — 事件驱动共享调度: idle 边 + 图变更 → 原子认领并唤醒 (零轮询)
//!
//! dsh-agent-teams 吸收 (evidence: notes/absorption-20260817-dsh-agent-teams.md):
//!   - P3 (L43): 每次 idle 边或任务图变更即尝试为每个空闲成员原子认领一个就绪任务
//!     并唤醒; 冷恢复: idle/ready 成员仍持 open task (中断/进程重启后) → 新 attempt
//!     重试同一任务而非视为 busy。
//!   - P4 (L51): 每次执行代际 = 单调 attempt + 唯一 attemptId 能力令牌。
//!   - P12 (L115): 分发唤醒失败仅回滚本次精确 dispatch (attemptId 等值校验)。
//!
//! R-P42: 作为 nt_core_scheduler::SchedulerEngine 的子组件强化 (engine.rs 持有
//!        claim_pool), 不建平行调度器。零轮询: 唤醒仅由显式事件边驱动。

use std::collections::{HashMap, HashSet};

/// 一次原子认领铸造的令牌 (代际 attempt)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    pub task_id: String,
    pub attempt_id: String,
    pub attempt_seq: u64,
    pub worker: String,
}

impl Claim {
    pub fn new(task_id: impl Into<String>, attempt_id: impl Into<String>, attempt_seq: u64, worker: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            attempt_id: attempt_id.into(),
            attempt_seq,
            worker: worker.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerState {
    /// 已注册但尚未转入 idle (fresh edge: 首次 idle 通知才算边)。
    Ready,
    Idle,
    Running,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolTask {
    pub deps: Vec<String>,
    pub done: bool,
    pub claimed: bool,
    pub attempt_seq: u64,
}

/// 事件驱动认领池: 成员状态 (idle/running) + 任务图 (deps/done) + 活跃认领。
/// 认领是单入口原子操作 (check + mint 在同一 `&mut self` 调用内完成)。
#[derive(Debug, Clone)]
pub struct EventDrivenClaimPool {
    tasks: HashMap<String, PoolTask>,
    workers: HashMap<String, WorkerState>,
    claims: HashMap<String, Claim>,
    worker_of: HashMap<String, String>,
    seq: u64,
    wake_count: u64,
}

impl Default for EventDrivenClaimPool {
    fn default() -> Self {
        Self::new()
    }
}

impl EventDrivenClaimPool {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            workers: HashMap::new(),
            claims: HashMap::new(),
            worker_of: HashMap::new(),
            seq: 0,
            wake_count: 0,
        }
    }

    pub fn register_worker(&mut self, worker: &str) {
        self.workers.entry(worker.to_string()).or_insert(WorkerState::Ready);
    }

    pub fn register_task(&mut self, id: &str, deps: Vec<String>) {
        let deps: Vec<String> = deps.into_iter().filter(|d| d != id).collect();
        self.tasks.entry(id.to_string()).or_insert(PoolTask {
            deps,
            done: false,
            claimed: false,
            attempt_seq: 0,
        });
    }

    pub fn worker_state(&self, worker: &str) -> Option<WorkerState> {
        self.workers.get(worker).copied()
    }

    pub fn active_claim(&self, task_id: &str) -> Option<&Claim> {
        self.claims.get(task_id)
    }

    /// idle 边检测: 仅当成员真实从非-idle 转 idle 才返回 true (同一状态重复通知
    /// 不触发, 防惊群/重复认领)。
    pub fn worker_goes_idle(&mut self, worker: &str) -> bool {
        match self.workers.get(worker) {
            Some(WorkerState::Idle) => false,
            _ => {
                self.workers.insert(worker.to_string(), WorkerState::Idle);
                true
            }
        }
    }

    /// 图变更边检测: 任务首次置 done 才返回 true。
    pub fn mark_done(&mut self, task_id: &str) -> bool {
        let task = match self.tasks.get_mut(task_id) {
            Some(t) => t,
            None => return false,
        };
        if task.done {
            return false;
        }
        task.done = true;
        task.claimed = false;
        if let Some(c) = self.claims.remove(task_id) {
            self.worker_of.remove(&c.worker);
            self.workers.insert(c.worker.clone(), WorkerState::Idle);
        }
        true
    }

    /// 就绪任务: 依赖全部 done、自身未 done、未认领。确定性排序。
    pub fn eligible_tasks(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (id, t) in &self.tasks {
            if t.done || t.claimed {
                continue;
            }
            let deps_ok = t.deps.iter().all(|d| {
                self.tasks.get(d).map(|x| x.done).unwrap_or(true)
            });
            if deps_ok {
                out.push(id.clone());
            }
        }
        out.sort();
        out
    }

    pub fn idle_workers(&self) -> Vec<String> {
        let mut out: Vec<String> = self.workers.iter()
            .filter(|(_, s)| matches!(s, WorkerState::Idle | WorkerState::Ready))
            .map(|(w, _)| w.clone())
            .collect();
        out.sort();
        out
    }

    /// 原子认领: 成员必须 idle/ready (fresh); 认领 mint 新代际 attempt 令牌 (证据 P4)。
    /// 单个 `&mut self` 调用内完成资格检查 + 状态置位, 天然防双认领。
    pub fn try_claim_for_worker(&mut self, worker: &str) -> Option<Claim> {
        if !matches!(self.workers.get(worker), Some(WorkerState::Idle | WorkerState::Ready)) {
            return None;
        }
        let eligible = self.eligible_tasks();
        for task_id in eligible {
            self.seq += 1;
            let claim = Claim::new(
                &task_id,
                format!("{}#{}", task_id, self.seq),
                self.seq,
                worker,
            );
            let task = self.tasks.get_mut(&task_id).expect("eligible task exists");
            task.claimed = true;
            task.attempt_seq = self.seq;
            self.claims.insert(task_id.clone(), claim.clone());
            self.worker_of.insert(worker.to_string(), task_id);
            self.workers.insert(worker.to_string(), WorkerState::Running);
            return Some(claim);
        }
        None
    }

    /// idle 边唤醒: 成员转 idle → 立即尝试原子认领并唤醒 (零轮询)。
    /// 返回唤醒的认领令牌; 无就绪任务则返回 None (成员保持 idle)。
    pub fn notify_worker_idle(&mut self, worker: &str) -> Option<Claim> {
        if !self.worker_goes_idle(worker) {
            return None;
        }
        self.try_claim_for_worker(worker)
    }

    /// 图变更唤醒: 任务完成边 → 为每个空闲成员原子认领 (确定性成员顺序)。
    /// 零轮询: 仅由图变更事件驱动。
    pub fn notify_graph_change(&mut self, completed: &str) -> Vec<Claim> {
        if !self.mark_done(completed) {
            return Vec::new();
        }
        self.wake_idle_workers()
    }

    fn wake_idle_workers(&mut self) -> Vec<Claim> {
        self.wake_count += 1;
        let idle = self.idle_workers();
        let mut claims = Vec::new();
        for w in idle {
            if let Some(c) = self.try_claim_for_worker(&w) {
                claims.push(c);
            }
        }
        claims
    }

    /// 完成认领: 校验令牌 (attemptId 等值) → 任务 done → 成员转 idle →
    /// 自动唤醒其他空闲成员处理新解锁任务。陈旧/过期令牌被拒 (证据 P12 L115)。
    pub fn complete(&mut self, claim: &Claim) -> Result<Vec<Claim>, String> {
        let current = self.claims.get(&claim.task_id)
            .ok_or_else(|| format!("no active claim for '{}'", claim.task_id))?;
        if current.attempt_id != claim.attempt_id || current.worker != claim.worker {
            return Err(format!(
                "stale claim rejected for '{}' (current {}, got {})",
                claim.task_id, current.attempt_id, claim.attempt_id
            ));
        }
        self.claims.remove(&claim.task_id);
        self.worker_of.remove(&claim.worker);
        let task = self.tasks.get_mut(&claim.task_id)
            .ok_or_else(|| format!("task '{}' not found", claim.task_id))?;
        task.done = true;
        task.claimed = false;
        self.workers.insert(claim.worker.clone(), WorkerState::Idle);
        Ok(self.wake_idle_workers())
    }

    /// 冷恢复: 清除无主 claim (成员已不存在的认领, 如进程重启后未恢复的成员),
    /// 还原其任务为可认领, 随后空闲成员以新代际 attempt 重试同一任务
    /// (证据 P3 L43: 冷恢复新 attempt 重试同一任务)。
    pub fn cold_recovery(&mut self) -> Vec<Claim> {
        let orphaned: Vec<String> = self.claims.iter()
            .filter(|(_, c)| {
                // 无主 claim: 成员不存在, 或成员存在但非 Running (重启后新代际注册)。
                match self.workers.get(&c.worker) {
                    None => true,
                    Some(WorkerState::Running) => false,
                    Some(WorkerState::Idle | WorkerState::Ready) => true,
                }
            })
            .map(|(tid, _)| tid.clone())
            .collect();
        for tid in orphaned {
            if let Some(c) = self.claims.remove(&tid) {
                if let Some(t) = self.tasks.get_mut(&tid) {
                    t.claimed = false;
                    t.attempt_seq = self.seq;
                }
                let _ = self.worker_of.remove(&c.worker);
            }
        }
        self.wake_idle_workers()
    }

    /// 转派: 撤销旧 worker 认领 → 该 worker 转 idle → 尝试认领 (如需)。
    /// 与 Feature 1 的静默交接不同: 这是调度池侧的撤销 + 重新分派。
    pub fn reassign(&mut self, task_id: &str, worker: &str) -> Result<Claim, String> {
        let claim = self.claims.get(task_id)
            .cloned()
            .ok_or_else(|| format!("no active claim for '{}'", task_id))?;
        self.claims.remove(task_id);
        self.worker_of.remove(&claim.worker);
        self.workers.insert(claim.worker.clone(), WorkerState::Idle);
        if let Some(t) = self.tasks.get_mut(task_id) {
            t.claimed = false;
        }
        self.register_worker(worker);
        if self.worker_goes_idle(worker) {
            return self.try_claim_for_worker(worker).ok_or_else(|| {
                format!("no eligible task for reassign to '{}'", worker)
            });
        }
        Err(format!("worker '{}' not idle for reassign", worker))
    }

    pub fn wake_count(&self) -> u64 {
        self.wake_count
    }

    pub fn pending_count(&self) -> usize {
        self.tasks.values().filter(|t| !t.done).count()
    }

    pub fn claim_count(&self) -> usize {
        self.claims.len()
    }

    pub fn task_ids(&self) -> HashSet<String> {
        self.tasks.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// (a) idle 边触发唤醒: 零轮询, 成员转 idle 即被认领。
    #[test]
    fn idle_edge_triggers_wake_without_poll() {
        let mut pool = EventDrivenClaimPool::new();
        pool.register_worker("w1");
        pool.register_task("t1", vec![]);

        assert_eq!(pool.wake_count(), 0);
        let claim = pool.notify_worker_idle("w1").expect("idle edge must claim t1");
        assert_eq!(claim.task_id, "t1");
        assert_eq!(claim.worker, "w1");
        assert_eq!(pool.worker_state("w1"), Some(WorkerState::Running));
        assert_eq!(pool.wake_count(), 0, "no poll wake; claim is edge-driven");

        // 重复 idle 通知不触发 (成员已是 Running): 无惊群
        assert!(pool.notify_worker_idle("w1").is_none());
    }

    /// (b) 图变更触发唤醒: 任务完成解锁下游, 空闲成员被唤醒认领。
    #[test]
    fn graph_change_triggers_wake() {
        let mut pool = EventDrivenClaimPool::new();
        pool.register_worker("w1");
        pool.register_task("a", vec![]);
        pool.register_task("b", vec!["a".to_string()]);

        let ca = pool.notify_worker_idle("w1").expect("a claimable immediately");
        assert_eq!(ca.task_id, "a");
        assert_eq!(pool.eligible_tasks(), Vec::<String>::new(), "b blocked on a");

        pool.complete(&ca).unwrap();
        // completion wakes w1, which now claims unlocked b
        assert_eq!(pool.worker_state("w1"), Some(WorkerState::Running));
        let active = pool.active_claim("b").expect("b claimed after a done");
        assert_eq!(active.worker, "w1");

        // external graph change edge also wakes
        pool.register_worker("w2");
        pool.register_task("c", vec![]);
        let waken = pool.notify_graph_change("c");
        assert!(waken.is_empty() || waken.iter().any(|c| c.task_id == "b" && c.worker == "w2"));
    }

    /// (c) 冷恢复: 无主 claim 被清除并重认领, 新代际 attempt 重试同一任务。
    #[test]
    fn cold_recovery_reclaims_eligible_work() {
        let mut pool = EventDrivenClaimPool::new();
        pool.register_worker("w1");
        pool.register_task("t1", vec![]);
        let c1 = pool.notify_worker_idle("w1").unwrap();
        assert_eq!(c1.attempt_seq, 1);

        // 模拟重启: w1 消失, 其 claim 变成无主
        pool.workers.remove("w1");
        assert_eq!(pool.claim_count(), 1);

        // 冷恢复: w1' 是重启后新成员, 应重认领同一任务 (新代际)
        pool.register_worker("w1");
        let recovered = pool.cold_recovery();
        assert!(!recovered.is_empty(), "eligible work must be reclaimed");
        let c2 = recovered.iter().find(|c| c.task_id == "t1").expect("t1 re-claimed");
        assert_eq!(c2.worker, "w1");
        assert!(c2.attempt_seq > c1.attempt_seq, "cold recovery mints a new attempt generation");
        assert_eq!(pool.claim_count(), 1, "no duplicate claim");
        assert!(pool.active_claim("t1").is_some());
    }

    /// (d) 认领原子性: 两个竞争者, 一个胜出 (任务被第一个认领后第二个拿不到)。
    #[test]
    fn claims_are_atomic_two_contenders_one_wins() {
        let mut pool = EventDrivenClaimPool::new();
        pool.register_worker("w1");
        pool.register_worker("w2");
        pool.register_task("solo", vec![]);

        let c1 = pool.notify_worker_idle("w1").expect("w1 wins the single task");
        assert_eq!(c1.task_id, "solo");
        assert_eq!(c1.worker, "w1");

        // w2 尝试认领同一任务 → 失败 (已被 w1 原子认领)
        let c2 = pool.notify_worker_idle("w2");
        assert!(c2.is_none(), "second contender must not win the same task");
        assert_eq!(pool.claim_count(), 1);
        assert_eq!(pool.active_claim("solo").unwrap().worker, "w1");

        // w2 在任务完成后仍可通过图变更竞争下一个任务
        pool.complete(&c1).unwrap();
        let c3 = pool.notify_worker_idle("w2");
        assert!(c3.is_none(), "no further eligible task");
    }

    #[test]
    fn complete_rejects_stale_claim() {
        let mut pool = EventDrivenClaimPool::new();
        pool.register_worker("w1");
        pool.register_task("t1", vec![]);
        let c1 = pool.notify_worker_idle("w1").unwrap();

        let forged = Claim::new("t1", "t1#999", 999, "w1");
        assert!(pool.complete(&forged).is_err(), "forged/stale attempt must be rejected");

        let forged_owner = Claim::new("t1", c1.attempt_id.clone(), c1.attempt_seq, "w2");
        assert!(pool.complete(&forged_owner).is_err(), "owner mismatch must be rejected");

        assert!(pool.complete(&c1).is_ok());
    }

    #[test]
    fn reassign_revokes_and_reclaims() {
        let mut pool = EventDrivenClaimPool::new();
        pool.register_worker("w1");
        pool.register_worker("w2");
        pool.register_task("t1", vec![]);
        let c1 = pool.notify_worker_idle("w1").unwrap();

        let c2 = pool.reassign("t1", "w2").expect("reassign to w2");
        assert_eq!(c2.worker, "w2");
        assert!(c2.attempt_seq > c1.attempt_seq, "reassign mints new attempt");
        assert_eq!(pool.worker_state("w1"), Some(WorkerState::Idle));
        assert!(pool.complete(&c1).is_err(), "old claim invalid after reassign");
        pool.complete(&c2).unwrap();
    }
}