//! Tauri commands for MCP server hosting
//!
//! NeoTrix can expose its tools as MCP servers so other AI assistants can consume them.
//!
//! 真实 MCP host 实现 (JSON-RPC 2.0 over stdio):
//! - `mcp_host_start` 以子进程方式拉起当前可执行文件 (env `NEOTRIX_MCP_STDIO=1`)，
//!   子进程进入 MCP stdio 循环，父进程通过管道通信并持有 child handle。
//! - 子进程入口见 `run_mcp_stdio` (protocolVersion "2025-03-26"，暴露 6 个真实能力工具)。
//!   main.rs 需在检测到 `NEOTRIX_MCP_STDIO` 时调用它并退出 (注册由主程序 agent 负责)。

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ── Data types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpHostConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub auth_token: Option<String>,
}

impl Default for McpHostConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "127.0.0.1".into(),
            port: 8311,
            max_connections: 10,
            auth_token: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpHostEndpoint {
    pub id: String,
    pub name: String,
    pub description: String,
    pub endpoint_type: String,
    pub parameters: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpHostSession {
    pub client_id: String,
    pub connected_at: u64,
    pub tool_calls: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct McpHostStatus {
    pub running: bool,
    pub port: u16,
    pub uptime_secs: u64,
    pub active_sessions: usize,
    pub total_endpoints: usize,
    pub total_calls: u64,
    pub pid: Option<u32>,
    pub registered_tools: usize,
}

// ── MCP stdio server (JSON-RPC 2.0) ────────────────────────────────────────
//
// 子进程入口: 逐行读取 stdin 上的 JSON-RPC 请求并回写响应。
// 必须遵守: 除响应行外不向 stdout 写入任何内容 (启动横幅走 stderr)。

const MCP_PROTOCOL_VERSION: &str = "2025-03-26";

#[derive(Debug, Deserialize)]
struct McpRequest {
    jsonrpc: String,
    #[serde(default)]
    id: Option<serde_json::Value>,
    method: String,
    #[serde(default)]
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct McpResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<McpError>,
}

#[derive(Debug, Serialize)]
struct McpError {
    code: i32,
    message: String,
}

impl McpError {
    fn new(code: i32, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }
}

/// MCP stdio 主循环入口。由 main.rs 在检测到 `NEOTRIX_MCP_STDIO=1` 时调用，然后退出进程。
/// 当前未接入 main.rs 时该函数是死代码，由注册 agent 接线。
#[allow(dead_code)]
pub fn run_mcp_stdio() {
    eprintln!("neotrix-mcp {} stdio JSON-RPC 2.0 ready", env!("CARGO_PKG_VERSION"));
    let stdin = std::io::stdin();
    let mut out = std::io::BufWriter::new(std::io::stdout());
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("neotrix-mcp stdio read error: {}", e);
                break;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let is_exit = serde_json::from_str::<McpRequest>(line.trim())
            .map(|r| r.method == "exit")
            .unwrap_or(false);
        if let Some(resp) = handle_line(&line) {
            write_json(&mut out, &serde_json::from_str(&resp).unwrap_or_default());
        }
        if is_exit {
            break;
        }
    }
    eprintln!("neotrix-mcp stdio exiting");
}

fn write_json(out: &mut impl Write, value: &serde_json::Value) {
    if let Ok(s) = serde_json::to_string(value) {
        if writeln!(out, "{}", s).is_ok() {
            let _ = out.flush();
        }
    }
}

fn parse_error_response() -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": null,
        "error": { "code": -32700, "message": "Parse error" }
    })
}

/// 单行请求处理入口 (供 stdio 循环与单元测试复用)。
/// 返回序列化响应字符串；通知 (无 id / id=null) 返回 None。
fn handle_line(line: &str) -> Option<String> {
    let req: McpRequest = match serde_json::from_str(line.trim()) {
        Ok(r) => r,
        Err(_) => return Some(parse_error_response().to_string()),
    };
    if req.id.as_ref().map_or(true, serde_json::Value::is_null) {
        return None;
    }
    dispatch(&req).map(|resp| serde_json::to_string(&resp).unwrap_or_default())
}

