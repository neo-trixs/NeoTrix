//! # MCP Registry — Legacy backward-compat type system
//!
//! Migration layer preserving the pre-v2 MCP API (`agent::tool::mcp::*`)
//! for all existing consumers.  Kept as a standalone module so the new
//! `nt_agent_mcp_transport` / `nt_agent_mcp_discovery` crates have a
//! single import target.

use serde_json::Value;
use std::collections::HashMap;

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
    pub fn call_tool(&self, name: &str, _args: &Value) -> Result<String, String> {
        Err(format!(
            "McpRegistry::call_tool is a stub; use nt_agent_mcp_transport::mcp_call_tool instead. (tool={name})"
        ))
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
