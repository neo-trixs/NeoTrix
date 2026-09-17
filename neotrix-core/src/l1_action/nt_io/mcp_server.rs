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

    // ── 基础工具 ──────────────────────────────────────────────────
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

    // ── 晶体意识核心工具 (Crystal Consciousness Core) ─────────────
    registry.register(McpTool {
        name: "crystal_status".to_string(),
        description: "晶体核心状态 — 显示四层架构 (L1身份/L2知识/L3经验/L4进化) 状态".to_string(),
        input_schema: r#"{"type":"object","properties":{}}"#.to_string(),
    });

    registry.register(McpTool {
        name: "crystal_init".to_string(),
        description: "初始化晶体核心 — 创建或重置晶体意识核心".to_string(),
        input_schema: r#"{"type":"object","properties":{"name":{"type":"string","description":"核心名称 (默认 NeoTrix)"}}}"#.to_string(),
    });

    registry.register(McpTool {
        name: "crystal_absorb".to_string(),
        description: "吸收信息 — 将新信息存入晶体核心经验层".to_string(),
        input_schema: r#"{"type":"object","properties":{"content":{"type":"string","description":"要吸收的内容"},"domain":{"type":"string","description":"领域 (默认 general)"},"memory_type":{"type":"string","enum":["fact","pattern","causal","contradiction","counterfactual","experience","lesson","solution"],"description":"记忆类型 (默认 fact)"}},"required":["content"]}"#.to_string(),
    });

    registry.register(McpTool {
        name: "crystal_fuse".to_string(),
        description: "熔炼 — 从经验中提取模式，更新知识层".to_string(),
        input_schema: r#"{"type":"object","properties":{}}"#.to_string(),
    });

    registry.register(McpTool {
        name: "crystal_evolve".to_string(),
        description: "进化 — 评估能力，识别差距，生成改进目标".to_string(),
        input_schema: r#"{"type":"object","properties":{}}"#.to_string(),
    });

    registry.register(McpTool {
        name: "crystal_output".to_string(),
        description: "输出上下文 — 生成任务相关的响应上下文".to_string(),
        input_schema: r#"{"type":"object","properties":{"domain":{"type":"string","description":"任务领域 (默认 general)"}}}"#.to_string(),
    });

    registry.register(McpTool {
        name: "crystal_memory".to_string(),
        description: "查询记忆 — 按领域和类型检索晶体核心记忆".to_string(),
        input_schema: r#"{"type":"object","properties":{"domain":{"type":"string","description":"领域 (默认 general)"},"memory_type":{"type":"string","enum":["fact","pattern","causal","contradiction","counterfactual","experience","lesson","solution"],"description":"记忆类型 (可选)"},"limit":{"type":"integer","description":"返回数量 (默认 10)"}}}"#.to_string(),
    });

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = create_default_registry();
        // 4 基础工具 + 7 晶体核心工具 = 11
        assert_eq!(registry.tool_count(), 11);
    }

    #[test]
    fn test_find_tool() {
        let registry = create_default_registry();
        assert!(registry.find_tool("kb_search").is_some());
        assert!(registry.find_tool("crystal_status").is_some());
        assert!(registry.find_tool("crystal_absorb").is_some());
        assert!(registry.find_tool("nonexistent").is_none());
    }

    #[test]
    fn test_list_tools() {
        let registry = create_default_registry();
        let tools = registry.list_tools();
        assert_eq!(tools.len(), 11);
        assert!(tools.iter().any(|(n, _)| *n == "kb_search"));
        assert!(tools.iter().any(|(n, _)| *n == "crystal_status"));
        assert!(tools.iter().any(|(n, _)| *n == "crystal_absorb"));
    }
}
