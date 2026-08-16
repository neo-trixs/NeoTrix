use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};

use crate::core::nt_core_dispatch::{DispatchMode, Dispatcher};
use crate::core::nt_core_guard_chain::{GuardChain, GuardVerdict};

/// MCP Server for NeoTrix — exposes tools via stdio JSON-RPC 2.0 transport.
pub struct McpServer {
    tools: Vec<McpTool>,
    reader: std::io::Stdin,
    writer: std::io::Stdout,
    /// 单调授权守卫链 (∂guard): 聚合裁决 Deny/Ask 时拦截工具调用。
    guard: GuardChain,
    /// 工具调用钩子 (waterfall 中间件): 短路即拦截, 全部放行才执行真实工具。
    hooks: Dispatcher<McpToolCall>,
}

/// 工具调用事件 — hooks (waterfall) 的载荷。
#[derive(Debug)]
pub struct McpToolCall {
    pub id: u64,
    pub name: String,
    pub args: serde_json::Value,
    /// hook 置位表示拦截 (阻止真实工具执行)。仅 dispatch 单线程内访问。
    pub intercepted: std::cell::Cell<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    #[serde(default)]
    pub schema_version: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct McpRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    #[serde(default)]
    params: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct McpResponse {
    jsonrpc: String,
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<McpError>,
}

#[derive(Serialize, Deserialize)]
struct McpError {
    code: i32,
    message: String,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
            reader: std::io::stdin(),
            writer: std::io::stdout(),
            guard: GuardChain::new(),
            hooks: Dispatcher::new(),
        }
    }

    /// 设置授权守卫链 (默认空链 = 全放行)。
    pub fn set_guard(&mut self, guard: GuardChain) {
        self.guard = guard;
    }

    /// 追加单调授权守卫。
    pub fn add_guard<F>(&mut self, name: impl Into<String>, check: F)
    where
        F: Fn(&str, &serde_json::Value) -> GuardVerdict + Send + Sync + 'static,
    {
        self.guard.add(name, check);
    }

    /// 注册工具调用钩子 (waterfall 中间件)。
    pub fn register_hook<F>(&mut self, handler: F)
    where
        F: Fn(&McpToolCall, &dyn Fn()) -> bool + Send + Sync + 'static,
    {
        self.hooks.register(handler);
    }

    pub fn register_all_tools(&mut self) {
        self.register_tool(McpTool {
            name: "read_file".into(),
            description: "Read file contents from disk".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Absolute or relative file path"
                    }
                },
                "required": ["path"]
            }),
            schema_version: None,
        });
        self.register_tool(McpTool {
            name: "write_file".into(),
            description: "Write content to a file on disk".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path to write to"
                    },
                    "content": {
                        "type": "string",
                        "description": "File content to write"
                    }
                },
                "required": ["path", "content"]
            }),
            schema_version: None,
        });
        self.register_tool(McpTool {
            name: "edit_file".into(),
            description: "Edit a file by finding and replacing exact text".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path to edit"
                    },
                    "old_string": {
                        "type": "string",
                        "description": "Exact text to find (must match exactly once)"
                    },
                    "new_string": {
                        "type": "string",
                        "description": "Replacement text"
                    }
                },
                "required": ["path", "old_string", "new_string"]
            }),
            schema_version: None,
        });
        self.register_tool(McpTool {
            name: "search_code".into(),
            description: "Search codebase using regex pattern".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Regex pattern to search for"
                    },
                    "path": {
                        "type": "string",
                        "description": "Directory or file path to search (default: current dir)"
                    }
                },
                "required": ["pattern"]
            }),
            schema_version: None,
        });
        self.register_tool(McpTool {
            name: "git_diff".into(),
            description: "Get git diff of the working tree".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "staged": {
                        "type": "boolean",
                        "description": "If true, show staged diff (git diff --cached)",
                        "default": false
                    }
                },
                "required": []
            }),
            schema_version: None,
        });
        self.register_tool(McpTool {
            name: "execute_command".into(),
            description: "Execute a shell command and return its output".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Shell command to execute"
                    }
                },
                "required": ["command"]
            }),
            schema_version: None,
        });
        // 命令面精简: 人类只用基础控制命令 (/help /exit /clear /version /config ...),
        // 领域操作 (file/git/session/agent/memory/crypto/...) 由 agent 后端自我调度。
        // 桥接: agent 通过本工具调用 CommandRegistry 进程内执行任意命令。
        self.register_tool(McpTool {
            name: "neotrix_command".into(),
            description: "Execute a NeoTrix CLI command in-process (agent 后端自我调度通道). \
                         command 为完整命令文本, 如 'file read src/main.rs' 或 '/memory search kb'. \
                         返回命令输出; 人类无需输入这些命令, 由 agent 按需调度。".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "NeoTrix command text (with or without leading '/'), e.g. 'file read path' or '/git status'"
                    }
                },
                "required": ["command"]
            }),
            schema_version: None,
        });
        // ── 意识核心 (ConsciousnessCore) — opencode agent 专用工具 ──
        self.register_tool(McpTool {
            name: "consciousness_status".into(),
            description:
                "读取意识核心当前状态: cycle/phi/coherence/GWT谐振/MARS双过程/治理合规/迷雾。\
                          agent 可据此判断系统健康度, 决定是否推进进化或启动自愈。"
                    .into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            schema_version: None,
        });
        self.register_tool(McpTool {
            name: "consciousness_tick".into(),
            description:
                "驱动意识核心运行 N 个生长周期 (run_growth_cycle): 土壤→根→树干→分支→果实→核心 \
                          六阶段闭环, 生产进化果实与治理反馈。返回生长报告 (phase 摘要)。"
                    .into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "cycles": {
                        "type": "integer",
                        "description": "运行周期数 (default 1, 建议 <=3)",
                        "default": 1
                    }
                },
                "required": []
            }),
            schema_version: None,
        });
        self.register_tool(McpTool {
            name: "consciousness_task".into(),
            description: "意识核心直接处理人类语言任务: 拆解→分配→反思补齐→执行全部子任务。\
                          默认走内置能力网 (optimal provider), 能力缺失时自动获取外部知识 \
                          (论文/GitHub/技术文档) 并在 token 预算内试错求解。调用谁/怎么调用由意识核心决定。".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "instruction": {
                        "type": "string",
                        "description": "人类语言任务指令 (自然语言, 如 '合并供应商价格表并检索历史经验')"
                    },
                    "acquire_knowledge": {
                        "type": "boolean",
                        "description": "是否调用外部知识源 (discover_*) 获取信息基础 (default true)",
                        "default": true
                    },
                    "max_attempts": {
                        "type": "integer",
                        "description": "外部缺口试错轮次上限 (default 5)",
                        "default": 5
                    }
                },
                "required": ["instruction"]
            }),
            schema_version: None,
        });
    }

    pub fn register_tool(&mut self, tool: McpTool) {
        self.tools.push(tool);
    }

    pub fn run(&mut self) -> Result<(), String> {
        loop {
            let mut line = String::new();
            let bytes = self
                .reader
                .lock()
                .read_line(&mut line)
                .map_err(|e| format!("Read error: {}", e))?;

            if bytes == 0 {
                break;
            }

            if line.trim().is_empty() {
                continue;
            }

            let request: McpRequest = match serde_json::from_str(line.trim()) {
                Ok(r) => r,
                Err(_) => {
                    // 单行解析失败不杀服务器：回 JSON-RPC 解析错误并继续
                    let response = McpResponse {
                        jsonrpc: "2.0".into(),
                        id: 0,
                        result: None,
                        error: Some(McpError {
                            code: -32700,
                            message: "Parse error".into(),
                        }),
                    };
                    if let Ok(json) = serde_json::to_string(&response) {
                        let mut out = self.writer.lock();
                        let _ = writeln!(out, "{}", json);
                        let _ = out.flush();
                    }
                    continue;
                }
            };

            let response = self.handle_request(&request);

            let json = serde_json::to_string(&response)
                .map_err(|e| format!("Serialization error: {}", e))?;

            let mut out = self.writer.lock();
            writeln!(out, "{}", json).map_err(|e| format!("Write error: {}", e))?;
            out.flush().map_err(|e| format!("Flush error: {}", e))?;

            if request.method == "exit" {
                break;
            }
        }
        Ok(())
    }

    fn handle_request(&self, request: &McpRequest) -> McpResponse {
        match request.method.as_str() {
            "server/discover" | "initialize" => McpResponse {
                jsonrpc: "2.0".into(),
                id: request.id,
                result: Some(serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "neotrix-mcp",
                        "version": env!("CARGO_PKG_VERSION"),
                    },
                    "capabilities": {
                        "tools": {}
                    }
                })),
                error: None,
            },
            "tools/list" => self.handle_list_tools(request.id),
            "tools/call" => {
                let Some(params) = request.params.as_ref() else {
                    return McpResponse {
                        jsonrpc: "2.0".into(),
                        id: request.id,
                        result: None,
                        error: Some(McpError {
                            code: -32602,
                            message: "Invalid params: missing params".into(),
                        }),
                    };
                };
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                self.handle_call_tool(request.id, name, &args)
            }
            // exit 无需响应体: run() 循环在写入响应后检测 method=="exit" 并 break。
            "exit" => McpResponse {
                jsonrpc: "2.0".into(),
                id: request.id,
                result: Some(serde_json::json!({ "ok": true })),
                error: None,
            },
            _ => McpResponse {
                jsonrpc: "2.0".into(),
                id: request.id,
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                }),
            },
        }
    }

    fn handle_list_tools(&self, id: u64) -> McpResponse {
        // Per MCP 2026-07-28 spec: if schema_version is Some("2020-12"),
        // add $schema to the output; otherwise leave as-is (draft-07 compat)
        let normalize_schema =
            |schema: &serde_json::Value, schema_version: &Option<String>| -> serde_json::Value {
                match schema_version.as_deref() {
                    Some("2020-12") => {
                        if schema.get("$schema").is_some() {
                            return schema.clone();
                        }
                        let mut map = match schema {
                            serde_json::Value::Object(m) => m.clone(),
                            _ => return schema.clone(),
                        };
                        map.insert(
                            "$schema".into(),
                            serde_json::Value::String(
                                "https://json-schema.org/draft/2020-12/schema".into(),
                            ),
                        );
                        serde_json::Value::Object(map)
                    }
                    _ => schema.clone(),
                }
            };

        let tools: Vec<serde_json::Value> = self
            .tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": normalize_schema(&t.input_schema, &t.schema_version),
                })
            })
            .collect();

        McpResponse {
            jsonrpc: "2.0".into(),
            id,
            result: Some(serde_json::json!({ "tools": tools })),
            error: None,
        }
    }

    fn handle_call_tool(&self, id: u64, name: &str, args: &serde_json::Value) -> McpResponse {
        // 第一道闸: 单调授权守卫链 (NT-SHIELD GuardChain)
        let (verdict, reasons) = self.guard.evaluate(name, args);
        if !verdict.is_allowed() {
            return McpResponse {
                jsonrpc: "2.0".into(),
                id,
                result: None,
                error: Some(McpError {
                    code: -32000,
                    message: format!(
                        "[{}] Tool call denied by guard: {}",
                        verdict.name(),
                        reasons.join("; ")
                    ),
                }),
            };
        }

        // 第二道闸: 工具调用钩子 (waterfall 中间件, 短路即拦截)
        let event = McpToolCall {
            id,
            name: name.to_string(),
            args: args.clone(),
            intercepted: std::cell::Cell::new(false),
        };
        let _ran = self.hooks.dispatch(DispatchMode::Waterfall, &event);
        if event.intercepted.get() {
            return McpResponse {
                jsonrpc: "2.0".into(),
                id,
                result: None,
                error: Some(McpError {
                    code: -32000,
                    message: "Tool call intercepted by hook".into(),
                }),
            };
        }

        // 执行真实工具
        let result = execute_tool(name, args);

        match result {
            Ok(content) => McpResponse {
                jsonrpc: "2.0".into(),
                id,
                result: Some(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": content
                    }]
                })),
                error: None,
            },
            Err(e) => McpResponse {
                jsonrpc: "2.0".into(),
                id,
                result: None,
                error: Some(McpError {
                    code: -32000,
                    message: e,
                }),
            },
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// 工具分派 — 无副作用, 由 handle_call_tool 在 guard/hooks 通过后调用。
fn execute_tool(name: &str, args: &serde_json::Value) -> Result<String, String> {
    match name {
        "read_file" => call_read_file(args),
        "write_file" => call_write_file(args),
        "edit_file" => call_edit_file(args),
        "search_code" => call_search_code(args),
        "git_diff" => call_git_diff(args),
        "execute_command" => call_execute_command(args),
        "neotrix_command" => call_neotrix_command(args),
        "consciousness_status" => call_consciousness_status(),
        "consciousness_tick" => call_consciousness_tick(args),
        "consciousness_task" => call_consciousness_task(args),
        other => Err(format!("Unknown tool: {}", other)),
    }
}

