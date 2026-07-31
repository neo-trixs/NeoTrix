#![deny(clippy::unwrap_used)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[derive(Default)]
pub enum GooseAgentStatus {
    #[default]
    Idle,
    Running,
    WaitingForInput,
    Error(String),
    Completed,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooseSession {
    pub id: String,
    pub name: String,
    pub agent_type: String,
    pub status: GooseAgentStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub metadata: HashMap<String, String>,
    pub tool_ids: Vec<String>,
}

impl GooseSession {
    pub fn new(id: String, name: String, agent_type: String) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        Self {
            id,
            name,
            agent_type,
            status: GooseAgentStatus::Idle,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
            tool_ids: Vec::new(),
        }
    }

    pub fn set_status(&mut self, status: GooseAgentStatus) {
        self.status = status;
        self.updated_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    }

    pub fn add_tool(&mut self, tool_id: String) {
        if !self.tool_ids.contains(&tool_id) {
            self.tool_ids.push(tool_id);
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, GooseAgentStatus::Running | GooseAgentStatus::WaitingForInput)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooseMessage {
    pub role: String,
    pub content: String,
    pub tool_calls: Vec<GooseToolCall>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooseToolCall {
    pub id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooseToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub handler_type: String,
}

#[derive(Debug, Clone)]
pub struct GooseRuntimeConfig {
    pub max_sessions: usize,
    pub idle_timeout_secs: u64,
    pub max_history_per_session: usize,
    pub allowed_tools: Vec<String>,
}

impl Default for GooseRuntimeConfig {
    fn default() -> Self {
        Self {
            max_sessions: 10,
            idle_timeout_secs: 3600,
            max_history_per_session: 100,
            allowed_tools: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GooseRuntime {
    pub config: GooseRuntimeConfig,
    sessions: HashMap<String, GooseSession>,
    tools: HashMap<String, GooseToolDef>,
}

impl GooseRuntime {
    pub fn new(config: GooseRuntimeConfig) -> Self {
        Self {
            config,
            sessions: HashMap::new(),
            tools: HashMap::new(),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(GooseRuntimeConfig::default())
    }

    pub fn create_session(&mut self, name: &str, agent_type: &str) -> GooseSession {
        let id = format!("goose-{}", self.sessions.len() + 1);
        let session = GooseSession::new(id.clone(), name.to_string(), agent_type.to_string());
        self.sessions.insert(id.clone(), session.clone());
        session
    }

    pub fn get_session(&self, id: &str) -> Option<&GooseSession> {
        self.sessions.get(id)
    }

    pub fn get_session_mut(&mut self, id: &str) -> Option<&mut GooseSession> {
        self.sessions.get_mut(id)
    }

    pub fn list_sessions(&self) -> Vec<&GooseSession> {
        self.sessions.values().filter(|s| {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
            s.is_active() || (now - s.updated_at) < self.config.idle_timeout_secs
        }).collect()
    }

    pub fn remove_session(&mut self, id: &str) -> Option<GooseSession> {
        self.sessions.remove(id)
    }

    pub fn register_tool(&mut self, tool: GooseToolDef) {
        self.tools.insert(tool.name.clone(), tool);
    }

    pub fn get_tool(&self, name: &str) -> Option<&GooseToolDef> {
        self.tools.get(name)
    }

    pub fn list_tools(&self) -> Vec<&GooseToolDef> {
        self.tools.values().collect()
    }

    pub fn execute_tool(&self, session_id: &str, tool_call: &GooseToolCall) -> Result<GooseToolCall, String> {
        let session = self.sessions.get(session_id).ok_or("Session not found")?;
        if !session.is_active() {
            return Err("Session not active".to_string());
        }
        let tool = self.tools.get(&tool_call.tool_name).ok_or_else(|| format!("Tool '{}' not found", tool_call.tool_name))?;
        if !self.config.allowed_tools.is_empty() && !self.config.allowed_tools.contains(&tool_call.tool_name) {
            return Err(format!("Tool '{}' not in allowed list", tool_call.tool_name));
        }
        Ok(GooseToolCall {
            id: tool_call.id.clone(),
            tool_name: tool_call.tool_name.clone(),
            arguments: tool_call.arguments.clone(),
            result: Some(serde_json::json!({"status": "executed", "tool": tool.name})),
        })
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn active_session_count(&self) -> usize {
        self.sessions.values().filter(|s| s.is_active()).count()
    }
}

impl Default for GooseRuntime {
    fn default() -> Self {
        Self::with_defaults()
    }
}

pub struct GooseSessionManager {
    inner: GooseRuntime,
}

impl GooseSessionManager {
    pub fn new(config: GooseRuntimeConfig) -> Self {
        Self {
            inner: GooseRuntime::new(config),
        }
    }

    pub fn with_defaults() -> Self {
        Self {
            inner: GooseRuntime::with_defaults(),
        }
    }

    pub fn run(&mut self, name: &str, agent_type: &str, task: &str) -> Result<GooseSession, String> {
        let session = self.inner.create_session(name, agent_type);
        let session_id = session.id.clone();
        if let Some(s) = self.inner.get_session_mut(&session_id) {
            s.set_status(GooseAgentStatus::Running);
        }
        log::info!("[goose] session {} started: {}", session_id, task);

        if let Some(s) = self.inner.get_session_mut(&session_id) {
            s.set_status(GooseAgentStatus::Completed);
        }
        log::info!("[goose] session {} completed", session_id);

        self.inner.get_session(&session_id).cloned().ok_or("Session vanished".to_string())
    }

    pub fn inner(&self) -> &GooseRuntime {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut GooseRuntime {
        &mut self.inner
    }
}

impl Default for GooseSessionManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goose_session_creation() {
        let mut runtime = GooseRuntime::with_defaults();
        let session = runtime.create_session("test-agent", "code");
        assert_eq!(session.name, "test-agent");
        assert_eq!(session.agent_type, "code");
        assert_eq!(session.status, GooseAgentStatus::Idle);
        assert!(session.created_at > 0);
    }

    #[test]
    fn test_session_status_transitions() {
        let mut runtime = GooseRuntime::with_defaults();
        let session = runtime.create_session("worker", "chat");
        let id = session.id.clone();

        let s = runtime.get_session_mut(&id).unwrap();
        s.set_status(GooseAgentStatus::Running);
        assert_eq!(s.status, GooseAgentStatus::Running);
        assert!(s.is_active());

        let s = runtime.get_session_mut(&id).unwrap();
        s.set_status(GooseAgentStatus::Completed);
        assert_eq!(s.status, GooseAgentStatus::Completed);
        assert!(!s.is_active());
    }

    #[test]
    fn test_tool_registration() {
        let mut runtime = GooseRuntime::with_defaults();
        runtime.register_tool(GooseToolDef {
            name: "read_file".to_string(),
            description: "Read a file from disk".to_string(),
            input_schema: serde_json::json!({"type": "object", "properties": {"path": {"type": "string"}}}),
            handler_type: "fs".to_string(),
        });
        assert!(runtime.get_tool("read_file").is_some());
        assert!(runtime.get_tool("nonexistent").is_none());
    }

    #[test]
    fn test_tool_execution() {
        let mut runtime = GooseRuntime::with_defaults();
        runtime.register_tool(GooseToolDef {
            name: "echo".to_string(),
            description: "Echo input".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            handler_type: "builtin".to_string(),
        });

        let session = runtime.create_session("test", "code");
        let id = session.id.clone();
        runtime.get_session_mut(&id).unwrap().set_status(GooseAgentStatus::Running);

        let result = runtime.execute_tool(&id, &GooseToolCall {
            id: "call-1".to_string(),
            tool_name: "echo".to_string(),
            arguments: serde_json::json!({"text": "hello"}),
            result: None,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_tool_execution_inactive_session() {
        let runtime = GooseRuntime::with_defaults();
        let result = runtime.execute_tool("nonexistent", &GooseToolCall {
            id: "call-1".to_string(),
            tool_name: "echo".to_string(),
            arguments: serde_json::json!({}),
            result: None,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_execution_disallowed_tool() {
        let mut runtime = GooseRuntime::new(GooseRuntimeConfig {
            allowed_tools: vec!["safe_tool".to_string()],
            ..Default::default()
        });
        runtime.register_tool(GooseToolDef {
            name: "unsafe_tool".to_string(),
            description: "Dangerous".to_string(),
            input_schema: serde_json::json!({}),
            handler_type: "shell".to_string(),
        });

        let session = runtime.create_session("test", "code");
        let id = session.id.clone();
        runtime.get_session_mut(&id).unwrap().set_status(GooseAgentStatus::Running);

        let result = runtime.execute_tool(&id, &GooseToolCall {
            id: "call-1".to_string(),
            tool_name: "unsafe_tool".to_string(),
            arguments: serde_json::json!({}),
            result: None,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_list_sessions() {
        let mut runtime = GooseRuntime::with_defaults();
        runtime.create_session("agent-a", "chat");
        runtime.create_session("agent-b", "code");
        let sessions = runtime.list_sessions();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_remove_session() {
        let mut runtime = GooseRuntime::with_defaults();
        let session = runtime.create_session("temp", "test");
        let id = session.id.clone();
        assert_eq!(runtime.session_count(), 1);

        let removed = runtime.remove_session(&id);
        assert!(removed.is_some());
        assert_eq!(runtime.session_count(), 0);
    }

    #[test]
    fn test_add_tool_to_session() {
        let mut runtime = GooseRuntime::with_defaults();
        let session = runtime.create_session("agent", "code");
        let id = session.id.clone();

        let s = runtime.get_session_mut(&id).unwrap();
        s.add_tool("tool-1".to_string());
        s.add_tool("tool-2".to_string());
        s.add_tool("tool-1".to_string());
        assert_eq!(s.tool_ids.len(), 2);
    }

    #[test]
    fn test_session_manager_run() {
        let mut manager = GooseSessionManager::with_defaults();
        let result = manager.run("test-agent", "code", "write hello world");
        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.name, "test-agent");
    }

    #[test]
    fn test_goose_tool_call_serde() {
        let call = GooseToolCall {
            id: "call-abc".to_string(),
            tool_name: "search".to_string(),
            arguments: serde_json::json!({"q": "rust"}),
            result: Some(serde_json::json!({"results": []})),
        };
        let json = serde_json::to_string(&call).unwrap();
        let deserialized: GooseToolCall = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "call-abc");
        assert!(deserialized.result.is_some());
    }

    #[test]
    fn test_goose_message_tool_calls() {
        let msg = GooseMessage {
            role: "assistant".to_string(),
            content: "Let me search for that".to_string(),
            tool_calls: vec![GooseToolCall {
                id: "call-1".to_string(),
                tool_name: "search".to_string(),
                arguments: serde_json::json!({"q": "rust"}),
                result: None,
            }],
            timestamp: 1000,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: GooseMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tool_calls.len(), 1);
    }

    #[test]
    fn test_session_metadata() {
        let mut runtime = GooseRuntime::with_defaults();
        let session = runtime.create_session("agent", "code");
        let id = session.id.clone();
        let s = runtime.get_session_mut(&id).unwrap();
        s.metadata.insert("model".to_string(), "gpt-4".to_string());
        s.metadata.insert("provider".to_string(), "openai".to_string());
        assert_eq!(s.metadata.len(), 2);
    }
}