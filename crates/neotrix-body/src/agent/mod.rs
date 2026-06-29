//! # AgentBus — Agent Orchestration
//!
//! Architecture: AgentBus ← Agent trait ← AgentInstance
//! Supports sub-agents, tool execution, and inter-agent communication.
//! Each agent runs in its own cortical column (future MAPE-K loop).

use std::collections::HashMap;
use std::fmt;

pub type AgentResult<T> = Result<T, AgentError>;

#[derive(Debug, Clone)]
pub enum AgentError {
    NotFound(String),
    Busy(String),
    ExecutionFailed { agent: String, reason: String },
    Timeout(String),
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(a) => write!(f, "agent not found: {a}"),
            Self::Busy(a) => write!(f, "agent busy: {a}"),
            Self::ExecutionFailed { agent, reason } => write!(f, "agent {agent} failed: {reason}"),
            Self::Timeout(a) => write!(f, "agent timeout: {a}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    Running,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct AgentTask {
    pub id: String,
    pub name: String,
    pub input: String,
    pub created_ms: u64,
}

#[derive(Debug, Clone)]
pub struct AgentOutput {
    pub task_id: String,
    pub result: String,
    pub latency_ms: u64,
    pub success: bool,
}

/// Agent trait — each agent implements this
pub trait Agent: std::fmt::Debug + Send + Sync {
    fn name(&self) -> &str;
    fn execute(&self, task: AgentTask) -> AgentResult<AgentOutput>;
    fn status(&self) -> AgentStatus;
    fn capabilities(&self) -> Vec<String>;
}

/// AgentBus — registry and executor for agents
#[derive(Debug)]
pub struct AgentBus {
    agents: HashMap<String, Box<dyn Agent>>,
    execution_history: Vec<AgentOutput>,
    max_history: usize,
}

impl Clone for AgentBus {
    fn clone(&self) -> Self {
        Self {
            agents: HashMap::new(),
            execution_history: self.execution_history.clone(),
            max_history: self.max_history,
        }
    }
}

impl AgentBus {
    pub fn new(max_history: usize) -> Self {
        Self {
            agents: HashMap::new(),
            execution_history: Vec::with_capacity(max_history),
            max_history,
        }
    }

    pub fn register(&mut self, agent: Box<dyn Agent>) {
        let name = agent.name().to_string();
        self.agents.insert(name, agent);
    }

    pub fn unregister(&mut self, name: &str) -> Option<Box<dyn Agent>> {
        self.agents.remove(name)
    }

    pub fn get(&self, name: &str) -> Option<&dyn Agent> {
        self.agents.get(name).map(|a| a.as_ref())
    }

    pub fn execute(&mut self, agent_name: &str, task: AgentTask) -> AgentResult<AgentOutput> {
        let agent = self.agents.get(agent_name).ok_or_else(|| AgentError::NotFound(agent_name.into()))?;
        let output = agent.execute(task)?;
        self.execution_history.push(output.clone());
        if self.execution_history.len() > self.max_history {
            self.execution_history.remove(0);
        }
        Ok(output)
    }

    pub fn list_agents(&self) -> Vec<&dyn Agent> {
        self.agents.values().map(|a| a.as_ref()).collect()
    }

    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    pub fn recent_outputs(&self, count: usize) -> Vec<&AgentOutput> {
        self.execution_history.iter().rev().take(count).collect()
    }

    pub fn report(&self) -> String {
        let names: Vec<&str> = self.agents.keys().map(|s| s.as_str()).collect();
        format!("agentbus:agents_{}_exec_{}", names.join(","), self.execution_history.len())
    }
}

pub mod browser_agent;
pub mod core;
pub mod js_render;
pub mod registry;
pub mod self_healing_selector;
pub mod struct_extractor;
pub mod tools;
pub mod quant_engine;
pub mod perception_gateway;
pub mod network_evolution;
pub mod extraction_bridge;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MockAgent {
        name: String,
    }

