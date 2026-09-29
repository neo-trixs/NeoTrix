//! NT-MCP stdio 会话客户端 —— 与**长驻式** MCP 服务器做完整 JSON-RPC 会话。
//!
//! 吸收源: QwenLM/Qwen-MM-Plugins (Apache-2.0) `src/capabilities/*/.mcp.json`
//! 的 stdio 启动约定（`command` + `args`，以 `uvx --from ...@tag` 或
//! `python3 <pkgdir>` 拉起长驻进程）+ MCP 基础协议的 `initialize` 握手。
//!
//! 对治缺口: `crate::agent::tool::mcp::StdioNativeTool`
//! （`neotrix-core/src/agent.rs:734-736`）只会裸发 `tools/call`、
//! 不做 `initialize` 握手，长驻式服务器 30s 超时 —— **调不通任何真 MCP
//! 服务器**（2026-09-28 实测断言，见吸收文档）。
//!
//! 设计（刻意保持无状态，与现有 30s 超时循环风格一致）：
//! - per-call spawn 全握手：`initialize`(id=1) → `notifications/initialized`
//!   → `tools/list`|`tools/call`(id=2) → **stdin 保持打开等回包**
//!   → 匹配到 id=2 后关 stdin → 读至 EOF。不维持常驻进程/线程；
//!   代价是每次调用一次握手（本地约数十 ms）。
//! - ⚠️ stdin 必须在回包前保持打开：真 MCP 服务器在 stdin EOF 时拆除
//!   会话（in-flight 请求被取消），先关 stdin 再等回包会永远等不到响应
//!   —— 2026-09-28 对真服务器实测证实（`CallToolRequest` 已处理但无回包）。
//!   关 stdin 只作为"会话结束"信号（长驻服务器以此退出）。
//! - stdout/stderr 双泵线程：stdout 逐行进 channel 供 id 匹配；stderr
//!   截尾缓存（防 64KB pipe 填满导致的服务端阻塞）。
//! - `tools/call` 返回的 `image` 块**不塞进返回字符串**（base64 会炸上下文）：
//!   解码落盘到 `artifacts_dir`，文本里只留文件路径。调用方可把路径喂给
//!   已有的 vision 通路（neobot `image_url`，见 `nt_types.rs:256`）。
//!   这是诚实设计：图像"确实到了磁盘"，而不是"假装进了模型"。
//!
//! 零新增依赖：只用 `std` + `serde_json` + `base64`（后两者已在
//! `neotrix-core/Cargo.toml`）。

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};

/// 握手声明的协议版本 —— `mcp` 1.x 全系接受的地板版本。
///
/// 服务端按规范回自己的版本；本客户端不校验回值（只要求它是合法响应），
/// 避免把"版本前向兼容"做成"版本门禁"。 bump 点：出现只认新版本的服务器时。
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// 握手 `clientInfo.name`。
pub const MCP_CLIENT_NAME: &str = "neotrix";

/// 超时轮询粒度（与 `agent.rs` 的 `StdioNativeTool` 同风格：100ms）。
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// 默认调用超时（与既有 30s 一致）。
pub const DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// stderr 诊断尾（错误信息里最多带这么多服务端 stderr）。
const STDERR_TAIL_BYTES: usize = 2048;

/// stdout 诊断头（协议错误时最多带这么多服务端 stdout）。
const STDOUT_HEAD_CHARS: usize = 500;

// ---------------------------------------------------------------------------
// 错误
// ---------------------------------------------------------------------------

/// 会话错误 —— 五类，不折叠（`attempt` 与 `outcome` 独立，见 5.3）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpSessionError {
    /// 进程起不来（命令不存在/无权限）。
    Spawn { command: String, detail: String },
    /// 限时内没拿到匹配响应（进程已 kill；`detail` 带服务端 stderr 尾）。
    Timeout {
        tool: String,
        timeout_ms: u64,
        detail: String,
    },
    /// 进程退出了但没给出合法会话（非 0 退出 / 无匹配 id / 非 JSON）。
    Protocol { detail: String },
    /// 服务端返回了 JSON-RPC `error` 对象。
    Server { code: i64, message: String },
    /// 本地 IO（写 stdin / 读 pipe / 落盘 image 块目录不可写等）。
    Io { detail: String },
}