/// JSON-RPC 方法分发 (仅处理带 id 的请求)。
fn dispatch(req: &McpRequest) -> Option<McpResponse> {
    let id = req.id.clone().unwrap_or(serde_json::Value::Null);
    if req.jsonrpc != "2.0" {
        return Some(make_response(
            &id,
            Err(McpError::new(-32600, "Invalid Request: jsonrpc must be \"2.0\"")),
        ));
    }
    Some(match req.method.as_str() {
        "initialize" => make_response(&id, Ok(serde_json::json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": { "name": "neotrix-mcp", "version": env!("CARGO_PKG_VERSION") }
        }))),
        "ping" => make_response(&id, Ok(serde_json::json!({}))),
        "tools/list" => make_response(&id, Ok(serde_json::json!({ "tools": mcp_tool_defs() }))),
        "tools/call" => handle_tools_call(&id, req.params.as_ref()),
        "exit" => make_response(&id, Ok(serde_json::json!({ "status": "bye" }))),
        other => make_response(&id, Err(McpError::new(-32601, format!("Method not found: {}", other)))),
    })
}

fn make_response(id: &serde_json::Value, result: Result<serde_json::Value, McpError>) -> McpResponse {
    McpResponse {
        jsonrpc: "2.0".into(),
        id: Some(id.clone()),
        result: result.as_ref().ok().cloned(),
        error: result.err(),
    }
}

fn handle_tools_call(id: &serde_json::Value, params: Option<&serde_json::Value>) -> McpResponse {
    let Some(params) = params else {
        return make_response(
            id,
            Err(McpError::new(-32602, "Invalid params: missing params")),
        );
    };
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let args = params.get("arguments").cloned().unwrap_or_else(|| serde_json::json!({}));
    let payload = match call_tool(&name, &args) {
        Ok(output) => serde_json::json!({
            "content": [{ "type": "text", "text": output }],
            "isError": false
        }),
        Err(e) => serde_json::json!({
            "content": [{ "type": "text", "text": e }],
            "isError": true
        }),
    };
    make_response(id, Ok(payload))
}

/// MCP 暴露的真实 NeoTrix 能力工具列表。
fn mcp_tool_defs() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "name": "read_file",
            "description": "Read file contents from disk (returns up to 64KB of text).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path of the file to read." }
                },
                "required": ["path"]
            }
        }),
        serde_json::json!({
            "name": "write_file",
            "description": "Write content to a file on disk, creating parent directories as needed.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path of the file to write." },
                    "content": { "type": "string", "description": "Content to write." }
                },
                "required": ["path", "content"]
            }
        }),
        serde_json::json!({
            "name": "execute_terminal_command",
            "description": "Execute a shell command via /bin/sh and return its stdout/stderr output.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "command": { "type": "string", "description": "Shell command to execute." }
                },
                "required": ["command"]
            }
        }),
        serde_json::json!({
            "name": "web_search",
            "description": "Search the web and return ranked results (DuckDuckGo + fallback).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query." },
                    "count": { "type": "integer", "description": "Max results (default 8, max 20)." }
                },
                "required": ["query"]
            }
        }),
        serde_json::json!({
            "name": "tool_search",
            "description": "Semantic search over NeoTrix tool surface and the web.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query." },
                    "count": { "type": "integer", "description": "Max results." }
                },
                "required": ["query"]
            }
        }),
        serde_json::json!({
            "name": "kb_search",
            "description": "Search the NeoTrix knowledge base (~/.neotrix/knowledge.db).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Knowledge base search query." },
                    "limit": { "type": "integer", "description": "Max results (default 10)." }
                },
                "required": ["query"]
            }
        }),
    ]
}

// ── Tool 真实执行 ───────────────────────────────────────────────────────────