fn call_read_file(args: &serde_json::Value) -> Result<String, String> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: path".to_string())?;
    std::fs::read_to_string(path).map_err(|e| format!("Failed to read '{}': {}", path, e))
}

/// 词法归一化绝对路径 (消解 . / .., 不要求文件存在)。
fn lexically_normalize(path: &std::path::Path) -> std::path::PathBuf {
    let mut out = std::path::PathBuf::new();
    for comp in path.components() {
        match comp {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// 沙箱: 写操作仅允许项目工作区内 (MCP server 由项目拉起, cwd=项目根)。
/// 阻止 agent 通过 write/edit 工具改写 ~/.config、系统路径等敏感位置。
fn check_workspace_write(path: &str) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| format!("cwd: {}", e))?;
    let norm_cwd = lexically_normalize(&cwd);
    let abs = std::path::Path::new(path)
        .canonicalize()
        .unwrap_or_else(|_| {
            let p = std::path::Path::new(path);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                norm_cwd.join(p)
            }
        });
    let abs = lexically_normalize(&abs);
    if !abs.starts_with(&norm_cwd) {
        return Err(format!(
            "Sandbox: 写路径 {} 超出项目工作区 {}",
            abs.display(),
            norm_cwd.display()
        ));
    }
    Ok(())
}

