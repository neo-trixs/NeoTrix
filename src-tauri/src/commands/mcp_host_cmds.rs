//! Tauri commands for MCP server hosting
//!
//! NeoTrix can expose its tools as MCP servers so other AI assistants can consume them.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

// ── Data types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpHostConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub auth_token: Option<String>,
}

impl Default for McpHostConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "127.0.0.1".into(),
            port: 8311,
            max_connections: 10,
            auth_token: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpHostEndpoint {
    pub id: String,
    pub name: String,
    pub description: String,
    pub endpoint_type: String,
    pub parameters: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpHostSession {
    pub client_id: String,
    pub connected_at: u64,
    pub tool_calls: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct McpHostStatus {
    pub running: bool,
    pub port: u16,
    pub uptime_secs: u64,
    pub active_sessions: usize,
    pub total_endpoints: usize,
    pub total_calls: u64,
}

// ── State ────────────────────────────────────────────────────────────────────

const MAX_SESSIONS: usize = 100;
const MAX_LOG: usize = 200;

struct McpHostState {
    config: McpHostConfig,
    running: bool,
    start_time: u64,
    endpoints: Vec<McpHostEndpoint>,
    sessions: Vec<McpHostSession>,
    activity_log: VecDeque<serde_json::Value>,
    total_calls: u64,
}

impl McpHostState {
    fn new() -> Self {
        Self {
            config: McpHostConfig::default(),
            running: false,
            start_time: 0,
            endpoints: builtin_endpoints(),
            sessions: Vec::new(),
            activity_log: VecDeque::with_capacity(MAX_LOG),
            total_calls: 0,
        }
    }
}

fn builtin_endpoints() -> Vec<McpHostEndpoint> {
    vec![
        McpHostEndpoint {
            id: "tool_execute".into(),
            name: "Execute Tool".into(),
            description: "Execute a general-purpose tool (web search, file read/write, bash, etc.)".into(),
            endpoint_type: "tool".into(),
            parameters: vec!["tool".into(), "args".into()],
            enabled: true,
        },
        McpHostEndpoint {
            id: "tool_search".into(),
            name: "Web Search".into(),
            description: "Search the web and return ranked results".into(),
            endpoint_type: "tool".into(),
            parameters: vec!["query".into(), "count".into()],
            enabled: true,
        },
        McpHostEndpoint {
            id: "brain_stats".into(),
            name: "Brain Statistics".into(),
            description: "Get the current NeoTrix reasoning brain stats".into(),
            endpoint_type: "resource".into(),
            parameters: vec![],
            enabled: true,
        },
        McpHostEndpoint {
            id: "kb_search".into(),
            name: "Knowledge Base Search".into(),
            description: "Search the NeoTrix knowledge base".into(),
            endpoint_type: "resource".into(),
            parameters: vec!["query".into()],
            enabled: true,
        },
        McpHostEndpoint {
            id: "computer_screen_capture".into(),
            name: "Screen Capture".into(),
            description: "Capture the current computer screen".into(),
            endpoint_type: "tool".into(),
            parameters: vec![],
            enabled: true,
        },
        McpHostEndpoint {
            id: "computer_mouse_click".into(),
            name: "Mouse Click".into(),
            description: "Click at a specified screen coordinate".into(),
            endpoint_type: "tool".into(),
            parameters: vec!["x".into(), "y".into(), "button".into()],
            enabled: true,
        },
        McpHostEndpoint {
            id: "computer_keyboard_type".into(),
            name: "Keyboard Type".into(),
            description: "Type text using the keyboard".into(),
            endpoint_type: "tool".into(),
            parameters: vec!["text".into()],
            enabled: true,
        },
    ]
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn log_activity(state: &mut McpHostState, entry: serde_json::Value) {
    if state.activity_log.len() >= MAX_LOG {
        state.activity_log.pop_front();
    }
    state.activity_log.push_back(entry);
}

static MCP_HOST: LazyLock<Mutex<McpHostState>> = LazyLock::new(|| Mutex::new(McpHostState::new()));

// ── Commands ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn mcp_host_start(config: McpHostConfig) -> Result<String, String> {
    let mut state = MCP_HOST.lock().map_err(|e| e.to_string())?;
    if state.running {
        return Err("MCP host is already running".into());
    }
    let now = now_secs();
    state.config = config;
    state.running = true;
    state.start_time = now;
    state.sessions.clear();
    state.total_calls = 0;
    let port = state.config.port;
    log_activity(&mut state, serde_json::json!({
        "event": "start", "port": port, "ts": now
    }));
    Ok(format!("mcp-host-{}", port))
}

#[tauri::command]
pub fn mcp_host_stop() -> Result<(), String> {
    let mut state = MCP_HOST.lock().map_err(|e| e.to_string())?;
    if !state.running {
        return Err("MCP host is not running".into());
    }
    state.running = false;
    state.sessions.clear();
    log_activity(&mut state, serde_json::json!({
        "event": "stop", "ts": now_secs()
    }));
    Ok(())
}

#[tauri::command]
pub fn mcp_host_status() -> Result<McpHostStatus, String> {
    let state = MCP_HOST.lock().map_err(|e| e.to_string())?;
    let uptime = if state.running {
        now_secs().saturating_sub(state.start_time)
    } else {
        0
    };
    Ok(McpHostStatus {
        running: state.running,
        port: state.config.port,
        uptime_secs: uptime,
        active_sessions: state.sessions.iter().filter(|s| s.status == "active").count(),
        total_endpoints: state.endpoints.len(),
        total_calls: state.total_calls,
    })
}

#[tauri::command]
pub fn mcp_host_list_endpoints() -> Result<Vec<McpHostEndpoint>, String> {
    let state = MCP_HOST.lock().map_err(|e| e.to_string())?;
    Ok(state.endpoints.clone())
}

#[tauri::command]
pub fn mcp_host_register_endpoint(
    name: String,
    description: String,
    params: Vec<String>,
) -> Result<(), String> {
    let mut state = MCP_HOST.lock().map_err(|e| e.to_string())?;
    if state.endpoints.iter().any(|ep| ep.name == name) {
        return Err(format!("Endpoint '{}' already exists", name));
    }
    let log_name = name.clone();
    let id = name.to_lowercase().replace(' ', "_");
    state.endpoints.push(McpHostEndpoint {
        id,
        name,
        description,
        endpoint_type: "tool".into(),
        parameters: params,
        enabled: true,
    });
    log_activity(&mut state, serde_json::json!({
        "event": "register_endpoint", "name": log_name, "ts": now_secs()
    }));
    Ok(())
}

#[tauri::command]
pub fn mcp_host_unregister_endpoint(name: String) -> Result<(), String> {
    let mut state = MCP_HOST.lock().map_err(|e| e.to_string())?;
    let len_before = state.endpoints.len();
    state.endpoints.retain(|ep| ep.name != name);
    if state.endpoints.len() == len_before {
        return Err(format!("Endpoint '{}' not found", name));
    }
    log_activity(&mut state, serde_json::json!({
        "event": "unregister_endpoint", "name": name, "ts": now_secs()
    }));
    Ok(())
}

#[tauri::command]
pub fn mcp_host_sessions() -> Result<Vec<McpHostSession>, String> {
    let state = MCP_HOST.lock().map_err(|e| e.to_string())?;
    Ok(state.sessions.clone())
}

#[tauri::command]
pub fn mcp_host_log(count: usize) -> Result<Vec<serde_json::Value>, String> {
    let state = MCP_HOST.lock().map_err(|e| e.to_string())?;
    let take = count.min(state.activity_log.len());
    let entries: Vec<serde_json::Value> = state.activity_log.iter().rev().take(take).cloned().collect();
    Ok(entries)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_host_start_stop() {
        let config = McpHostConfig {
            port: 18311,
            ..Default::default()
        };
        let id = mcp_host_start(config).unwrap();
        assert_eq!(id, "mcp-host-18311");

        let status = mcp_host_status().unwrap();
        assert!(status.running);
        assert_eq!(status.port, 18311);
        assert!(status.uptime_secs > 0);

        mcp_host_stop().unwrap();
        let status = mcp_host_status().unwrap();
        assert!(!status.running);
    }

    #[test]
    fn test_mcp_host_list_endpoints() {
        // start to reset state
        let _ = mcp_host_start(McpHostConfig::default());

        let endpoints = mcp_host_list_endpoints().unwrap();
        assert_eq!(endpoints.len(), 7);
        assert!(endpoints.iter().any(|ep| ep.name == "Execute Tool"));
        assert!(endpoints.iter().any(|ep| ep.name == "Screen Capture"));
        assert!(endpoints.iter().any(|ep| ep.name == "Keyboard Type"));
    }

    #[test]
    fn test_mcp_host_register_endpoint() {
        let _ = mcp_host_start(McpHostConfig::default());

        mcp_host_register_endpoint(
            "Custom API".into(),
            "Call a custom REST API".into(),
            vec!["url".into(), "method".into()],
        )
        .unwrap();

        let endpoints = mcp_host_list_endpoints().unwrap();
        assert_eq!(endpoints.len(), 8);
        assert!(endpoints.iter().any(|ep| ep.name == "Custom API"));

        // duplicate should fail
        assert!(mcp_host_register_endpoint(
            "Custom API".into(),
            "Duplicate".into(),
            vec![],
        )
        .is_err());
    }

    #[test]
    fn test_mcp_host_sessions() {
        let _ = mcp_host_start(McpHostConfig::default());

        let sessions = mcp_host_sessions().unwrap();
        assert!(sessions.is_empty());

        // add a session manually through state
        {
            let mut state = MCP_HOST.lock().unwrap();
            state.sessions.push(McpHostSession {
                client_id: "test-client".into(),
                connected_at: now_secs(),
                tool_calls: 3,
                status: "active".into(),
            });
        }

        let sessions = mcp_host_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].client_id, "test-client");
        assert_eq!(sessions[0].tool_calls, 3);
        assert_eq!(sessions[0].status, "active");
    }
}
