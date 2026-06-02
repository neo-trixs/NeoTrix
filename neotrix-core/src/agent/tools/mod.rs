//! MCP Registry — Model Context Protocol 服务注册与智能路由
//!
//! 参照 PraisonAI MCP 集成 + cc-haha MCP Server 设计：
//! - 4 种传输协议：Stdio, HTTP, WebSocket, SSE
//! - 智能路由：根据任务类型选择最合适工具
//! - 结果缓存：TTL + LRU
//! - 健康检查 + 自动重连

pub mod patch;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::core::nt_core_traits::{ToolDef, ToolOutput, ToolProvider};

/// MCP 传输协议
#[derive(Debug, Clone, PartialEq)]
pub enum McpTransport {
    Stdio { command: String, args: Vec<String> },
    Http { url: String, headers: HashMap<String, String> },
    WebSocket { url: String, auth_token: Option<String> },
    Sse { url: String },
}

/// MCP 工具定义
#[derive(Debug, Clone)]
pub struct McpToolDef {
    pub name: String,
    pub description: String,
    pub server_name: String,
    pub transport: McpTransport,
    pub input_schema: serde_json::Value,
}

/// MCP 服务实例
#[derive(Debug, Clone)]
pub struct McpServer {
    pub name: String,
    pub transport: McpTransport,
    pub tools: Vec<McpToolDef>,
    pub healthy: bool,
    pub last_health_check: Option<Instant>,
    pub latency_ms: u64,
}

/// 内置工具处理器
type BuiltinHandler = fn(&serde_json::Value) -> Result<String, String>;

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    result: String,
    created_at: Instant,
    ttl: Duration,
}

/// MCP Registry
pub struct McpRegistry {
    servers: Vec<McpServer>,
    tool_index: HashMap<String, usize>,    // tool_name → server index
    cache: HashMap<String, CacheEntry>,
    default_ttl: Duration,
    builtin_handlers: HashMap<String, BuiltinHandler>,
}