fn call_tool(name: &str, args: &serde_json::Value) -> Result<String, String> {
    match name {
        "read_file" => tool_read_file(args),
        "write_file" => tool_write_file(args),
        "execute_terminal_command" => tool_execute_terminal_command(args),
        "web_search" => tool_web_search(args),
        "tool_search" => tool_tool_search(args),
        "kb_search" => tool_kb_search(args),
        other => Err(format!("Unknown tool: {}", other)),
    }
}

fn tool_read_file(args: &serde_json::Value) -> Result<String, String> {
    let path = args.get("path").and_then(|v| v.as_str()).ok_or("Missing required field: path")?;
    // 路径校验：仅允许主目录内文件（复用 project_cmds 的 resolve_safe_path）
    let safe = super::project_cmds::resolve_safe_path(path)
        .map_err(|e| format!("Path rejected: {}", e))?;
    let content = std::fs::read_to_string(&safe).map_err(|e| format!("Failed to read '{}': {}", safe.display(), e))?;
    let capped: String = content.chars().take(64 * 1024).collect();
    Ok(capped)
}

fn tool_write_file(args: &serde_json::Value) -> Result<String, String> {
    let path = args.get("path").and_then(|v| v.as_str()).ok_or("Missing required field: path")?;
    let content = args.get("content").and_then(|v| v.as_str()).ok_or("Missing required field: content")?;
    // 路径校验：仅允许主目录内文件
    let safe = super::project_cmds::resolve_safe_path(path)
        .map_err(|e| format!("Path rejected: {}", e))?;
    if let Some(parent) = safe.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent directories: {}", e))?;
        }
    }
    std::fs::write(&safe, content).map_err(|e| format!("Failed to write '{}': {}", safe.display(), e))?;
    Ok(format!("Wrote {} bytes to {}", content.len(), safe.display()))
}

fn tool_execute_terminal_command(args: &serde_json::Value) -> Result<String, String> {
    let command = args.get("command").and_then(|v| v.as_str()).ok_or("Missing required field: command")?;
    // 命令白名单校验：复用 remote_cmds 的 validate_remote_command
    super::remote_cmds::validate_remote_command(command)?;
    let output = Command::new("sh")
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
        result.push_str(&format!("\n(exit code: {})", output.status.code().unwrap_or(-1)));
    }
    Ok(result)
}

fn tool_web_search(args: &serde_json::Value) -> Result<String, String> {
    let query = args.get("query").and_then(|v| v.as_str()).ok_or("Missing required field: query")?;
    let count = args.get("count").and_then(|v| v.as_u64()).map(|c| c as usize);
    let results = super::websearch_cmds::web_search(query.to_string(), count)
        .map_err(|e| format!("Web search failed: {}", e))?;
    let mut lines: Vec<String> = results.into_iter()
        .map(|r| format!("{}\n  {} | {}", r.title, r.url, r.snippet))
        .collect();
    if lines.is_empty() {
        lines.push("No results.".into());
    }
    Ok(lines.join("\n"))
}

fn tool_tool_search(args: &serde_json::Value) -> Result<String, String> {
    let query = args.get("query").and_then(|v| v.as_str()).ok_or("Missing required field: query")?;
    let count = args.get("count").and_then(|v| v.as_u64()).map(|c| c as usize);
    let results = super::tool_cmds::tool_search(query.to_string(), count)
        .map_err(|e| format!("Tool search failed: {}", e))?;
    let mut lines: Vec<String> = results.into_iter()
        .map(|r| format!("{}\n  {} | {}", r.title, r.url, r.snippet))
        .collect();
    if lines.is_empty() {
        lines.push("No results.".into());
    }
    Ok(lines.join("\n"))
}

