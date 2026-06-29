#![forbid(unsafe_code)]

//! MCP 2026-07-28 Full Client with server lifecycle management

use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use serde_json::Value;

/// Transport protocol for MCP server
#[derive(Clone, Debug)]
pub enum McpTransport {
    Stdio { command: String, args: Vec<String> },
    Http { url: String },
}

/// A running MCP server instance
pub struct McpServerInstance {
    pub name: String,
    pub transport: McpTransport,
    pub tools: Vec<String>,
    process: Option<Child>,
    pub healthy: bool,
    pub last_ping: Instant,
}

/// MCP server configuration from .mcp.json
#[derive(Clone, Debug, serde::Deserialize)]
pub struct McpServerConfig {
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Full MCP client managing multiple server instances
pub struct McpFullClient {
    servers: HashMap<String, McpServerInstance>,
}

impl McpFullClient {
    pub fn new() -> Self {
        Self { servers: HashMap::new() }
    }

    /// Parse McpServerConfig map from a JSON value, supporting both
    /// the flat format (keys are server names) and the mcpServers wrapper.
    fn parse_configs(value: &Value) -> HashMap<String, McpServerConfig> {
        // Try nested mcpServers first (compatible with ToolRegistry::discover_mcp_servers)
        if let Some(nested) = value.get("mcpServers").and_then(|v| v.as_object()) {
            let mut map = HashMap::new();
            for (name, cfg) in nested {
                if let Ok(config) = serde_json::from_value::<McpServerConfig>(cfg.clone()) {
                    map.insert(name.clone(), config);
                }
            }
            return map;
        }
        // Fall back to flat format: entire object is { name -> config }
        if let Some(obj) = value.as_object() {
            let mut map = HashMap::new();
            for (name, cfg) in obj {
                if let Ok(config) = serde_json::from_value::<McpServerConfig>(cfg.clone()) {
                    map.insert(name.clone(), config);
                }
            }
            return map;
        }
        HashMap::new()
    }

