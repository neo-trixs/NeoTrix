use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolPermission {
    Network,
    FileRead,
    FileWrite,
    ProcessSpawn,
    ShellExec,
    EnvRead,
    Custom(String),
}

pub trait AgentTool: Send + Sync {
    fn id(&self) -> &str;
    fn manifest(&self) -> &ToolManifest;
    fn start(&mut self, api: ToolApi) -> Result<(), ToolError>;
    fn execute(&self, ctx: ToolContext) -> Result<ToolOutput, ToolError>;
    fn stop(&mut self) -> Result<(), ToolError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub permissions: Vec<ToolPermission>,
    pub mcp: Option<McpServerDecl>,
    pub min_runtime: String,
    pub description: String,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerDecl {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

pub struct ToolApi {
    pub storage: Box<dyn ToolStorage>,
    pub fs: Box<dyn ToolFs>,
    pub ipc: Box<dyn ToolIpc>,
    pub log: Box<dyn ToolLogger>,
}

pub trait ToolStorage: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str);
    fn delete(&self, key: &str);
}

pub trait ToolFs: Send + Sync {
    fn read(&self, path: &str) -> Result<String, ToolError>;
    fn write(&self, path: &str, contents: &str) -> Result<(), ToolError>;
    fn exists(&self, path: &str) -> bool;
}

pub trait ToolIpc: Send + Sync {
    fn on(&self, channel: &str, handler: Box<dyn Fn(&[u8])>);
    fn send(&self, channel: &str, data: &[u8]);
}

pub trait ToolLogger: Send + Sync {
    fn info(&self, msg: &str);
    fn warn(&self, msg: &str);
    fn error(&self, msg: &str);
}

#[derive(Debug)]
pub struct ToolContext {
    pub input: String,
    pub session_id: String,
}

#[derive(Debug)]
pub struct ToolOutput {
    pub result: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ToolError {
    Runtime { id: String, message: String },
    PermissionDenied { tool_id: String, missing: ToolPermission },
    Io(std::io::Error),
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolError::Runtime { id, message } => {
                write!(f, "Tool {} error: {}", id, message)
            }
            ToolError::PermissionDenied { tool_id, missing } => {
                write!(f, "Permission denied: tool {} missing {:?}", tool_id, missing)
            }
            ToolError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for ToolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ToolError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ToolError {
    fn from(e: std::io::Error) -> Self {
        ToolError::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_manifest_creation() {
        let m = ToolManifest {
            id: "test".into(), name: "Test".into(), version: "1.0".into(),
            permissions: vec![ToolPermission::Network, ToolPermission::FileRead],
            mcp: None, min_runtime: "0.18".into(), description: "A test tool".into(),
            author: Some("NeoTrix".into()),
        };
        assert_eq!(m.id, "test");
        assert_eq!(m.permissions.len(), 2);
    }

    #[test]
    fn test_tool_manifest_with_mcp() {
        let mcp = McpServerDecl {
            command: "python".into(),
            args: vec!["server.py".into()],
            env: [("KEY".into(), "val".into())].into(),
        };
        let m = ToolManifest {
            id: "mcp-tool".into(), name: "MCP Tool".into(), version: "0.1".into(),
            permissions: vec![], mcp: Some(mcp), min_runtime: "0.18".into(),
            description: "MCP tool".into(), author: None,
        };
        assert!(m.mcp.is_some());
        assert_eq!(m.mcp.as_ref().unwrap().command, "python");
    }

    #[test]
    fn test_tool_context_creation() {
        let ctx = ToolContext {
            input: "test input".into(),
            session_id: "session-1".into(),
        };
        assert_eq!(ctx.input, "test input");
    }

    #[test]
    fn test_tool_output_creation() {
        let out = ToolOutput {
            result: "done".into(),
            metadata: [("key".into(), "val".into())].into(),
        };
        assert_eq!(out.result, "done");
    }

    #[test]
    fn test_tool_error_runtime_display() {
        let err = ToolError::Runtime { id: "web_scrape".into(), message: "timeout".into() };
        let msg = format!("{}", err);
        assert!(msg.contains("web_scrape"));
        assert!(msg.contains("timeout"));
    }

    #[test]
    fn test_tool_error_permission_denied_display() {
        let err = ToolError::PermissionDenied {
            tool_id: "shell".into(),
            missing: ToolPermission::ShellExec,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("shell"));
    }

    #[test]
    fn test_tool_error_io_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = ToolError::Io(io_err);
        let msg = format!("{}", err);
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn test_tool_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err: ToolError = io_err.into();
        match err {
            ToolError::Io(_) => {},
            _ => panic!("expected Io variant"),
        }
    }

    #[test]
    fn test_tool_error_source_for_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "err");
        let err = ToolError::Io(io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_tool_error_source_for_runtime() {
        let err = ToolError::Runtime { id: "x".into(), message: "err".into() };
        assert!(err.source().is_none());
    }
}