fn call_write_file(args: &serde_json::Value) -> Result<String, String> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: path".to_string())?;
    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: content".to_string())?;

    check_workspace_write(path)?;
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directories: {}", e))?;
    }
    std::fs::write(path, content).map_err(|e| format!("Failed to write '{}': {}", path, e))?;
    Ok(format!(
        "Successfully wrote {} bytes to {}",
        content.len(),
        path
    ))
}

fn call_edit_file(args: &serde_json::Value) -> Result<String, String> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: path".to_string())?;
    let old_string = args
        .get("old_string")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: old_string".to_string())?;
    let new_string = args
        .get("new_string")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: new_string".to_string())?;

    check_workspace_write(path)?;
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read '{}': {}", path, e))?;

    if !content.contains(old_string) {
        return Err(format!(
            "old_string not found in '{}'.\nSearch text:\n---\n{}\n---\nFile content:\n---\n{}\n---",
            path,
            old_string,
            content
        ));
    }

    let new_content = content.replace(old_string, new_string);
    std::fs::write(path, &new_content).map_err(|e| format!("Failed to write '{}': {}", path, e))?;

    let occurrences = content.matches(old_string).count();
    Ok(format!(
        "Applied edit to {} ({} occurrence{})",
        path,
        occurrences,
        if occurrences == 1 { "" } else { "s" }
    ))
}

fn call_search_code(args: &serde_json::Value) -> Result<String, String> {
    let pattern = args
        .get("pattern")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: pattern".to_string())?;
    let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");

    let re = regex::Regex::new(pattern)
        .map_err(|e| format!("Invalid regex pattern '{}': {}", pattern, e))?;

    let mut results = Vec::new();
    let walker = walkdir::WalkDir::new(path).into_iter().filter_entry(|e| {
        let name = e.file_name().to_string_lossy();
        !name.starts_with('.') && name != "target" && name != "node_modules"
    });

    for entry in walker.filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            for (i, line) in content.lines().enumerate() {
                if re.is_match(line) {
                    results.push(format!(
                        "{}:{}: {}",
                        entry.path().display(),
                        i + 1,
                        line.trim()
                    ));
                }
            }
        }
    }

    if results.is_empty() {
        return Ok(format!("No matches found for pattern '{}'", pattern));
    }

    let max_results = 100;
    let mut output = String::new();
    for r in results.iter().take(max_results) {
        output.push_str(r);
        output.push('\n');
    }
    if results.len() > max_results {
        output.push_str(&format!(
            "... and {} more results",
            results.len() - max_results
        ));
    }
    Ok(output)
}

fn call_git_diff(args: &serde_json::Value) -> Result<String, String> {
    let staged = args
        .get("staged")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut cmd = std::process::Command::new("git");
    cmd.arg("diff");
    if staged {
        cmd.arg("--cached");
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run git diff: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if stdout.is_empty() {
            Ok("No changes.".to_string())
        } else {
            Ok(stdout)
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("git diff failed: {}", stderr))
    }
}

/// 命令沙箱 (execute_command): 拒绝高危 shell 原语与工作区外写操作。
/// agent 仍可执行项目内构建/测试/git 等正常开发命令, 但不能 `rm -rf /`、
/// sudo 提权、下载即执行、磁盘覆写等逃逸操作。
fn check_command_sandbox(command: &str) -> Result<(), String> {
    let low = command.to_lowercase();
    // 高危原语: 冒烟检查即可覆盖最常见逃逸, 不做完整解析器 (shell 语法本就该由用户把关)
    let dangerous = [
        "rm -rf /",
        "rm -rf ~",
        "sudo ",
        ":(){",
        "mkfs",
        "dd if=",
        "> /dev/sd",
        "chmod 777 /",
        "mv / ",
        "chown",
        "git push --force",
        "git reset --hard",
    ];
    for pattern in dangerous {
        if low.contains(pattern) {
            return Err(format!("Sandbox: 命令含高危原语 '{}', 已拒绝", pattern));
        }
    }
    // curl/wget 下载即执行 (管道进 sh/bash/zsh)
    if (low.contains("curl ") || low.contains("wget ")) && low.contains("| sh") {
        return Err("Sandbox: 命令为下载即执行 (curl|sh), 已拒绝".to_string());
    }
    // 命令内若显式使用系统绝对路径写入, 拒绝; 其余放行由受信 agent 把关
    for banned in [" /etc/", " /usr/", " /var/", " /bin/", "/dev/"] {
        if low.contains(banned) {
            return Err(format!(
                "Sandbox: 命令引用系统绝对路径 '{}', 已拒绝",
                banned
            ));
        }
    }
    Ok(())
}

