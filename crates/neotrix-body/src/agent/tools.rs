//! # Tool Registry
//!
//! Central registry for executable tools that agents can invoke.

use std::collections::HashMap;

/// Tool definition
#[derive(Debug, Clone)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParam>,
    pub return_type: String,
}

#[derive(Debug, Clone)]
pub struct ToolParam {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,
}

/// Tool execution result
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

/// Tool trait
pub trait ToolExecutor: std::fmt::Debug + Send + Sync {
    fn execute(&self, tool: &str, args: &HashMap<String, String>) -> ToolResult;
    fn list_tools(&self) -> Vec<ToolDef>;
}

/// Tool registry — holds tool definitions and executors
#[derive(Debug)]
pub struct ToolRegistry {
    executors: Vec<Box<dyn ToolExecutor>>,
}

impl Clone for ToolRegistry {
    fn clone(&self) -> Self {
        Self {
            executors: Vec::new(),
        }
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            executors: Vec::new(),
        }
    }

    pub fn add_executor(&mut self, executor: Box<dyn ToolExecutor>) {
        self.executors.push(executor);
    }

    pub fn execute(&self, tool: &str, args: &HashMap<String, String>) -> Option<ToolResult> {
        for executor in &self.executors {
            let tools = executor.list_tools();
            if tools.iter().any(|t| t.name == tool) {
                return Some(executor.execute(tool, args));
            }
        }
        None
    }

    pub fn all_tools(&self) -> Vec<ToolDef> {
        self.executors.iter().flat_map(|e| e.list_tools()).collect()
    }

    pub fn len(&self) -> usize {
        self.executors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.executors.is_empty()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock tool executor for testing
#[derive(Debug, Clone)]
pub struct MockToolExecutor;

impl ToolExecutor for MockToolExecutor {
    fn execute(&self, tool: &str, _args: &HashMap<String, String>) -> ToolResult {
        ToolResult {
            success: true,
            output: format!("{tool} executed"),
            error: None,
        }
    }

    fn list_tools(&self) -> Vec<ToolDef> {
        vec![
            ToolDef {
                name: "read_file".into(),
                description: "Read a file".into(),
                parameters: vec![ToolParam {
                    name: "path".into(),
                    param_type: "string".into(),
                    required: true,
                    description: "file path".into(),
                }],
                return_type: "string".into(),
            },
            ToolDef {
                name: "search".into(),
                description: "Search the web".into(),
                parameters: vec![ToolParam {
                    name: "query".into(),
                    param_type: "string".into(),
                    required: true,
                    description: "search query".into(),
                }],
                return_type: "string".into(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_empty() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_tool_registry_execute() {
        let mut registry = ToolRegistry::new();
        registry.add_executor(Box::new(MockToolExecutor));
        let result = registry.execute("read_file", &HashMap::new());
        assert!(result.is_some());
        assert!(result.unwrap().success);
    }

    #[test]
    fn test_tool_registry_unknown_tool() {
        let registry = ToolRegistry::new();
        let result = registry.execute("nonexistent", &HashMap::new());
        assert!(result.is_none());
    }

    #[test]
    fn test_mock_tool_list() {
        let executor = MockToolExecutor;
        let tools = executor.list_tools();
        assert_eq!(tools.len(), 2);
    }
}