    impl Agent for MockAgent {
        fn name(&self) -> &str { &self.name }
        fn execute(&self, task: AgentTask) -> AgentResult<AgentOutput> {
            Ok(AgentOutput {
                task_id: task.id,
                result: format!("executed by {}", self.name),
                latency_ms: 10,
                success: true,
            })
        }
        fn status(&self) -> AgentStatus { AgentStatus::Idle }
        fn capabilities(&self) -> Vec<String> { vec!["mock".into()] }
    }

    #[test]
    fn test_agent_bus_register() {
        let mut bus = AgentBus::new(100);
        bus.register(Box::new(MockAgent { name: "test".into() }));
        assert_eq!(bus.agent_count(), 1);
    }

    #[test]
    fn test_agent_bus_execute() {
        let mut bus = AgentBus::new(100);
        bus.register(Box::new(MockAgent { name: "worker".into() }));
        let output = bus.execute("worker", AgentTask {
            id: "1".into(),
            name: "task1".into(),
            input: "do something".into(),
            created_ms: 0,
        }).unwrap();
        assert!(output.success);
    }

    #[test]
    fn test_agent_bus_not_found() {
        let mut bus = AgentBus::new(100);
        let result = bus.execute("nonexistent", AgentTask {
            id: "1".into(),
            name: "task1".into(),
            input: "".into(),
            created_ms: 0,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_bus_recent_outputs() {
        let mut bus = AgentBus::new(10);
        bus.register(Box::new(MockAgent { name: "a".into() }));
        bus.execute("a", AgentTask {
            id: "1".into(), name: "t1".into(), input: "".into(), created_ms: 0,
        }).unwrap();
        assert_eq!(bus.recent_outputs(10).len(), 1);
    }
}

/// Bootstrap the cross-crate MetaEvolutionController with all Body layer modules registered.
/// Body depends on mind, so we can wire directly.
pub fn bootstrap_body_evolution() -> neotrix_mind::evolution::meta_controller::MetaEvolutionController {
    use neotrix_mind::evolution::evolution_task::TargetLayer;
    let mut ctrl = neotrix_mind::evolution::meta_controller::MetaEvolutionController::new(5);

    ctrl.register_module("agent_bus", TargetLayer::Body, &["orchestration", "agent_execution"]);
    ctrl.register_module("browser_agent", TargetLayer::Body, &["browsing", "navigation", "interaction"]);
    ctrl.register_module("js_render", TargetLayer::Body, &["javascript", "rendering", "dom"]);
    ctrl.register_module("self_healing_selector", TargetLayer::Body, &["selector", "css", "xpath", "healing"]);
    ctrl.register_module("struct_extractor", TargetLayer::Body, &["extraction", "schema", "parsing"]);
    ctrl.register_module("quant_engine", TargetLayer::Body, &["quantitative", "finance", "analysis"]);
    ctrl.register_module("perception_gateway", TargetLayer::Body, &["perception", "gwt", "attention", "routing"]);
    ctrl.register_module("network_evolution", TargetLayer::Body, &["evolution", "seal", "heuristic", "self_improvement"]);
    ctrl.register_module("extraction_bridge", TargetLayer::Body, &["bridge", "knowledge", "data_integration"]);
    ctrl.register_module("tls_fingerprint", TargetLayer::Body, &["tls", "fingerprint", "security"]);
    ctrl.register_module("proxy_rotator", TargetLayer::Body, &["proxy", "rotation", "anonymity"]);
    ctrl.register_module("captcha_handler", TargetLayer::Body, &["captcha", "solver", "challenge"]);
    ctrl.register_module("queue_persist", TargetLayer::Body, &["queue", "persistence", "storage"]);
    ctrl.register_module("doc_converter", TargetLayer::Body, &["document", "conversion", "markdown"]);
    ctrl.register_module("finance_pipeline", TargetLayer::Body, &["finance", "data", "pipeline"]);

    ctrl
}
