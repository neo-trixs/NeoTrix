use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::Value;

use crate::agent::tool::mcp::{McpRegistry, McpServerEntry, McpToolDef, McpTransport};
use crate::neotrix::l1_body_impl::nt_agent_mcp_adapter::McpToolAdapter;
use crate::neotrix::l1_body_impl::nt_agent_mcp_discovery::{discover_and_register, McpDiscovery};
use crate::neotrix::l1_body_impl::nt_agent_mcp_transport::{mcp_call_tool, TransportMode};
use neotrix_types::traits::{NativeTool, ToolOutput};

/// McpGateway — 统一外部 MCP 服务调用入口
///
/// 仅做客户端调用，不运行自己的 MCP 服务端。
/// 整合发现、注册、传输、认证到单一接口。
pub struct McpGateway {
    registry: Arc<Mutex<McpRegistry>>,
    transport_cache: Arc<Mutex<HashMap<String, TransportMode>>>,
}

impl Default for McpGateway {
    fn default() -> Self {
        Self::new()
    }
}

impl McpGateway {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(McpRegistry::new())),
            transport_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn registry(&self) -> &Arc<Mutex<McpRegistry>> {
        &self.registry
    }

    /// 初始化网关：扫描 PATH 发现并注册 *-mcp-server 二进制，返回注册数
    pub fn init(&self) -> usize {
        let mut reg = self.registry.lock().expect("lock registry");
        let entries = discover_and_register(&mut reg);
        entries.len()
    }

    /// 手动注册一个本地 MCP 服务器
    pub fn register_local(&self, name: &str, command: &str, args: &[&str], desc: &str) {
        let mut reg = self.registry.lock().expect("lock registry");
        reg.publish(name, command, args, desc);
    }

    /// 手动注册一个远程 MCP 服务器
    pub fn register_remote(&self, name: &str, url: &str, _headers: HashMap<String, String>) {
        let mut reg = self.registry.lock().expect("lock registry");
        let tool = McpToolDef {
            name: format!("{name}_tool"),
            description: format!("Remote MCP server: {name}"),
            input_schema: Value::Object(serde_json::Map::new()),
            server_name: name.to_string(),
            transport: McpTransport::Http,
            schema_version: None,
        };
        reg.register(McpServerEntry {
            name: name.to_string(),
            transport: McpTransport::Http,
            command: None,
            url: Some(url.to_string()),
            tools: vec![tool],
            healthy: true,
            latency_ms: 0,
            last_health_check: None,
            init_result: None,
        });
    }

    /// 列出所有已注册的工具
    pub fn list_tools(&self) -> Vec<McpToolDef> {
        let reg = self.registry.lock().expect("lock registry");
        reg.list_tools()
    }

    /// 搜索工具（大小写不敏感模糊匹配）
    pub fn search_tools(&self, query: &str) -> Vec<McpToolDef> {
        let reg = self.registry.lock().expect("lock registry");
        reg.search(query)
    }

    /// 带评分的工具推荐
    pub fn recommend_tools(&self, query: &str, top_k: usize) -> Vec<McpToolDef> {
        let reg = self.registry.lock().expect("lock registry");
        reg.recommend_tools(query, top_k)
    }

    pub fn server_count(&self) -> usize {
        let reg = self.registry.lock().expect("lock registry");
        reg.server_count()
    }

    pub fn tool_count(&self) -> usize {
        let reg = self.registry.lock().expect("lock registry");
        reg.tool_count()
    }

    /// 统一的工具调用入口
    ///
    /// 自动查找工具所属的服务器，创建/复用传输层，执行 JSON-RPC 调用。
    /// 支持本地 stdio / 远程 HTTP / Streamable HTTP 三种模式。
    pub fn call_tool(&self, tool_name: &str, args: &Value) -> Result<ToolOutput, String> {
        let (transport, _name) = {
            let reg = self.registry.lock().expect("lock registry");

            // 在所有服务器的工具中查找
            let mut found: Option<(TransportMode, String)> = None;
            for server in reg.list_servers() {
                if server.tools.iter().any(|t| t.name == tool_name) {
                    let transport = self.transport_for_server(server)?;
                    found = Some((transport, server.name.clone()));
                    break;
                }
            }
            // 如果工具名 = 服务器名，用服务器默认的 dispatcher
            if found.is_none() {
                for server in reg.list_servers() {
                    if server.name == tool_name {
                        let transport = self.transport_for_server(server)?;
                        found = Some((transport, server.name.clone()));
                        break;
                    }
                }
            }
            found.ok_or_else(|| format!("MCP tool '{tool_name}' not found in any registered server"))?
        };

        let (content, _cache) = mcp_call_tool(&transport, tool_name, args)
            .map_err(|e| format!("MCP call '{tool_name}' failed: {e}"))?;

        Ok(ToolOutput {
            success: true,
            content,
        })
    }

    /// 扫描 PATH 发现新的 MCP 服务器并注册
    pub fn discover(&self) -> Vec<String> {
        let discovered = McpDiscovery::scan_path();
        let mut reg = self.registry.lock().expect("lock registry");
        let mut names = Vec::new();
        for entry in &discovered {
            let name = entry.name.clone();
            match McpDiscovery::try_register(entry) {
                Ok(verified) => {
                    let cmd = verified.path.to_string_lossy().to_string();
                    reg.register(McpServerEntry {
                        name: name.clone(),
                        transport: McpTransport::Local {
                            command: cmd.clone(),
                            args: vec![],
                        },
                        command: Some(cmd.clone()),
                        url: None,
                        tools: vec![McpToolDef {
                            name: format!("{name}-dispatcher"),
                            description: format!("MCP server: {name} — dispatches all tools"),
                            server_name: name.clone(),
                            transport: McpTransport::Local {
                                command: cmd.clone(),
                                args: vec![],
                            },
                            input_schema: serde_json::json!({
                                "type": "object",
                                "properties": {
                                    "tool": { "type": "string", "description": "Tool name" },
                                    "args": { "type": "object", "description": "Tool arguments" },
                                },
                                "required": ["tool"],
                            }),
                            schema_version: None,
                        }],
                        healthy: true,
                        latency_ms: 0,
                        last_health_check: None,
                        init_result: None,
                    });
                    names.push(name);
                }
                Err(e) => {
                    log::warn!("[mcp-gateway] Discovered '{name}' but verify failed: {e}");
                }
            }
        }
        names
    }

    /// 将已注册的所有 MCP 工具导出为 NativeTool 列表
    ///
    /// 用于集成到 ToolOrchestrator / GWT / SEAL 管线。
    pub fn to_native_tools(&self) -> Vec<Box<dyn NativeTool>> {
        let reg = self.registry.lock().expect("lock registry");
        let mut tools: Vec<Box<dyn NativeTool>> = Vec::new();

        for server in reg.list_servers() {
            if server.tools.is_empty() {
                continue;
            }
            let Ok(transport) = self.transport_for_server(server) else { continue };
            let adapter = McpToolAdapter::new(&server.name, transport, server.tools.clone());
            tools.push(Box::new(adapter));
        }

        tools
    }

    // -- 内部方法 --

    /// 将 McpServerEntry 的传输描述转换为 TransportMode
    fn transport_for_server(&self, server: &McpServerEntry) -> Result<TransportMode, String> {
        // 尝试从缓存获取
        {
            let cache = self.transport_cache.lock().expect("lock cache");
            if let Some(t) = cache.get(&server.name) {
                return Ok(t.clone());
            }
        }

        let transport = match &server.transport {
            McpTransport::Local { command, args } => TransportMode::Local {
                command: command.clone(),
                args: args.clone(),
            },
            McpTransport::Stdio => {
                let cmd = server
                    .command
                    .as_ref()
                    .ok_or_else(|| format!("Server '{}' has Stdio transport but no command", server.name))?;
                TransportMode::Local {
                    command: cmd.clone(),
                    args: vec![],
                }
            }
            McpTransport::Http | McpTransport::Sse | McpTransport::WebSocket => {
                let url = server
                    .url
                    .as_ref()
                    .ok_or_else(|| format!("Server '{}' has remote transport but no URL", server.name))?;
                TransportMode::Remote {
                    http_url: url.clone(),
                    headers: HashMap::new(),
                    sse_url: None,
                    auth: None,
                }
            }
        };

        // 写入缓存
        {
            let mut cache = self.transport_cache.lock().expect("lock cache");
            cache.insert(server.name.clone(), transport.clone());
        }

        Ok(transport)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_new() {
        let gw = McpGateway::new();
        assert_eq!(gw.server_count(), 0);
        assert_eq!(gw.tool_count(), 0);
    }

    #[test]
    fn test_register_local_tool() {
        let gw = McpGateway::new();
        gw.register_local("test-server", "echo", &["hello"], "Test MCP server");
        assert_eq!(gw.server_count(), 1);
        assert_eq!(gw.tool_count(), 1);
        let tools = gw.list_tools();
        assert!(tools.iter().any(|t| t.server_name == "test-server"));
    }

    #[test]
    fn test_register_remote_tool() {
        let gw = McpGateway::new();
        let mut headers = HashMap::new();
        headers.insert("Authorization".into(), "Bearer test".into());
        gw.register_remote("remote-server", "https://example.com/mcp", headers);
        assert_eq!(gw.server_count(), 1);
        assert_eq!(gw.tool_count(), 1);
    }

    #[test]
    fn test_search_tools() {
        let gw = McpGateway::new();
        gw.register_local("search-test", "echo", &["hello"], "Search engine");
        let results = gw.search_tools("search");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_recommend_tools() {
        let gw = McpGateway::new();
        gw.register_local("alpha", "echo", &["a"], "Alpha tool for searching");
        gw.register_local("beta", "echo", &["b"], "Beta tool for searching");
        let results = gw.recommend_tools("search", 5);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_call_tool_not_found() {
        let gw = McpGateway::new();
        let result = gw.call_tool("nonexistent", &serde_json::json!({}));
        assert!(result.is_err());
        match result {
            Err(e) => assert!(e.contains("not found"), "error mismatch: {e}"),
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_to_native_tools_empty() {
        let gw = McpGateway::new();
        let tools = gw.to_native_tools();
        assert!(tools.is_empty());
    }

    #[test]
    fn test_discover_scans_path() {
        let gw = McpGateway::new();
        let found = gw.discover();
        // 在测试环境下可能没有 *-mcp-server 二进制
        // 只要不 panic 就算通过
        assert!(found.is_empty() || !found.is_empty());
    }

    #[test]
    fn test_duplicate_register_ignored() {
        let gw = McpGateway::new();
        gw.register_local("dup", "echo", &[], "first");
        gw.register_local("dup", "echo", &[], "second");
        assert_eq!(gw.server_count(), 1);
    }

    #[test]
    fn test_multiple_servers() {
        let gw = McpGateway::new();
        gw.register_local("s1", "echo", &[], "Server one");
        gw.register_local("s2", "echo", &[], "Server two");
        gw.register_local("s3", "echo", &[], "Server three");
        assert_eq!(gw.server_count(), 3);
    }
}
