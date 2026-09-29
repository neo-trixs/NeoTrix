//! NT-MCP stdio 会话客户端 —— 与**长驻式** MCP 服务器做完整 JSON-RPC 会话。
//!
//! 位置说明（2026-09-29 从 `neotrix-core/src/nt_mcp_stdio_session.rs` 下移）：
//! **唯一真在执行的工具环是本 crate 的 `nt_agent::execute_tool`**
//! （2026-09-29 实测：`neotrix-core` 的 `ToolOrchestrator.call` 仅测试在调、
//! `AgentLoop.with_tools` 仅测试在用、crystal 活环零消费 `NativeTool`）。
//! 客户端住在执行环所在 crate，模型才能自主调它；core 侧经
//! `neotrix::nt_mcp_stdio_session` re-export 保持旧路径可用。
//!
//! 吸收源: QwenLM/Qwen-MM-Plugins (Apache-2.0) `src/capabilities/*/.mcp.json`
//! 的 stdio 启动约定（`command` + `args`，以 `uvx --from ...@tag` 或
//! `python3 <pkgdir>` 拉起长驻进程）+ MCP 基础协议的 `initialize` 握手。
//!
//! 对治缺口: `neotrix-core/src/agent.rs` 的 `StdioNativeTool`（`:795-810`）
//! 只会裸发 `tools/call`、不做 `initialize` 握手，长驻式服务器 30s 超时 ——
//! **调不通任何真 MCP 服务器**（2026-09-28 实测断言，见吸收文档）。
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
//!   解码落盘到 `artifacts_dir`，文本里只留文件路径。调用方（本 crate 的
//!   `execute_read_image` 兄弟路径）把文件读成 `ImagePart`，随下一轮请求
//!   以 `image_url` 部件抵达模型（`nt_types.rs` `TranscriptItem::image`）。
//!   这是诚实设计：图像"确实到了磁盘→进了请求"，而不是"假装进了模型"。
//!
//! 零新增依赖：只用 `std` + `serde_json` + `base64`（后两者在本 crate 的
//! `Cargo.toml`；`base64` 与 neotrix-core 同为 0.21，`Cargo.lock` 已有）。

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
// Qwen-MM-Plugins 启动解析（无 key 路径）—— 2026-09-29 自 neotrix-core 搬入。
//
// 命名/版本钉死上游 `plugin-versions.json`（core/search 均 1.1.0）与 `.mcp.json`。
// 探测**零 spawn**：只查 PATH 与目录存在性 + 环境变量，离线可用；探测不到时
// 上层不挂载工具（fail-closed：不支持 ≠ 已列出）。
// ---------------------------------------------------------------------------

/// core 能力：插件名 / 版本。
pub const QWEN_MM_CORE_PLUGIN: &str = "qwen-mm-plugins-core";
pub const QWEN_MM_CORE_VERSION: &str = "1.1.0";

/// search 能力：插件名 / 版本。
pub const QWEN_MM_SEARCH_PLUGIN: &str = "qwen-mm-plugins-search";
pub const QWEN_MM_SEARCH_VERSION: &str = "1.1.0";

/// 上游 tag 格式：`qwen-mm-plugins-{cap}-v{version}`。
pub fn release_tag(capability: &str, version: &str) -> String {
    format!("qwen-mm-plugins-{capability}-v{version}")
}

/// core 工具默认超时：帧渲染类有界但慢，90s（kill 开关仍在）。
pub const CORE_TIMEOUT_MS: u64 = 90_000;

/// search 工具超时：网络调用服务端自带超时，这里只做上限。
pub const SEARCH_TIMEOUT_MS: u64 = DEFAULT_TIMEOUT_MS;

/// 源码 checkout 定位环境变量（sparse-checkout 只需 core 源＋shared＋framework）。
pub const QWEN_MM_CHECKOUT_ENV: &str = "QWEN_MM_PLUGINS_CHECKOUT";