impl std::fmt::Display for McpSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpSessionError::Spawn { command, detail } => {
                write!(f, "mcp spawn '{command}' failed: {detail}")
            }
            McpSessionError::Timeout {
                tool,
                timeout_ms,
                detail,
            } => {
                if detail.is_empty() {
                    write!(f, "mcp tool '{tool}' timed out after {timeout_ms}ms")
                } else {
                    write!(
                        f,
                        "mcp tool '{tool}' timed out after {timeout_ms}ms: {detail}"
                    )
                }
            }
            McpSessionError::Protocol { detail } => {
                write!(f, "mcp protocol error: {detail}")
            }
            McpSessionError::Server { code, message } => {
                write!(f, "mcp server error {code}: {message}")
            }
            McpSessionError::Io { detail } => write!(f, "mcp io error: {detail}"),
        }
    }
}

impl std::error::Error for McpSessionError {}

// ---------------------------------------------------------------------------
// 类型
// ---------------------------------------------------------------------------

/// `tools/list` 带回的单工具清单。
#[derive(Debug, Clone, PartialEq)]
pub struct McpListedTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// `tools/call` 的结构化结果。
#[derive(Debug, Clone, PartialEq)]
pub struct McpCallResult {
    /// 全部 `text` 块拼接（`image` 块转为落盘路径行，不内联 base64）。
    pub text: String,
    /// 本次调用落盘的图像文件（`image` 块解码所得）。
    pub saved_images: Vec<PathBuf>,
    /// 服务端 `isError` 标志。
    pub is_error: bool,
}

// ---------------------------------------------------------------------------
// 会话
// ---------------------------------------------------------------------------

/// MCP stdio 会话（无状态：每次调用独立 spawn + 全握手）。
#[derive(Debug, Clone)]
pub struct McpStdioSession {
    command: String,
    args: Vec<String>,
    timeout_ms: u64,
    artifacts_dir: PathBuf,
}