fn tool_kb_search(args: &serde_json::Value) -> Result<String, String> {
    let query = args.get("query").and_then(|v| v.as_str()).ok_or("Missing required field: query")?;
    let limit = args.get("limit").and_then(|v| v.as_u64()).map(|c| c as usize);
    let results = super::kb_cmds::kb_search(query.to_string(), limit)
        .map_err(|e| format!("KB search failed: {}", e))?;
    let mut lines: Vec<String> = results.into_iter()
        .map(|r| {
            let domain = r.domain.as_deref().unwrap_or("?");
            let snippet = r.summary.as_deref().unwrap_or("").chars().take(200).collect::<String>();
            format!("[{}] {} (conf {:.2})\n  {} | {}", r.node_type, r.title, r.confidence, domain, snippet)
        })
        .collect();
    if lines.is_empty() {
        lines.push("No results.".into());
    }
    Ok(lines.join("\n"))
}

// ── State ────────────────────────────────────────────────────────────────────

const MAX_LOG: usize = 200;

struct McpHostState {
    config: McpHostConfig,
    running: bool,
    start_time: u64,
    pid: Option<u32>,
    child: Option<Child>,
    child_stdin: Option<ChildStdin>,
    child_stdout: Option<ChildStdout>,
    endpoints: Vec<McpHostEndpoint>,
    sessions: Vec<McpHostSession>,
    activity_log: VecDeque<serde_json::Value>,
    total_calls: u64,
}

impl McpHostState {
    fn new() -> Self {
        Self {
            config: McpHostConfig::default(),
            running: false,
            start_time: 0,
            pid: None,
            child: None,
            child_stdin: None,
            child_stdout: None,
            endpoints: builtin_endpoints(),
            sessions: Vec::new(),
            activity_log: VecDeque::with_capacity(MAX_LOG),
            total_calls: 0,
        }
    }
}

fn builtin_endpoints() -> Vec<McpHostEndpoint> {
    vec![
        McpHostEndpoint {
            id: "tool_execute".into(),
            name: "Execute Tool".into(),
            description: "Execute a general-purpose tool (web search, file read/write, bash, etc.)".into(),
            endpoint_type: "tool".into(),
            parameters: vec!["tool".into(), "args".into()],
            enabled: true,
        },
        McpHostEndpoint {
            id: "tool_search".into(),
            name: "Web Search".into(),
            description: "Search the web and return ranked results".into(),
            endpoint_type: "tool".into(),
            parameters: vec!["query".into(), "count".into()],
            enabled: true,
        },
        McpHostEndpoint {
            id: "brain_stats".into(),
            name: "Brain Statistics".into(),
            description: "Get the current NeoTrix reasoning brain stats".into(),
            endpoint_type: "resource".into(),
            parameters: vec![],
            enabled: true,
        },
        McpHostEndpoint {
            id: "kb_search".into(),
            name: "Knowledge Base Search".into(),
            description: "Search the NeoTrix knowledge base".into(),
            endpoint_type: "resource".into(),
            parameters: vec!["query".into()],
            enabled: true,
        },
        McpHostEndpoint {
            id: "computer_screen_capture".into(),
            name: "Screen Capture".into(),
            description: "Capture the current computer screen".into(),
            endpoint_type: "tool".into(),
            parameters: vec![],
            enabled: true,
        },
        McpHostEndpoint {
            id: "computer_mouse_click".into(),
            name: "Mouse Click".into(),
            description: "Click at a specified screen coordinate".into(),
            endpoint_type: "tool".into(),
            parameters: vec!["x".into(), "y".into(), "button".into()],
            enabled: true,
        },
        McpHostEndpoint {
            id: "computer_keyboard_type".into(),
            name: "Keyboard Type".into(),
            description: "Type text using the keyboard".into(),
            endpoint_type: "tool".into(),
            parameters: vec!["text".into()],
            enabled: true,
        },
    ]
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn log_activity(state: &mut McpHostState, entry: serde_json::Value) {
    if state.activity_log.len() >= MAX_LOG {
        state.activity_log.pop_front();
    }
    state.activity_log.push_back(entry);
}

static MCP_HOST: LazyLock<Mutex<McpHostState>> = LazyLock::new(|| Mutex::new(McpHostState::new()));

/// 拉起子进程运行 MCP stdio 服务器 (当前可执行文件 + env `NEOTRIX_MCP_STDIO=1`)。
fn spawn_mcp_child() -> Result<(Child, ChildStdin, ChildStdout, u32), String> {
    let exe = std::env::current_exe().map_err(|e| format!("cannot locate current executable: {}", e))?;
    let mut cmd = Command::new(&exe);
    cmd.env("NEOTRIX_MCP_STDIO", "1");
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::inherit());
    let mut child = cmd.spawn().map_err(|e| format!("failed to spawn MCP host child ({}): {}", exe.display(), e))?;
    let pid = child.id();
    let stdin = child.stdin.take().ok_or_else(|| "child stdin unavailable".to_string())?;
    let stdout = child.stdout.take().ok_or_else(|| "child stdout unavailable".to_string())?;
    Ok((child, stdin, stdout, pid))
}

