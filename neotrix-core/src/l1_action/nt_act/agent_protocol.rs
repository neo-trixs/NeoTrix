//! AgentProtocol — sagent-inspired multi-agent coordination
//!
//! Three primitives:
//! - **AgentSelf**: agents can reconfigure themselves (swap tools, adjust parameters)
//! - **AgentSpawn**: agents can create child agents with scoped capabilities
//! - **AgentSend**: agents can send messages to peer agents via inbox
//!
//! Design: Inbox-Loop pattern with typed messages.
//! Zero unsafe. Uses tokio for async coordination.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

// ============================================================================
// Messages
// ============================================================================

/// Typed message envelope for inter-agent communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    /// Request another agent to perform a task.
    TaskRequest {
        task: String,
        context: serde_json::Value,
    },
    /// Response with a task result.
    TaskResult {
        task_id: String,
        result: serde_json::Value,
    },
    /// Request a tool invocation.
    ToolCall {
        call_id: String,
        tool: String,
        args: serde_json::Value,
    },
    /// Result of a tool invocation.
    ToolResult {
        call_id: String,
        result: serde_json::Value,
    },
    /// Request self-mutation (swap tools, adjust parameters).
    MutationRequest {
        changes: AgentMutation,
    },
    /// Request to spawn a child agent.
    SpawnRequest {
        config: AgentConfig,
        task: String,
    },
}

/// Describes a set of changes to apply to an agent (AgentSelf primitive).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMutation {
    /// Replace the full tool list (if set).
    pub tools: Option<Vec<String>>,
    /// Replace the agent name (if set).
    pub name: Option<String>,
    /// Override max budget (if set).
    pub max_budget_usd: Option<Option<f64>>,
}

// ============================================================================
// Inbox
// ============================================================================

/// Thread-safe agent inbox — VecDeque of messages behind an RwLock.
#[derive(Debug, Clone, Default)]
pub struct AgentInbox {
    queue: VecDeque<AgentMessage>,
}

impl AgentInbox {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// Enqueue a message.
    pub fn send(&mut self, msg: AgentMessage) {
        self.queue.push_back(msg);
    }

    /// Dequeue the next message, if any.
    pub fn recv(&mut self) -> Option<AgentMessage> {
        self.queue.pop_front()
    }

    /// Drain all messages and return them in order.
    pub fn drain(&mut self) -> Vec<AgentMessage> {
        self.queue.drain(..).collect()
    }

    /// Number of pending messages.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Whether the inbox is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

// ============================================================================
// Agent Config & Handle
// ============================================================================

/// Static configuration for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_id: String,
    pub name: String,
    pub tools: Vec<String>,
    pub max_budget_usd: Option<f64>,
    pub parent_id: Option<String>,
}

impl AgentConfig {
    pub fn new(agent_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            name: name.into(),
            tools: Vec::new(),
            max_budget_usd: None,
            parent_id: None,
        }
    }

    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_budget(mut self, budget: f64) -> Self {
        self.max_budget_usd = Some(budget);
        self
    }

    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }
}

/// Live handle to an agent — holds inbox + config.
#[derive(Debug, Clone)]
pub struct AgentHandle {
    pub id: String,
    pub inbox: Arc<RwLock<AgentInbox>>,
    pub config: AgentConfig,
}

impl AgentHandle {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            id: config.agent_id.clone(),
            inbox: Arc::new(RwLock::new(AgentInbox::new())),
            config,
        }
    }

    /// Send a message to this agent's inbox (AgentSend primitive).
    pub async fn send(&self, msg: AgentMessage) {
        self.inbox.write().await.send(msg);
    }

    /// Receive the next message, if any.
    pub async fn recv(&self) -> Option<AgentMessage> {
        self.inbox.write().await.recv()
    }

    /// Drain all pending messages.
    pub async fn drain(&self) -> Vec<AgentMessage> {
        self.inbox.write().await.drain()
    }

    /// Apply a mutation to this agent's config (AgentSelf primitive).
    pub fn apply_mutation(&mut self, mutation: &AgentMutation) {
        if let Some(ref new_tools) = mutation.tools {
            self.config.tools = new_tools.clone();
        }
        if let Some(ref new_name) = mutation.name {
            self.config.name = new_name.clone();
        }
        if mutation.max_budget_usd.is_some() {
            self.config.max_budget_usd = mutation.max_budget_usd.unwrap();
        }
    }
}

