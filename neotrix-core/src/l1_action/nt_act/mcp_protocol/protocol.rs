//! MCP Protocol
//!
//! Top-level protocol struct that wires together tool and resource
//! registries, implements the `initialize` handshake, capability
//! discovery, and request routing.

use std::sync::Arc;

use super::resource_endpoint::{McpResourceEndpoint, McpResourceRegistry};
use super::tool_endpoint::{McpToolEndpoint, McpToolRegistry};

// ============================================================================
// MCP Protocol
// ============================================================================

/// MCP (Model Context Protocol) server-side implementation.
///
/// Holds the tool and resource registries, handles the `initialize`
/// handshake, and routes incoming JSON-RPC-style requests to the
/// correct handler.
pub struct McpProtocol {
    tools: McpToolRegistry,
    resources: McpResourceRegistry,
    version: String,
}

impl McpProtocol {
    /// Create a new protocol with the given MCP version string.
    pub fn new(version: &str) -> Self {
        Self {
            tools: McpToolRegistry::new(),
            resources: McpResourceRegistry::new(),
            version: version.to_string(),
        }
    }

    /// Register a tool endpoint.
    pub fn register_tool(&self, endpoint: Arc<dyn McpToolEndpoint>) {
        self.tools.register(endpoint);
    }

    /// Register a resource endpoint.
    pub fn register_resource(&self, endpoint: Arc<dyn McpResourceEndpoint>) {
        self.resources.register(endpoint);
    }

    /// Protocol version.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Reference to the tool registry.
    pub fn tools(&self) -> &McpToolRegistry {
        &self.tools
    }

    /// Reference to the resource registry.
    pub fn resources(&self) -> &McpResourceRegistry {
        &self.resources
    }

    /// Handle the MCP `initialize` request.
    ///
    /// Returns a capabilities object describing what this server supports.
    pub fn initialize(&self) -> serde_json::Value {
        let caps = self.list_capabilities();
        serde_json::json!({
            "protocolVersion": self.version,
            "capabilities": caps,
            "serverInfo": {
                "name": "neotrix-mcp",
                "version": env!("CARGO_PKG_VERSION"),
            }
        })
    }

    /// Discover and return the full capability map.
    pub fn list_capabilities(&self) -> serde_json::Value {
        serde_json::json!({
            "tools": {
                "listChanged": false,
            },
            "resources": {
                "listChanged": false,
                "subscribe": false,
            },
        })
    }

    /// Route an incoming JSON-RPC-style request.
    ///
    /// - `method` — the RPC method name (e.g. `"tools/list"`, `"tools/call"`,
    ///   `"resources/list"`, `"resources/read"`).
    /// - `params` — method parameters.
    ///
    /// Returns the result payload or an error object.
    pub fn handle_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match method {
            "initialize" => Ok(self.initialize()),
            "capabilities/list" | "capabilities" => Ok(self.list_capabilities()),
            "tools/list" => Ok(self.handle_tools_list()),
            "tools/call" => self.handle_tools_call(params),
            "resources/list" => Ok(self.handle_resources_list()),
            "resources/read" => self.handle_resources_read(params),
            _ => Err(format!("Unknown method: {}", method)),
        }
    }

    /// Handle `tools/list` — return registered tool descriptions.
    fn handle_tools_list(&self) -> serde_json::Value {
        let descriptions = self.tools.list_tools();
        serde_json::json!({ "tools": descriptions })
    }

    /// Handle `tools/call` — invoke a tool by name.
    fn handle_tools_call(&self, params: serde_json::Value) -> Result<serde_json::Value, String> {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("missing 'name' parameter")?;
        let input = params
            .get("arguments")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        match self.tools.call_tool(name, input) {
            Ok(result) => Ok(serde_json::json!({
                "content": [{ "type": "text", "text": result.to_string() }],
            })),
            Err(e) => Ok(serde_json::json!({
                "content": [{ "type": "text", "text": format!("error: {}", e) }],
                "isError": true,
            })),
        }
    }

    /// Handle `resources/list` — return registered resource descriptions.
    fn handle_resources_list(&self) -> serde_json::Value {
        let descriptions = self.resources.list_resources();
        serde_json::json!({ "resources": descriptions })
    }

    /// Handle `resources/read` — read a resource by URI.
    fn handle_resources_read(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let uri = params
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or("missing 'uri' parameter")?;

        match self.resources.read_resource(uri) {
            Ok(content) => Ok(serde_json::json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "text/plain",
                    "text": content,
                }],
            })),
            Err(e) => Err(e),
        }
    }
}

