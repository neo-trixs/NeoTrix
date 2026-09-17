//! # MCP/A2A Protocol Bridge
//!
//! Implements the Model Context Protocol (MCP) and Agent-to-Agent Protocol (A2A)
//! for agent interoperability. MCP handles agent-to-tool communication, while
//! A2A enables agent-to-agent collaboration.
//!
//! ## Design Principles
//! - Complementary protocols: MCP for tools, A2A for agents
//! - Capability discovery: Agents publish "agent cards" with capabilities
//! - Task delegation: Agents can delegate tasks to other agents
//! - Secure & opaque: Agents interact without sharing internal state
//! - Multi-turn conversations: Support for extended agent interactions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub server_id: String,
    pub server_name: String,
    pub endpoint: String,
    pub capabilities: Vec<McpCapability>,
    pub tools: Vec<McpTool>,
    pub resources: Vec<McpResource>,
    pub prompts: Vec<McpPrompt>,
}

/// MCP Capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpCapability {
    /// Server supports tool execution
    Tools,
    /// Server supports resource access
    Resources,
    /// Server supports prompt templates
    Prompts,
    /// Server supports sampling (LLM completion requests)
    Sampling,
    /// Server supports elicitation (requesting user input)
    Elicitation,
    /// Server supports notifications
    Notifications,
}

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: Option<serde_json::Value>,
}

/// MCP Resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
}

/// MCP Prompt template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<McpPromptArgument>,
}

/// MCP Prompt argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default: Option<String>,
}

/// A2A Agent Card (capability advertisement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCard {
    pub agent_id: String,
    pub agent_name: String,
    pub description: String,
    pub capabilities: Vec<AgentCapability>,
    pub supported_modalities: Vec<String>,
    pub endpoint: String,
    pub version: String,
    pub provider: String,
}

/// Agent capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentCapability {
    /// Agent can handle coding tasks
    Coding,
    /// Agent can handle research tasks
    Research,
    /// Agent can handle data analysis
    DataAnalysis,
    /// Agent can handle creative writing
    CreativeWriting,
    /// Agent can handle mathematical reasoning
    MathReasoning,
    /// Agent can handle multi-step planning
    Planning,
    /// Agent can handle tool use
    ToolUse,
    /// Agent can handle multimodal input
    Multimodal,
    /// Custom capability
    Custom(String),
}

/// A2A Task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2ATask {
    pub task_id: String,
    pub context_id: String,
    pub status: TaskStatus,
    pub description: String,
    pub artifacts: Vec<A2AArtifact>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// Task status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Submitted,
    Working,
    InputRequired,
    Completed,
    Failed,
    Canceled,
    Rejected,
}