impl McpStdioSession {
    /// 新建会话。`artifacts_dir` 默认走系统临时目录下 `neotrix-mcp-artifacts`。
    pub fn new(command: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            command: command.into(),
            args,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            artifacts_dir: std::env::temp_dir().join("neotrix-mcp-artifacts"),
        }
    }

    /// 覆盖调用超时（毫秒）。
    pub fn with_timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// 覆盖 `image` 块落盘目录。
    pub fn with_artifacts_dir(mut self, dir: PathBuf) -> Self {
        self.artifacts_dir = dir;
        self
    }

    /// `tools/list` —— 取服务器的真实工具清单（调用方不用再手写 manifest）。
    pub fn list_tools(&self) -> Result<Vec<McpListedTool>, McpSessionError> {
        let params = serde_json::json!({});
        let result = self.run_request("tools/list", params, "tools/list")?;
        let tools = result
            .get("tools")
            .and_then(|v| v.as_array())
            .ok_or_else(|| McpSessionError::Protocol {
                detail: "tools/list result missing 'tools' array".to_string(),
            })?;
        let mut out = Vec::with_capacity(tools.len());
        for t in tools {
            let name = t.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
                McpSessionError::Protocol {
                    detail: "tools/list entry missing 'name'".to_string(),
                }
            })?;
            out.push(McpListedTool {
                name: name.to_string(),
                description: t
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                input_schema: t
                    .get("inputSchema")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({"type": "object"})),
            });
        }
        Ok(out)
    }

    /// `tools/call` —— 调一个工具，`image` 块落盘后只返回路径。
    pub fn call_tool(
        &self,
        name: &str,
        arguments: &serde_json::Value,
    ) -> Result<McpCallResult, McpSessionError> {
        let params = serde_json::json!({"name": name, "arguments": arguments});
        let result = self.run_request("tools/call", params, name)?;
        self.decode_call_result(&result)
    }

    // -- 会话主干 ---------------------------------------------------------

    /// 一次完整会话：spawn → initialize → initialized 通知 → 目标请求 →
    /// stdin 保持打开等 id=2 回包 → 关 stdin → 收尸 → 返回 `result` 对象。
    fn run_request(
        &self,
        method: &str,
        params: serde_json::Value,
        label: &str,
    ) -> Result<serde_json::Value, McpSessionError> {
        use std::sync::mpsc;

        let mut child = Command::new(&self.command)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| McpSessionError::Spawn {
                command: self.command.clone(),
                detail: e.to_string(),
            })?;

        // 按序写入三帧。stdin 句柄**必须留到回包之后**（见模块头 ⚠️）。
        let frames: [(&str, serde_json::Value); 3] = [
            (
                "initialize",
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": MCP_PROTOCOL_VERSION,
                        "capabilities": {},
                        "clientInfo": {"name": MCP_CLIENT_NAME, "version": env!("CARGO_PKG_VERSION")},
                    },
                }),
            ),
            (
                "notifications/initialized",
                serde_json::json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
            ),
            (
                method,
                serde_json::json!({"jsonrpc": "2.0", "id": 2, "method": method, "params": params}),
            ),
        ];
        let mut stdin = child.stdin.take().ok_or_else(|| McpSessionError::Io {
            detail: format!("take stdin of '{}'", self.command),
        })?;
        for (frame_name, frame) in &frames {
            writeln!(stdin, "{frame}").map_err(|e| McpSessionError::Protocol {
                detail: format!("stdin write failed at frame '{frame_name}': {e}"),
            })?;
        }
        stdin.flush().map_err(|e| McpSessionError::Io {
            detail: format!("flush stdin of '{}': {e}", self.command),
        })?;

        // stdout 泵线程：逐行进 channel。stderr 泵线程：截尾缓存防 pipe 填满。
        let (out_tx, out_rx) = mpsc::channel::<String>();
        if let Some(stdout) = child.stdout.take() {
            std::thread::spawn(move || {
                use std::io::BufRead;
                let reader = std::io::BufReader::new(stdout);
                for line in reader.lines() {
                    match line {
                        Ok(l) => {
                            if out_tx.send(l).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            });
        }
        let (err_tx, err_rx) = mpsc::channel::<String>();
        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                use std::io::BufRead;
                let reader = std::io::BufReader::new(stderr);
                // 只保尾：超限后丢弃最旧行（VecDeque 开销不值，行数少时直接 Vec）。
                let mut kept: Vec<String> = Vec::new();
                for line in reader.lines() {
                    match line {
                        Ok(l) => {
                            kept.push(l);
                            if kept.len() > 50 {
                                kept.remove(0);
                            }
                        }
                        Err(_) => break,
                    }
                }
                let _ = err_tx.send(kept.join("\n"));
            });
        }

        // 等 id=2 回包（跳过日志噪音行），直到超时。
        let deadline = Instant::now() + Duration::from_millis(self.timeout_ms);
        let mut stdout_head = String::new();
        let response: Option<serde_json::Value> = loop {
            let remain = deadline.saturating_duration_since(Instant::now());
            if remain.is_zero() {
                let stderr_tail = drain_stderr(&err_rx);
                kill_and_reap(&mut child);
                let mut detail = format!("tool '{label}' timed out after {}ms", self.timeout_ms);
                if !stderr_tail.is_empty() {
                    detail.push_str(&format!("; server stderr: {stderr_tail}"));
                }
                return Err(McpSessionError::Timeout {
                    tool: label.to_string(),
                    timeout_ms: self.timeout_ms,
                    detail,
                });
            }
            match out_rx.recv_timeout(remain) {
                Ok(line) => {
                    let line = line.trim().to_string();
                    if line.is_empty() {
                        continue;
                    }
                    if stdout_head.len() < STDOUT_HEAD_CHARS {
                        stdout_head.push_str(&line);
                        stdout_head.push('\n');
                    }
                    match serde_json::from_str::<serde_json::Value>(&line) {
                        Ok(v) => {
                            if v.get("id").and_then(|id| id.as_u64()) == Some(2) {
                                break Some(v);
                            }
                            // 非 id=2 的帧（通知/日志）跳过，不判死刑。
                        }
                        Err(_) => continue, // 非 JSON 行（日志噪音）跳过。
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue, // 回环顶部判 deadline。
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    // stdout EOF 但没匹配到：服务器提前退出。
                    break None;
                }
            }
        };

        // 会话结束：关 stdin（服务器以此退出）→ 收尸（5s 宽限）→ 读 stderr。
        drop(stdin);
        let exited_ok = wait_for_exit(&mut child, Duration::from_secs(5));
        let stderr_tail = drain_stderr(&err_rx);

        match response {
            Some(v) => {
                if let Some(err) = v.get("error") {
                    return Err(McpSessionError::Server {
                        code: err.get("code").and_then(|c| c.as_i64()).unwrap_or(-1),
                        message: err
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("unknown server error")
                            .to_string(),
                    });
                }
                if let Some(result) = v.get("result") {
                    return Ok(result.clone());
                }
                Err(McpSessionError::Protocol {
                    detail: format!("response id=2 has neither 'result' nor 'error' ({label})"),
                })
            }
            None => {
                // 没匹配到：进程行为异常，附诊断（退出码 + stderr 尾 + stdout 头）。
                let mut detail =
                    format!("no JSON-RPC response with id=2 for '{label}' (exited_ok={exited_ok})");
                if !stderr_tail.is_empty() {
                    detail.push_str(&format!("; server stderr: {stderr_tail}"));
                }
                let head: String = stdout_head.chars().take(STDOUT_HEAD_CHARS).collect();
                if !head.trim().is_empty() {
                    detail.push_str(&format!("; server stdout head: {head}"));
                }
                Err(McpSessionError::Protocol { detail })
            }
        }
    }

    /// `tools/call` 的 `result` → `McpCallResult`（`image` 块落盘）。
    fn decode_call_result(
        &self,
        result: &serde_json::Value,
    ) -> Result<McpCallResult, McpSessionError> {
        let is_error = result
            .get("isError")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let content = result.get("content").and_then(|v| v.as_array());
        let mut text_parts: Vec<String> = Vec::new();
        let mut saved_images: Vec<PathBuf> = Vec::new();
        if let Some(items) = content {
            for item in items {
                let kind = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
                match kind {
                    "text" => {
                        if let Some(t) = item.get("text").and_then(|v| v.as_str()) {
                            text_parts.push(t.to_string());
                        }
                    }
                    "image" => {
                        let b64 = item.get("data").and_then(|v| v.as_str()).unwrap_or("");
                        let mime = item
                            .get("mimeType")
                            .and_then(|v| v.as_str())
                            .unwrap_or("image/png");
                        match self.save_image_block(b64, mime, saved_images.len()) {
                            Ok(path) => {
                                saved_images.push(path.clone());
                                text_parts
                                    .push(format!("[image block saved to {}]", path.display()));
                            }
                            Err(e) => {
                                // 降级不断链：文本里如实记录失败，调用方看得见。
                                text_parts.push(format!("[image block NOT saved: {e}]"));
                            }
                        }
                    }
                    other => {
                        text_parts.push(format!("[unsupported content block: {other}]"));
                    }
                }
            }
        }
        Ok(McpCallResult {
            text: text_parts.join("\n"),
            saved_images,
            is_error,
        })
    }

    /// base64 `image` 块 → `artifacts_dir/mcp_img_<millis>_<pid>_<n>.<ext>`。
    fn save_image_block(
        &self,
        b64: &str,
        mime: &str,
        index: usize,
    ) -> Result<PathBuf, McpSessionError> {
        std::fs::create_dir_all(&self.artifacts_dir).map_err(|e| McpSessionError::Io {
            detail: format!(
                "create artifacts dir '{}': {e}",
                self.artifacts_dir.display()
            ),
        })?;
        let ext = match mime {
            "image/jpeg" => "jpg",
            "image/webp" => "webp",
            "image/gif" => "gif",
            _ => "png",
        };
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let path = self.artifacts_dir.join(format!(
            "mcp_img_{millis}_{}_{index}.{ext}",
            std::process::id()
        ));
        let bytes = B64.decode(b64).map_err(|e| McpSessionError::Protocol {
            detail: format!("image block base64 decode failed: {e}"),
        })?;
        std::fs::write(&path, &bytes).map_err(|e| McpSessionError::Io {
            detail: format!("write image '{}': {e}", path.display()),
        })?;
        Ok(path)
    }
}

