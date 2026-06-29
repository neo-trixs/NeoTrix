//! # Core Agent Types
//!
//! Common agent implementations used across the system.

use super::{Agent, AgentOutput, AgentResult, AgentStatus, AgentTask};

/// A generic agent that uses a tool registry to fulfill tasks
#[derive(Debug)]
#[allow(dead_code)]
pub struct ToolAgent {
    name: String,
    description: String,
    status: AgentStatus,
}

impl ToolAgent {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            status: AgentStatus::Idle,
        }
    }
}

impl Agent for ToolAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, task: AgentTask) -> AgentResult<AgentOutput> {
        Ok(AgentOutput {
            task_id: task.id,
            result: format!("{} executed: {}", self.name, task.name),
            latency_ms: 0,
            success: true,
        })
    }

    fn status(&self) -> AgentStatus {
        self.status.clone()
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["tool_execution".into()]
    }
}

/// Orchestrator agent — delegates subtasks to sub-agents
#[derive(Debug)]
pub struct OrchestratorAgent {
    name: String,
    status: AgentStatus,
    sub_agent_names: Vec<String>,
}

impl OrchestratorAgent {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            status: AgentStatus::Idle,
            sub_agent_names: Vec::new(),
        }
    }

    pub fn add_sub_agent(&mut self, name: &str) {
        self.sub_agent_names.push(name.into());
    }

    pub fn sub_agents(&self) -> &[String] {
        &self.sub_agent_names
    }
}

impl Agent for OrchestratorAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, task: AgentTask) -> AgentResult<AgentOutput> {
        Ok(AgentOutput {
            task_id: task.id,
            result: format!("orchestrated with {} sub-agents: {}", self.sub_agent_names.len(), task.input),
            latency_ms: 0,
            success: true,
        })
    }

    fn status(&self) -> AgentStatus {
        self.status.clone()
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["orchestration".into(), "delegation".into()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_agent_execution() {
        let agent = ToolAgent::new("test", "test agent");
        let task = AgentTask {
            id: "1".into(),
            name: "task1".into(),
            input: "do work".into(),
            created_ms: 0,
        };
        let output = agent.execute(task).unwrap();
        assert!(output.success);
    }

    #[test]
    fn test_orchestrator_sub_agents() {
        let mut agent = OrchestratorAgent::new("orchestrator");
        agent.add_sub_agent("agent_a");
        agent.add_sub_agent("agent_b");
        assert_eq!(agent.sub_agents().len(), 2);
    }
}
