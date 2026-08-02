//! # MCP Registry — Legacy backward-compat type system
//!
//! Migration layer preserving the pre-v2 MCP API (`agent::tool::mcp::*`)
//! for all existing consumers.  Kept as a standalone module so the new
//! `nt_agent_mcp_transport` / `nt_agent_mcp_discovery` crates have a
//! single import target.

use serde_json::Value;
use std::collections::HashMap;

use crate::neotrix::l1_body_impl::nt_agent_mcp_transport::TransportMode;

// ---------------------------------------------------------------------------
// Legacy types — API-identical to the original inline module in agent.rs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct McpToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub server_name: String,
    pub transport: McpTransport,
    pub schema_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum McpTransport {
    Stdio,
    Http,
    WebSocket,
    Sse,
    Local { command: String, args: Vec<String> },
}

impl McpTransport {
    pub fn transport_type(&self) -> &str {
        match self {
            McpTransport::Stdio => "stdio",
            McpTransport::Http => "http",
            McpTransport::WebSocket => "ws",
            McpTransport::Sse => "sse",
            McpTransport::Local { .. } => "local",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct McpServerEntry {
    pub name: String,
    pub transport: McpTransport,
    pub command: Option<String>,
    pub url: Option<String>,
    pub tools: Vec<McpToolDef>,
    pub healthy: bool,
    pub latency_ms: u64,
    pub last_health_check: Option<String>,
    pub init_result: Option<String>,
}

/// Backward-compat alias
pub type McpServer = McpServerEntry;

// ---------------------------------------------------------------------------
// McpRegistry — Vec-backed for backward compat with list_servers() -> &[Entry]
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct McpRegistry {
    servers: Vec<McpServerEntry>,
    /// Fast name → index lookup (kept in sync by all mutating methods)
    by_name: HashMap<String, usize>,
}

impl Default for McpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl McpRegistry {
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
            by_name: HashMap::new(),
        }
    }

    // -- Registration -------------------------------------------------------

    pub fn register(&mut self, server: McpServerEntry) {
        if self.by_name.contains_key(&server.name) {
            return;
        }
        let name = server.name.clone();
        self.servers.push(server);
        self.by_name.insert(name, self.servers.len() - 1);
    }

    pub fn register_stdio(
        &mut self,
        name: &str,
        cmd: &str,
        _args: &[&str],
        tools: Vec<McpToolDef>,
    ) {
        if self.by_name.contains_key(name) {
            return;
        }
        let tools: Vec<McpToolDef> = tools
            .into_iter()
            .map(|mut t| {
                t.server_name = name.to_string();
                t
            })
            .collect();
        let server = McpServerEntry {
            name: name.to_string(),
            transport: McpTransport::Stdio,
            command: Some(cmd.to_string()),
            url: None,
            tools,
            healthy: true,
            latency_ms: 0,
            last_health_check: None,
            init_result: Some("registered".to_string()),
        };
        self.by_name
            .insert(name.to_string(), self.servers.len());
        self.servers.push(server);
    }

    // -- Query --------------------------------------------------------------

    /// Search all servers for a tool whose **name** matches exactly.
    pub fn find_tool(&self, name: &str) -> Option<McpToolDef> {
        self.servers
            .iter()
            .flat_map(|s| s.tools.iter())
            .find(|t| t.name == name)
            .cloned()
    }

    /// Case-insensitive substring search over tool name + description.
    pub fn search(&self, query: &str) -> Vec<McpToolDef> {
        let q = query.to_lowercase();
        self.servers
            .iter()
            .flat_map(|s| s.tools.iter())
            .filter(|t| {
                t.name.to_lowercase().contains(&q)
                    || t.description.to_lowercase().contains(&q)
            })
            .cloned()
            .collect()
    }

    /// Scored search: name match = 2 pts, description match = 1 pt.
    pub fn recommend_tools(&self, query: &str, top_k: usize) -> Vec<McpToolDef> {
        let q = query.to_lowercase();
        let mut scored: Vec<(usize, &McpToolDef)> = self
            .servers
            .iter()
            .flat_map(|s| s.tools.iter())
            .filter_map(|t| {
                let name_match = if t.name.to_lowercase().contains(&q) {
                    2
                } else {
                    0
                };
                let desc_match = if t.description.to_lowercase().contains(&q) {
                    1
                } else {
                    0
                };
                let score = name_match + desc_match;
                if score > 0 {
                    Some((score, t))
                } else {
                    None
                }
            })
            .collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored
            .into_iter()
            .take(top_k)
            .map(|(_, t)| t.clone())
            .collect()
    }

    pub fn list_tools(&self) -> Vec<McpToolDef> {
        self.servers
            .iter()
            .flat_map(|s| s.tools.iter().cloned())
            .collect()
    }

    // -- Publish (legacy) ---------------------------------------------------

    pub fn publish(
        &mut self,
        name: &str,
        cmd: &str,
        args: &[&str],
        desc: &str,
    ) -> usize {
        if self.by_name.contains_key(name) {
            return 0;
        }
        let tool_name = format!("{name}_tool");
        let transport = McpTransport::Local {
            command: cmd.to_string(),
            args: args.iter().map(|a| a.to_string()).collect(),
        };
        let tool = McpToolDef {
            name: tool_name,
            description: format!("{desc} [published]"),
            input_schema: Value::Object(serde_json::Map::new()),
            server_name: name.to_string(),
            transport: transport.clone(),
            schema_version: None,
        };
        self.servers.push(McpServerEntry {
            name: name.to_string(),
            transport,
            command: Some(cmd.to_string()),
            url: None,
            tools: vec![tool],
            healthy: true,
            latency_ms: 0,
            last_health_check: None,
            init_result: Some("published".to_string()),
        });
        self.by_name
            .insert(name.to_string(), self.servers.len() - 1);
        1
    }

    // -- Stats --------------------------------------------------------------

    pub fn server_count(&self) -> usize {
        self.servers.len()
    }

    pub fn tool_count(&self) -> usize {
        self.servers.iter().map(|s| s.tools.len()).sum()
    }

    pub fn list_servers(&self) -> &[McpServerEntry] {
        &self.servers
    }

    // -- Stubs (for forward compat) -----------------------------------------

    /// Always returns `true` (stub)
    pub fn health_check(&self) -> bool {
        true
    }

    /// Stub — returns Err with a descriptive message
    pub fn call_tool(&self, name: &str, args: &Value) -> Result<String, String> {
        let (server, tool) = self
            .servers
            .iter()
            .filter_map(|s| {
                s.tools
                    .iter()
                    .find(|t| t.name == name)
                    .map(|t| (s, t))
            })
            .next()
            .ok_or_else(|| format!("MCP tool '{}' not registered", name))?;

        let mode = to_transport_mode(&server.transport, server.url.as_deref());
        let (content, _cache) = crate::neotrix::l1_body_impl::nt_agent_mcp_transport::mcp_call_tool(&mode, &tool.name, args)
            .map_err(|e| format!("MCP tool '{}' failed: {}", name, e))?;
        Ok(content)
    }

    /// Stub
    pub fn cache_result(&self) -> Option<()> {
        None
    }

    /// Stub
    pub fn prune_cache(&self) -> usize {
        0
    }
}

/// Convert legacy `McpTransport` into the v3 `TransportMode` used by
/// `nt_agent_mcp_transport::mcp_call_tool`. Falls back to Stdio for unknown
/// variants so a missing URL never panics.
fn to_transport_mode(t: &McpTransport, url: Option<&str>) -> TransportMode {
    match t {
        McpTransport::Local { command, args } => TransportMode::Local {
            command: command.clone(),
            args: args.clone(),
        },
        McpTransport::Http | McpTransport::WebSocket | McpTransport::Sse => {
            if let Some(url) = url {
                TransportMode::Remote {
                    http_url: url.to_string(),
                    headers: std::collections::HashMap::new(),
                    sse_url: None,
                    auth: None,
                }
            } else {
                TransportMode::Local {
                    command: "mcp".to_string(),
                    args: vec![],
                }
            }
        }
        McpTransport::Stdio => TransportMode::Local {
            command: "mcp".to_string(),
            args: vec![],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_transport_mode_local() {
        let mode = to_transport_mode(
            &McpTransport::Local { command: "server".into(), args: vec!["--flag".into()] },
            None,
        );
        match mode {
            TransportMode::Local { command, args } => {
                assert_eq!(command, "server");
                assert_eq!(args, vec!["--flag".to_string()]);
            }
            _ => panic!("expected Local"),
        }
    }

    #[test]
    fn test_to_transport_mode_remote_with_url() {
        let mode = to_transport_mode(&McpTransport::Http, Some("https://mcp.example.com"));
        match mode {
            TransportMode::Remote { http_url, sse_url, .. } => {
                assert_eq!(http_url, "https://mcp.example.com");
                assert!(sse_url.is_none());
            }
            _ => panic!("expected Remote"),
        }
    }

    #[test]
    fn test_to_transport_mode_remote_without_url_falls_back() {
        let mode = to_transport_mode(&McpTransport::Sse, None);
        assert!(matches!(mode, TransportMode::Local { .. }));
    }

    #[test]
    fn test_call_tool_unregistered() {
        let reg = McpRegistry::new();
        let err = reg.call_tool("ghost", &serde_json::json!({})).unwrap_err();
        assert!(err.contains("not registered"));
    }

    #[test]
    fn test_call_tool_registered_no_command() {
        let mut reg = McpRegistry::new();
        reg.publish("echo", "nonexistent-cmd-xyz", &[], "echo tool");
        // Tool resolves but spawn of a nonexistent binary fails → returns Err,
        // proving we left the stub (which also returned Err but with "stub").
        let err = reg.call_tool("echo_tool", &serde_json::json!({})).unwrap_err();
        assert!(!err.contains("stub"), "call_tool must not be a stub anymore");
    }
}