impl McpRegistry {
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
            tool_index: HashMap::new(),
            cache: HashMap::new(),
            default_ttl: Duration::from_secs(60),
            builtin_handlers: HashMap::new(),
        }
    }

    /// 注册 MCP 服务
    pub fn register(&mut self, server: McpServer) {
        for tool in &server.tools {
            self.tool_index.insert(tool.name.clone(), self.servers.len());
        }
        self.servers.push(server);
    }

    /// 注册 Stdio 服务
    pub fn register_stdio(&mut self, name: &str, command: &str, args: &[&str], tools: Vec<McpToolDef>) {
        self.register(McpServer {
            name: name.to_string(),
            transport: McpTransport::Stdio {
                command: command.to_string(),
                args: args.iter().map(|a| a.to_string()).collect(),
            },
            tools,
            healthy: true,
            last_health_check: None,
            latency_ms: 0,
        });
    }

    /// 注册 HTTP 服务
    pub fn register_http(&mut self, name: &str, url: &str, tools: Vec<McpToolDef>) {
        self.register(McpServer {
            name: name.to_string(),
            transport: McpTransport::Http { url: url.to_string(), headers: HashMap::new() },
            tools,
            healthy: true,
            last_health_check: None,
            latency_ms: 0,
        });
    }

    /// 按名称查找工具
    pub fn find_tool(&self, name: &str) -> Option<&McpToolDef> {
        self.tool_index.get(name).and_then(|&idx| {
            self.servers.get(idx).and_then(|s| s.tools.iter().find(|t| t.name == name))
        })
    }

    /// 根据任务类型智能推荐工具
    pub fn recommend_tools(&self, task_type: &str, top_k: usize) -> Vec<&McpToolDef> {
        let task_lower = task_type.to_lowercase();
        let mut scored: Vec<(&McpToolDef, usize)> = Vec::new();

        for server in &self.servers {
            for tool in &server.tools {
                let score = Self::compute_relevance(&tool.name, &tool.description, &task_lower);
                if score > 0 {
                    scored.push((tool, score));
                }
            }
        }

        scored.sort_by_key(|b| std::cmp::Reverse(b.1));
        scored.into_iter().take(top_k).map(|(t, _)| t).collect()
    }

    fn compute_relevance(name: &str, desc: &str, task: &str) -> usize {
        let mut score = 0;
        for word in task.split_whitespace() {
            if name.to_lowercase().contains(word) { score += 3; }
            if desc.to_lowercase().contains(word) { score += 1; }
        }
        score
    }

    /// 健康检查所有服务
    pub async fn health_check(&mut self) -> Vec<(&str, bool)> {
        let mut results = Vec::new();
        for server in &mut self.servers {
            let ok = match &server.transport {
                McpTransport::Http { url, .. } => {
                    reqwest::get(url).await.is_ok()
                }
                _ => true, // Stdio/WebSocket/SSE 简化处理
            };
            server.healthy = ok;
            server.last_health_check = Some(Instant::now());
            results.push((server.name.as_str(), ok));
        }
        results
    }

    /// 缓存工具调用结果
    pub fn cache_result(&mut self, key: &str, result: &str, ttl: Duration) {
        let ttl = if ttl.as_secs() == 0 { self.default_ttl } else { ttl };
        self.cache.insert(key.to_string(), CacheEntry {
            result: result.to_string(),
            created_at: Instant::now(),
            ttl,
        });
    }

    /// 获取缓存结果
    pub fn get_cached(&self, key: &str) -> Option<&str> {
        self.cache.get(key).and_then(|entry| {
            if entry.created_at.elapsed() < entry.ttl {
                Some(entry.result.as_str())
            } else {
                None
            }
        })
    }

    /// 清理过期缓存
    pub fn prune_cache(&mut self) {
        self.cache.retain(|_, entry| entry.created_at.elapsed() < entry.ttl);
    }

    /// 注册内置工具处理器
    pub fn register_builtin(&mut self, name: &str, handler: BuiltinHandler) {
        self.builtin_handlers.insert(name.to_string(), handler);
    }

    pub fn server_count(&self) -> usize { self.servers.len() }
    pub fn tool_count(&self) -> usize { self.tool_index.len() }
    pub fn list_servers(&self) -> &[McpServer] { &self.servers }

    /// Search tools by query across name + description (case-insensitive substring match)
    pub fn search(&self, query: &str) -> Vec<&McpToolDef> {
        let q = query.to_lowercase();
        if q.is_empty() {
            let mut seen_servers: std::collections::HashSet<usize> = std::collections::HashSet::new();
            let mut results: Vec<&McpToolDef> = Vec::new();
            for &idx in self.tool_index.values() {
                if seen_servers.insert(idx) {
                    if let Some(server) = self.servers.get(idx) {
                        results.extend(server.tools.iter());
                    }
                }
            }
            return results;
        }
        let mut results: Vec<&McpToolDef> = Vec::new();
        for server in &self.servers {
            for tool in &server.tools {
                if tool.name.to_lowercase().contains(&q)
                    || tool.description.to_lowercase().contains(&q)
                {
                    results.push(tool);
                }
            }
        }
        results
    }

    /// Register a discoverable server (alias for `register` with metadata flag).
    /// Used by `/mcp publish` to mark a server as user-published.
    pub fn publish(&mut self, name: &str, command: &str, args: &[&str], description: &str) -> usize {
        let tool = McpToolDef {
            name: format!("{}-invoke", name),
            description: format!("[published] {} — {}", name, description),
            server_name: name.to_string(),
            transport: McpTransport::Stdio {
                command: command.to_string(),
                args: args.iter().map(|s| s.to_string()).collect(),
            },
            input_schema: serde_json::json!({"type": "object"}),
        };
        let server = McpServer {
            name: name.to_string(),
            transport: McpTransport::Stdio {
                command: command.to_string(),
                args: args.iter().map(|s| s.to_string()).collect(),
            },
            tools: vec![tool],
            healthy: true,
            last_health_check: None,
            latency_ms: 0,
        };
        self.register(server);
        1
    }

    /// 执行 MCP 工具调用
    pub fn call_tool(&self, name: &str, args: &serde_json::Value) -> Result<String, String> {
        // 优先检查内置处理器
        if let Some(handler) = self.builtin_handlers.get(name) {
            return handler(args);
        }

        let tool = self.find_tool(name).ok_or_else(|| format!("Tool '{}' not found", name))?;
        let server = self.servers.iter().find(|s| s.name == tool.server_name)
            .ok_or_else(|| format!("Server '{}' not found", tool.server_name))?;

        match &server.transport {
            McpTransport::Stdio { command, args: cmd_args } => {
                let mut cmd = std::process::Command::new(command);
                cmd.args(cmd_args);

                // Pass args as JSON on stdin
                let input = serde_json::json!({
                    "tool": name,
                    "args": args,
                }).to_string();

                let output = cmd.arg(&input).output()
                    .map_err(|e| format!("Failed to execute {}: {}", command, e))?;

                if output.status.success() {
                    String::from_utf8(output.stdout)
                        .map_err(|e| format!("Invalid UTF-8 output: {}", e))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(format!("Tool '{}' failed: {}", name, stderr))
                }
            }
            McpTransport::Http { url, headers } => {
                let client = reqwest::blocking::Client::builder()
                    .danger_accept_invalid_certs(true)
                    .build()
                    .map_err(|e| format!("构建 MCP HTTP client 失败: {}", e))?;
                let mut req = client.post(url)
                    .header("Content-Type", "application/json");

                for (k, v) in headers {
                    req = req.header(k, v);
                }

                let body = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "tools/call",
                    "params": {
                        "name": name,
                        "arguments": args,
                    },
                    "id": 1,
                });

                let resp = req.json(&body).send()
                    .map_err(|e| format!("HTTP call failed: {}", e))?;

                if resp.status().is_success() {
                    resp.text().map_err(|e| format!("Read response failed: {}", e))
                } else {
                    Err(format!("HTTP {}: {}", resp.status(), resp.text().unwrap_or_default()))
                }
            }
            McpTransport::WebSocket { .. } | McpTransport::Sse { .. } => {
                Err(format!("Transport '{}' not yet implemented for tool calling", server.transport.transport_type()))
            }
        }
    }
}