/// 启动方式（探测结论，按优先级排序）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchVia {
    /// `uvx --from "qwen-mm-plugins[core] @ git+...@tag" <entry>`（官方方式）。
    Uvx,
    /// PATH 上的已安装入口（`qwen-mm-plugins-core`）。
    Path,
    /// 源码 checkout：`python3 <pkgdir>`（`__main__.py` 自带 sys.path 注入）。
    SourceCheckout,
}

/// 探测到的启动方式。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QwenLaunch {
    pub server_name: String,
    pub command: String,
    pub args: Vec<String>,
    pub via: LaunchVia,
}

impl QwenLaunch {
    /// 按 `timeout_ms` 造一个就绪会话。
    pub fn session(&self, timeout_ms: u64, artifacts_dir: std::path::PathBuf) -> McpStdioSession {
        McpStdioSession::new(self.command.clone(), self.args.clone())
            .with_timeout_ms(timeout_ms)
            .with_artifacts_dir(artifacts_dir)
    }
}

/// 探测失败：带可操作的安装指引（不挂载、不断链）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QwenResolveError {
    pub server: String,
    pub reason: String,
    /// 用户照做即生产（精确命令）。
    pub hint: String,
}

impl std::fmt::Display for QwenResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} unavailable: {}. {}",
            self.server, self.reason, self.hint
        )
    }
}

impl std::error::Error for QwenResolveError {}

/// 可注入的环境 —— 生产走真实环境，测试走隔离值，**零全局变更**。
///
/// 背景：直接改 `PATH` 跑测试会和同进程并行测试（12k）竞态。本结构让探测逻辑
/// 可测，又不碰进程全局状态。
#[derive(Debug, Clone, Default)]
pub struct ResolveEnv {
    /// `None` = 读真实 `PATH`；`Some(dirs)` = 只在这些目录里找。
    pub path_dirs: Option<Vec<PathBuf>>,
    /// `None` = 读真实 `QWEN_MM_PLUGINS_CHECKOUT`；`Some(x)` = 覆盖
    ///（`Some(None)` = 视为未设）。
    pub checkout: Option<Option<PathBuf>>,
}

impl ResolveEnv {
    /// 生产环境（读真实进程环境）。
    pub fn live() -> Self {
        Self::default()
    }
}

/// PATH 查找（只判存在性，不 spawn）。
fn path_lookup(name: &str) -> Option<PathBuf> {
    path_lookup_in(name, None)
}

fn path_lookup_in(name: &str, dirs: Option<&[PathBuf]>) -> Option<PathBuf> {
    let owned: Vec<PathBuf>;
    let search_dirs: &[PathBuf] = match dirs {
        Some(d) => d,
        None => {
            let path_var = std::env::var_os("PATH")?;
            owned = std::env::split_paths(&path_var).collect();
            &owned
        }
    };
    for dir in search_dirs {
        if dir.as_os_str().is_empty() {
            continue;
        }
        let cand = dir.join(name);
        if cand.is_file() {
            return Some(cand);
        }
    }
    None
}

/// core 能力需要的外部二进制（上游 `SYSTEM_DEPS` 子集——只含"装即有、无需 pip"的项）。
pub fn core_system_dep_status() -> Vec<(String, bool)> {
    let probe = |label: &str, bins: &[&str]| {
        (
            label.to_string(),
            bins.iter().any(|b| path_lookup(b).is_some()),
        )
    };
    vec![
        probe(
            "read_video / media_info (video & audio)",
            &["ffmpeg", "ffprobe"],
        ),
        probe(
            "visualize: Office / DrawIO (LibreOffice)",
            &["libreoffice", "soffice"],
        ),
        probe("visualize: LaTeX (.tex)", &["pdflatex"]),
        probe("visualize: 3D best-quality render (Blender)", &["blender"]),
    ]
}