/// 单元测试环境不真实 spawn (当前可执行文件是测试 harness)，模拟运行中状态。
#[cfg(test)]
fn spawn_simulated() -> bool {
    true
}

#[cfg(not(test))]
fn spawn_simulated() -> bool {
    false
}

// ── Commands ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn mcp_host_start(config: McpHostConfig) -> Result<String, String> {
    let mut state = MCP_HOST.lock().map_err(|e| e.to_string())?;
    if state.running {
        return Err("MCP host is already running".into());
    }
    let now = now_secs();
    state.config = config;
    state.sessions.clear();
    state.total_calls = 0;

    if spawn_simulated() {
        state.pid = Some(std::process::id());
        state.running = true;
        state.start_time = now;
    } else {
        let (child, stdin, stdout, pid) = spawn_mcp_child()?;
        state.child = Some(child);
        state.child_stdin = Some(stdin);
        state.child_stdout = Some(stdout);
        state.pid = Some(pid);
        state.running = true;
        state.start_time = now;
    }

    let port = state.config.port;
    log_activity(&mut state, serde_json::json!({
        "event": "start", "port": port, "ts": now
    }));
    Ok(format!("mcp-host-{}", port))
}

#[tauri::command]
pub fn mcp_host_stop() -> Result<(), String> {
    let child = {
        let mut state = MCP_HOST.lock().map_err(|e| e.to_string())?;
        if !state.running {
            return Err("MCP host is not running".into());
        }
        let child = state.child.take();
        state.child = None;
        state.child_stdin = None;
        state.child_stdout = None;
        state.pid = None;
        state.running = false;
        state.sessions.clear();
        log_activity(&mut state, serde_json::json!({
            "event": "stop", "ts": now_secs()
        }));
        child
    };
    // 锁外 wait：child 对 kill 无响应时不应阻塞其他 MCP 命令
    if let Some(mut child) = child {
        let _ = child.kill();
        let _ = child.wait();
    }
    Ok(())
}

#[tauri::command]
pub fn mcp_host_status() -> Result<McpHostStatus, String> {
    let mut state = MCP_HOST.lock().map_err(|e| e.to_string())?;
    if state.running {
        if let Some(child) = state.child.as_mut() {
            // 子进程已退出 → 视为 host 停止并清理
            if let Ok(Some(_)) = child.try_wait() {
                state.running = false;
                state.child = None;
                state.child_stdin = None;
                state.child_stdout = None;
                state.pid = None;
                state.sessions.clear();
            }
        }
    }
    let uptime = if state.running {
        now_secs().saturating_sub(state.start_time)
    } else {
        0
    };
    Ok(McpHostStatus {
        running: state.running,
        port: state.config.port,
        uptime_secs: uptime,
        active_sessions: state.sessions.iter().filter(|s| s.status == "active").count(),
        total_endpoints: state.endpoints.len(),
        total_calls: state.total_calls,
        pid: state.pid,
        registered_tools: mcp_tool_defs().len(),
    })
}