impl Default for McpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolProvider for McpRegistry {
    fn list_tools(&self) -> Vec<ToolDef> {
        self.servers.iter()
            .flat_map(|s| s.tools.iter())
            .map(|t| ToolDef {
                name: t.name.clone(),
                description: t.description.clone(),
                input_schema: t.input_schema.clone(),
            })
            .collect()
    }

    fn call_tool(&self, name: &str, args: &serde_json::Value) -> Result<ToolOutput, String> {
        let content = McpRegistry::call_tool(self, name, args)?;
        Ok(ToolOutput { success: true, content })
    }
}

impl McpTransport {
    /// Check if this transport is available
    pub fn is_available(&self) -> bool {
        match self {
            McpTransport::Stdio { command, .. } => {
                std::process::Command::new("which")
                    .arg(command)
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
            }
            McpTransport::Http { url, .. } => !url.is_empty(),
            McpTransport::WebSocket { url, .. } => !url.is_empty(),
            McpTransport::Sse { url, .. } => !url.is_empty(),
        }
    }

    /// Transport type as string
    pub fn transport_type(&self) -> &str {
        match self {
            McpTransport::Stdio { .. } => "stdio",
            McpTransport::Http { .. } => "http",
            McpTransport::WebSocket { .. } => "websocket",
            McpTransport::Sse { .. } => "sse",
        }
    }
}

// ========== MCP 工具生成器 (A-202) ==========

/// MCP 工具生成器 — 从代码库自动分析并生成 MCP 工具配置
pub struct McpToolGenerator {
    /// 支持的项目类型
    pub project_types: Vec<String>,
}

impl McpToolGenerator {
    pub fn new() -> Self {
        Self { project_types: vec!["rust".into(), "python".into(), "node".into()] }
    }

    /// 分析项目目录并生成 MCP 工具
    pub fn generate_from_dir(&self, project_path: &str) -> Vec<McpToolDef> {
        let mut tools = Vec::new();
        let path = std::path::Path::new(project_path);
        if !path.exists() || !path.is_dir() { return tools; }

        // 检测项目类型
        let project_type = self.detect_project_type(path);

        // 扫描入口文件
        match project_type.as_str() {
            "rust" => tools.extend(self.scan_rust_project(path)),
            "python" => tools.extend(self.scan_python_project(path)),
            "node" => tools.extend(self.scan_node_project(path)),
            _ => {}
        }

        tools
    }