/// 取 stderr 泵线程的截尾缓存（线程已结束则拿全量，否则拿已到部分；永不阻塞）。
fn drain_stderr(rx: &std::sync::mpsc::Receiver<String>) -> String {
    // 泵线程结束时恰好发一条；先耗尽再取最后一条（只要尾）。
    let mut last = String::new();
    while let Ok(s) = rx.try_recv() {
        last = s;
    }
    tail_bytes(&last, STDERR_TAIL_BYTES)
}

/// 超时 kill 并收尸（收不了也认，调用方已判 Timeout）。
fn kill_and_reap(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// 等进程退出（轮询风格与既有 `StdioNativeTool` 一致）；宽限耗尽则 kill。
/// 返回进程是否自行退出（true）还是被 kill（false）。
fn wait_for_exit(child: &mut std::process::Child, grace: Duration) -> bool {
    let deadline = Instant::now() + grace;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return true,
            Ok(None) => {
                if Instant::now() >= deadline {
                    kill_and_reap(child);
                    return false;
                }
                std::thread::sleep(POLL_INTERVAL);
            }
            Err(_) => return false,
        }
    }
}

/// 取字符串尾部字节（按字符边界切，不 panic）。
fn tail_bytes(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let cut = s.len() - max;
    let mut start = cut;
    while start < s.len() && !s.is_char_boundary(start) {
        start += 1;
    }
    s[start..].to_string()
}

