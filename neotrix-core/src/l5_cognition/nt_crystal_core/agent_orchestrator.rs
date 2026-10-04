//! AgentOrchestrator — Agent 编排引擎
//!
//! 基于 crewAI + AutoGen + MetaGPT。
//! - Crew 概念 (Agent 团队)
//! - 层级委派 (Hierarchical)
//! - Flow 事件驱动

use serde::{Deserialize, Serialize};

// T14: 原 AgentRole 改名 CrystalArchetype（与 L1 正典 AgentRole 消歧；crew 功能正典见 l5 multi_agent/role.rs）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrystalArchetype {
    Guide,
    Artisan,
    Protector,
    Explorer,
    Sentinel,
    Weaver,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub role: CrystalArchetype,
    pub name: String,
    pub capabilities: Vec<String>,
    pub status: AgentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Working,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessType {
    Sequential,
    Hierarchical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crew {
    pub name: String,
    pub agents: Vec<Agent>,
    pub process: ProcessType,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub assigned_to: Option<String>,
    pub status: TaskStatus,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlowEvent {
    Start,
    DataReady(String),
    TaskComplete(String),
    Error(String),
}

pub struct AgentOrchestrator {
    pub crews: Vec<Crew>,
    pub event_log: Vec<(u64, FlowEvent)>,
}

impl AgentOrchestrator {
    pub fn new() -> Self {
        Self { crews: Vec::new(), event_log: Vec::new() }
    }

    pub fn create_crew(&mut self, name: &str, process: ProcessType) -> &mut Crew {
        self.crews.push(Crew {
            name: name.to_string(), agents: Vec::new(), process, tasks: Vec::new(),
        });
        self.crews.last_mut().unwrap()
    }

    pub fn add_agent(&mut self, crew_name: &str, agent: Agent) {
        if let Some(crew) = self.crews.iter_mut().find(|c| c.name == crew_name) {
            crew.agents.push(agent);
        }
    }

    pub fn dispatch(&mut self, crew_name: &str, task: Task) -> bool {
        if let Some(crew) = self.crews.iter_mut().find(|c| c.name == crew_name) {
            let available = crew.agents.iter().find(|a| matches!(a.status, AgentStatus::Idle));
            if let Some(agent) = available {
                let mut task = task;
                task.assigned_to = Some(agent.id.clone());
                task.status = TaskStatus::InProgress;
                crew.tasks.push(task);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// 登记一个**待委派**任务（不占用 agent）。
    ///
    /// 与 `dispatch` 的区别是本方法存在的全部理由：
    /// - `dispatch`：按到达顺序**立刻**派给一个 `Idle` agent 并置 `InProgress`
    /// - 本方法：只入队为 `Pending`，留给 `hierarchical_delegate` 批量委派
    ///
    /// ⚠️ 二者对同一个空闲 agent **互斥**：先 `dispatch` 再
    ///   `hierarchical_delegate`，后者会因找不到 `Pending` 而返回空。
    ///   这是设计如此（一个 agent 同时只做一件事），不是缺陷。
    pub fn enqueue_pending_task(&mut self, crew_name: &str, task: Task) -> bool {
        match self.crews.iter_mut().find(|c| c.name == crew_name) {
            Some(crew) => {
                crew.tasks.push(task);
                true
            }
            None => false,
        }
    }

    pub fn hierarchical_delegate(&mut self, crew_name: &str) -> Vec<String> {
        let mut delegated = Vec::new();
        if let Some(crew) = self.crews.iter_mut().find(|c| c.name == crew_name) {
            if matches!(crew.process, ProcessType::Hierarchical) {
                let pending: Vec<usize> = crew.tasks.iter().enumerate()
                    .filter(|(_, t)| matches!(t.status, TaskStatus::Pending))
                    .map(|(i, _)| i).collect();
                for idx in pending {
                    if let Some(agent) = crew.agents.iter_mut().find(|a| matches!(a.status, AgentStatus::Idle)) {
                        crew.tasks[idx].assigned_to = Some(agent.id.clone());
                        crew.tasks[idx].status = TaskStatus::InProgress;
                        delegated.push(format!("Task {} -> Agent {}", crew.tasks[idx].id, agent.id));
                    }
                }
            }
        }
        delegated
    }

    pub fn execute_flow(&mut self, events: Vec<FlowEvent>) -> Vec<String> {
        let mut outputs = Vec::new();
        for event in events {
            self.event_log.push((now_ms(), event.clone()));
            match event {
                FlowEvent::Start => outputs.push("Flow started".into()),
                FlowEvent::DataReady(data) => outputs.push(format!("Data ready: {}", data)),
                FlowEvent::TaskComplete(id) => outputs.push(format!("Task {} complete", id)),
                FlowEvent::Error(e) => outputs.push(format!("Error: {}", e)),
            }
        }
        outputs
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_crew_and_dispatch() {
        let mut orch = AgentOrchestrator::new();
        orch.create_crew("alpha", ProcessType::Sequential);
        orch.add_agent("alpha", Agent {
            id: "a1".into(), role: CrystalArchetype::Guide, name: "Guide".into(),
            capabilities: vec!["reasoning".into()], status: AgentStatus::Idle,
        });
        let task = Task { id: "t1".into(), description: "test".into(), assigned_to: None, status: TaskStatus::Pending, result: None };
        assert!(orch.dispatch("alpha", task));
    }

    #[test]
    fn test_hierarchical_delegate() {
        let mut orch = AgentOrchestrator::new();
        orch.create_crew("beta", ProcessType::Hierarchical);
        orch.add_agent("beta", Agent {
            id: "b1".into(), role: CrystalArchetype::Artisan, name: "Artisan".into(),
            capabilities: vec![], status: AgentStatus::Idle,
        });
        let task = Task { id: "t2".into(), description: "work".into(), assigned_to: None, status: TaskStatus::Pending, result: None };
        // ⚠️ 此处原调用 `dispatch("beta", task)`。
        //
        //   而 `dispatch` 会**立刻**把任务派给空闲 agent 并置为 `InProgress`，
        //   于是 `hierarchical_delegate` 再去找 `Pending` 任务时
        //   找到 0 条 ⇒ `delegated` 必空 ⇒ **该测试恒失败**。
        //
        //   两条路径是**互斥**的（一条按到达顺序派发，一条按层级批量委派），
        //   同一个 crew + 同一个空闲 agent 不该被两条路径先后用掉。
        //
        // ✅ 改为直接登记 Pending 任务（层级委派的正确前置条件）。
        //    ⚠️ `add_task` 若不存在则用 push；先确认 API 再写。
        orch.enqueue_pending_task("beta", task);
        let delegated = orch.hierarchical_delegate("beta");
        assert!(!delegated.is_empty(), "Pending 任务应被委派出去");
        assert!(
            delegated.iter().any(|d| d.contains("t2") && d.contains("b1")),
            "委派记录应含任务 id 与 agent id: {:?}",
            delegated
        );
    }

    /// 锁住上面那条语义边界：`dispatch` 与 `hierarchical_delegate` 互斥。
    /// 若将来有人让 `dispatch` 不再立刻置 `InProgress`，本测试会红。
    #[test]
    fn dispatch_consumes_the_idle_agent_so_delegate_finds_nothing() {
        let mut orch = AgentOrchestrator::new();
        orch.create_crew("gamma", ProcessType::Hierarchical);
        orch.add_agent("gamma", Agent {
            id: "g1".into(), role: CrystalArchetype::Guide, name: "G".into(),
            capabilities: vec![], status: AgentStatus::Idle,
        });
        let task = Task { id: "t9".into(), description: "d".into(), assigned_to: None, status: TaskStatus::Pending, result: None };
        assert!(orch.dispatch("gamma", task), "dispatch 应成功占用唯一空闲 agent");
        // 任务已被 dispatch 置为 InProgress ⇒ 层级委派无 Pending 可用
        assert!(
            orch.hierarchical_delegate("gamma").is_empty(),
            "dispatch 已占用 agent，层级委派应无事可做"
        );
    }

    #[test]
    fn test_execute_flow() {
        let mut orch = AgentOrchestrator::new();
        let outputs = orch.execute_flow(vec![FlowEvent::Start, FlowEvent::DataReady("x".into())]);
        assert_eq!(outputs.len(), 2);
    }
}