/// search 后端 key 存在性（只返回"有/无"，值永不外露）。
pub fn search_credential_status() -> Vec<(String, bool)> {
    [
        "SERPER_API_KEY",
        "TAVILY_API_KEY",
        "EXA_API_KEY",
        "SERPLY_API_KEY",
    ]
    .iter()
    .map(|k| {
        let present = std::env::var_os(k).is_some_and(|v| !v.is_empty());
        (k.to_string(), present)
    })
    .collect()
}

/// 探测 core 服务器启动方式（读真实环境）。
pub fn resolve_core_launch() -> Result<QwenLaunch, QwenResolveError> {
    resolve_launch_with(
        QWEN_MM_CORE_PLUGIN,
        "core",
        QWEN_MM_CORE_VERSION,
        &ResolveEnv::live(),
    )
}

/// 探测 search 服务器启动方式（读真实环境）。
pub fn resolve_search_launch() -> Result<QwenLaunch, QwenResolveError> {
    resolve_launch_with(
        QWEN_MM_SEARCH_PLUGIN,
        "search",
        QWEN_MM_SEARCH_VERSION,
        &ResolveEnv::live(),
    )
}

/// 通用探测（环境可注入，供测试隔离）。
fn resolve_launch_with(
    plugin: &str,
    extra: &str,
    version: &str,
    env: &ResolveEnv,
) -> Result<QwenLaunch, QwenResolveError> {
    let lookup = |name: &str| path_lookup_in(name, env.path_dirs.as_deref());
    let capability = plugin.strip_prefix("qwen-mm-plugins-").unwrap_or(plugin);
    let checkout_dir: Option<PathBuf> = match &env.checkout {
        None => std::env::var_os(QWEN_MM_CHECKOUT_ENV).map(PathBuf::from),
        Some(o) => o.clone(),
    };
    let install_hint = format!(
        "install: uvx --from \"qwen-mm-plugins[{extra}] @ git+https://github.com/QwenLM/Qwen-MM-Plugins.git@{} \" {plugin} \
         | or sparse-checkout the repo and set {QWEN_MM_CHECKOUT_ENV}=<dir>",
        release_tag(capability, version),
    );

    // 1) 官方方式：uvx。
    if lookup("uvx").is_some() {
        return Ok(QwenLaunch {
            server_name: plugin.to_string(),
            command: "uvx".to_string(),
            args: vec![
                "--from".to_string(),
                format!(
                    "qwen-mm-plugins[{extra}] @ git+https://github.com/QwenLM/Qwen-MM-Plugins.git@{}",
                    release_tag(capability, version)
                ),
                plugin.to_string(),
            ],
            via: LaunchVia::Uvx,
        });
    }
    // 2) PATH 上已有入口。
    if lookup(plugin).is_some() {
        return Ok(QwenLaunch {
            server_name: plugin.to_string(),
            command: plugin.to_string(),
            args: Vec::new(),
            via: LaunchVia::Path,
        });
    }
    // 3) 源码 checkout（`__main__.py` 自举 sys.path，直接 `python3 <pkgdir>`）。
    if let Some(dir) = checkout_dir {
        let pkg = dir
            .join("src")
            .join("capabilities")
            .join(capability)
            .join(plugin.replace('-', "_"));
        if pkg.join("__main__.py").is_file() {
            let python = lookup("python3")
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "python3".to_string());
            return Ok(QwenLaunch {
                server_name: plugin.to_string(),
                command: python,
                args: vec![pkg.to_string_lossy().to_string()],
                via: LaunchVia::SourceCheckout,
            });
        }
    }
    Err(QwenResolveError {
        server: plugin.to_string(),
        reason: "no uvx, no PATH entry, and no source checkout".to_string(),
        hint: install_hint,
    })
}

/// 挂到模型面前的 4 个工具名（只读 3 + 写盘 1）。
///
/// **刻意不暴露 `read_image`**：上游 core 有 `read_image`，本 crate 已有同名的
/// `ToolName::ReadImage`（工作区内图片 → 多模态部件，且带 vision 门）。同名两个
/// 工具会让模型随机挑，且绕过 `ReadImage` 的视觉能力门 —— 那一门正是"看不见
/// 画面的模型不该被喂图"的诚实性保证。`visualize` 已覆盖"看文件"。
pub const QWEN_MM_TOOL_NAMES: [&str; 4] = ["media_info", "read_video", "visualize", "save_view"];