// ---------------------------------------------------------------------------
// NativeTool 适配 —— 供 `agent::tool::mcp::as_native_tools` 的会话分支使用。
// ---------------------------------------------------------------------------

/// 会话式 MCP 工具的 `NativeTool` 适配（与 `StdioNativeTool` 对应）。
///
/// `agent.rs` 的 `McpServer.use_session == true` 时由此执行；
/// 旧 `StdioNativeTool` 保持不动（短命令式工具仍走它）。
#[derive(Debug, Clone)]
pub struct McpSessionTool {
    /// 工具名（`tools/call` 的 `name`）。
    pub def_name: String,
    /// 工具描述（透传清单）。
    pub def_description: String,
    /// 工具输入 schema（透传清单）。
    pub def_schema: serde_json::Value,
    /// 服务器启动命令。
    pub command: String,
    /// 服务器启动参数。
    pub args: Vec<String>,
    /// 单次调用超时（毫秒）。
    pub timeout_ms: u64,
    /// `image` 块落盘目录（`None` = 默认临时目录）。
    pub artifacts_dir: Option<PathBuf>,
}

impl crate::l0_substrate::nt_core_traits::NativeTool for McpSessionTool {
    fn id(&self) -> &str {
        &self.def_name
    }

    fn description(&self) -> &str {
        &self.def_description
    }

    fn input_schema(&self) -> serde_json::Value {
        self.def_schema.clone()
    }

