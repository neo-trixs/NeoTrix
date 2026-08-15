//! TaskContract — agent-loop 契约层
//!
//! 在 `nt_core_parallel` 之上叠加"契约化任务生命周期" (C1-C6):
//!   C1 目标明确 (goal set) → C2 拆解为可验证步骤 (subtasks) →
//!   C3 自主执行 (in-flight) → C4 证据接地 (acceptance) → C5 产出交付 (done) → C6 收尾反馈 (handled)
//!
//! 状态机: Defined → Accepted → InFlight → Done / DoneLate / Failed / Cancelled
//! 持久化: KB kv_store `agent_loop` namespace，按 contract_id upsert (幂等)。
//!
//! 遵循 R-P42 (强化现有节点，不建平行模块) — 复用 `TodoTask`/`TaskState` 数据类型。

use serde::{Deserialize, Serialize};

use super::types::TodoTask;

/// 契约状态 (C0-C6 的离散表示)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractState {
    /// C1 已发起，目标明确但未接受执行
    Defined,
    /// C2 已拆解并接受，进入自主执行
    Accepted,
    /// C3 正在执行
    InFlight,
    /// C4+C5 验收通过且产出已交付
    Done,
    /// C5 延期但最终交付
    DoneLate,
    /// C4 验收失败 (证据不足/未通过门)
    Failed,
    /// 取消 (注入 C6 终止)
    Cancelled,
}

impl ContractState {
    pub fn label(&self) -> String {
        match *self {
            ContractState::Defined => "C1 Defined ⚪".into(),
            ContractState::Accepted => "C2 Accepted 🔵".into(),
            ContractState::InFlight => "C3 In-Flight 🟠".into(),
            ContractState::Done => "C4/C5 Done 🟢".into(),
            ContractState::DoneLate => "C5 Done (late) 🟡".into(),
            ContractState::Failed => "C4 Failed 🔴".into(),
            ContractState::Cancelled => "C6 Cancelled ⚫".into(),
        }
    }

    /// 是否为终态
    pub fn is_terminal(&self) -> bool {
        matches!(self, ContractState::Done | ContractState::DoneLate | ContractState::Failed | ContractState::Cancelled)
    }
}

/// 一条契约化的 todo 记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContract {
    pub id: String,
    pub description: String,
    pub task_type: String,
    pub state: ContractState,
    pub subtasks: Vec<String>,
    /// 验收证据 (C4 grounding): 文件/命令/结果片段
    pub evidence: Vec<String>,
    pub priority: i32,
    pub created_at: i64,
    pub updated_at: i64,
    /// 超时阈值 (秒)；0 = 不限
    pub timeout_secs: u64,
}