impl Default for McpProtocol {
    fn default() -> Self {
        Self::new("2025-03-26")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct AddTool;

    impl McpToolEndpoint for AddTool {
        fn name(&self) -> &str {
            "add"
        }
        fn description(&self) -> &str {
            "Adds two numbers"
        }
        fn input_schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "a": { "type": "number" },
                    "b": { "type": "number" }
                },
                "required": ["a", "b"]
            })
        }
        fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, String> {
            let a = input.get("a").and_then(|v| v.as_f64()).ok_or("missing a")?;
            let b = input.get("b").and_then(|v| v.as_f64()).ok_or("missing b")?;
            Ok(serde_json::json!({ "result": a + b }))
        }
    }

    struct ConfigResource;

    impl McpResourceEndpoint for ConfigResource {
        fn uri(&self) -> &str {
            "config://app"
        }
        fn name(&self) -> &str {
            "app-config"
        }
        fn mime_type(&self) -> &str {
            "application/json"
        }
        fn read(&self) -> Result<String, String> {
            Ok(r#"{"debug": true}"#.into())
        }
    }

    #[test]
    fn test_initialize() {
        let proto = McpProtocol::new("2025-03-26");
        let init = proto.initialize();
        assert_eq!(init["protocolVersion"], "2025-03-26");
        assert_eq!(init["serverInfo"]["name"], "neotrix-mcp");
    }

    #[test]
    fn test_default_version() {
        let proto = McpProtocol::default();
        assert_eq!(proto.version(), "2025-03-26");
    }

    #[test]
    fn test_tools_list_empty() {
        let proto = McpProtocol::new("1.0");
        let result = proto.handle_request("tools/list", serde_json::json!({}));
        let tools = result.unwrap()["tools"].as_array().unwrap().clone();
        assert!(tools.is_empty());
    }

    #[test]
    fn test_tools_list_with_registered() {
        let proto = McpProtocol::new("1.0");
        proto.register_tool(Arc::new(AddTool));

        let result = proto.handle_request("tools/list", serde_json::json!({}));
        let tools = result.unwrap()["tools"].as_array().unwrap().clone();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "add");
    }

    #[test]
    fn test_tools_call_success() {
        let proto = McpProtocol::new("1.0");
        proto.register_tool(Arc::new(AddTool));

        let result = proto
            .handle_request(
                "tools/call",
                serde_json::json!({
                    "name": "add",
                    "arguments": { "a": 3, "b": 5 }
                }),
            )
            .unwrap();
        assert!(result["content"][0]["text"].as_str().unwrap().contains("8"));
    }

    #[test]
    fn test_tools_call_missing_name() {
        let proto = McpProtocol::new("1.0");
        let result = proto.handle_request("tools/call", serde_json::json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_resources_list_empty() {
        let proto = McpProtocol::new("1.0");
        let result = proto.handle_request("resources/list", serde_json::json!({}));
        let resources = result.unwrap()["resources"].as_array().unwrap().clone();
        assert!(resources.is_empty());
    }

    #[test]
    fn test_resources_list_with_registered() {
        let proto = McpProtocol::new("1.0");
        proto.register_resource(Arc::new(ConfigResource));

        let result = proto.handle_request("resources/list", serde_json::json!({}));
        let resources = result.unwrap()["resources"].as_array().unwrap().clone();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0]["uri"], "config://app");
    }

    #[test]
    fn test_resources_read_success() {
        let proto = McpProtocol::new("1.0");
        proto.register_resource(Arc::new(ConfigResource));

        let result = proto
            .handle_request(
                "resources/read",
                serde_json::json!({ "uri": "config://app" }),
            )
            .unwrap();
        let text = result["contents"][0]["text"].as_str().unwrap();
        assert!(text.contains("debug"));
    }

    #[test]
    fn test_resources_read_not_found() {
        let proto = McpProtocol::new("1.0");
        let result = proto.handle_request(
            "resources/read",
            serde_json::json!({ "uri": "missing://x" }),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_method() {
        let proto = McpProtocol::new("1.0");
        let result = proto.handle_request("unknown/method", serde_json::json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown method"));
    }

    #[test]
    fn test_capabilities() {
        let proto = McpProtocol::new("1.0");
        let caps = proto.list_capabilities();
        assert!(caps.is_object());
    }
}
