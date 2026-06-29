//! # MCP Protocol Driver
//!
//! Model Context Protocol — standardized tool/connector interface.

use std::collections::HashMap;

/// MCP request
#[derive(Debug, Clone)]
pub struct McpRequest {
    pub server: String,
    pub method: String,
    pub params: HashMap<String, String>,
    pub timeout_ms: u64,
}

impl Default for McpRequest {
    fn default() -> Self {
        Self {
            server: String::new(),
            method: String::new(),
            params: HashMap::new(),
            timeout_ms: 5000,
        }
    }
}

/// MCP response
#[derive(Debug, Clone)]
pub struct McpResponse {
    pub success: bool,
    pub data: String,
    pub server: String,
    pub latency_ms: u64,
}

/// MCP driver trait
pub trait McpDriver: std::fmt::Debug + Send + Sync {
    fn execute(&self, request: McpRequest) -> Result<McpResponse, String>;
    fn list_servers(&self) -> Vec<String>;
}

/// Mock MCP driver
#[derive(Debug, Clone)]
pub struct MockMcpDriver {
    pub servers: Vec<String>,
}

impl MockMcpDriver {
    pub fn new(servers: &[&str]) -> Self {
        Self {
            servers: servers.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl McpDriver for MockMcpDriver {
    fn execute(&self, request: McpRequest) -> Result<McpResponse, String> {
        Ok(McpResponse {
            success: true,
            data: format!("{}:{} executed", request.server, request.method),
            server: request.server,
            latency_ms: 50,
        })
    }

    fn list_servers(&self) -> Vec<String> {
        self.servers.clone()
    }
}

/// MCP state
#[derive(Debug, Clone)]
pub struct McpState {
    pub connected_servers: Vec<String>,
    pub total_calls: u64,
    pub failed_calls: u64,
}

impl McpState {
    pub fn new() -> Self {
        Self {
            connected_servers: Vec::new(),
            total_calls: 0,
            failed_calls: 0,
        }
    }

    pub fn record_call(&mut self, success: bool) {
        self.total_calls += 1;
        if !success {
            self.failed_calls += 1;
        }
    }

    pub fn report(&self) -> String {
        format!("mcp:servers_{}_calls_{}_fail_{}", self.connected_servers.len(), self.total_calls, self.failed_calls)
    }
}

impl Default for McpState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_mcp_driver() {
        let driver = MockMcpDriver::new(&["server1", "server2"]);
        let servers = driver.list_servers();
        assert_eq!(servers.len(), 2);
    }

    #[test]
    fn test_mcp_state() {
        let mut state = McpState::new();
        state.connected_servers.push("test".into());
        state.record_call(true);
        state.record_call(false);
        assert_eq!(state.total_calls, 2);
        assert_eq!(state.failed_calls, 1);
    }
}
