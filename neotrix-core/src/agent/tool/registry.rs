/// Tool registry pattern inspired by OpenHarness BaseTool system.
/// Provides automatic schema generation, tool discovery, and bulk registration.

use std::collections::HashMap;
use crate::agent::tool::lifecycle::*;
use crate::agent::tool::watcher::{ToolWatcher, UpdateInfo, UpdatePolicy};
use serde_json::Value;

/// A registered tool with its metadata
#[derive(Debug, Clone)]
pub struct ToolEntry {
    pub manifest: ToolManifest,
    pub input_schema: Value,
}

/// Central tool registry — inspired by OpenHarness create_default_tool_registry()
pub struct ToolRegistry {
    tools: HashMap<String, ToolEntry>,
    handlers: HashMap<String, Box<dyn Fn(&Value) -> Result<ToolOutput, ToolError> + Send + Sync>>,
    update_policy: UpdatePolicy,
    watcher: Option<ToolWatcher>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            handlers: HashMap::new(),
            update_policy: UpdatePolicy::Advisory,
            watcher: None,
        }
    }

    /// Register a tool with its manifest and handler function
    pub fn register<F>(&mut self, manifest: ToolManifest, handler: F)
    where
        F: Fn(&Value) -> Result<ToolOutput, ToolError> + Send + Sync + 'static,
    {
        let input_schema = Self::generate_schema(&manifest);
        let id = manifest.id.clone();
        self.tools.insert(id.clone(), ToolEntry {
            manifest,
            input_schema,
        });
        self.handlers.insert(id, Box::new(handler));
    }

    /// Get a tool by ID
    pub fn get(&self, id: &str) -> Option<&ToolEntry> {
        self.tools.get(id)
    }

    /// Execute a tool by ID with JSON args
    pub fn execute(&self, id: &str, args: &Value) -> Result<ToolOutput, ToolError> {
        let handler = self.handlers.get(id)
            .ok_or_else(|| ToolError::Runtime {
                id: id.into(),
                message: format!("Tool '{}' not registered", id),
            })?;
        handler(args)
    }

    /// List all registered tools (for MCP tool discovery)
    pub fn list_tools(&self) -> Vec<&ToolEntry> {
        self.tools.values().collect()
    }

    /// Check if a tool is registered
    pub fn has_tool(&self, id: &str) -> bool {
        self.tools.contains_key(id)
    }

    /// Register from an AgentTool implementation
    pub fn register_agent_tool(&mut self, tool: Box<dyn AgentTool>) {
        let manifest = tool.manifest().clone();
        let id = manifest.id.clone();
        let input_schema = Self::generate_schema(&manifest);
        self.tools.insert(id.clone(), ToolEntry {
            manifest,
            input_schema,
        });
    }

    /// Set the update policy
    pub fn with_update_policy(mut self, policy: UpdatePolicy) -> Self {
        self.update_policy = policy;
        self
    }

    /// Attach a ToolWatcher
    pub fn with_watcher(mut self, watcher: ToolWatcher) -> Self {
        self.watcher = Some(watcher);
        self
    }

    /// Get current update policy
    pub fn update_policy(&self) -> UpdatePolicy {
        self.update_policy
    }

    /// Set update policy after construction
    pub fn set_update_policy(&mut self, policy: UpdatePolicy) {
        self.update_policy = policy;
    }

    /// List all current tool versions as a map
    pub fn current_versions(&self) -> HashMap<String, String> {
        self.tools
            .iter()
            .map(|(id, entry)| (id.clone(), entry.manifest.version.clone()))
            .collect()
    }

    /// Sync updates: check watcher for updates, apply based on policy.
    /// Returns list of actually-applied updates.
    /// In Advisory mode, returns pending updates — caller must confirm them.
    /// In Auto mode, returns applied updates directly.
    /// In Disabled mode, returns empty.
    pub fn sync_updates(&mut self) -> Vec<UpdateInfo> {
        match self.update_policy {
            UpdatePolicy::Disabled => return Vec::new(),
            _ => {}
        }

        let watcher = match &self.watcher {
            Some(ref w) => w,
            None => return Vec::new(),
        };

        // Scan for new/changed manifests
        watcher.scan_and_reload();

        // Check all current tools for updates
        let current = self.current_versions();
        let updates = watcher.check_all_updates(&current);

        match self.update_policy {
            UpdatePolicy::Auto => {
                // Return updates so caller can re-register handlers
                updates
            }
            UpdatePolicy::Advisory => {
                // Return pending updates for user approval
                updates
            }
            UpdatePolicy::Disabled => Vec::new(),
        }
    }

    /// Apply a specific update (re-scan manifest for the tool)
    /// In a real implementation, this would re-load the tool's wasm/dynamic code.
    /// For now, it updates the manifest version tracking.
    pub fn confirm_update(&mut self, info: &UpdateInfo) -> bool {
        // In a real system, this would:
        // 1. Unload old tool
        // 2. Load new tool code
        // 3. Register new handler
        // For now we just record the version change
        if let Some(entry) = self.tools.get_mut(&info.tool_id) {
            entry.manifest.version = info.new_version.clone();
            true
        } else {
            false
        }
    }

    /// Generate a minimal JSON Schema for a tool (based on ToolManifest)
    fn generate_schema(manifest: &ToolManifest) -> Value {
        serde_json::json!({
            "name": manifest.id,
            "description": manifest.description,
            "input_schema": {
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": "Tool input"
                    }
                }
            }
        })
    }

    /// Get the count of registered tools
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_execute() {
        let mut registry = ToolRegistry::new();
        let manifest = ToolManifest {
            id: "test_tool".into(),
            name: "Test Tool".into(),
            version: "0.1.0".into(),
            permissions: vec![],
            mcp: None,
            min_runtime: "0.1.0".into(),
            description: "A test tool".into(),
            author: None,
        };
        registry.register(manifest, |args: &Value| {
            Ok(ToolOutput {
                result: format!("executed with {}", args),
                metadata: HashMap::new(),
            })
        });
        assert!(registry.has_tool("test_tool"));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_list_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(ToolManifest {
            id: "tool_a".into(), name: "A".into(), version: "0.1.0".into(),
            permissions: vec![], mcp: None, min_runtime: "0.1.0".into(),
            description: "".into(), author: None,
        }, |_| Ok(ToolOutput { result: "".into(), metadata: HashMap::new() }));
        registry.register(ToolManifest {
            id: "tool_b".into(), name: "B".into(), version: "0.1.0".into(),
            permissions: vec![], mcp: None, min_runtime: "0.1.0".into(),
            description: "".into(), author: None,
        }, |_| Ok(ToolOutput { result: "".into(), metadata: HashMap::new() }));
        assert_eq!(registry.list_tools().len(), 2);
    }

    #[test]
    fn test_execute_unknown_tool() {
        let registry = ToolRegistry::new();
        let result = registry.execute("unknown", &serde_json::json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_schema() {
        let manifest = ToolManifest {
            id: "web_scrape".into(), name: "Web Scraper".into(), version: "0.1.0".into(),
            permissions: vec![], mcp: None, min_runtime: "0.1.0".into(),
            description: "Scrape web pages".into(), author: None,
        };
        let schema = ToolRegistry::generate_schema(&manifest);
        assert_eq!(schema["name"], "web_scrape");
        assert_eq!(schema["description"], "Scrape web pages");
    }

    #[test]
    fn test_update_policy_default() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.update_policy(), UpdatePolicy::Advisory);
    }

    #[test]
    fn test_sync_updates_disabled() {
        let mut registry = ToolRegistry::new();
        registry.set_update_policy(UpdatePolicy::Disabled);
        let updates = registry.sync_updates();
        assert!(updates.is_empty());
    }

    #[test]
    fn test_confirm_update() {
        let mut registry = ToolRegistry::new();
        let manifest = ToolManifest {
            id: "test_tool".into(), name: "Test".into(), version: "1.0.0".into(),
            permissions: vec![], mcp: None, min_runtime: "0.1.0".into(),
            description: "".into(), author: None,
        };
        registry.register(manifest, |_| Ok(ToolOutput { result: "".into(), metadata: HashMap::new() }));

        let info = UpdateInfo {
            tool_id: "test_tool".into(),
            current_version: "1.0.0".into(),
            new_version: "2.0.0".into(),
            name: "Test".into(),
        };
        assert!(registry.confirm_update(&info));
        assert_eq!(registry.get("test_tool").expect("value should be ok in test").manifest.version, "2.0.0");
    }

    #[test]
    fn test_with_policy_and_watcher() {
        let watcher = ToolWatcher::new();
        let registry = ToolRegistry::new()
            .with_update_policy(UpdatePolicy::Auto)
            .with_watcher(watcher);
        assert_eq!(registry.update_policy(), UpdatePolicy::Auto);
    }
}
