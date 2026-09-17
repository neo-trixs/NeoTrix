//! MCP Bridge — Model Context Protocol 桥接
//!
//! 暴露 NeoTrix 工具给外部 MCP 客户端
//! 调用外部 MCP 服务器的工具

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// MCP 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// MCP 工具结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResult {
    pub call_id: String,
    pub content: String,
    pub is_error: bool,
}

/// MCP Bridge — 管理工具注册和调用
pub struct McpBridge {
    /// 已注册的本地工具 (供外部调用)
    local_tools: HashMap<String, McpTool>,
    /// 外部 MCP 服务器
    servers: HashMap<String, McpServer>,
}

/// 外部 MCP 服务器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub url: String,
    pub transport: McpTransport,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpTransport {
    Stdio {
        command: String,
        args: Vec<String>,
    },
    Http {
        url: String,
    },
    Sse {
        url: String,
    },
}

impl McpBridge {
    pub fn new() -> Self {
        Self {
            local_tools: HashMap::new(),
            servers: HashMap::new(),
        }
    }

    /// 注册本地工具
    pub fn register_local_tool(&mut self, tool: McpTool) {
        self.local_tools.insert(tool.name.clone(), tool);
    }

    /// 列出本地工具
    pub fn list_local_tools(&self) -> Vec<&McpTool> {
        self.local_tools.values().collect()
    }

    /// 添加外部 MCP 服务器
    pub fn add_server(&mut self, server: McpServer) {
        self.servers.insert(server.id.clone(), server);
    }

    /// 列出外部服务器
    pub fn list_servers(&self) -> Vec<&McpServer> {
        self.servers.values().collect()
    }

    /// 调用本地工具 (供外部 MCP 客户端)
    pub fn call_local_tool(&self, call: &McpToolCall) -> McpToolResult {
        match self.local_tools.get(&call.name) {
            Some(_tool) => {
                // 实际调用逻辑需要根据工具类型实现
                // 这里返回一个示例结果
                McpToolResult {
                    call_id: call.id.clone(),
                    content: format!("Tool '{}' executed successfully", call.name),
                    is_error: false,
                }
            }
            None => McpToolResult {
                call_id: call.id.clone(),
                content: format!("Tool '{}' not found", call.name),
                is_error: true,
            },
        }
    }
}

impl Default for McpBridge {
    fn default() -> Self {
        let mut bridge = Self::new();

        // 预注册 NeoTrix 核心工具
        bridge.register_local_tool(McpTool {
            name: "kb_query".to_string(),
            description: "知识库查询 — 从知识库中检索信息".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "查询内容"
                    }
                },
                "required": ["query"]
            }),
        });

        bridge.register_local_tool(McpTool {
            name: "kb_search".to_string(),
            description: "知识库搜索 — 在知识库中搜索相关信息".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "keyword": {
                        "type": "string",
                        "description": "搜索关键词"
                    }
                },
                "required": ["keyword"]
            }),
        });

        bridge.register_local_tool(McpTool {
            name: "experience_query".to_string(),
            description: "经验查询 — 查询历史经验记录".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "查询条件"
                    }
                },
                "required": ["query"]
            }),
        });

        bridge.register_local_tool(McpTool {
            name: "agent_list".to_string(),
            description: "列出 agents — 显示所有可用的 agents".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        });

        bridge
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_local_tool() {
        let mut bridge = McpBridge::new();
        let tool = McpTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            parameters: serde_json::json!({}),
        };

        bridge.register_local_tool(tool);
        assert_eq!(bridge.local_tools.len(), 1);
        assert!(bridge.local_tools.contains_key("test_tool"));
    }

    #[test]
    fn test_list_tools() {
        let mut bridge = McpBridge::new();
        let tool1 = McpTool {
            name: "tool1".to_string(),
            description: "Tool 1".to_string(),
            parameters: serde_json::json!({}),
        };
        let tool2 = McpTool {
            name: "tool2".to_string(),
            description: "Tool 2".to_string(),
            parameters: serde_json::json!({}),
        };

        bridge.register_local_tool(tool1);
        bridge.register_local_tool(tool2);

        let tools = bridge.list_local_tools();
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn test_add_server() {
        let mut bridge = McpBridge::new();
        let server = McpServer {
            id: "server1".to_string(),
            name: "Test Server".to_string(),
            url: "http://localhost:8080".to_string(),
            transport: McpTransport::Http {
                url: "http://localhost:8080".to_string(),
            },
            available: true,
        };

        bridge.add_server(server);
        assert_eq!(bridge.servers.len(), 1);
        assert!(bridge.servers.contains_key("server1"));
    }

    #[test]
    fn test_call_local_tool() {
        let mut bridge = McpBridge::new();
        let tool = McpTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            parameters: serde_json::json!({}),
        };

        bridge.register_local_tool(tool);

        let call = McpToolCall {
            id: "call1".to_string(),
            name: "test_tool".to_string(),
            arguments: serde_json::json!({}),
        };

        let result = bridge.call_local_tool(&call);
        assert_eq!(result.call_id, "call1");
        assert!(!result.is_error);
        assert!(result.content.contains("test_tool"));
    }
}
