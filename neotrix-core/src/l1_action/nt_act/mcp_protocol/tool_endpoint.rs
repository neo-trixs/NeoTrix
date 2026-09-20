//! MCP Tool Endpoint
//!
//! Tool trait and registry for the Model Context Protocol.
//! Each tool exposes metadata and an execute handler; the registry
//! performs lookup and dispatch.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// ============================================================================
// Trait
// ============================================================================

/// A single MCP tool endpoint.
pub trait McpToolEndpoint: Send + Sync + 'static {
    /// Unique tool name (e.g. "file.read").
    fn name(&self) -> &str;

    /// Human-readable description.
    fn description(&self) -> &str;

    /// JSON Schema describing the expected input.
    fn input_schema(&self) -> serde_json::Value;

    /// Execute the tool with the given input and return the result.
    fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, String>;
}

// ============================================================================
// Tool Description (serializable snapshot)
// ============================================================================

/// Serialized description of a tool for listing.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDescription {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

// ============================================================================
// Registry
// ============================================================================

/// Thread-safe registry of MCP tool endpoints.
pub struct McpToolRegistry {
    tools: RwLock<HashMap<String, Arc<dyn McpToolEndpoint>>>,
}

impl McpToolRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
        }
    }

    /// Register a tool. Overwrites any existing tool with the same name.
    pub fn register(&self, endpoint: Arc<dyn McpToolEndpoint>) {
        let tools = self.tools.write().map_err(|e| e.to_string());
        if let Ok(mut map) = tools {
            map.insert(endpoint.name().to_string(), endpoint);
        }
    }

    /// List all registered tools as serializable descriptions.
    pub fn list_tools(&self) -> Vec<ToolDescription> {
        let tools = self.tools.read().map_err(|e| e.to_string());
        match tools {
            Ok(map) => map
                .values()
                .map(|t| ToolDescription {
                    name: t.name().to_string(),
                    description: t.description().to_string(),
                    input_schema: t.input_schema(),
                })
                .collect(),
            Err(_) => vec![],
        }
    }

    /// Call a tool by name with the given input.
    pub fn call_tool(
        &self,
        name: &str,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let endpoint = {
            let tools = self.tools.read().map_err(|e| e.to_string())?;
            tools
                .get(name)
                .cloned()
                .ok_or_else(|| format!("Tool not found: {}", name))?
        };
        endpoint.execute(input)
    }

    /// Number of registered tools.
    pub fn tool_count(&self) -> usize {
        self.tools.read().map(|m| m.len()).unwrap_or(0)
    }
}

impl Default for McpToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct EchoTool;

    impl McpToolEndpoint for EchoTool {
        fn name(&self) -> &str {
            "echo"
        }
        fn description(&self) -> &str {
            "Echoes input"
        }
        fn input_schema(&self) -> serde_json::Value {
            json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                },
                "required": ["message"]
            })
        }
        fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, String> {
            let msg = input
                .get("message")
                .and_then(|v| v.as_str())
                .ok_or("missing 'message' field")?;
            Ok(json!({ "echo": msg }))
        }
    }

    struct FailTool;

    impl McpToolEndpoint for FailTool {
        fn name(&self) -> &str {
            "fail"
        }
        fn description(&self) -> &str {
            "Always fails"
        }
        fn input_schema(&self) -> serde_json::Value {
            json!({ "type": "object" })
        }
        fn execute(&self, _input: serde_json::Value) -> Result<serde_json::Value, String> {
            Err("intentional failure".into())
        }
    }

    #[test]
    fn test_register_and_call() {
        let registry = McpToolRegistry::new();
        assert_eq!(registry.tool_count(), 0);

        registry.register(Arc::new(EchoTool));
        assert_eq!(registry.tool_count(), 1);

        let result = registry
            .call_tool("echo", json!({ "message": "hello" }))
            .unwrap();
        assert_eq!(result, json!({ "echo": "hello" }));
    }

    #[test]
    fn test_list_tools() {
        let registry = McpToolRegistry::new();
        registry.register(Arc::new(EchoTool));
        registry.register(Arc::new(FailTool));

        let tools = registry.list_tools();
        assert_eq!(tools.len(), 2);

        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"echo"));
        assert!(names.contains(&"fail"));
    }

    #[test]
    fn test_call_tool_not_found() {
        let registry = McpToolRegistry::new();
        let result = registry.call_tool("missing", json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_call_tool_failure_propagation() {
        let registry = McpToolRegistry::new();
        registry.register(Arc::new(FailTool));
        let result = registry.call_tool("fail", json!({}));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "intentional failure");
    }

    #[test]
    fn test_overwrite_registration() {
        let registry = McpToolRegistry::new();
        registry.register(Arc::new(EchoTool));
        registry.register(Arc::new(EchoTool));
        assert_eq!(registry.tool_count(), 1);
    }
}