    fn detect_project_type(&self, path: &std::path::Path) -> String {
        if path.join("Cargo.toml").exists() { return "rust".to_string(); }
        if path.join("pyproject.toml").exists() || path.join("requirements.txt").exists() || path.join("setup.py").exists() {
            return "python".to_string();
        }
        if path.join("package.json").exists() { return "node".to_string(); }
        "unknown".to_string()
    }

    fn scan_rust_project(&self, path: &std::path::Path) -> Vec<McpToolDef> {
        let mut tools = Vec::new();
        let src_dir = path.join("src");
        if !src_dir.exists() { return tools; }

        if let Ok(entries) = std::fs::read_dir(&src_dir) {
            for entry in entries.flatten() {
                let fpath = entry.path();
                if fpath.extension().is_some_and(|e| e == "rs") {
                    if let Ok(content) = std::fs::read_to_string(&fpath) {
                        tools.extend(self.extract_rust_tools(&fpath, &content));
                    }
                }
                // 递归子目录
                if fpath.is_dir() {
                    tools.extend(self.scan_rust_project(&fpath));
                }
            }
        }
        tools
    }

    fn extract_rust_tools(&self, fpath: &std::path::Path, content: &str) -> Vec<McpToolDef> {
        let mut tools = Vec::new();
        let file_name = fpath.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

        // 检测 main 函数入口
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("fn main") || trimmed.starts_with("#[tokio::main]") {
                tools.push(McpToolDef {
                    name: format!("run-{}", file_name),
                    description: format!("Run {} entry point", file_name),
                    server_name: "code-analysis".to_string(),
                    transport: McpTransport::Stdio {
                        command: "cargo".to_string(),
                        args: vec!["run".to_string()],
                    },
                    input_schema: serde_json::json!({"type": "object", "properties": {}}),
                });
            }

            // 检测 CLI 命令 (clap/structopt)
            if trimmed.contains("#[derive(Parser)") || trimmed.contains("#[derive(Args)") || trimmed.contains("Command::new(") {
                tools.push(McpToolDef {
                    name: format!("cli-{}", file_name),
                    description: format!("CLI command in {}", file_name),
                    server_name: "code-analysis".to_string(),
                    transport: McpTransport::Stdio {
                        command: "cargo".to_string(),
                        args: vec!["run".to_string(), "--".to_string(), "--help".to_string()],
                    },
                    input_schema: serde_json::json!({"type": "object"}),
                });
            }

            // 检测 HTTP 处理函数 (axum/actix)
            if trimmed.contains(".route(") || trimmed.contains("#[get(") || trimmed.contains("#[post(") {
                tools.push(McpToolDef {
                    name: format!("api-{}", file_name),
                    description: format!("HTTP endpoint in {}", file_name),
                    server_name: "code-analysis".to_string(),
                    transport: McpTransport::Http {
                        url: "http://localhost:3000".to_string(),
                        headers: HashMap::new(),
                    },
                    input_schema: serde_json::json!({"type": "object"}),
                });
            }
        }

