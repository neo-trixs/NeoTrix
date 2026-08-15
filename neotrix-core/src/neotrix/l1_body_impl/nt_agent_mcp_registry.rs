//! # MCP Registry — Legacy backward-compat type system
//!
//! Migration layer preserving the pre-v2 MCP API (`agent::tool::mcp::*`)
//! for all existing consumers.  Kept as a standalone module so the new
//! `nt_agent_mcp_transport` / `nt_agent_mcp_discovery` crates have a
//! single import target.

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::neotrix::l1_body_impl::nt_agent_mcp_adapter::McpToolAdapter;
use crate::neotrix::l1_body_impl::nt_agent_mcp_gateway::{EvidenceEntry, HashChain};
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

#[derive(Debug)]
pub struct McpRegistry {
    servers: Vec<McpServerEntry>,
    /// Fast name → index lookup (kept in sync by all mutating methods)
    by_name: HashMap<String, usize>,
    /// G15/G16 治理证据哈希链 (interior mutability: McpGateway 持 &McpRegistry 也能 record)。
    evidence: Mutex<HashChain>,
}

impl Clone for McpRegistry {
    fn clone(&self) -> Self {
        let evidence = self
            .evidence
            .lock()
            .map(|c| c.clone())
            .unwrap_or_else(|e| e.into_inner().clone());
        Self {
            servers: self.servers.clone(),
            by_name: self.by_name.clone(),
            evidence: Mutex::new(evidence),
        }
    }
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
            evidence: Mutex::new(HashChain::new()),
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

    // -- Absorption into NativeTool -------------------------------------------------

    /// 把每个已注册且含工具的服务器包装为 `McpToolAdapter`，供 ToolOrchestrator
    /// 以普通 NativeTool 身份消费（吸收管线：MCP → adapter → orchestrator）。
    pub fn as_native_tools(
        &self,
    ) -> Vec<Box<dyn crate::core::nt_core_traits::NativeTool>> {
        self.servers
            .iter()
            .filter(|s| !s.tools.is_empty())
            .map(|s| {
                let mode = to_transport_mode(&s.transport, s.url.as_deref());
                Box::new(McpToolAdapter::new(&s.name, mode, s.tools.clone()))
                    as Box<dyn crate::core::nt_core_traits::NativeTool>
            })
            .collect()
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

        // 优先使用工具自身声明的 transport (工具级 Local 命令/URL 优先于服务器级)。
        let mode = to_transport_mode(&tool.transport, server.url.as_deref());
        let (content, _cache) = crate::neotrix::l1_body_impl::nt_agent_mcp_transport::mcp_call_tool(&mode, &tool.name, args)
            .map_err(|e| format!("MCP tool '{}' failed: {}", name, e))?;
        Ok(content)
    }

    /// Stub
    pub fn cache_result(&self) -> Option<()> {
        None
    }

    // -- G15/G16 Governance gateway (hash-chain evidence) --------------------

    /// Governed wrapper over the same registry: N→4 folding, allow/deny/HITL,
    /// SHA-256 chain evidence. Every existing caller that holds `&McpRegistry`
    /// can reach the production path via this accessor.
    pub fn gateway(&self) -> crate::neotrix::l1_body_impl::nt_agent_mcp_gateway::McpGateway<'_> {
        crate::neotrix::l1_body_impl::nt_agent_mcp_gateway::McpGateway::new(self)
    }

    /// Governed one-shot call with an explicit policy (registry-level surface).
    /// Returns transport content on approval, records every outcome on the chain.
    pub fn call_tool_governed(
        &self,
        name: &str,
        args: &Value,
        policy: &crate::neotrix::l1_body_impl::nt_agent_mcp_gateway::GovernancePolicy,
    ) -> Result<String, String> {
        let verdict = policy.check(name);
        match verdict {
            crate::neotrix::l1_body_impl::nt_act_sandbox::SandboxVerdict::Denied => {
                self.record_evidence(name, args, verdict, false, None);
                Err(format!(
                    "MCP governance: tool '{}' denied by policy",
                    name
                ))
            }
            crate::neotrix::l1_body_impl::nt_act_sandbox::SandboxVerdict::RequiresApproval => {
                self.record_evidence(name, args, verdict, false, None);
                Err(format!(
                    "MCP governance: tool '{}' requires human approval (not granted)",
                    name
                ))
            }
            crate::neotrix::l1_body_impl::nt_act_sandbox::SandboxVerdict::Approved => {
                match self.call_tool(name, args) {
                    Ok(content) => {
                        self.record_evidence(
                            name,
                            args,
                            verdict,
                            false,
                            Some(truncate_for_evidence(&content, 256)),
                        );
                        Ok(content)
                    }
                    Err(e) => {
                        self.record_evidence(
                            name,
                            args,
                            verdict,
                            false,
                            Some(format!("ERROR: {}", e)),
                        );
                        Err(e)
                    }
                }
            }
        }
    }

    /// Append a hash-chain evidence entry (used by `McpGateway` and
    /// `call_tool_governed`). Returns the appended entry.
    pub fn record_evidence(
        &self,
        name: &str,
        args: &Value,
        verdict: crate::neotrix::l1_body_impl::nt_act_sandbox::SandboxVerdict,
        approved_by_hitl: bool,
        result: Option<String>,
    ) -> EvidenceEntry {
        let mut chain = self
            .evidence
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        chain.append(name, args.clone(), verdict, approved_by_hitl, result)
    }

    /// Whether the full evidence chain is intact.
    pub fn chain_valid(&self) -> bool {
        self.evidence
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .verify()
    }

    /// Clone of the current evidence chain (append-only audit view).
    pub fn evidence_chain(&self) -> HashChain {
        self.evidence
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Stub
    pub fn prune_cache(&self) -> usize {
        0
    }
}

/// Truncate a result preview for evidence (keep it bounded for audit).
fn truncate_for_evidence(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.min(s.len())])
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

    #[test]
    fn test_as_native_tools_maps_servers() {
        let mut reg = McpRegistry::new();
        reg.publish("alpha", "cmd-a", &[], "alpha tool");
        reg.register_stdio("beta", "cmd-b", &[], vec![McpToolDef {
            name: "beta_t1".into(),
            description: "beta tool".into(),
            input_schema: serde_json::json!({"type": "object", "properties": {}}),
            server_name: "beta".into(),
            transport: McpTransport::Stdio,
            schema_version: None,
        }]);
        // Empty server (no tools) must be skipped.
        reg.register_stdio("empty", "cmd-e", &[], vec![]);

        let native = reg.as_native_tools();
        // alpha (1 published tool) + beta (1 tool) = 2 adapters; "empty" filtered out.
        assert_eq!(native.len(), 2, "empty server should be excluded");
        let ids: Vec<String> = native.iter().map(|t| t.id().to_string()).collect();
        assert!(ids.contains(&"alpha_tool".to_string()));
        assert!(ids.contains(&"beta_t1".to_string()));
        assert_eq!(native[0].capability_tags(), vec!["mcp_absorbed"]);
    }

    #[test]
    fn test_as_native_tools_empty_registry() {
        let reg = McpRegistry::new();
        assert!(reg.as_native_tools().is_empty());
    }
}