/// 向子进程发送 MCP `ping` 请求并读取响应，验证 stdio 通道与 JSON-RPC 循环存活。
/// 注册由主程序 agent 负责，注册前允许死代码。
#[allow(dead_code)]
#[tauri::command]
pub fn mcp_host_ping() -> Result<serde_json::Value, String> {
    // 只在作用域内取所需句柄，随后立即释放锁——避免阻塞等待期间
    // 所有 MCP 命令排队 (std Mutex 跨 recv_timeout 阻塞)。
    let (stdin, stdout) = {
        let mut state = MCP_HOST.lock().map_err(|e| e.to_string())?;
        if !state.running {
            return Err("MCP host is not running".into());
        }
        if spawn_simulated() {
            return Ok(serde_json::json!({ "pong": true, "simulated": true }));
        }
        let stdin = state.child_stdin.take().ok_or("child stdin unavailable")?;
        let stdout = state.child_stdout.take().ok_or("child stdout unavailable")?;
        (stdin, stdout)
    };

    let request = serde_json::json!({ "jsonrpc": "2.0", "id": 0, "method": "ping", "params": {} });
    let mut stdin = stdin;
    writeln!(stdin, "{}", request).map_err(|e| format!("failed to write ping: {}", e))?;
    stdin.flush().map_err(|e| format!("failed to flush ping: {}", e))?;

    // 超时保护: 读线程持有 stdout，主线程最多等 5 秒
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        let read_ok = reader.read_line(&mut line).is_ok();
        let _ = tx.send((read_ok, line, reader));
    });

    match rx.recv_timeout(Duration::from_secs(5)) {
        Ok((read_ok, line, reader)) => {
            let mut state = MCP_HOST.lock().map_err(|e| e.to_string())?;
            // 并发 stop 可能已把 host 停掉：此时把手柄塞回 running=false 的 state
            // 只会留下过期句柄。直接丢弃并如实报错。
            if !state.running {
                drop(reader);
                drop(stdin);
                return Err("MCP host stopped during ping".into());
            }
            state.child_stdout = Some(reader.into_inner());
            state.child_stdin = Some(stdin);
            if !read_ok {
                return Err(format!("failed to read MCP response: {}", line));
            }
            serde_json::from_str::<serde_json::Value>(line.trim())
                .map_err(|e| format!("invalid MCP response '{}': {}", line.trim(), e))
        }
        Err(_) => {
            // 超时：stdin 已移出 state，直接 drop 会对子进程写 EOF；读线程持有 stdout
            // 无法回收。为防孤儿管道/子进程，此处主动杀掉子进程并复位状态。
            drop(stdin);
            let mut state = MCP_HOST.lock().map_err(|e| e.to_string())?;
            if let Some(mut child) = state.child.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
            state.child_stdin = None;
            state.child_stdout = None;
            state.running = false;
            Err("MCP ping timed out (child unresponsive, killed)".into())
        }
    }
}

#[tauri::command]
pub fn mcp_host_list_endpoints() -> Result<Vec<McpHostEndpoint>, String> {
    let state = MCP_HOST.lock().map_err(|e| e.to_string())?;
    Ok(state.endpoints.clone())
}

#[tauri::command]
pub fn mcp_host_register_endpoint(
    name: String,
    description: String,
    params: Vec<String>,
) -> Result<(), String> {
    let mut state = MCP_HOST.lock().map_err(|e| e.to_string())?;
    if state.endpoints.iter().any(|ep| ep.name == name) {
        return Err(format!("Endpoint '{}' already exists", name));
    }
    let log_name = name.clone();
    let id = name.to_lowercase().replace(' ', "_");
    state.endpoints.push(McpHostEndpoint {
        id,
        name,
        description,
        endpoint_type: "tool".into(),
        parameters: params,
        enabled: true,
    });
    log_activity(&mut state, serde_json::json!({
        "event": "register_endpoint", "name": log_name, "ts": now_secs()
    }));
    Ok(())
}