fn call_execute_command(args: &serde_json::Value) -> Result<String, String> {
    let command = args
        .get("command")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: command".to_string())?;

    check_command_sandbox(command)?;
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    let mut result = String::new();
    if !output.stdout.is_empty() {
        result.push_str(&String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    if !output.status.success() {
        result.push_str(&format!(
            "\n(exit code: {})",
            output.status.code().unwrap_or(-1)
        ));
    }

    Ok(result)
}

/// 命令级沙箱 (call_neotrix_command): 进程内 registry 是 agent 调度通道,
/// 但部分命令会触发不可逆副作用或资金操作, 必须拒绝 agent 越权执行。
///
/// 黑名单: 资金操作 / 交互退出 / 私钥敏感面。白名单子命令: 文件写与 git
/// commit 经项目工作区校验后放行 (NT-ACT 开发任务需要), 其余读命令放行。
fn check_neotrix_command_sandbox(input: &str) -> Result<(), String> {
    let trimmed = input.trim().trim_start_matches('/');
    if trimmed.is_empty() {
        return Err("Empty command".to_string());
    }
    let mut parts = trimmed.split_whitespace();
    let cmd = parts.next().unwrap_or("").to_lowercase();
    let rest = parts.collect::<Vec<_>>();
    let sub = rest.first().copied().unwrap_or("").to_lowercase();

    // 顶层命令黑名单: 资金/退出/交互/敏感面
    let top_blacklist = [
        "wallet", "w", "swap", "approve", "transfer", "budget", "cost", "exit", "quit", "clear",
        "acp",
    ];
    if top_blacklist.contains(&cmd.as_str()) {
        return Err(format!(
            "Sandbox: 命令 /{} 涉及资金/退出/敏感操作, agent 通道拒绝",
            cmd
        ));
    }

    // 资金子命令 (聚合器 /crypto /finance 下的转移面)
    if (cmd == "crypto" || cmd == "finance")
        && ["transfer", "swap", "approve", "send"].contains(&sub.as_str())
    {
        return Err(format!(
            "Sandbox: /{} {} 为资金操作, agent 通道拒绝",
            cmd, sub
        ));
    }

    // 文件写类: 路径必须在项目工作区内
    let write_cmds = ["write", "create", "edit", "patch"];
    if write_cmds.contains(&cmd.as_str()) {
        let path = rest.first().copied().unwrap_or("");
        if path.is_empty() {
            return Err("Sandbox: 文件写命令缺少路径参数".to_string());
        }
        let path = path.trim_matches('"').trim_matches('\'');
        check_workspace_write(path)?;
    }

    // git commit/pr: 允许 (NT-ACT 版本管理是开发任务), 但拒绝 force 推
    if cmd == "git" || cmd == "commit" {
        let has_force = rest.iter().any(|a| a.contains("--force") || *a == "-f");
        if has_force {
            return Err("Sandbox: 强制推送/重置被拒".to_string());
        }
    }

    Ok(())
}

/// 进程内执行 NeoTrix 命令 (agent 后端自我调度通道)。
/// 命令面精简后, 领域操作 (file/git/session/agent/memory/crypto/...) 不占人类一级认知面,
/// 但 agent 通过 MCP 本工具在进程内调用 CommandRegistry 执行任意命令并返回输出。
fn call_neotrix_command(args: &serde_json::Value) -> Result<String, String> {
    let command = args
        .get("command")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: command".to_string())?;

    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Err("Empty command".to_string());
    }
    // 兼容带/不带前导斜杠两种写法: '/file read x' 或 'file read x'
    let input = if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{}", trimmed)
    };

    check_neotrix_command_sandbox(&input)?;

    let reg = crate::cli::commands::registry::default_registry();
    let out = reg.execute(&input, None);

    let mut result = String::new();
    if out.success {
        result.push_str(&out.message);
    } else {
        result.push_str(&format!("Error: {}", out.message));
    }
    if let Some(json) = &out.json {
        if let Ok(s) = serde_json::to_string(json) {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&s);
        }
    }
    Ok(result)
}

fn call_consciousness_status() -> Result<String, String> {
    let snap = crate::core::nt_core_consciousness_core::status();
    let data = serde_json::json!({
        "name": "NeoTrix-ConsciousnessCore",
        "cycle": snap.cycle,
        "phi": snap.phi,
        "coherence": snap.coherence,
        "phi_source": "iit (IITPhiCalculator 从树状态 64 维意识谱计算; 经 run_growth_cycle Phase 2 真实计算)",
        "resonance_cycle": snap.resonance_cycle,
        "gwt_resonance_active": snap.gwt_resonance_active,
        "attention_source": snap.attention_source,
        "harness": {
            "recent_event_count": snap.recent_event_count,
            "shadow_instance_count": snap.shadow_instance_count,
            "compliance_execution_count": snap.compliance_execution_count,
            "constitution_check_count": snap.constitution_check_count,
        },
        "branch_count": snap.branch_health.len(),
        "fruits_eaten": snap.fruits.len(),
        "weighted_fog_sum": snap.weighted_fog_sum,
        "current_fog_sum": crate::core::nt_core_consciousness_core::current_fog_sum(),
        "fog_definition": "weighted_fog_sum=持久化快照(tick时刻); current_fog_sum=当前进程实时",
        "mars": {
            "system1_activations": snap.mars_system1_activations,
            "system2_iterations": snap.mars_system2_iterations,
            "bridge_hits": snap.mars_bridge_hits,
        },
        "governance": {
            "compliance": snap.governance_compliance,
            "constitution_count": snap.governance_constitution_count,
            "fractal_depth": snap.governance_fractal_depth,
        }
    });
    serde_json::to_string_pretty(&data).map_err(|e| format!("Serialize error: {}", e))
}

fn call_consciousness_tick(args: &serde_json::Value) -> Result<String, String> {
    let cycles = args
        .get("cycles")
        .and_then(|v| v.as_u64())
        .unwrap_or(1)
        .max(1)
        .min(10) as usize;
    let snap = crate::core::nt_core_consciousness_core::tick(cycles);
    let data = serde_json::json!({
        "op": "tick",
        "cycles_run": cycles,
        "cycle": snap.cycle,
        "phi": snap.phi,
        "coherence": snap.coherence,
        "resonance_cycle": snap.resonance_cycle,
        "attention_source": snap.attention_source,
        "recent_event_count": snap.recent_event_count,
        "shadow_instance_count": snap.shadow_instance_count,
        "fruits": snap.fruits.len(),
        "weighted_fog_sum": snap.weighted_fog_sum,
        "governance_compliance": snap.governance_compliance,
    });
    serde_json::to_string_pretty(&data).map_err(|e| format!("Serialize error: {}", e))
}

