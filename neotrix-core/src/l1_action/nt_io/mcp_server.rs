//! MCP (Model Context Protocol) 服务器模块
//!
//! 将 NT 核心能力暴露为 MCP 工具，供外部 agent 调用。

/// MCP 工具定义
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: String,
}

/// 工具调用结果
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

/// MCP 工具注册表
pub struct McpToolRegistry {
    tools: Vec<McpTool>,
}

impl McpToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    /// 注册工具
    pub fn register(&mut self, tool: McpTool) {
        self.tools.push(tool);
    }

    /// 列出所有工具名称和描述
    pub fn list_tools(&self) -> Vec<(&str, &str)> {
        self.tools.iter().map(|t| (t.name.as_str(), t.description.as_str())).collect()
    }

    /// 按名称查找工具
    pub fn find_tool(&self, name: &str) -> Option<&McpTool> {
        self.tools.iter().find(|t| t.name == name)
    }

    /// 工具数量
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }
}

/// 创建默认工具注册表
pub fn create_default_registry() -> McpToolRegistry {
    let mut registry = McpToolRegistry::new();

    registry.register(McpTool {
        name: "kb_search".to_string(),
        description: "搜索知识库 — 在 KB 中检索相关信息".to_string(),
        input_schema: r#"{"type":"object","properties":{"query":{"type":"string","description":"搜索查询"}},"required":["query"]}"#.to_string(),
    });

    registry.register(McpTool {
        name: "memory_search".to_string(),
        description: "搜索记忆 — 在经验库中检索相关记忆".to_string(),
        input_schema: r#"{"type":"object","properties":{"keyword":{"type":"string","description":"关键词"}},"required":["keyword"]}"#.to_string(),
    });

    registry.register(McpTool {
        name: "skill_route".to_string(),
        description: "技能路由 — 根据任务类型选择合适的技能".to_string(),
        input_schema: r#"{"type":"object","properties":{"task":{"type":"string","description":"任务描述"}},"required":["task"]}"#.to_string(),
    });

    registry.register(McpTool {
        name: "context_manage".to_string(),
        description: "上下文管理 — 管理 LLM 上下文窗口".to_string(),
        input_schema: r#"{"type":"object","properties":{"action":{"type":"string","enum":["compress","expand","summarize"]}},"required":["action"]}"#.to_string(),
    });

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = create_default_registry();
        assert_eq!(registry.tool_count(), 4);
    }

    #[test]
    fn test_find_tool() {
        let registry = create_default_registry();
        assert!(registry.find_tool("kb_search").is_some());
        assert!(registry.find_tool("nonexistent").is_none());
    }

    #[test]
    fn test_list_tools() {
        let registry = create_default_registry();
        let tools = registry.list_tools();
        assert_eq!(tools.len(), 4);
        assert!(tools.iter().any(|(n, _)| *n == "kb_search"));
    }
}