/// 写盘工具（Medium 风险；只读工具 Low）。
pub fn qwen_mm_tool_writes(tool: &str) -> bool {
    tool == "save_view"
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
        McpStdioSession::new(bash_cmd(), vec![script.to_string_lossy().to_string()])
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
    fn test_release_tag_format_matches_upstream() {
        assert_eq!(release_tag("core", "1.1.0"), "qwen-mm-plugins-core-v1.1.0");
        assert_eq!(
            release_tag("search", "1.1.0"),
            "qwen-mm-plugins-search-v1.1.0"
        );
    }

    #[test]
    fn test_resolve_without_anything_reports_install_hint() {
        // 注入空环境 → 三路探测全灭，必须回可操作的 hint。零全局变更。
        let dir = std::env::temp_dir().join(format!(
            "nt_qwen_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).expect("test temp dir");
        let env = ResolveEnv {
            path_dirs: Some(vec![dir.clone()]),
            checkout: Some(None),
        };
        let e = resolve_launch_with(QWEN_MM_CORE_PLUGIN, "core", QWEN_MM_CORE_VERSION, &env)
            .expect_err("must fail with empty env");
        assert!(e.hint.contains("qwen-mm-plugins-core-v1.1.0"));
        assert!(e.to_string().contains("unavailable"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_source_checkout_layout() {
        // 伪造最小 checkout 树 → 必须走 SourceCheckout。
        let base = std::env::temp_dir().join(format!(
            "nt_qwen_checkout_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let pkg = base
            .join("src")
            .join("capabilities")
            .join("core")
            .join("qwen_mm_plugins_core");
        std::fs::create_dir_all(&pkg).expect("fake checkout tree");
        std::fs::write(pkg.join("__main__.py"), "# fake\n").expect("fake main");
        let env = ResolveEnv {
            path_dirs: Some(vec![base.join("no-such-bin-dir")]),
            checkout: Some(Some(base.clone())),
        };
        let l = resolve_launch_with(QWEN_MM_CORE_PLUGIN, "core", QWEN_MM_CORE_VERSION, &env)
            .expect("checkout resolves");
        assert_eq!(l.via, LaunchVia::SourceCheckout);
        assert!(l.args.iter().any(|a| a.contains("qwen_mm_plugins_core")));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn test_tool_names_exclude_read_image_collision() {
        // 4 个工具，且**不含** read_image（与本 crate 的 ToolName::ReadImage 撞名）。
        assert_eq!(QWEN_MM_TOOL_NAMES.len(), 4);
        assert!(!QWEN_MM_TOOL_NAMES.contains(&"read_image"));
        assert!(QWEN_MM_TOOL_NAMES.contains(&"media_info"));
        assert!(QWEN_MM_TOOL_NAMES.contains(&"visualize"));
        // 写盘判定：只有 save_view。
        assert!(qwen_mm_tool_writes("save_view"));
        assert!(!qwen_mm_tool_writes("media_info"));
        assert!(!qwen_mm_tool_writes("read_video"));
        assert!(!qwen_mm_tool_writes("visualize"));
    }

    #[test]
    fn test_system_dep_and_credential_status_shape() {
        let deps = core_system_dep_status();
        let labels: Vec<&str> = deps.iter().map(|(l, _)| l.as_str()).collect();
        assert_eq!(
            labels,
            [
                "read_video / media_info (video & audio)",
                "visualize: Office / DrawIO (LibreOffice)",
                "visualize: LaTeX (.tex)",
                "visualize: 3D best-quality render (Blender)",
            ]
        );
        let creds = search_credential_status();
        assert_eq!(creds.len(), 4);
        assert!(creds.iter().all(|(k, _)| k.ends_with("_API_KEY")));
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