// ============================================================================
// Orchestrator
// ============================================================================

/// Manages all agents, routes messages, and tracks spawn lineage.
pub struct AgentOrchestrator {
    agents: HashMap<String, AgentHandle>,
    lineage: HashMap<String, Vec<String>>, // parent_id -> child_ids
}

impl AgentOrchestrator {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            lineage: HashMap::new(),
        }
    }

    /// Register a root agent.
    pub fn register(&mut self, handle: AgentHandle) {
        let id = handle.id.clone();
        self.agents.insert(id, handle);
    }

    /// Spawn a child agent (AgentSpawn primitive).
    pub fn spawn_agent(
        &mut self,
        parent_id: &str,
        config: AgentConfig,
        _task: &str,
    ) -> Result<AgentHandle, AgentProtocolError> {
        if !self.agents.contains_key(parent_id) {
            return Err(AgentProtocolError::AgentNotFound(parent_id.to_string()));
        }

        let child_config = config.with_parent(parent_id);
        let handle = AgentHandle::new(child_config);

        let child_id = handle.id.clone();
        self.agents.insert(child_id.clone(), handle.clone());

        self.lineage
            .entry(parent_id.to_string())
            .or_default()
            .push(child_id);

        Ok(handle)
    }

    /// Send a message between two agents (AgentSend primitive).
    pub async fn send_message(
        &self,
        from: &str,
        to: &str,
        message: AgentMessage,
    ) -> Result<(), AgentProtocolError> {
        if !self.agents.contains_key(from) {
            return Err(AgentProtocolError::AgentNotFound(from.to_string()));
        }
        let target = self
            .agents
            .get(to)
            .ok_or_else(|| AgentProtocolError::AgentNotFound(to.to_string()))?;

        target.send(message).await;
        Ok(())
    }

    /// Apply a mutation to an agent (AgentSelf primitive).
    pub fn mutate_agent(
        &mut self,
        agent_id: &str,
        mutation: &AgentMutation,
    ) -> Result<(), AgentProtocolError> {
        let agent = self
            .agents
            .get_mut(agent_id)
            .ok_or_else(|| AgentProtocolError::AgentNotFound(agent_id.to_string()))?;

        agent.apply_mutation(mutation);
        Ok(())
    }

    /// Get a handle to an agent by id.
    pub fn get_agent(&self, agent_id: &str) -> Option<&AgentHandle> {
        self.agents.get(agent_id)
    }

    /// Get all child agent ids for a parent.
    pub fn children_of(&self, parent_id: &str) -> &[String] {
        self.lineage.get(parent_id).map_or(&[], |v| v.as_slice())
    }

    /// Get all registered agent ids.
    pub fn agent_ids(&self) -> Vec<&str> {
        self.agents.keys().map(|s| s.as_str()).collect()
    }

    /// Remove an agent and all its descendants.
    pub fn remove_agent(&mut self, agent_id: &str) -> Result<(), AgentProtocolError> {
        if !self.agents.contains_key(agent_id) {
            return Err(AgentProtocolError::AgentNotFound(agent_id.to_string()));
        }

        // Recursively remove children.
        if let Some(children) = self.lineage.remove(agent_id) {
            for child_id in children {
                let _ = self.remove_agent(&child_id);
            }
        }

        self.agents.remove(agent_id);
        Ok(())
    }

    /// Drain all messages from an agent's inbox.
    pub async fn drain_inbox(&self, agent_id: &str) -> Result<Vec<AgentMessage>, AgentProtocolError> {
        let agent = self
            .agents
            .get(agent_id)
            .ok_or_else(|| AgentProtocolError::AgentNotFound(agent_id.to_string()))?;

        Ok(agent.drain().await)
    }
}

