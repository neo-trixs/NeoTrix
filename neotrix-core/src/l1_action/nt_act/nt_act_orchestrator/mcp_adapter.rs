use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct McpRequest {
    pub tool: String,
    pub arguments: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct McpResponse {
    pub content: String,
    pub is_error: bool,
}

pub struct McpAdapter {
    tools: Vec<McpTool>,
}

impl McpAdapter {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    pub fn register_tool(&mut self, tool: McpTool) {
        self.tools.push(tool);
    }

    pub fn list_tools(&self) -> Vec<&McpTool> {
        self.tools.iter().collect()
    }

    pub fn call(&self, req: &McpRequest) -> Result<McpResponse, String> {
        if !self.tools.iter().any(|t| t.name == req.tool) {
            return Err(format!("Tool not found: {}", req.tool));
        }
        Ok(McpResponse {
            content: format!("Called {}", req.tool),
            is_error: false,
        })
    }

    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }
}

impl Default for McpAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_call() {
        let mut a = McpAdapter::new();
        a.register_tool(McpTool {
            name: "search".into(),
            description: "Search".into(),
            input_schema: HashMap::new(),
        });
        let r = a
            .call(&McpRequest {
                tool: "search".into(),
                arguments: HashMap::new(),
            })
            .unwrap();
        assert!(!r.is_error);
    }

    #[test]
    fn test_not_found() {
        let a = McpAdapter::new();
        assert!(a
            .call(&McpRequest {
                tool: "missing".into(),
                arguments: HashMap::new(),
            })
            .is_err());
    }
}