impl TaskContract {
    /// C1: 定义契约 (初始状态 Defined)
    pub fn define(description: &str, task_type: &str, priority: i32) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            description: description.to_string(),
            task_type: task_type.to_string(),
            state: ContractState::Defined,
            subtasks: Vec::new(),
            evidence: Vec::new(),
            priority,
            created_at: now,
            updated_at: now,
            timeout_secs: 0,
        }
    }

    /// C2: 拆解为子步骤并接受契约
    pub fn accept(mut self, subtasks: Vec<String>) -> Self {
        self.subtasks = subtasks;
        self.state = ContractState::Accepted;
        self.touch();
        self
    }

    /// C3: 标记执行中
    pub fn start(mut self) -> Self {
        self.state = ContractState::InFlight;
        self.touch();
        self
    }

    /// C4: 验收通过 (成功)
    pub fn accept_mut(&mut self) {
        self.state = ContractState::InFlight;
        self.touch();
    }

    /// C4+C5: 标记完成
    pub fn complete(self, late: bool) -> Self {
        let mut completed = self;
        completed.state = if late { ContractState::DoneLate } else { ContractState::Done };
        completed.touch();
        completed
    }

    /// C4: 验收失败 (回归 Defined 重试，或 Failed)
    pub fn fail(mut self) -> Self {
        if self.state.is_terminal() {
            return self;
        }
        self.state = ContractState::Failed;
        self.evidence.push("[C4] acceptance gate failed".into());
        self.touch();
        self
    }

    /// 追加验收证据 (C4 grounding)
    pub fn add_evidence(&mut self, evidence: &str) {
        self.evidence.push(evidence.to_string());
        self.touch();
    }

    /// C6: 取消
    pub fn cancel(mut self) -> Self {
        self.state = ContractState::Cancelled;
        self.touch();
        self
    }

    /// 是否已超时 (基于 timeout_secs)
    pub fn is_timed_out(&self) -> bool {
        if self.timeout_secs == 0 {
            return false;
        }
        let now = chrono::Utc::now().timestamp() as u64;
        (now.saturating_sub(self.updated_at as u64)) > self.timeout_secs
    }

    /// 合法状态迁移校验
    pub fn can_transition_to(&self, next: ContractState) -> bool {
        if self.state.is_terminal() {
            return false; // 终态不可再迁移
        }
        matches!(
            (self.state, next),
            (ContractState::Defined, ContractState::Accepted | ContractState::Cancelled | ContractState::Failed)
                | (ContractState::Accepted, ContractState::InFlight | ContractState::Cancelled | ContractState::Failed)
                | (ContractState::InFlight, ContractState::Done | ContractState::DoneLate | ContractState::Failed | ContractState::Cancelled)
        )
    }

    /// 从 TodoTask 生成契约 (复用已有拆解数据类型)
    pub fn from_todo(todo: &TodoTask) -> Self {
        Self::define(&todo.description, &todo.task_type, todo.priority)
    }

    /// 汇总指标
    pub fn summary(&self) -> String {
        format!(
            "[{}] {} — {} (subtasks: {}), evidence: {}, priority: {}",
            self.id.get(..8).unwrap_or(&self.id),
            self.state.label(),
            self.description,
            self.subtasks.len(),
            self.evidence.len(),
            self.priority,
        )
    }

    fn touch(&mut self) {
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

impl Default for TaskContract {
    fn default() -> Self {
        Self::define("", "", 0)
    }
}

/// TaskContractWarden — 契约持久化 + 生命周期管理
pub struct TaskContractWarden {
    kb: Option<crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase>,
    contracts: std::collections::HashMap<String, TaskContract>,
    ns: String,
    /// 契约完成率 (验收通过 / 总额，C4 定义)
    completion_rate: f64,
}

impl Default for TaskContractWarden {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskContractWarden {
    pub fn new() -> Self {
        let kb = crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase::open(None).ok();
        Self {
            kb,
            contracts: std::collections::HashMap::new(),
            ns: "agent_loop".to_string(),
            completion_rate: 0.0,
        }
    }

    /// C1: 定义并持久化新契约
    pub fn define(&mut self, description: &str, task_type: &str, priority: i32) -> String {
        let c = TaskContract::define(description, task_type, priority);
        let id = c.id.clone();
        self.persist(&c);
        self.contracts.insert(id.clone(), c);
        id
    }

    /// 记录一条契约 (覆盖式)
    pub fn record(&mut self, contract: &TaskContract) {
        self.persist(contract);
        self.contracts.insert(contract.id.clone(), contract.clone());
        self.recompute_rate();
    }

    /// 获取契约 (内存优先，未命中回落 KB)
    pub fn get(&mut self, id: &str) -> Option<TaskContract> {
        if let Some(c) = self.contracts.get(id) {
            return Some(c.clone());
        }
        let k = self.key(id);
        let raw = self.kb.as_ref()?.kv_get(&self.ns, &k).ok()??;
        serde_json::from_str(&raw).ok()
    }

    /// 列出全部契约
    pub fn list(&self) -> Vec<TaskContract> {
        let mut all: Vec<TaskContract> = self.contracts.values().cloned().collect();
        all.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        all
    }

    /// 汇总状态分布
    pub fn stats(&self) -> ContractStats {
        let total = self.contracts.len() as u64;
        let done = self.contracts.values().filter(|c| matches!(c.state, ContractState::Done | ContractState::DoneLate)).count() as u64;
        let in_flight = self.contracts.values().filter(|c| !c.state.is_terminal()).count() as u64;
        ContractStats {
            total,
            done,
            in_flight,
            failed: self.contracts.values().filter(|c| c.state == ContractState::Failed).count() as u64,
            completion_rate: self.completion_rate,
        }
    }

    /// 持久化契约到 KB kv_store
    pub fn persist(&self, contract: &TaskContract) {
        if let Some(kb) = &self.kb {
            let json = serde_json::to_string(contract).unwrap_or_default();
            let _ = kb.kv_set(&self.ns, &self.key(&contract.id), &json);
        }
    }

    fn key(&self, id: &str) -> String {
        format!("contract:{}", id)
    }

    fn recompute_rate(&mut self) {
        let total = self.contracts.len();
        if total == 0 {
            self.completion_rate = 0.0;
            return;
        }
        let done = self.contracts.values().filter(|c| matches!(c.state, ContractState::Done | ContractState::DoneLate)).count();
        self.completion_rate = done as f64 / total as f64;
    }
}

/// 契约统计
#[derive(Debug, Clone, Serialize)]
pub struct ContractStats {
    pub total: u64,
    pub done: u64,
    pub in_flight: u64,
    pub failed: u64,
    pub completion_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_accept_complete_lifecycle() {
        let c = TaskContract::define("build search", "code", 1)
            .accept(vec!["scaffold".into(), "wire".into()])
            .start()
            .complete(false);
        assert_eq!(c.state, ContractState::Done);
        assert_eq!(c.subtasks.len(), 2);
        assert!(c.state.is_terminal());
    }

    #[test]
    fn test_legal_transitions() {
        let c = TaskContract::define("t", "x", 0);
        assert!(c.can_transition_to(ContractState::Accepted));
        assert!(!c.can_transition_to(ContractState::Done)); // Defined → Done 非法
        let done = c.accept(vec![]).start().complete(false);
        assert!(!done.can_transition_to(ContractState::InFlight)); // 终态不可迁移
    }

    #[test]
    fn test_fail_captures_evidence() {
        let c = TaskContract::define("t", "x", 0).accept(vec![]).fail();
        assert_eq!(c.state, ContractState::Failed);
        assert!(c.evidence.iter().any(|e| e.contains("acceptance gate")));
    }

    #[test]
    fn test_timeout() {
        let mut c = TaskContract::define("t", "x", 0);
        c.timeout_secs = 1;
        c.updated_at = chrono::Utc::now().timestamp() - 5;
        assert!(c.is_timed_out());
    }

    #[test]
    fn test_cancel() {
        let c = TaskContract::define("t", "x", 0).accept(vec!["s".into()]).cancel();
        assert_eq!(c.state, ContractState::Cancelled);
    }
}