impl Default for AgentOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

// ════════════════════════════════════════════════════════════════
// Agent trait — nt_core_platform unified Agent interface
// ════════════════════════════════════════════════════════════════

#[async_trait::async_trait]
impl crate::core::nt_core_platform::Agent for AgentOrchestrator {
    fn agent_id(&self) -> &str { "act.agent_orchestrator" }
    fn agent_name(&self) -> &str { "AgentOrchestrator" }
    fn agent_layer(&self) -> crate::core::nt_core_capability::Layer {
        crate::core::nt_core_capability::Layer::L1Action
    }
    fn agent_domain(&self) -> crate::core::nt_core_capability::Domain {
        crate::core::nt_core_capability::Domain::NtAct
    }

    async fn initialize(&mut self) -> Result<(), crate::core::nt_core_platform::AgentError> {
        Ok(())
    }
    async fn start(&self) -> Result<(), crate::core::nt_core_platform::AgentError> {
        Ok(())
    }
    async fn stop(&self) -> Result<(), crate::core::nt_core_platform::AgentError> {
        Ok(())
    }
    fn status(&self) -> crate::core::nt_core_platform::AgentStatus {
        crate::core::nt_core_platform::AgentStatus::Running
    }
    fn metrics(&self) -> crate::core::nt_core_platform::AgentMetrics {
        crate::core::nt_core_platform::AgentMetrics::default()
    }
}

// ════════════════════════════════════════════════════════════════
// UnifiedCapability — nt_core_capability unified capability interface
// ════════════════════════════════════════════════════════════════

impl crate::core::nt_core_capability::UnifiedCapability for AgentOrchestrator {
    fn meta(&self) -> crate::core::nt_core_capability::CapabilityMeta {
        crate::core::nt_core_capability::CapabilityMeta {
            id: "act.agent_orchestrator".into(),
            name: "AgentOrchestrator".into(),
            layer: crate::core::nt_core_capability::Layer::L1Action,
            domain: crate::core::nt_core_capability::Domain::NtAct,
            version: env!("CARGO_PKG_VERSION").into(),
            description: "Multi-agent coordination orchestrator with spawn/send/mutate primitives".into(),
            tags: vec!["agent".into(), "orchestrator".into(), "l1".into()],
            status: crate::core::nt_core_capability::CapabilityStatus::Healthy,
            metrics: crate::core::nt_core_capability::CapabilityMetrics::default(),
            cost_weight: 0.2,
            priority: 1.0,
        }
    }

    fn health(&self) -> crate::core::nt_core_capability::CapabilityHealth {
        crate::core::nt_core_capability::CapabilityHealth {
            state: crate::core::nt_core_capability::CapabilityState::Healthy,
            success_rate: 1.0,
            avg_latency_ms: 0.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(
        &self,
        _input: crate::core::nt_core_capability::CapabilityInput,
    ) -> Result<crate::core::nt_core_capability::CapabilityOutput, crate::core::nt_core_capability::CapabilityError> {
        Ok(crate::core::nt_core_capability::CapabilityOutput::Text(
            format!("AgentOrchestrator: {} agents registered", self.agents.len()),
        ))
    }

    fn supports(&self, _input: &crate::core::nt_core_capability::CapabilityInput) -> bool {
        true
    }
}

// ============================================================================
// Errors
// ============================================================================

/// Errors from agent protocol operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentProtocolError {
    AgentNotFound(String),
    SpawnDenied(String),
    MutationDenied(String),
    MessageDeliveryFailed(String),
}

impl std::fmt::Display for AgentProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AgentNotFound(id) => write!(f, "agent not found: {}", id),
            Self::SpawnDenied(msg) => write!(f, "spawn denied: {}", msg),
            Self::MutationDenied(msg) => write!(f, "mutation denied: {}", msg),
            Self::MessageDeliveryFailed(msg) => write!(f, "message delivery failed: {}", msg),
        }
    }
}