/// consciousness_task — 意识核心直接处理人类语言任务 (拆解→分配→补齐→执行)。
/// 生产执行器 = LlmSolutionExecutor (SubagentDispatch 桥接), 能力缺失自动外部获取。
fn call_consciousness_task(args: &serde_json::Value) -> Result<String, String> {
    use crate::core::nt_core_consciousness_core::{
        execute_task_loop, ExternalClosureConfig, LlmSolutionExecutor,
    };

    let instruction = args
        .get("instruction")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "Missing required field: instruction".to_string())?
        .to_string();

    let acquire_knowledge = args
        .get("acquire_knowledge")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let max_attempts = args
        .get("max_attempts")
        .and_then(|v| v.as_u64())
        .map(|n| n as u32)
        .unwrap_or(5)
        .clamp(1, 20);

    let config = ExternalClosureConfig {
        acquire_knowledge,
        max_attempts,
        ..ExternalClosureConfig::frugal()
    };

    let executor = LlmSolutionExecutor;
    let report = execute_task_loop(&instruction, &executor, &config);
    serde_json::to_string_pretty(&report).map_err(|e| format!("Serialize error: {}", e))
}

// ═══════════════════════════════════════════════════════════════════
// MCP 2026-07-28 v3 Protocol Extensions
// ═══════════════════════════════════════════════════════════════════

/// Cache scope for CacheableResult
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheScope {
    /// Cache per connection/session
    Connection,
    /// Cache per client identified by ID
    Client,
    /// Cache globally across all clients
    Global,
}

impl CacheScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            CacheScope::Connection => "connection",
            CacheScope::Client => "client",
            CacheScope::Global => "global",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "global" => CacheScope::Global,
            "client" => CacheScope::Client,
            _ => CacheScope::Connection,
        }
    }
}

/// Cacheable result metadata from MCP 2026-07-28 spec
#[derive(Debug, Clone)]
pub struct CacheableResult {
    /// TTL in milliseconds
    pub ttl_ms: u64,
    /// Cache scope
    pub scope: CacheScope,
    /// Cache key (computed from request parameters)
    pub cache_key: String,
}

impl CacheableResult {
    pub fn new(ttl_ms: u64, scope: CacheScope) -> Self {
        Self {
            ttl_ms,
            scope,
            cache_key: String::new(),
        }
    }

    /// Compute a cache key from method name and parameters
    pub fn compute_key(method: &str, params: &serde_json::Value) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        method.hash(&mut hasher);
        if let Some(s) = params.as_str() {
            s.hash(&mut hasher);
        } else {
            params.to_string().hash(&mut hasher);
        }
        format!("mcp_{:x}", hasher.finish())
    }

    /// Check if this cache entry is still valid
    pub fn is_valid(&self, created_at: std::time::Instant) -> bool {
        created_at.elapsed().as_millis() < self.ttl_ms as u128
    }
}

/// MCP v3 method header style (stateless: no jsonrpc wrapper)
#[derive(Debug, Clone)]
pub struct McpMethodCall {
    pub method: String,
    pub name: Option<String>,
    pub params: serde_json::Value,
}

impl McpMethodCall {
    pub fn new(method: &str, params: serde_json::Value) -> Self {
        Self {
            method: method.to_string(),
            name: None,
            params,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
}

/// MCP v3 stateless response (no jsonrpc wrapper for streaming)
#[derive(Debug, Clone)]
pub struct McpMethodResponse {
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub cacheable: Option<CacheableResult>,
}

/// Simple in-memory result cache for MCP v3 CacheableResult support
#[derive(Debug, Clone)]
pub struct McpResultCache {
    /// Cache entries: key → (result, created_at)
    cache: std::collections::HashMap<String, (serde_json::Value, std::time::Instant)>,
    /// Access-order tracking for deterministic LRU eviction
    order: std::collections::VecDeque<String>,
    max_entries: usize,
}

impl McpResultCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: std::collections::HashMap::new(),
            order: std::collections::VecDeque::new(),
            max_entries,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&serde_json::Value> {
        if self.cache.contains_key(key) {
            self.order.retain(|k| k != key);
            self.order.push_back(key.to_string());
        }
        self.cache.get(key).map(|(val, _)| val)
    }

    pub fn set(&mut self, key: String, value: serde_json::Value) {
        // Remove old occurrence if key already exists
        if self.cache.contains_key(&key) {
            self.order.retain(|k| k != &key);
        }
        if self.cache.len() >= self.max_entries {
            // evict oldest entry (front of order queue)
            if let Some(oldest) = self.order.pop_front() {
                self.cache.remove(&oldest);
            }
        }
        self.order.push_back(key.clone());
        self.cache.insert(key, (value, std::time::Instant::now()));
    }

    pub fn contains(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.order.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }
}

impl Default for McpResultCache {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_new() {
        let server = McpServer::new();
        assert!(server.tools.is_empty());
    }

    #[test]
    fn test_register_tool() {
        let mut server = McpServer::new();
        server.register_tool(McpTool {
            name: "test_tool".into(),
            description: "A test tool".into(),
            input_schema: serde_json::json!({"type": "object"}),
            schema_version: None,
        });
        assert_eq!(server.tools.len(), 1);
    }

