use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct McpCallback {
    pub tool_name: String,
    pub handler: String,
    pub schema: String,
}

#[derive(Debug, Clone)]
pub struct McpCallbackBridge {
    pub callbacks: HashMap<String, McpCallback>,
}

impl McpCallbackBridge {
    pub fn new() -> Self {
        Self {
            callbacks: HashMap::new(),
        }
    }
    pub fn register(&mut self, tool: &str, handler: &str, schema: &str) {
        self.callbacks.insert(
            tool.into(),
            McpCallback {
                tool_name: tool.into(),
                handler: handler.into(),
                schema: schema.into(),
            },
        );
    }
    pub fn invoke(&self, tool: &str) -> Option<&str> {
        self.callbacks.get(tool).map(|c| c.handler.as_str())
    }
    pub fn list_tools(&self) -> Vec<&str> {
        let mut v: Vec<&str> = self.callbacks.keys().map(|k| k.as_str()).collect();
        v.sort();
        v
    }
}