impl std::error::Error for AgentProtocolError {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inbox_send_recv() {
        let mut inbox = AgentInbox::new();
        assert!(inbox.is_empty());

        inbox.send(AgentMessage::TaskRequest {
            task: "do stuff".into(),
            context: serde_json::json!({}),
        });
        assert_eq!(inbox.len(), 1);

        let msg = inbox.recv().unwrap();
        match msg {
            AgentMessage::TaskRequest { task, .. } => assert_eq!(task, "do stuff"),
            _ => panic!("unexpected message variant"),
        }
        assert!(inbox.is_empty());
    }

    #[test]
    fn inbox_drain() {
        let mut inbox = AgentInbox::new();
        inbox.send(AgentMessage::ToolCall {
            call_id: "c1".into(),
            tool: "search".into(),
            args: serde_json::json!({}),
        });
        inbox.send(AgentMessage::ToolCall {
            call_id: "c2".into(),
            tool: "write".into(),
            args: serde_json::json!({}),
        });

        let drained = inbox.drain();
        assert_eq!(drained.len(), 2);
        assert!(inbox.is_empty());
    }

    #[test]
    fn agent_self_mutation() {
        let config = AgentConfig::new("a1", "worker")
            .with_tools(vec!["search".into(), "write".into()]);
        let mut handle = AgentHandle::new(config);

        handle.apply_mutation(&AgentMutation {
            tools: Some(vec!["execute".into()]),
            name: Some("executor".into()),
            max_budget_usd: Some(Some(5.0)),
        });

        assert_eq!(handle.config.name, "executor");
        assert_eq!(handle.config.tools, vec!["execute"]);
        assert_eq!(handle.config.max_budget_usd, Some(5.0));
    }

    #[tokio::test]
    async fn agent_send_between_handles() {
        let a = AgentHandle::new(AgentConfig::new("a1", "alpha"));
        let b = AgentHandle::new(AgentConfig::new("b1", "beta"));

        a.send(AgentMessage::TaskRequest {
            task: "hello".into(),
            context: serde_json::json!({}),
        })
        .await;

        let msg = b.inbox.read().await.len();
        assert_eq!(msg, 0); // b's inbox is empty — message went to a

        let msg = a.recv().await.unwrap();
        match msg {
            AgentMessage::TaskRequest { task, .. } => assert_eq!(task, "hello"),
            _ => panic!("unexpected"),
        }
    }

    #[test]
    fn orchestrator_spawn_and_lineage() {
        let mut orch = AgentOrchestrator::new();
        let parent = AgentHandle::new(AgentConfig::new("p1", "parent"));
        orch.register(parent);

        let child = orch
            .spawn_agent(
                "p1",
                AgentConfig::new("c1", "child"),
                "analyze data",
            )
            .unwrap();

        assert_eq!(child.config.parent_id.as_deref(), Some("p1"));
        assert_eq!(orch.children_of("p1"), &["c1"]);
    }

    #[test]
    fn orchestrator_remove_cascades() {
        let mut orch = AgentOrchestrator::new();
        orch.register(AgentHandle::new(AgentConfig::new("p", "parent")));
        orch.spawn_agent("p", AgentConfig::new("c1", "child1"), "t").unwrap();
        orch.spawn_agent("c1", AgentConfig::new("gc1", "grandchild"), "t").unwrap();

        orch.remove_agent("p").unwrap();
        assert!(orch.get_agent("p").is_none());
        assert!(orch.get_agent("c1").is_none());
        assert!(orch.get_agent("gc1").is_none());
    }
}


