use serde::{Serialize, Deserialize};
use std::fmt;

pub mod auth;
pub mod connection;
pub mod server;
pub mod client;

pub use auth::{AuthMethod, Authenticator, ApiKeyAuthenticator, NoAuthAuthenticator, ClientAuth};
pub use connection::{ConnectionManager, Connection, ConnectionState, ConnectionStats};
pub use server::RemoteServer;
pub use client::RemoteClient;

/// Trait for handling remote commands externally.
/// Implement this to wire up the server to your application logic.
#[async_trait::async_trait]
pub trait CommandHandler: Send + Sync {
    async fn execute_task(&self, prompt: &str, session_id: Option<&str>) -> Result<String, String>;
    async fn run_command(&self, command: &str, args: &[String]) -> Result<serde_json::Value, String>;
    async fn query_state(&self, scope: &StateScope) -> Result<serde_json::Value, String>;
    async fn list_sessions(&self) -> Result<Vec<String>, String>;
    async fn get_session(&self, id: &str) -> Result<serde_json::Value, String>;
    async fn health(&self) -> Result<serde_json::Value, String>;
    async fn shutdown(&self) -> Result<(), String>;
}

/// Scope of state to query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateScope {
    All,
    ReasoningBrain,
    Sessions,
    Agents,
    Memory,
    Health,
    Custom(String),
}

/// Commands accepted by the RemoteServer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemoteCommand {
    ExecuteTask {
        prompt: String,
        session_id: Option<String>,
    },
    RunCommand {
        command: String,
        args: Vec<String>,
    },
    QueryState {
        scope: StateScope,
    },
    ListSessions,
    GetSession {
        id: String,
    },
    HealthCheck,
    Shutdown,
}

/// Response from the RemoteServer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteResponse {
    pub status: ResponseStatus,
    pub data: serde_json::Value,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResponseStatus {
    Success,
    Error,
    Unauthorized,
    RateLimited,
    ShuttingDown,
}

impl RemoteResponse {
    pub fn ok(data: impl Serialize) -> Self {
        Self {
            status: ResponseStatus::Success,
            data: serde_json::to_value(data).unwrap_or_default(),
            error: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            status: ResponseStatus::Error,
            data: serde_json::Value::Null,
            error: Some(msg.into()),
        }
    }

    pub fn unauthorized() -> Self {
        Self {
            status: ResponseStatus::Unauthorized,
            data: serde_json::Value::Null,
            error: Some("Unauthorized".into()),
        }
    }

    pub fn rate_limited() -> Self {
        Self {
            status: ResponseStatus::RateLimited,
            data: serde_json::Value::Null,
            error: Some("Rate limited".into()),
        }
    }
}

impl fmt::Display for RemoteCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RemoteCommand::ExecuteTask { prompt, .. } => {
                write!(f, "ExecuteTask(prompt={:.60})", prompt)
            }
            RemoteCommand::RunCommand { command, args } => {
                write!(f, "RunCommand({} {:?})", command, args)
            }
            RemoteCommand::QueryState { scope } => {
                write!(f, "QueryState({:?})", scope)
            }
            RemoteCommand::ListSessions => write!(f, "ListSessions"),
            RemoteCommand::GetSession { id } => write!(f, "GetSession({})", id),
            RemoteCommand::HealthCheck => write!(f, "HealthCheck"),
            RemoteCommand::Shutdown => write!(f, "Shutdown"),
        }
    }
}