    /// Discover servers from .mcp.json files in home dir and CWD.
    /// Returns list of discovered server names.
    pub fn discover_servers(&mut self) -> Result<Vec<String>, String> {
        let mut found = Vec::new();
        let paths = self.config_paths();
        for path in &paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(value) = serde_json::from_str::<Value>(&content) {
                    let configs = Self::parse_configs(&value);
                    for (name, config) in &configs {
                        if !self.servers.contains_key(name) {
                            found.push(name.clone());
                        }
                        // Start server if not already running
                        if !self.servers.contains_key(name) {
                            let _ = self.start_server(name, config.clone());
                        }
                    }
                }
            }
        }
        Ok(found)
    }

    /// Return the list of config file paths to check.
    fn config_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        // CWD
        paths.push(".mcp.json".to_string());
        // Home directory
        {
            let home = dirs::home_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "/tmp".to_string());
            paths.push(format!("{}/.mcp.json", home));
            paths.push(format!("{}/.config/mcp/servers.json", home));
        }
        // Config path from env
        if let Ok(custom) = std::env::var("MCP_CONFIG_PATH") {
            paths.push(custom);
        }
        paths
    }

    /// Start an MCP server from config
    pub fn start_server(&mut self, name: &str, config: McpServerConfig) -> Result<(), String> {
        if !config.command.is_empty() {
            let mut cmd = Command::new(&config.command);
            for arg in &config.args { cmd.arg(arg); }
            for (k, v) in &config.env { cmd.env(k, v); }

            // If no stdin/stdout capture needed, just manage lifecycle
            // with stderr visible for debugging
            cmd.stdin(Stdio::null())
               .stdout(Stdio::piped())
               .stderr(Stdio::inherit());

            let child = cmd.spawn().map_err(|e| format!("Failed to start '{}': {}", name, e))?;
            let instance = McpServerInstance {
                name: name.to_string(),
                transport: McpTransport::Stdio {
                    command: config.command.clone(),
                    args: config.args.clone(),
                },
                tools: Vec::new(),
                process: Some(child),
                healthy: true,
                last_ping: Instant::now(),
            };
            self.servers.insert(name.to_string(), instance);
            Ok(())
        } else if !config.url.is_empty() {
            let instance = McpServerInstance {
                name: name.to_string(),
                transport: McpTransport::Http { url: config.url.clone() },
                tools: Vec::new(),
                process: None,
                healthy: true,
                last_ping: Instant::now(),
            };
            self.servers.insert(name.to_string(), instance);
            Ok(())
        } else {
            Err(format!("No command or url for server '{}'", name))
        }
    }

    /// Stop an MCP server and kill its process
    pub fn stop_server(&mut self, name: &str) -> Result<(), String> {
        if let Some(mut instance) = self.servers.remove(name) {
            if let Some(mut child) = instance.process.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
            Ok(())
        } else {
            Err(format!("Server '{}' not found", name))
        }
    }

    /// Stop all managed servers
    pub fn stop_all(&mut self) {
        let names: Vec<String> = self.servers.keys().cloned().collect();
        for name in names {
            let _ = self.stop_server(&name);
        }
    }

    /// Restart a server by stopping then starting it
    pub fn restart_server(&mut self, name: &str, config: McpServerConfig) -> Result<(), String> {
        let _ = self.stop_server(name);
        self.start_server(name, config)
    }

    /// List all managed server names
    pub fn list_servers(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.servers.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Check if a specific server is healthy
    pub fn is_healthy(&self, name: &str) -> bool {
        self.servers.get(name).map(|s| s.healthy).unwrap_or(false)
    }

    /// Get registered tool names from all servers
    pub fn all_tools(&self) -> Vec<String> {
        let mut tools = Vec::new();
        for instance in self.servers.values() {
            tools.extend(instance.tools.clone());
        }
        tools.sort();
        tools
    }

    /// Get a server instance by name
    pub fn get_server(&self, name: &str) -> Option<&McpServerInstance> {
        self.servers.get(name)
    }

    /// Number of managed servers
    pub fn server_count(&self) -> usize {
        self.servers.len()
    }

    /// Health check all servers, returning names of unhealthy ones
    pub fn health_check(&mut self) -> Vec<String> {
        let mut issues = Vec::new();
        let names: Vec<String> = self.servers.keys().cloned().collect();
        for name in names {
            let should_check = self.servers.get(&name)
                .map(|s| !s.healthy && s.last_ping.elapsed() > Duration::from_secs(30))
                .unwrap_or(false);

            if should_check {
                // Try to restart unhealthy server
                issues.push(format!("{} unhealthy >30s", name));
                if let Some(instance) = self.servers.get(&name) {
                    let config = McpServerConfig {
                        command: match &instance.transport {
                            McpTransport::Stdio { command, .. } => command.clone(),
                            McpTransport::Http { .. } => String::new(),
                        },
                        args: match &instance.transport {
                            McpTransport::Stdio { args, .. } => args.clone(),
                            McpTransport::Http { .. } => Vec::new(),
                        },
                        url: match &instance.transport {
                            McpTransport::Http { url } => url.clone(),
                            McpTransport::Stdio { .. } => String::new(),
                        },
                        env: HashMap::new(),
                    };
                    let _ = self.restart_server(&name, config);
                }
            }
        }
        issues
    }

    /// Mark a server as unhealthy (e.g. after a failed ping)
    pub fn mark_unhealthy(&mut self, name: &str) {
        if let Some(server) = self.servers.get_mut(name) {
            server.healthy = false;
        }
    }

    /// Set tool list for a server
    pub fn set_tools(&mut self, name: &str, tools: Vec<String>) {
        if let Some(server) = self.servers.get_mut(name) {
            server.tools = tools;
        }
    }
}