/// A2A Artifact (output of a task)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AArtifact {
    pub artifact_id: String,
    pub name: String,
    pub parts: Vec<A2APart>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A2A Part (content unit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2APart {
    pub part_type: A2APartType,
    pub content: String,
    pub mime_type: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A2A Part type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum A2APartType {
    Text,
    File,
    Data,
    Image,
    Audio,
    Video,
}

/// A2A Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AMessage {
    pub message_id: String,
    pub role: MessageRole,
    pub parts: Vec<A2APart>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Message role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Agent,
    System,
}

/// MCP/A2A Protocol Bridge
pub struct ProtocolBridge {
    mcp_servers: Vec<McpServerConfig>,
    known_agents: Vec<AgentCard>,
    active_tasks: HashMap<String, A2ATask>,
    message_history: Vec<A2AMessage>,
}

impl ProtocolBridge {
    pub fn new() -> Self {
        Self {
            mcp_servers: Vec::new(),
            known_agents: Vec::new(),
            active_tasks: HashMap::new(),
            message_history: Vec::new(),
        }
    }

    /// Register an MCP server
    pub fn register_mcp_server(&mut self, config: McpServerConfig) {
        self.mcp_servers.push(config);
    }

    /// Discover available agents (A2A)
    pub fn discover_agents(&self) -> Vec<&AgentCard> {
        self.known_agents.iter().collect()
    }

    /// Register an agent (A2A)
    pub fn register_agent(&mut self, card: AgentCard) {
        self.known_agents.push(card);
    }

    /// Find agents with specific capability
    pub fn find_agents_with_capability(&self, capability: &AgentCapability) -> Vec<&AgentCard> {
        self.known_agents
            .iter()
            .filter(|agent| {
                agent
                    .capabilities
                    .iter()
                    .any(|c| self.capabilities_match(c, capability))
            })
            .collect()
    }

    /// Check if capabilities match
    fn capabilities_match(&self, a: &AgentCapability, b: &AgentCapability) -> bool {
        match (a, b) {
            (AgentCapability::Coding, AgentCapability::Coding) => true,
            (AgentCapability::Research, AgentCapability::Research) => true,
            (AgentCapability::DataAnalysis, AgentCapability::DataAnalysis) => true,
            (AgentCapability::CreativeWriting, AgentCapability::CreativeWriting) => true,
            (AgentCapability::MathReasoning, AgentCapability::MathReasoning) => true,
            (AgentCapability::Planning, AgentCapability::Planning) => true,
            (AgentCapability::ToolUse, AgentCapability::ToolUse) => true,
            (AgentCapability::Multimodal, AgentCapability::Multimodal) => true,
            (AgentCapability::Custom(a), AgentCapability::Custom(b)) => a == b,
            _ => false,
        }
    }

    /// Create a new A2A task
    pub fn create_task(&mut self, description: String, context_id: String) -> A2ATask {
        let task_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let task = A2ATask {
            task_id: task_id.clone(),
            context_id,
            status: TaskStatus::Submitted,
            description,
            artifacts: Vec::new(),
            metadata: HashMap::new(),
            created_at: now.clone(),
            updated_at: now,
        };

        self.active_tasks.insert(task_id.clone(), task.clone());
        task
    }

    /// Delegate task to an agent (A2A)
    pub fn delegate_task(
        &mut self,
        task_id: &str,
        agent_id: &str,
    ) -> Result<A2ATask, ProtocolError> {
        let task = self
            .active_tasks
            .get_mut(task_id)
            .ok_or_else(|| ProtocolError::TaskNotFound(task_id.to_string()))?;

        let agent = self
            .known_agents
            .iter()
            .find(|a| a.agent_id == agent_id)
            .ok_or_else(|| ProtocolError::AgentNotFound(agent_id.to_string()))?;

        // Update task status
        task.status = TaskStatus::Working;
        task.updated_at = chrono::Utc::now().to_rfc3339();
        task.metadata
            .insert("delegated_to".into(), serde_json::json!(agent.agent_name));

        Ok(task.clone())
    }

    /// Get available tools from MCP servers
    pub fn get_available_tools(&self) -> Vec<&McpTool> {
        self.mcp_servers
            .iter()
            .flat_map(|server| server.tools.iter())
            .collect()
    }

    /// Find MCP server with specific tool
    pub fn find_server_with_tool(&self, tool_name: &str) -> Option<&McpServerConfig> {
        self.mcp_servers
            .iter()
            .find(|server| server.tools.iter().any(|t| t.name == tool_name))
    }

    /// Send A2A message
    pub fn send_message(&mut self, message: A2AMessage) {
        self.message_history.push(message);
    }

    /// Get conversation history
    pub fn get_conversation_history(&self) -> &[A2AMessage] {
        &self.message_history
    }

    /// Get active tasks
    pub fn get_active_tasks(&self) -> Vec<&A2ATask> {
        self.active_tasks.values().collect()
    }

    /// Get task by ID
    pub fn get_task(&self, task_id: &str) -> Option<&A2ATask> {
        self.active_tasks.get(task_id)
    }

    /// Update task status
    pub fn update_task_status(
        &mut self,
        task_id: &str,
        status: TaskStatus,
    ) -> Result<(), ProtocolError> {
        let task = self
            .active_tasks
            .get_mut(task_id)
            .ok_or_else(|| ProtocolError::TaskNotFound(task_id.to_string()))?;

        task.status = status;
        task.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }

    /// Add artifact to task
    pub fn add_artifact(
        &mut self,
        task_id: &str,
        artifact: A2AArtifact,
    ) -> Result<(), ProtocolError> {
        let task = self
            .active_tasks
            .get_mut(task_id)
            .ok_or_else(|| ProtocolError::TaskNotFound(task_id.to_string()))?;

        task.artifacts.push(artifact);
        task.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }
}

/// Protocol errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolError {
    TaskNotFound(String),
    AgentNotFound(String),
    ToolNotFound(String),
    InvalidMessage(String),
    ConnectionError(String),
    TimeoutError(String),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::TaskNotFound(id) => write!(f, "Task not found: {}", id),
            ProtocolError::AgentNotFound(id) => write!(f, "Agent not found: {}", id),
            ProtocolError::ToolNotFound(name) => write!(f, "Tool not found: {}", name),
            ProtocolError::InvalidMessage(msg) => write!(f, "Invalid message: {}", msg),
            ProtocolError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            ProtocolError::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_mcp_server() {
        let mut bridge = ProtocolBridge::new();
        let server = McpServerConfig {
            server_id: "github".into(),
            server_name: "GitHub MCP Server".into(),
            endpoint: "https://api.github.com".into(),
            capabilities: vec![McpCapability::Tools, McpCapability::Resources],
            tools: vec![McpTool {
                name: "create_issue".into(),
                description: "Create a GitHub issue".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "title": {"type": "string"},
                        "body": {"type": "string"}
                    }
                }),
                output_schema: None,
            }],
            resources: Vec::new(),
            prompts: Vec::new(),
        };

        bridge.register_mcp_server(server);
        assert_eq!(bridge.mcp_servers.len(), 1);
    }

    #[test]
    fn test_discover_agents() {
        let mut bridge = ProtocolBridge::new();
        let agent = AgentCard {
            agent_id: "coder-1".into(),
            agent_name: "Code Assistant".into(),
            description: "Specialized in coding tasks".into(),
            capabilities: vec![AgentCapability::Coding, AgentCapability::ToolUse],
            supported_modalities: vec!["text".into()],
            endpoint: "https://coder.example.com".into(),
            version: "1.0.0".into(),
            provider: "Example Corp".into(),
        };

        bridge.register_agent(agent);
        let agents = bridge.discover_agents();
        assert_eq!(agents.len(), 1);
    }

    #[test]
    fn test_create_task() {
        let mut bridge = ProtocolBridge::new();
        let task = bridge.create_task("Implement a new feature".into(), "context-1".into());

        assert_eq!(task.status, TaskStatus::Submitted);
        assert!(bridge.active_tasks.contains_key(&task.task_id));
    }

    #[test]
    fn test_delegate_task() {
        let mut bridge = ProtocolBridge::new();

        // Register agent
        let agent = AgentCard {
            agent_id: "coder-1".into(),
            agent_name: "Code Assistant".into(),
            description: "Specialized in coding tasks".into(),
            capabilities: vec![AgentCapability::Coding],
            supported_modalities: vec!["text".into()],
            endpoint: "https://coder.example.com".into(),
            version: "1.0.0".into(),
            provider: "Example Corp".into(),
        };
        bridge.register_agent(agent);

        // Create task
        let task = bridge.create_task("Implement a new feature".into(), "context-1".into());

        // Delegate task
        let result = bridge.delegate_task(&task.task_id, "coder-1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, TaskStatus::Working);
    }
}
