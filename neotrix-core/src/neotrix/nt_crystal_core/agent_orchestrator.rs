//! AgentOrchestrator — Agent 编排引擎
//!
//! 基于 crewAI + AutoGen + MetaGPT。
//! - Crew 概念 (Agent 团队)
//! - 层级委派 (Hierarchical)
//! - Flow 事件驱动

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentRole {
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
    pub role: AgentRole,
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
            id: "a1".into(), role: AgentRole::Guide, name: "Guide".into(),
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
            id: "b1".into(), role: AgentRole::Artisan, name: "Artisan".into(),
            capabilities: vec![], status: AgentStatus::Idle,
        });
        let task = Task { id: "t2".into(), description: "work".into(), assigned_to: None, status: TaskStatus::Pending, result: None };
        orch.dispatch("beta", task);
        let delegated = orch.hierarchical_delegate("beta");
        assert!(!delegated.is_empty());
    }

    #[test]
    fn test_execute_flow() {
        let mut orch = AgentOrchestrator::new();
        let outputs = orch.execute_flow(vec![FlowEvent::Start, FlowEvent::DataReady("x".into())]);
        assert_eq!(outputs.len(), 2);
    }
}