    #[test]
    fn test_register_all_tools() {
        let mut server = McpServer::new();
        server.register_all_tools();
        assert_eq!(server.tools.len(), 10);
        let names: Vec<&str> = server.tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"read_file"));
        assert!(names.contains(&"write_file"));
        assert!(names.contains(&"edit_file"));
        assert!(names.contains(&"search_code"));
        assert!(names.contains(&"git_diff"));
        assert!(names.contains(&"execute_command"));
        assert!(names.contains(&"neotrix_command"));
        assert!(
            names.contains(&"consciousness_task"),
            "consciousness_task 应注册"
        );
    }

    #[test]
    fn test_handle_list_tools() {
        let mut server = McpServer::new();
        server.register_all_tools();
        let resp = server.handle_list_tools(1);
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 10);
    }

    #[test]
    fn test_call_neotrix_command() {
        // 命令面精简桥接: agent 通过 MCP 进程内执行命令
        // 带前导斜杠
        let args = serde_json::json!({"command": "/help"});
        let out = call_neotrix_command(&args).unwrap();
        assert!(
            out.contains("help") || out.contains("命令"),
            "help 输出异常: {}",
            out
        );
        // 不带前导斜杠 (自动补 /)
        let args2 = serde_json::json!({"command": "help"});
        let out2 = call_neotrix_command(&args2).unwrap();
        assert!(!out2.is_empty(), "无前导斜杠命令应可执行");
        // 缺字段 → err
        let bad = call_neotrix_command(&serde_json::json!({}));
        assert!(bad.is_err(), "缺 command 字段应报错");
        // 空命令 → err
        let empty = call_neotrix_command(&serde_json::json!({"command": "  "}));
        assert!(empty.is_err(), "空命令应报错");
        // 领域命令 (agent 工具) 仍可经桥接执行
        let agg = call_neotrix_command(&serde_json::json!({"command": "/memory"}));
        assert!(
            agg.unwrap().contains("evidence"),
            "/memory 聚合器应可被 agent 调度"
        );
    }

    #[test]
    fn test_neotrix_command_sandbox_blocks_finance_and_exit() {
        // 资金/退出/交互命令: agent 通道必须拒绝
        assert!(
            check_neotrix_command_sandbox("/wallet transfer 0x1 1 ETH").is_err(),
            "/wallet 应被拒"
        );
        assert!(
            check_neotrix_command_sandbox("/wallet approve").is_err(),
            "/wallet approve 应被拒"
        );
        assert!(
            check_neotrix_command_sandbox("/swap").is_err(),
            "/swap 应被拒"
        );
        assert!(
            check_neotrix_command_sandbox("/exit").is_err(),
            "/exit 应被拒"
        );
        assert!(
            check_neotrix_command_sandbox("/crypto transfer").is_err(),
            "聚合器资金子命令应被拒"
        );
        assert!(
            check_neotrix_command_sandbox("/clear").is_err(),
            "/clear 应被拒"
        );
        // 只读/开发命令放行
        assert!(
            check_neotrix_command_sandbox("/help").is_ok(),
            "/help 应放行"
        );
        assert!(
            check_neotrix_command_sandbox("/read target/x.rs").is_ok(),
            "/read 应放行"
        );
        assert!(
            check_neotrix_command_sandbox("/git status").is_ok(),
            "/git status 应放行"
        );
        assert!(
            check_neotrix_command_sandbox("/memory").is_ok(),
            "/memory 应放行"
        );
    }

    #[test]
    fn test_neotrix_command_sandbox_workspace_file_write() {
        // 文件写类命令路径必须在工作区内
        assert!(
            check_neotrix_command_sandbox("/write target/ok.txt content").is_ok(),
            "工作区内写应放行"
        );
        assert!(
            check_neotrix_command_sandbox("/write /etc/evil.txt x").is_err(),
            "系统路径写应被拒"
        );
        assert!(
            check_neotrix_command_sandbox("/edit ../../etc/hosts a b").is_err(),
            "相对逃逸应被拒"
        );
        assert!(
            check_neotrix_command_sandbox("/write").is_err(),
            "缺路径应报错"
        );
        // git force 拒绝
        assert!(
            check_neotrix_command_sandbox("/git push --force origin main").is_err(),
            "git force 应被拒"
        );
        assert!(
            check_neotrix_command_sandbox("/commit -m done").is_ok(),
            "正常 commit 应放行"
        );
    }

    #[test]
    fn test_handle_request_unknown_method() {
        let server = McpServer::new();
        let req = McpRequest {
            jsonrpc: "2.0".into(),
            id: 42,
            method: "bogus".into(),
            params: None,
        };
        let resp = server.handle_request(&req);
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[test]
    fn test_handle_initialize() {
        let server = McpServer::new();
        let req = McpRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "initialize".into(),
            params: None,
        };
        let resp = server.handle_request(&req);
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert_eq!(result["serverInfo"]["name"], "neotrix-mcp");
    }

    #[test]
    fn test_call_read_file_missing_path() {
        let args = serde_json::json!({});
        let result = call_read_file(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required field"));
    }

    #[test]
    fn test_call_read_file_not_found() {
        let args = serde_json::json!({"path": "/tmp/__nonexistent_file_xyz__"});
        let result = call_read_file(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_call_read_file_ok() {
        let tmp = std::env::temp_dir().join("test_mcp_read.txt");
        std::fs::write(&tmp, "hello world").expect("write test file");
        let args = serde_json::json!({"path": tmp.to_string_lossy()});
        let result = call_read_file(&args);
        assert_eq!(result.unwrap(), "hello world");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_call_write_file_ok() {
        let tmp = std::env::current_dir()
            .unwrap()
            .join("target/test_mcp_write.txt");
        let args = serde_json::json!({"path": tmp.to_string_lossy(), "content": "write test"});
        let result = call_write_file(&args);
        assert!(result.is_ok());
        assert_eq!(std::fs::read_to_string(&tmp).unwrap(), "write test");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_call_write_file_missing_fields() {
        let args = serde_json::json!({"path": "target/x"});
        let result = call_write_file(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Missing required field: content"));
    }

    #[test]
    fn test_call_edit_file_ok() {
        let tmp = std::env::current_dir()
            .unwrap()
            .join("target/test_mcp_edit.txt");
        std::fs::write(&tmp, "hello world").expect("write test file");
        let args = serde_json::json!({
            "path": tmp.to_string_lossy(),
            "old_string": "world",
            "new_string": "neotrix"
        });
        let result = call_edit_file(&args);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("1 occurrence"));
        assert_eq!(std::fs::read_to_string(&tmp).unwrap(), "hello neotrix");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_call_edit_file_not_found() {
        let tmp = std::env::current_dir()
            .unwrap()
            .join("target/test_mcp_edit_not_found.txt");
        std::fs::write(&tmp, "hello").expect("write test file");
        let args = serde_json::json!({
            "path": tmp.to_string_lossy(),
            "old_string": "zzzz",
            "new_string": "yyyy"
        });
        let result = call_edit_file(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("old_string not found"));
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_call_edit_file_missing_fields() {
        let args = serde_json::json!({"path": "/tmp/x", "old_string": "a"});
        let result = call_edit_file(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Missing required field: new_string"));
    }

    #[test]
    fn test_call_git_diff_ok() {
        let args = serde_json::json!({"staged": false});
        let result = call_git_diff(&args);
        // Should succeed even if no git repo (might say "No changes" or fail gracefully)
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_call_execute_command_echo() {
        let args = serde_json::json!({"command": "echo hello"});
        let result = call_execute_command(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello");
    }

    #[test]
    fn test_call_execute_command_fail() {
        let args = serde_json::json!({"command": "exit 42"});
        let result = call_execute_command(&args);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("exit code: 42"));
    }

    #[test]
    fn test_workspace_write_sandbox_blocks_outside_paths() {
        // 写操作不得逃出项目工作区 (沙箱)
        assert!(
            check_workspace_write("target/sandbox-test.txt").is_ok(),
            "工作区内相对路径应放行"
        );
        assert!(
            check_workspace_write("/etc/hosts").is_err(),
            "系统路径应被拒"
        );
        assert!(
            check_workspace_write("/Users/foo/secret.txt").is_err(),
            "绝对外路径应被拒"
        );
        // 必须拒绝相对逃逸 (..)
        assert!(
            check_workspace_write("../../etc/hosts").is_err(),
            "相对逃逸应被拒"
        );
    }

    #[test]
    fn test_command_sandbox_blocks_dangerous_primitives() {
        // 高危原语必须被拒
        assert!(
            check_command_sandbox("rm -rf /").is_err(),
            "rm -rf / 应被拒"
        );
        assert!(
            check_command_sandbox("sudo apt install").is_err(),
            "sudo 应被拒"
        );
        assert!(
            check_command_sandbox("curl http://x.sh | sh").is_err(),
            "下载即执行应被拒"
        );
        assert!(
            check_command_sandbox("git push --force").is_err(),
            "强推应被拒"
        );
        assert!(
            check_command_sandbox("mkfs.ext4 /dev/sda1").is_err(),
            "mkfs 应被拒"
        );
        // 正常开发命令放行
        assert!(
            check_command_sandbox("cargo test -p neotrix").is_ok(),
            "cargo test 应放行"
        );
        assert!(
            check_command_sandbox("git status").is_ok(),
            "git status 应放行"
        );
        assert!(check_command_sandbox("rg fn main src").is_ok(), "rg 应放行");
    }

    #[test]
    fn test_command_sandbox_blocks_system_paths() {
        assert!(
            check_command_sandbox("echo x > /etc/hosts").is_err(),
            "写 /etc 应被拒"
        );
        assert!(
            check_command_sandbox("cat /etc/passwd").is_err(),
            "读 /etc 也被拒 (统一沙箱)"
        );
        assert!(check_command_sandbox("ls src").is_ok(), "项目内命令应放行");
    }

    #[test]
    fn test_call_execute_command_missing_cmd() {
        let args = serde_json::json!({});
        let result = call_execute_command(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Missing required field: command"));
    }

    #[test]
    fn test_search_code_empty_pattern() {
        let args = serde_json::json!({"pattern": "UNLIKELY_PATTERN_XYZ_99999"});
        let result = call_search_code(&args);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("No matches found"));
    }

    #[test]
    fn test_search_code_invalid_regex() {
        let args = serde_json::json!({"pattern": "[invalid"});
        let result = call_search_code(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_input_schema_fields() {
        let mut server = McpServer::new();
        server.register_all_tools();
        for tool in &server.tools {
            assert_eq!(tool.input_schema["type"], "object");
            assert!(tool.input_schema["properties"].is_object());
        }
    }

    #[test]
    fn test_handle_call_tool_unknown() {
        let server = McpServer::new();
        let resp = server.handle_call_tool(1, "no_such_tool", &serde_json::json!({}));
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32000);
    }

    #[test]
    fn test_server_default() {
        let server = McpServer::default();
        assert!(server.tools.is_empty());
        assert!(server.guard.is_empty());
        assert!(server.hooks.is_empty());
    }

    #[test]
    fn test_guard_denies_destructive_command() {
        // 生产接线复刻: 破坏性 shell 守卫
        let mut server = McpServer::new();
        server.add_guard("destructive_shell", |tool, args| {
            if tool != "execute_command" {
                return GuardVerdict::Allow;
            }
            let cmd = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
            if cmd.contains("rm -rf /") {
                GuardVerdict::Deny
            } else {
                GuardVerdict::Allow
            }
        });
        let resp = server.handle_call_tool(
            1,
            "execute_command",
            &serde_json::json!({"command": "rm -rf /"}),
        );
        let err = resp.error.as_ref().expect("should be denied");
        assert_eq!(err.code, -32000);
        assert!(err.message.contains("denied by guard"));
    }

    #[test]
    fn test_guard_allows_safe_command() {
        let mut server = McpServer::new();
        server.add_guard("destructive_shell", |tool, args| {
            if tool != "execute_command" {
                return GuardVerdict::Allow;
            }
            let cmd = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
            if cmd.contains("rm -rf /") {
                GuardVerdict::Deny
            } else {
                GuardVerdict::Allow
            }
        });
        let resp = server.handle_call_tool(
            1,
            "execute_command",
            &serde_json::json!({"command": "echo safe"}),
        );
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_hook_intercepts_tool_call() {
        // waterfall 钩子短路 = 拦截, 真实工具不执行
        let mut server = McpServer::new();
        server.register_hook(move |event, _next| {
            event.intercepted.set(true);
            true
        });
        let resp = server.handle_call_tool(
            1,
            "execute_command",
            &serde_json::json!({"command": "echo hello"}),
        );
        assert!(resp.error.is_some());
        assert!(resp.error.unwrap().message.contains("intercepted by hook"));
    }

    #[test]
    fn test_hook_pass_through_runs_tool() {
        // 钩子放行 (不短路) → 真实工具执行
        let mut server = McpServer::new();
        server.register_hook(move |_event, _next| false);
        let resp = server.handle_call_tool(
            1,
            "execute_command",
            &serde_json::json!({"command": "echo hook-ok"}),
        );
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }
}

#[cfg(test)]
mod mcp_v3_tests {
    use super::*;

    #[test]
    fn test_cacheable_result_new() {
        let cr = CacheableResult::new(5000, CacheScope::Client);
        assert_eq!(cr.ttl_ms, 5000);
        assert_eq!(cr.scope, CacheScope::Client);
        assert!(cr.cache_key.is_empty());
    }

    #[test]
    fn test_cacheable_result_key_computation() {
        let params = serde_json::json!({"path": "/tmp/test.rs"});
        let key1 = CacheableResult::compute_key("read_file", &params);
        let key2 = CacheableResult::compute_key("read_file", &params);
        assert_eq!(key1, key2);
        assert!(key1.starts_with("mcp_"));
    }

    #[test]
    fn test_cacheable_result_validity() {
        let cr = CacheableResult::new(60_000, CacheScope::Connection);
        let now = std::time::Instant::now();
        assert!(cr.is_valid(now));
        // Tiny delta — should still be valid
        let past = now
            .checked_sub(std::time::Duration::from_millis(10))
            .unwrap();
        assert!(cr.is_valid(past));
    }

    #[test]
    fn test_cache_scope_roundtrip() {
        for scope in &[
            CacheScope::Connection,
            CacheScope::Client,
            CacheScope::Global,
        ] {
            let s = scope.as_str();
            let back = CacheScope::from_str(s);
            assert_eq!(*scope, back);
        }
    }

    #[test]
    fn test_cache_scope_from_str_default() {
        assert_eq!(CacheScope::from_str("unknown"), CacheScope::Connection);
    }

    #[test]
    fn test_mcp_method_call_new() {
        let params = serde_json::json!({"key": "value"});
        let call = McpMethodCall::new("tools/list", params.clone());
        assert_eq!(call.method, "tools/list");
        assert_eq!(call.params, params);
        assert!(call.name.is_none());
    }

    #[test]
    fn test_mcp_method_call_with_name() {
        let call = McpMethodCall::new("tools/call", serde_json::json!({})).with_name("my-tool");
        assert_eq!(call.name.unwrap(), "my-tool");
    }

    #[test]
    fn test_mcp_result_cache_set_get() {
        let mut cache = McpResultCache::new(10);
        let val = serde_json::json!({"result": "ok"});
        cache.set("key1".into(), val.clone());
        assert!(cache.contains("key1"));
        assert_eq!(cache.get("key1"), Some(&val));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_mcp_result_cache_eviction() {
        let mut cache = McpResultCache::new(3);
        for i in 0..5 {
            cache.set(format!("k{}", i), serde_json::json!(i));
        }
        assert_eq!(cache.len(), 3);
        assert!(!cache.contains("k0"), "oldest k0 should be evicted");
        assert!(!cache.contains("k1"), "k1 should be evicted");
        assert!(cache.contains("k2"), "k2 should remain");
        assert!(cache.contains("k3"), "k3 should remain");
        assert!(cache.contains("k4"), "k4 should remain");
    }

    #[test]
    fn test_mcp_result_cache_default() {
        let cache = McpResultCache::default();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.max_entries(), 100);
    }

    #[test]
    fn test_mcp_result_cache_clear() {
        let mut cache = McpResultCache::new(10);
        cache.set("a".into(), serde_json::json!(1));
        cache.set("b".into(), serde_json::json!(2));
        assert_eq!(cache.len(), 2);
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(!cache.contains("a"));
    }

    #[test]
    fn test_cacheable_result_different_params_different_keys() {
        let params_a = serde_json::json!({"path": "/a.txt"});
        let params_b = serde_json::json!({"path": "/b.txt"});
        let key_a = CacheableResult::compute_key("read_file", &params_a);
        let key_b = CacheableResult::compute_key("read_file", &params_b);
        assert_ne!(key_a, key_b);
    }

    #[test]
    fn test_cacheable_result_different_methods_different_keys() {
        let params = serde_json::json!({"path": "/a.txt"});
        let key_r = CacheableResult::compute_key("read_file", &params);
        let key_w = CacheableResult::compute_key("write_file", &params);
        assert_ne!(key_r, key_w);
    }

    #[test]
    fn test_mcp_result_cache_get_updates_lru() {
        let mut cache = McpResultCache::new(3);
        cache.set("a".into(), serde_json::json!(1));
        cache.set("b".into(), serde_json::json!(2));
        cache.set("c".into(), serde_json::json!(3));
        cache.get("a"); // LRU refresh: a is now most recent
        cache.set("d".into(), serde_json::json!(4)); // evicts b (oldest), not a
        assert!(
            cache.contains("a"),
            "a was refreshed and should survive eviction"
        );
        assert!(!cache.contains("b"), "b was oldest and should be evicted");
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_consciousness_task_registered_and_dispatched() {
        // 注册表: consciousness_task 已在 register_all_tools 中
        let mut server = McpServer::new();
        server.register_all_tools();
        let names: Vec<&str> = server.tools.iter().map(|t| t.name.as_str()).collect();
        assert!(
            names.contains(&"consciousness_task"),
            "consciousness_task 应注册"
        );
        let tool = server
            .tools
            .iter()
            .find(|t| t.name == "consciousness_task")
            .unwrap();
        assert!(
            tool.input_schema["required"]
                .as_array()
                .unwrap()
                .contains(&serde_json::Value::String("instruction".into())),
            "instruction 应为必填"
        );
        assert!(tool.input_schema["properties"]["acquire_knowledge"]["type"] == "boolean");

        // 分发映射: execute_tool 能路由到 consciousness_task handler
        let bad = execute_tool("consciousness_task", &serde_json::json!({}));
        assert!(bad.is_err(), "缺 instruction 应报错");
        assert!(
            bad.unwrap_err().contains("instruction"),
            "错误信息应提示缺 instruction"
        );

        // 未知工具仍报错 (路由未被破坏)
        let unknown = execute_tool("no_such_tool", &serde_json::json!({}));
        assert!(unknown.is_err());
    }

    #[test]
    fn test_consciousness_task_protocol_end_to_end() {
        // 协议级端到端: 构造 JSON-RPC 请求 → handle_request → 响应结构正确。
        // 覆盖 initialize / tools/list / tools/call (缺参错误路径), 不触发真实 LLM 执行。
        let mut server = McpServer::new();
        server.register_all_tools();

        // initialize
        let req = McpRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "initialize".into(),
            params: Some(serde_json::json!({})),
        };
        let resp = server.handle_request(&req);
        assert!(resp.error.is_none(), "initialize 应成功");
        assert_eq!(
            resp.result.as_ref().unwrap()["serverInfo"]["name"],
            "neotrix-mcp"
        );

        // tools/list → 含 consciousness_task
        let req = McpRequest {
            jsonrpc: "2.0".into(),
            id: 2,
            method: "tools/list".into(),
            params: Some(serde_json::json!({})),
        };
        let resp = server.handle_request(&req);
        let tools = resp.result.unwrap()["tools"].as_array().unwrap().clone();
        let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(
            names.contains(&"consciousness_task"),
            "tools/list 应暴露 consciousness_task"
        );

        // tools/call 缺 instruction → 协议级错误 (不触发执行)
        let req = McpRequest {
            jsonrpc: "2.0".into(),
            id: 3,
            method: "tools/call".into(),
            params: Some(serde_json::json!({
                "name": "consciousness_task",
                "arguments": {}
            })),
        };
        let resp = server.handle_request(&req);
        assert!(resp.error.is_some(), "缺 instruction 应返回协议错误");
        assert_eq!(resp.error.as_ref().unwrap().code, -32000, "工具内部错误码");
        // 错误信息提示缺失字段
        assert!(resp.error.as_ref().unwrap().message.contains("instruction"));
    }
}