    fn capability_tags(&self) -> Vec<&'static str> {
        Vec::new()
    }

    fn execute(
        &self,
        args: &serde_json::Value,
    ) -> Result<crate::l0_substrate::nt_core_traits::ToolOutput, String> {
        let mut session = McpStdioSession::new(self.command.clone(), self.args.clone())
            .with_timeout_ms(self.timeout_ms);
        if let Some(dir) = &self.artifacts_dir {
            session = session.with_artifacts_dir(dir.clone());
        }
        match session.call_tool(&self.def_name, args) {
            Ok(r) => {
                let mut content = r.text;
                if !r.saved_images.is_empty() {
                    content.push_str("\n[saved images]");
                    for p in &r.saved_images {
                        content.push_str(&format!("\n- {}", p.display()));
                    }
                }
                Ok(crate::l0_substrate::nt_core_traits::ToolOutput {
                    success: !r.is_error,
                    content,
                })
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// 测试 —— 以 bash 伪造 MCP 服务器做真进程级 framing 验证（零 Python 依赖）。
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const FAKE_SERVER: &str = r#"#!/usr/bin/env bash
# 伪造 MCP stdio 服务器：initialize → list/call → stdin EOF 即退出。
IMG1x1="iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg=="
while IFS= read -r line; do
  case "$line" in
    *'"method":"initialize"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{},"serverInfo":{"name":"fake","version":"0"}}}'
      ;;
    *'"method":"tools/list"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":2,"result":{"tools":[{"name":"echo_text","description":"echo back","inputSchema":{"type":"object"}},{"name":"make_image","description":"one image","inputSchema":{"type":"object"}}]}}'
      ;;
    *'"method":"tools/call"'*)
      if [[ "$line" == *'"make_image"'* ]]; then
        printf '%s\n' '{"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","text":"one"},{"type":"image","data":"'"$IMG1x1"'","mimeType":"image/png"}],"isError":false}}'
      elif [[ "$line" == *'"echo_text"'* ]]; then
        printf '%s\n' '{"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","text":"hi"}],"isError":false}}'
      else
        printf '%s\n' '{"jsonrpc":"2.0","id":2,"error":{"code":-32602,"message":"unknown tool"}}'
      fi
      ;;
  esac
done
"#;

    const HANG_SERVER: &str = "#!/usr/bin/env bash\ncat >/dev/null\nsleep 30\n";
    const DEAD_SERVER: &str = "#!/usr/bin/env bash\nexit 3\n";

    /// 写伪造服务器到唯一临时路径，返回脚本路径与 guard 目录。
    fn write_fake(name: &str, body: &str) -> (PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "nt_mcp_sess_test_{}_{}_{name}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).expect("test temp dir");
        let script = dir.join("fake.sh");
        std::fs::write(&script, body).expect("write fake server");
        (script, dir)
    }

    fn session_to(script: &Path) -> McpStdioSession {
        McpStdioSession::new(
            bash_cmd(),
            vec![script.to_string_lossy().to_string()],
        )
    }

    /// 测试 shell：绝对路径优先（`/bin/bash` 在 macOS/Linux 恒存在），
    /// 回退 PATH 查找。不用相对 `"bash"`：同进程并行测试若隔离式改 `PATH`
    /// 会让相对查找竞态失败（教训见 `nt_qwen_mm_manifests::ResolveEnv`）。
    fn bash_cmd() -> String {
        if Path::new("/bin/bash").is_file() {
            "/bin/bash".to_string()
        } else {
            "bash".to_string()
        }
    }

    #[test]
    fn test_session_list_tools() {
        let (script, dir) = write_fake("list", FAKE_SERVER);
        let tools = session_to(&script).list_tools().expect("list");
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].name, "echo_text");
        assert_eq!(tools[1].name, "make_image");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_session_call_text() {
        let (script, dir) = write_fake("call", FAKE_SERVER);
        let r = session_to(&script)
            .call_tool("echo_text", &serde_json::json!({}))
            .expect("call");
        assert!(!r.is_error);
        assert!(r.text.contains("hi"));
        assert!(r.saved_images.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_session_call_image_saved_to_disk() {
        let (script, dir) = write_fake("img", FAKE_SERVER);
        let arts = dir.join("arts");
        let r = session_to(&script)
            .with_artifacts_dir(arts.clone())
            .call_tool("make_image", &serde_json::json!({}))
            .expect("call image");
        assert!(!r.is_error);
        assert_eq!(r.saved_images.len(), 1);
        let p = &r.saved_images[0];
        assert!(p.starts_with(&arts));
        assert_eq!(p.extension().and_then(|e| e.to_str()), Some("png"));
        let bytes = std::fs::read(p).expect("saved image readable");
        assert!(!bytes.is_empty());
        assert!(r.text.contains(&p.display().to_string()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_session_server_error_maps_to_variant() {
        let (script, dir) = write_fake("err", FAKE_SERVER);
        let e = session_to(&script)
            .call_tool("nope", &serde_json::json!({}))
            .expect_err("unknown tool must error");
        assert!(matches!(e, McpSessionError::Server { code: -32602, .. }));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_session_spawn_failure() {
        let s = McpStdioSession::new("/nonexistent/nt_cmd_xyz".to_string(), vec![]);
        let e = s.list_tools().expect_err("bad command must error");
        assert!(matches!(e, McpSessionError::Spawn { .. }));
    }

    #[test]
    fn test_session_timeout_kills_hung_server() {
        let (script, dir) = write_fake("hang", HANG_SERVER);
        let e = session_to(&script)
            .with_timeout_ms(400)
            .call_tool("echo_text", &serde_json::json!({}))
            .expect_err("hung server must time out");
        assert!(matches!(e, McpSessionError::Timeout { .. }));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_session_dead_server_is_protocol_error() {
        let (script, dir) = write_fake("dead", DEAD_SERVER);
        let e = session_to(&script)
            .list_tools()
            .expect_err("dead server must error");
        assert!(matches!(e, McpSessionError::Protocol { .. }));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_session_tool_native_adapter() {
        use crate::l0_substrate::nt_core_traits::NativeTool;
        let (script, dir) = write_fake("native", FAKE_SERVER);
        let tool = McpSessionTool {
            def_name: "echo_text".to_string(),
            def_description: "echo back".to_string(),
            def_schema: serde_json::json!({"type": "object"}),
            command: bash_cmd(),
            args: vec![script.to_string_lossy().to_string()],
            timeout_ms: 10_000,
            artifacts_dir: None,
        };
        assert_eq!(tool.id(), "echo_text");
        let out = tool
            .execute(&serde_json::json!({}))
            .expect("native execute");
        assert!(out.success);
        assert!(out.content.contains("hi"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // -- 纯函数单测（无进程） -------------------------------------------------

    #[test]
    fn test_tail_bytes_respects_char_boundary() {
        assert_eq!(tail_bytes("abcdef", 3), "def");
        assert_eq!(tail_bytes("短", 10), "短");
        // 多字节字符边界：不 panic，不断裂。
        let s = "a中b文c";
        let t = tail_bytes(s, 4);
        assert!(s.ends_with(&t));
    }

    #[test]
    fn test_error_display_is_stable() {
        let e = McpSessionError::Timeout {
            tool: "x".to_string(),
            timeout_ms: 1,
            detail: String::new(),
        };
        assert!(e.to_string().contains('x'));
    }
}