#[tauri::command]
pub fn mcp_host_unregister_endpoint(name: String) -> Result<(), String> {
    let mut state = MCP_HOST.lock().map_err(|e| e.to_string())?;
    let len_before = state.endpoints.len();
    state.endpoints.retain(|ep| ep.name != name);
    if state.endpoints.len() == len_before {
        return Err(format!("Endpoint '{}' not found", name));
    }
    log_activity(&mut state, serde_json::json!({
        "event": "unregister_endpoint", "name": name, "ts": now_secs()
    }));
    Ok(())
}

#[tauri::command]
pub fn mcp_host_sessions() -> Result<Vec<McpHostSession>, String> {
    let state = MCP_HOST.lock().map_err(|e| e.to_string())?;
    Ok(state.sessions.clone())
}

#[tauri::command]
pub fn mcp_host_log(count: usize) -> Result<Vec<serde_json::Value>, String> {
    let state = MCP_HOST.lock().map_err(|e| e.to_string())?;
    let take = count.min(state.activity_log.len());
    let entries: Vec<serde_json::Value> = state.activity_log.iter().rev().take(take).cloned().collect();
    Ok(entries)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn reset_state() {
        let mut state = MCP_HOST.lock().unwrap();
        state.running = false;
        state.start_time = 0;
        state.pid = None;
        state.child = None;
        state.child_stdin = None;
        state.child_stdout = None;
        state.sessions.clear();
        state.endpoints = builtin_endpoints();
        state.activity_log.clear();
        state.total_calls = 0;
    }

    #[test]
    fn test_mcp_host_start_stop() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let config = McpHostConfig {
            port: 18311,
            ..Default::default()
        };
        let id = mcp_host_start(config).unwrap();
        assert_eq!(id, "mcp-host-18311");

        let status = mcp_host_status().unwrap();
        assert!(status.running);
        assert_eq!(status.port, 18311);
        assert_eq!(status.registered_tools, 6);

        mcp_host_stop().unwrap();
        let status = mcp_host_status().unwrap();
        assert!(!status.running);
    }

    #[test]
    fn test_mcp_host_list_endpoints() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        // start to reset state
        let _ = mcp_host_start(McpHostConfig::default());

        let endpoints = mcp_host_list_endpoints().unwrap();
        assert_eq!(endpoints.len(), 7);
        assert!(endpoints.iter().any(|ep| ep.name == "Execute Tool"));
        assert!(endpoints.iter().any(|ep| ep.name == "Screen Capture"));
        assert!(endpoints.iter().any(|ep| ep.name == "Keyboard Type"));
    }

    #[test]
    fn test_mcp_host_register_endpoint() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let _ = mcp_host_start(McpHostConfig::default());

        mcp_host_register_endpoint(
            "Custom API".into(),
            "Call a custom REST API".into(),
            vec!["url".into(), "method".into()],
        )
        .unwrap();

        let endpoints = mcp_host_list_endpoints().unwrap();
        assert_eq!(endpoints.len(), 8);
        assert!(endpoints.iter().any(|ep| ep.name == "Custom API"));

        // duplicate should fail
        assert!(mcp_host_register_endpoint(
            "Custom API".into(),
            "Duplicate".into(),
            vec![],
        )
        .is_err());
    }

    #[test]
    fn test_mcp_host_sessions() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let _ = mcp_host_start(McpHostConfig::default());

        let sessions = mcp_host_sessions().unwrap();
        assert!(sessions.is_empty());

        // add a session manually through state
        {
            let mut state = MCP_HOST.lock().unwrap();
            state.sessions.push(McpHostSession {
                client_id: "test-client".into(),
                connected_at: now_secs(),
                tool_calls: 3,
                status: "active".into(),
            });
        }

        let sessions = mcp_host_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].client_id, "test-client");
        assert_eq!(sessions[0].tool_calls, 3);
        assert_eq!(sessions[0].status, "active");
    }

    #[test]
    fn test_mcp_host_ping_simulated() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let _ = mcp_host_start(McpHostConfig::default());
        let pong = mcp_host_ping().unwrap();
        assert_eq!(pong["pong"], true);
        mcp_host_stop().unwrap();
        assert!(mcp_host_ping().is_err());
    }

    #[test]
    fn test_jsonrpc_initialize() {
        let resp = handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#).unwrap();
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["id"], 1);
        assert_eq!(v["result"]["protocolVersion"], "2025-03-26");
        assert_eq!(v["result"]["serverInfo"]["name"], "neotrix-mcp");
        assert!(v["error"].is_null());
    }

    #[test]
    fn test_jsonrpc_ping() {
        let resp = handle_line(r#"{"jsonrpc":"2.0","id":5,"method":"ping"}"#).unwrap();
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(v["id"], 5);
        assert_eq!(v["result"], serde_json::json!({}));
    }

    #[test]
    fn test_jsonrpc_tools_list_has_six_tools() {
        let resp = handle_line(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#).unwrap();
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        let tools = v["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 6);
        let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
        for n in ["read_file", "write_file", "execute_terminal_command", "web_search", "tool_search", "kb_search"] {
            assert!(names.contains(&n), "missing tool {}", n);
        }
    }

    #[test]
    fn test_jsonrpc_tools_call_read_file() {
        let tmp = std::env::temp_dir().join(format!("mcp_host_cmds_read_{}.txt", std::process::id()));
        std::fs::write(&tmp, "mcp hello").expect("write temp file");
        let line = format!(
            r#"{{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}}}"#,
            tmp.display()
        );
        let resp = handle_line(&line).unwrap();
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(v["result"]["isError"], false);
        let text = v["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("mcp hello"));
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_jsonrpc_tools_call_unknown_tool() {
        let line = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"nope","arguments":{}}}"#;
        let resp = handle_line(line).unwrap();
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(v["result"]["isError"], true);
        assert!(v["result"]["content"][0]["text"].as_str().unwrap().contains("Unknown tool"));
    }

    #[test]
    fn test_jsonrpc_unknown_method() {
        let resp = handle_line(r#"{"jsonrpc":"2.0","id":9,"method":"bogus/method"}"#).unwrap();
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(v["error"]["code"], -32601);
        assert_eq!(v["id"], 9);
    }

    #[test]
    fn test_jsonrpc_parse_error() {
        let resp = handle_line("this is not json").unwrap();
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(v["error"]["code"], -32700);
        assert!(v["id"].is_null());
    }

    #[test]
    fn test_jsonrpc_notification_ignored() {
        assert!(handle_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).is_none());
    }

    #[test]
    fn test_jsonrpc_bad_jsonrpc_version() {
        let resp = handle_line(r#"{"jsonrpc":"1.0","id":7,"method":"ping"}"#).unwrap();
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(v["error"]["code"], -32600);
    }

    #[test]
    fn test_call_tool_write_file() {
        let tmp = std::env::temp_dir().join(format!("mcp_host_cmds_write_{}.txt", std::process::id()));
        let args = serde_json::json!({ "path": tmp.to_string_lossy(), "content": "write test" });
        let out = call_tool("write_file", &args).unwrap();
        assert!(out.contains("Wrote 10 bytes"));
        assert_eq!(std::fs::read_to_string(&tmp).unwrap(), "write test");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_call_tool_missing_args() {
        let err = call_tool("read_file", &serde_json::json!({})).unwrap_err();
        assert!(err.contains("path"));
        let err = call_tool("web_search", &serde_json::json!({})).unwrap_err();
        assert!(err.contains("query"));
    }

    #[test]
    fn test_call_tool_unknown() {
        let err = call_tool("no_such_tool", &serde_json::json!({})).unwrap_err();
        assert!(err.contains("Unknown tool"));
    }
}