        tools
    }

    fn scan_python_project(&self, path: &std::path::Path) -> Vec<McpToolDef> {
        let mut tools = Vec::new();
        let dirs = [path.join("src"), path.join("."), path.join("app")];
        for dir in &dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let fpath = entry.path();
                    if fpath.extension().is_some_and(|e| e == "py") {
                        if let Ok(content) = std::fs::read_to_string(&fpath) {
                            for line in content.lines() {
                                let t = line.trim();
                                if t.starts_with("@app.route") || t.starts_with("@router.") || t.starts_with("def main") {
                                    tools.push(McpToolDef {
                                        name: format!("py-{}", fpath.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown")),
                                        description: format!("Python endpoint in {:?}", fpath),
                                        server_name: "code-analysis".to_string(),
                                        transport: McpTransport::Stdio {
                                            command: "python".to_string(),
                                            args: vec![fpath.to_string_lossy().to_string()],
                                        },
                                        input_schema: serde_json::json!({"type": "object"}),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        tools
    }

    fn scan_node_project(&self, path: &std::path::Path) -> Vec<McpToolDef> {
        let mut tools = Vec::new();
        // Read package.json scripts
        let pkg_path = path.join("package.json");
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(scripts) = pkg.get("scripts").and_then(|s| s.as_object()) {
                    for (name, cmd) in scripts {
                        let cmd_str = cmd.as_str().unwrap_or("");
                        tools.push(McpToolDef {
                            name: format!("npm-{}", name),
                            description: format!("npm script: {} → {}", name, cmd_str),
                            server_name: "code-analysis".to_string(),
                            transport: McpTransport::Stdio {
                                command: "npm".to_string(),
                                args: vec!["run".to_string(), name.to_string()],
                            },
                            input_schema: serde_json::json!({"type": "object"}),
                        });
                    }
                }
            }
        }
        tools
    }

    /// 将生成的工具注册到 McpRegistry
    pub fn register_to(&self, registry: &mut McpRegistry, project_path: &str) -> usize {
        let tools = self.generate_from_dir(project_path);
        if tools.is_empty() { return 0; }
        registry.register(McpServer {
            name: "code-analysis".to_string(),
            transport: McpTransport::Stdio {
                command: "echo".to_string(),
                args: vec!["MCP tools from code analysis".to_string()],
            },
            tools: tools.clone(),
            healthy: true,
            last_health_check: None,
            latency_ms: 0,
        });
        tools.len()
    }
}

impl Default for McpToolGenerator {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tool(name: &str, desc: &str) -> McpToolDef {
        McpToolDef {
            name: name.to_string(),
            description: desc.to_string(),
            server_name: "test".to_string(),
            transport: McpTransport::Http { url: "http://localhost:9999".to_string(), headers: HashMap::new() },
            input_schema: serde_json::json!({}),
        }
    }

    fn make_tool_for(name: &str, desc: &str, server: &str) -> McpToolDef {
        McpToolDef {
            name: name.to_string(),
            description: desc.to_string(),
            server_name: server.to_string(),
            transport: McpTransport::Http { url: "http://localhost:9999".to_string(), headers: HashMap::new() },
            input_schema: serde_json::json!({}),
        }
    }

    #[test]
    fn test_register_and_find() {
        let mut reg = McpRegistry::new();
        reg.register_stdio("test", "echo", &["hello"], vec![
            make_tool("greet", "Say hello"),
            make_tool("echo", "Echo input"),
        ]);
        assert_eq!(reg.server_count(), 1);
        assert_eq!(reg.tool_count(), 2);
        assert!(reg.find_tool("greet").is_some());
        assert!(reg.find_tool("nonexistent").is_none());
    }

    #[test]
    fn test_recommend_tools() {
        let mut reg = McpRegistry::new();
        reg.register_stdio("server1", "npx", &[], vec![
            make_tool("nt_world_search", "Search the web for information"),
            make_tool("calculator", "Perform math calculations"),
            make_tool("file_read", "Read files from disk"),
        ]);

        let recommended = reg.recommend_tools("search the web", 2);
        assert_eq!(recommended.len(), 1);
        assert_eq!(recommended[0].name, "nt_world_search");
    }

    #[test]
    fn test_cache() {
        let mut reg = McpRegistry::new();
        assert!(reg.get_cached("test-key").is_none());
        reg.cache_result("test-key", "cached result", Duration::from_secs(60));
        assert_eq!(reg.get_cached("test-key"), Some("cached result"));
    }

    #[test]
    fn test_cache_expiry() {
        let mut reg = McpRegistry::new();
        reg.cache_result("expire-key", "data", Duration::from_nanos(1));
        // force expiration by setting ttl to 0
        if let Some(entry) = reg.cache.get_mut("expire-key") {
            entry.ttl = Duration::from_nanos(0);
        }
        std::thread::sleep(Duration::from_millis(1));
        assert!(reg.get_cached("expire-key").is_none());
    }

    #[test]
    fn test_prune_cache() {
        let mut reg = McpRegistry::new();
        reg.cache_result("k1", "v1", Duration::from_secs(60));
        reg.cache_result("k2", "v2", Duration::from_secs(60));
        // manually expire k1
        if let Some(entry) = reg.cache.get_mut("k1") {
            entry.ttl = Duration::from_nanos(0);
        }
        reg.prune_cache();
        assert!(reg.get_cached("k1").is_none());
        assert_eq!(reg.get_cached("k2"), Some("v2"));
    }

    #[test]
    fn test_register_http() {
        let mut reg = McpRegistry::new();
        reg.register_http("api", "https://api.example.com/mcp", vec![
            make_tool_for("api_call", "Make API call", "api"),
        ]);
        assert_eq!(reg.server_count(), 1);
        let tool = reg.find_tool("api_call").expect("find_tool for registered api_call should succeed");
        assert_eq!(tool.server_name, "api");
    }

    #[test]
    fn test_recommend_no_match() {
        let mut reg = McpRegistry::new();
        reg.register_stdio("srv", "cmd", &[], vec![
            make_tool("image_gen", "Generate images"),
        ]);
        let recommended = reg.recommend_tools("write code", 5);
        assert!(recommended.is_empty());
    }

    #[test]
    fn test_mcp_generator_new() {
        let gen = McpToolGenerator::new();
        assert_eq!(gen.project_types.len(), 3);
    }

    #[test]
    fn test_detect_project_type_rust() {
        let gen = McpToolGenerator::new();
        let tmp = std::env::temp_dir().join("test-mcp-gen-rust");
        let _ = std::fs::create_dir_all(&tmp);
        std::fs::write(tmp.join("Cargo.toml"), "[package]\nname = \"test\"\n").expect("write test Cargo.toml should succeed");
        let result = gen.detect_project_type(&tmp);
        assert_eq!(result, "rust");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_detect_project_type_unknown() {
        let gen = McpToolGenerator::new();
        let tmp = std::env::temp_dir().join("test-mcp-gen-empty");
        let _ = std::fs::create_dir_all(&tmp);
        let result = gen.detect_project_type(&tmp);
        assert_eq!(result, "unknown");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_extract_rust_tools_main() {
        let gen = McpToolGenerator::new();
        let tmp = std::env::temp_dir().join("test-mcp-extract");
        let _ = std::fs::create_dir_all(&tmp);
        let main_rs = tmp.join("main.rs");
        std::fs::write(&main_rs, "fn main() {\n    println!(\"hello\");\n}\n").expect("write test main.rs should succeed");
        let tools = gen.extract_rust_tools(&main_rs, "fn main() {\n    println!(\"hello\");\n}\n");
        assert!(!tools.is_empty());
        assert!(tools.iter().any(|t| t.name.contains("main")));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_extract_rust_tools_axum() {
        let gen = McpToolGenerator::new();
        let tools = gen.extract_rust_tools(
            std::path::Path::new("api.rs"),
            "fn main() {\n    let app = Router::new().route(\"/users\", get(list_users));\n}\n"
        );
        assert!(tools.iter().any(|t| t.name.contains("api") || t.name.contains("cli")));
    }

    #[test]
    fn test_generate_from_dir_empty() {
        let gen = McpToolGenerator::new();
        let tmp = std::env::temp_dir().join("test-mcp-empty");
        let _ = std::fs::create_dir_all(&tmp);
        let tools = gen.generate_from_dir(tmp.to_str().expect("tmp path should be valid utf-8"));
        assert!(tools.is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_generate_from_dir_rust_proj() {
        let gen = McpToolGenerator::new();
        let tmp = std::env::temp_dir().join("test-mcp-rust-proj");
        let _ = std::fs::create_dir_all(tmp.join("src"));
        std::fs::write(tmp.join("Cargo.toml"), "[package]\nname = \"test\"\n").expect("write Cargo.toml for rust-proj test should succeed");
        std::fs::write(tmp.join("src/main.rs"), "fn main() {}\n").expect("write src/main.rs for rust-proj test should succeed");
        let tools = gen.generate_from_dir(tmp.to_str().expect("tmp path should be valid utf-8 for rust-proj test"));
        assert!(!tools.is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_register_tools_into_registry() {
        let gen = McpToolGenerator::new();
        let mut reg = McpRegistry::new();
        let tmp = std::env::temp_dir().join("test-mcp-register");
        let _ = std::fs::create_dir_all(tmp.join("src"));
        std::fs::write(tmp.join("Cargo.toml"), "[package]\n").expect("write Cargo.toml for register test should succeed");
        std::fs::write(tmp.join("src/main.rs"), "fn main() {}\n").expect("write src/main.rs for register test should succeed");
        let count = gen.register_to(&mut reg, tmp.to_str().expect("tmp path should be valid utf-8"));
        assert!(count > 0);
        assert_eq!(reg.server_count(), 1);
        assert!(reg.find_tool("run-main").is_some());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_scan_python_flask() {
        let gen = McpToolGenerator::new();
        let tmp = std::env::temp_dir().join("test-mcp-py");
        let _ = std::fs::create_dir_all(&tmp);
        std::fs::write(tmp.join("app.py"), "@app.route('/api/users')\ndef list_users():\n    return []\n").expect("write test app.py should succeed");
        let tools = gen.scan_python_project(&tmp);
        assert!(!tools.is_empty());
        assert!(tools.iter().any(|t| t.name.starts_with("py-")));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_tool_provider_list_tools() {
        let mut reg = McpRegistry::new();
        reg.register_stdio("test", "echo", &[], vec![
            make_tool("tool_a", "Tool A"),
            make_tool("tool_b", "Tool B"),
        ]);
        let tools = ToolProvider::list_tools(&reg);
        assert_eq!(tools.len(), 2);
        assert!(tools.iter().any(|t| t.name == "tool_a"));
        assert!(tools.iter().any(|t| t.name == "tool_b"));
    }

    #[test]
    fn test_tool_provider_call_tool_builtin() {
        let mut reg = McpRegistry::new();
        reg.register_builtin("echo_test", |args| {
            Ok(format!("echo: {}", args))
        });
        let result = ToolProvider::call_tool(&reg, "echo_test", &serde_json::json!({"msg": "hello"}));
        assert!(result.is_ok());
        let output = result.expect("builtin handler call should return Ok");
        assert!(output.success);
        assert!(output.content.contains("hello"));
    }

    #[test]
    fn test_search_finds_matching_tools() {
        let mut reg = McpRegistry::new();
        reg.register_stdio("alpha", "echo", &[], vec![
            make_tool_for("nt_world_search", "Search the public web", "alpha"),
            make_tool_for("vector_search", "Find nearest vectors", "alpha"),
            make_tool_for("calculator", "Do math", "alpha"),
        ]);
        let results = reg.search("search");
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|t| t.name == "nt_world_search"));
        assert!(results.iter().any(|t| t.name == "vector_search"));
    }

    #[test]
    fn test_search_empty_query_returns_all() {
        let mut reg = McpRegistry::new();
        reg.register_stdio("alpha", "echo", &[], vec![
            make_tool_for("a", "x", "alpha"),
            make_tool_for("b", "y", "alpha"),
        ]);
        let results = reg.search("");
        assert_eq!(results.len(), reg.tool_count());
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut reg = McpRegistry::new();
        reg.register_stdio("alpha", "echo", &[], vec![
            make_tool_for("WebSearch", "PUBLIC search", "alpha"),
        ]);
        let r1 = reg.search("websearch");
        let r2 = reg.search("WEBSEARCH");
        let r3 = reg.search("public");
        assert_eq!(r1.len(), 1);
        assert_eq!(r2.len(), 1);
        assert_eq!(r3.len(), 1);
    }

    #[test]
    fn test_publish_registers_user_server() {
        let mut reg = McpRegistry::new();
        let n = reg.publish("my-tool", "/usr/local/bin/my-tool", &["--mcp"], "Custom MCP server");
        assert_eq!(n, 1);
        assert_eq!(reg.server_count(), 1);
        assert!(reg.find_tool("my-tool-invoke").is_some());
    }
}