impl Drop for McpFullClient {
    fn drop(&mut self) {
        self.stop_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client() {
        let client = McpFullClient::new();
        assert!(client.list_servers().is_empty());
        assert_eq!(client.server_count(), 0);
    }

    #[test]
    fn test_discover_no_file() {
        let mut client = McpFullClient::new();
        let result = client.discover_servers();
        assert!(result.is_ok());
    }

    #[test]
    fn test_start_http_server() {
        let mut client = McpFullClient::new();
        let config = McpServerConfig {
            command: String::new(),
            args: Vec::new(),
            url: "http://localhost:8080/mcp".into(),
            env: HashMap::new(),
        };
        assert!(client.start_server("test-http", config).is_ok());
        assert!(client.is_healthy("test-http"));
        let servers = client.list_servers();
        assert_eq!(servers, vec!["test-http"]);
    }

    #[test]
    fn test_start_http_server_get_server() {
        let mut client = McpFullClient::new();
        let config = McpServerConfig {
            command: String::new(),
            args: Vec::new(),
            url: "http://localhost:8080/mcp".into(),
            env: HashMap::new(),
        };
        client.start_server("srv", config).unwrap();
        let srv = client.get_server("srv");
        assert!(srv.is_some());
        assert_eq!(srv.unwrap().name, "srv");
    }

    #[test]
    fn test_stop_server() {
        let mut client = McpFullClient::new();
        let config = McpServerConfig {
            command: String::new(),
            args: Vec::new(),
            url: "http://localhost:8081/mcp".into(),
            env: HashMap::new(),
        };
        client.start_server("to-stop", config).unwrap();
        assert!(client.is_healthy("to-stop"));
        assert!(client.stop_server("to-stop").is_ok());
        assert!(!client.is_healthy("to-stop"));
    }

    #[test]
    fn test_stop_nonexistent() {
        let mut client = McpFullClient::new();
        assert!(client.stop_server("ghost").is_err());
    }

    #[test]
    fn test_stop_all() {
        let mut client = McpFullClient::new();
        client.start_server("a", McpServerConfig {
            command: String::new(), args: Vec::new(),
            url: "http://a/mcp".into(), env: HashMap::new(),
        }).unwrap();
        client.start_server("b", McpServerConfig {
            command: String::new(), args: Vec::new(),
            url: "http://b/mcp".into(), env: HashMap::new(),
        }).unwrap();
        assert_eq!(client.server_count(), 2);
        client.stop_all();
        assert_eq!(client.server_count(), 0);
    }

    #[test]
    fn test_all_tools_empty() {
        let client = McpFullClient::new();
        assert!(client.all_tools().is_empty());
    }

    #[test]
    fn test_all_tools_with_data() {
        let mut client = McpFullClient::new();
        let config = McpServerConfig {
            command: String::new(), args: Vec::new(),
            url: "http://t/mcp".into(), env: HashMap::new(),
        };
        client.start_server("t1", config).unwrap();
        client.set_tools("t1", vec!["tool_a".into(), "tool_b".into()]);
        let tools = client.all_tools();
        assert_eq!(tools.len(), 2);
        assert!(tools.contains(&"tool_a".to_string()));
    }

    #[test]
    fn test_health_check_returns_empty() {
        let mut client = McpFullClient::new();
        assert!(client.health_check().is_empty());
    }

    #[test]
    fn test_mark_unhealthy() {
        let mut client = McpFullClient::new();
        let config = McpServerConfig {
            command: String::new(), args: Vec::new(),
            url: "http://u/mcp".into(), env: HashMap::new(),
        };
        client.start_server("u", config).unwrap();
        assert!(client.is_healthy("u"));
        client.mark_unhealthy("u");
        assert!(!client.is_healthy("u"));
    }

    #[test]
    fn test_config_deserialize() {
        let json = r#"{"my-server":{"command":"node","args":["server.js"],"url":"","env":{"KEY":"val"}}}"#;
        let configs: HashMap<String, McpServerConfig> = serde_json::from_str(json).unwrap();
        assert_eq!(configs["my-server"].command, "node");
        assert_eq!(configs["my-server"].args, vec!["server.js"]);
        assert_eq!(configs["my-server"].env["KEY"], "val");
        assert!(configs["my-server"].url.is_empty());
    }

    #[test]
    fn test_config_deserialize_mcp_servers_nested() {
        let json = r#"{"mcpServers":{"srv1":{"command":"python","args":["-m","server"],"env":{}}}}"#;
        let value: Value = serde_json::from_str(json).unwrap();
        let configs = McpFullClient::parse_configs(&value);
        assert_eq!(configs.len(), 1);
        assert_eq!(configs["srv1"].command, "python");
    }

    #[test]
    fn test_config_deserialize_flat() {
        let json = r#"{"flat-srv":{"command":"deno","args":["run","mod.ts"],"env":{}}}"#;
        let value: Value = serde_json::from_str(json).unwrap();
        let configs = McpFullClient::parse_configs(&value);
        assert_eq!(configs.len(), 1);
        assert_eq!(configs["flat-srv"].command, "deno");
    }

    #[test]
    fn test_config_deserialize_empty() {
        let json = r#"{}"#;
        let value: Value = serde_json::from_str(json).unwrap();
        let configs = McpFullClient::parse_configs(&value);
        assert!(configs.is_empty());
    }

    #[test]
    fn test_config_deserialize_partial() {
        let json = r#"{"srv":{"command":"node","unknown_field":"ignore"}}"#;
        let value: Value = serde_json::from_str(json).unwrap();
        let configs = McpFullClient::parse_configs(&value);
        assert_eq!(configs.len(), 1);
        assert_eq!(configs["srv"].command, "node");
    }

    #[test]
    fn test_restart_server() {
        let mut client = McpFullClient::new();
        let config = McpServerConfig {
            command: String::new(), args: Vec::new(),
            url: "http://restart/mcp".into(), env: HashMap::new(),
        };
        client.start_server("r", config.clone()).unwrap();
        assert!(client.is_healthy("r"));
        assert!(client.restart_server("r", config).is_ok());
        assert!(client.is_healthy("r"));
    }

    #[test]
    fn test_start_fails_on_empty_config() {
        let mut client = McpFullClient::new();
        let config = McpServerConfig {
            command: String::new(),
            args: Vec::new(),
            url: String::new(),
            env: HashMap::new(),
        };
        let result = client.start_server("empty", config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No command or url"));
    }

    #[test]
    fn test_start_fails_on_bad_command() {
        let mut client = McpFullClient::new();
        let config = McpServerConfig {
            command: "nonexistent-command-xyz".into(),
            args: Vec::new(),
            url: String::new(),
            env: HashMap::new(),
        };
        let result = client.start_server("bad", config);
        assert!(result.is_err());
    }

    #[test]
    fn test_drop_impl_no_panic() {
        let mut client = McpFullClient::new();
        let config = McpServerConfig {
            command: String::new(), args: Vec::new(),
            url: "http://drop/mcp".into(), env: HashMap::new(),
        };
        client.start_server("d", config).unwrap();
        // Drop will be called at end of scope; just verify no panic
        assert!(client.is_healthy("d"));
    }

    #[test]
    fn test_health_check_bad_server_reported() {
        let mut client = McpFullClient::new();
        let config = McpServerConfig {
            command: String::new(), args: Vec::new(),
            url: "http://bad/mcp".into(), env: HashMap::new(),
        };
        client.start_server("bad", config).unwrap();
        client.mark_unhealthy("bad");

        // last_ping was just set, so health_check won't report it yet
        // (only reports after 30s). Verify it stays quiet initially.
        let issues = client.health_check();
        assert!(issues.is_empty());
    }
}
