//! # nt_model_cli — 模型 CLI 问答桥（资源适配器）
//!
//! NeoTrix 自己的 CLI 终端没有独立模型配额，本模块把外部模型 CLI
//! （默认 `opencode run`，headless；任何同形 CLI 均可替换）包成晶体的
//! 同步 `NtLlmAsk`。外部项目永远只是可替换资源：类型名不带外部标签，
//! 命令/模型/后缀全是构造参数。
//!
//! ```text
//! NtCrystalTaskLoop ──▶ NtModelCliAsk::ask(prompt)
//!                          │  <cmd> run [--dir W] [-m M] [--log-level ERROR] <prompt>
//!                          ▼
//!                       stdout（trim 后）→ NtLlmReply { text, 结构置信度, model }
//! ```
//!
//! ## 设计要点
//! - 每次问答都是无状态单次 `run`（不接长会话），上下文膨胀由晶体侧
//!   的预算/剪枝管，不在外部会话里堆历史。
//! - 超时可杀：轮询 `try_wait`，超时 `kill`，不留僵尸进程。
//! - 可测性：`argv_template` 允许测试期替换命令（`/bin/echo` 验证透传，
//!   `/bin/sh` 验证超时，`/nonexistent` 验证失败路径）。
//!
//! # Safety
//! - 只用 `std::process`，无 shell 拼接（prompt 整体作单个 argv 传参），
//!   无 unsafe (R-P1)；生产代码无 `unwrap/expect/panic`。

// ⭐ 2026-10-03 跨域错位收敛：原先直取 `crate::l5_cognition::nt_crystal_core::…`
// （L1 越过 L2–L4），且本行在 3 个文件里逐字节重复。现经 **L1 自己的 facade** 转出
// —— `AGENTS.md` §4.2：「改跨层引用唯一合法通道是消费方自己那层的 facade」。
use crate::l1_action::nt_action_facade::{NtLlmAsk, NtLlmReply, NtTaskFusionError};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

/// 借模型问答桥。
pub struct NtModelCliAsk {
    command: String,
    /// 显式 argv 模板（测试/特殊调用用）；`None` = 标准 opencode run 构造。
    /// 模板 + prompt（单个 argv 追加）即完整命令。
    argv_template: Option<Vec<String>>,
    model: Option<String>,
    workdir: Option<PathBuf>,
    timeout: Duration,
    max_prompt_chars: usize,
    base_confidence: f64,
}

impl NtModelCliAsk {
    pub fn new() -> Self {
        Self {
            command: "opencode".to_string(),
            argv_template: None,
            model: None,
            workdir: None,
            timeout: Duration::from_secs(300),
            max_prompt_chars: 12_000,
            base_confidence: 0.65,
        }
    }

    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = command.into();
        self
    }

    pub fn with_argv_template(mut self, template: Vec<String>) -> Self {
        self.argv_template = Some(template);
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_workdir(mut self, dir: PathBuf) -> Self {
        self.workdir = Some(dir);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_base_confidence(mut self, confidence: f64) -> Self {
        self.base_confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn model_name(&self) -> String {
        self.model
            .clone()
            .unwrap_or_else(|| "opencode-default".to_string())
    }

    fn prompt_clipped(&self, prompt: &str) -> String {
        if prompt.chars().count() <= self.max_prompt_chars {
            return prompt.to_string();
        }
        let t: String = prompt.chars().take(self.max_prompt_chars).collect();
        format!("{t}\n…[prompt clipped to {} chars]", self.max_prompt_chars)
    }

    /// 组装完整 argv（含 prompt 作最后一个参数）。
    fn build_argv(&self, prompt: &str) -> Vec<String> {
        let clipped = self.prompt_clipped(prompt);
        if let Some(t) = &self.argv_template {
            let mut v = t.clone();
            v.push(clipped);
            return v;
        }
        let mut v = vec!["run".to_string()];
        if let Some(dir) = &self.workdir {
            v.push("--dir".to_string());
            v.push(dir.to_string_lossy().into_owned());
        }
        if let Some(m) = &self.model {
            v.push("-m".to_string());
            v.push(m.clone());
        }
        v.push("--log-level".to_string());
        v.push("ERROR".to_string());
        v.push(clipped);
        v
    }

    fn run_once(&self, prompt: &str) -> Result<String, String> {
        let argv = self.build_argv(prompt);
        let mut cmd = Command::new(&self.command);
        cmd.args(&argv)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        run_with_timeout(&mut cmd, self.timeout)
    }

    /// 流式 argv：在标准参数前插入 `--format json`（模板模式下不适用，回退整包）。
    fn build_stream_argv(&self, prompt: &str) -> Option<Vec<String>> {
        if self.argv_template.is_some() {
            return None;
        }
        let mut v = vec!["run".to_string(), "--format".to_string(), "json".to_string()];
        if let Some(dir) = &self.workdir {
            v.push("--dir".to_string());
            v.push(dir.to_string_lossy().into_owned());
        }
        if let Some(m) = &self.model {
            v.push("-m".to_string());
            v.push(m.clone());
        }
        v.push("--log-level".to_string());
        v.push("ERROR".to_string());
        v.push(self.prompt_clipped(prompt));
        Some(v)
    }
}

impl Default for NtModelCliAsk {
    fn default() -> Self {
        Self::new()
    }
}

/// opencode `--format json` 事件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NtStreamEvent {
    /// 助手文本增量（到达即拼）。
    Text(String),
    /// 步骤结束（reason 如 stop）。
    Done,
    /// 远端错误。
    RemoteError(String),
    /// 其他事件（step_start/tool/reasoning…），忽略。
    Other,
}

/// 纯函数：解析单行 json 事件（实测样本：step_start/text/step_finish）。
pub fn parse_stream_event(line: &str) -> NtStreamEvent {
    let v: serde_json::Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(_) => return NtStreamEvent::Other,
    };
    let part = v.get("part");
    let ptype = part
        .and_then(|p| p.get("type"))
        .and_then(|t| t.as_str())
        .unwrap_or("");
    match ptype {
        "text" => match part.and_then(|p| p.get("text")).and_then(|t| t.as_str()) {
            Some(s) => NtStreamEvent::Text(s.to_string()),
            None => NtStreamEvent::Other,
        },
        "step-finish" => NtStreamEvent::Done,
        _ => {
            if v.get("type").and_then(|t| t.as_str()) == Some("error") {
                let msg = v
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown remote error");
                return NtStreamEvent::RemoteError(msg.to_string());
            }
            NtStreamEvent::Other
        }
    }
}

/// 流式执行：逐行读 json 事件，文本增量即回调；回调 false 即 kill（Esc 取消）。
/// 读取泵跑独立线程，主线程 `recv_timeout` 节拍检查超时——即使远端全程静默
/// 也能按时 kill，不存在阻塞读 hung 死（第一单 PTY 实测曾踩中此坑）。
fn run_streaming(
    command: &str,
    argv: &[String],
    timeout: Duration,
    on_chunk: &dyn Fn(&str) -> bool,
) -> Result<String, String> {
    use std::io::BufRead;
    let mut child = Command::new(command)
        .args(argv)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn failed: {e}"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "stdout pipe missing".to_string())?;
    let child = std::sync::Arc::new(std::sync::Mutex::new(child));
    let killer = std::sync::Arc::clone(&child);
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    std::thread::spawn(move || {
        for line in std::io::BufReader::new(stdout).lines() {
            match line {
                Ok(l) => {
                    if tx.send(l).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    let kill_and_wait = |why: String| -> Result<String, String> {
        if let Ok(mut c) = killer.lock() {
            let _ = c.kill();
            let _ = c.wait();
        }
        Err(why)
    };
    let mut full = String::new();
    let mut raw = String::new();
    let start = std::time::Instant::now();
    loop {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(line) => {
                raw.push_str(&line);
                raw.push('\n');
                match parse_stream_event(&line) {
                NtStreamEvent::Text(delta) => {
                    full.push_str(&delta);
                    if !on_chunk(&delta) {
                        return kill_and_wait("cancelled by user".to_string());
                    }
                }
                NtStreamEvent::RemoteError(msg) => {
                    return kill_and_wait(format!("remote error: {msg}"));
                }
                NtStreamEvent::Done | NtStreamEvent::Other => {}
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if start.elapsed() >= timeout {
                    return kill_and_wait(format!(
                        "stream timed out after {}s",
                        timeout.as_secs()
                    ));
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    let ok = child
        .lock()
        .ok()
        .and_then(|mut c| c.wait().ok())
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok && full.trim().is_empty() {
        return Err("model command failed with empty stream".to_string());
    }
    let text = strip_ansi(&full).trim().to_string();
    if !text.is_empty() {
        return Ok(text);
    }
    // 宽容路径：远端说纯文本而非 json 事件（如测试替身 echo），且退出码正常
    // → 取原文一次性回调后返回。
    if ok {
        let raw_text = strip_ansi(&raw).trim().to_string();
        if !raw_text.is_empty() {
            let _ = on_chunk(&raw_text);
            return Ok(raw_text);
        }
    }
    Err("empty stream from model command".to_string())
}

/// crate 内复用：跑任意命令并取 stdout（发现源等调用方共用超时可杀语义）。
pub(crate) fn run_capture(
    program: &str,
    args: &[String],
    timeout: Duration,
) -> Result<String, String> {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    run_with_timeout(&mut cmd, timeout)
}

/// ⭐⭐ 带超时的同步执行：**并发读取两条管道** + 超时 kill，不留僵尸。
///
/// ## ⭐⭐⭐ 2026-10-03 重写：修「大输出 = 假超时」
///
/// ⛔ **旧实现**（轮询 `try_wait()`，**从不并发读管道**）：
/// ```ignore
/// loop { match child.try_wait()? {
///   Some(_) => { let out = child.wait_with_output()?; … }
///   None => { if 超时 { kill; return Err("timed out") } sleep(50ms) } } }
/// ```
/// ⭐⭐ **缺陷形态**：子进程 stdout/stderr 若填满管道缓冲区（macOS/Linux 通常 64 KiB）
/// 就会**永久阻塞在 write 上** ⇒ `try_wait()` 于是**永远**返回 `Ok(None)`
/// ⇒ 最后被判成超时 ⭐⭐ **即「输出越大越容易被误判超时」**。
///
/// ⭐ **本仓已有一份写对的实现**：`l2_perception/nt_world/social_access/probe.rs:50`
/// 的 `run_with_timeout` —— 它的 doc 逐字记录了这个 bug
/// （「本实现第一版把读取放在等待循环**之后**，被自测抓到」），
/// ⭐ 且带回归测试 `test_large_output_does_not_deadlock`（`probe.rs:363`）。
///
/// ⇒ ⭐⭐ **本函数此前是同一语义的「差版本」**（重复实现 + 缺陷）。
/// ⭐ 现改为**复用那份写对的实现**，⛔ 不再自带一份。
/// ⭐ 影响面：`run_capture` → `nt_io_provider/catalog/cli_free_source.rs`
/// （⭐ 免费模型发现路径会误判超时）。
fn run_with_timeout(cmd: &mut Command, timeout: Duration) -> Result<String, String> {
    // ⭐ 复用 `probe.rs` 的并发读实现（它返回 stdout/stderr/成功与否）。
    // ⭐ `get_program()` 给 `&OsStr`，而目标签名要 `&str`
    // ⛔ 非 UTF-8 的 program 名**不能静默替换成别的值** ⇒ 显式拒绝。
    let program = cmd
        .get_program()
        .to_str()
        .ok_or_else(|| "model command program is not valid UTF-8".to_string())?;
    let args: Vec<String> = cmd
        .get_args()
        .filter_map(|a| a.to_str().map(|s| s.to_owned()))
        .collect();
    let outcome = crate::l2_perception::nt_world::social_access::probe::run_with_timeout(
        program, &args, timeout,
    )
    .map_err(|e| format!("spawn failed: {e}"))?;
    // ⭐⭐⭐ **必须先判 `timed_out`** —— 我第一版替换时漏了它，
    // 导致 `test_ask_timeout_kills` 失败（实际文案 `model command failed: ` 而非
    // `timed out`）。⭐ `RunOutcome` 有 `timed_out: bool`（`probe.rs:124`），
    // ⭐⭐ **它排在 `success` 之前是有原因的**：超时被 kill 的子进程
    // **必然** `success == false`，⛔ 不先判它就会把「超时」误报成「命令失败」。
    if outcome.timed_out {
        return Err(format!(
            "model command timed out after {}s",
            timeout.as_secs()
        ));
    }
    if outcome.success {
        let text = strip_ansi(&outcome.stdout).trim().to_string();
        if text.is_empty() {
            return Err("empty stdout from model command".to_string());
        }
        Ok(text)
    } else {
        let err = strip_ansi(&outcome.stderr).trim().to_string();
        Err(format!("model command failed: {err}"))
    }
}

/// 去 ANSI 转义（`\x1b[...<字母>` CSI 序列 + 残留 ESC）：窗口展示与 `-free` 后缀
/// 判定都依赖干净文本；无匹配时原样返回。
pub fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // CSI: ESC [ params... final-byte(@..~)
            if chars.peek() == Some(&'[') {
                chars.next();
                for nc in chars.by_ref() {
                    if ('@'..='~').contains(&nc) {
                        break;
                    }
                }
                continue;
            }
            // 孤立 ESC：丢弃
            continue;
        }
        out.push(c);
    }
    out
}
/// 回复结构置信度（与 `nt_crystal_llm_bridge::confidence_for_text` 同规则）。
fn confidence_for_text(text: &str, base: f64) -> f64 {
    let t = text.trim();
    if t.is_empty() {
        return 0.0;
    }
    let lower = t.to_lowercase();
    let refusal = [
        "不知道",
        "无法回答",
        "抱歉",
        "不能",
        "无法提供",
        "as an ai",
        "i don't know",
        "i cannot",
        "unable to",
    ];
    let mut c = base.clamp(0.0, 1.0);
    if refusal.iter().any(|m| lower.contains(m)) {
        c = (c - 0.3).max(0.0);
    }
    if t.chars().count() < 8 {
        c = (c - 0.2).max(0.0);
    }
    c.clamp(0.0, 1.0)
}

impl NtLlmAsk for NtModelCliAsk {
    fn ask(&self, prompt: &str) -> Result<NtLlmReply, NtTaskFusionError> {
        let text = self
            .run_once(prompt)
            .map_err(NtTaskFusionError::Llm)?;
        let confidence = confidence_for_text(&text, self.base_confidence);
        Ok(NtLlmReply {
            text,
            confidence,
            model: self.model_name(),
        })
    }

    fn ask_stream(
        &self,
        prompt: &str,
        on_chunk: &dyn Fn(&str) -> bool,
    ) -> Result<NtLlmReply, NtTaskFusionError> {
        // 模板模式 = 自定义命令 + prompt 追加：同样走流式解析，
        // 纯文本输出由宽容路径一次性回调（测试替身 echo 即此）。
        let argv = match self.build_stream_argv(prompt) {
            Some(v) => v,
            None => {
                let mut v = self
                    .argv_template
                    .clone()
                    .unwrap_or_else(Vec::new);
                v.push(self.prompt_clipped(prompt));
                v
            }
        };
        let text = run_streaming(&self.command, &argv, self.timeout, on_chunk)
            .map_err(NtTaskFusionError::Llm)?;
        let confidence = confidence_for_text(&text, self.base_confidence);
        Ok(NtLlmReply {
            text,
            confidence,
            model: self.model_name(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_argv_default_shape() {
        let ask = NtModelCliAsk::new();
        let argv = ask.build_argv("你好");
        assert_eq!(argv[0], "run");
        assert!(argv.contains(&"--log-level".to_string()));
        assert!(argv.contains(&"ERROR".to_string()));
        assert_eq!(argv.last().map(String::as_str), Some("你好"));
        assert!(!argv.iter().any(|a| a == "-m"));
    }

    #[test]
    fn test_build_argv_with_model_and_dir() {
        let ask = NtModelCliAsk::new()
            .with_model("test/model")
            .with_workdir(PathBuf::from("/tmp"));
        let argv = ask.build_argv("p");
        let mpos = argv.iter().position(|a| a == "-m").unwrap();
        assert_eq!(argv[mpos + 1], "test/model");
        let dpos = argv.iter().position(|a| a == "--dir").unwrap();
        assert_eq!(argv[dpos + 1], "/tmp");
    }

    #[test]
    fn test_prompt_clipped() {
        let ask = NtModelCliAsk::new();
        let long = "x".repeat(12_001);
        let clipped = ask.prompt_clipped(&long);
        assert!(clipped.contains("clipped"));
        assert!(clipped.chars().count() < long.chars().count() + 100);
    }

    #[cfg(unix)]
    #[test]
    fn test_ask_echo_plumbs_prompt() {
        let ask = NtModelCliAsk::new()
            .with_command("/bin/echo")
            .with_argv_template(vec![])
            .with_model("echo-test");
        let reply = ask.ask("透传验证内容足够长").unwrap();
        assert!(reply.text.contains("透传验证内容足够长"));
        assert_eq!(reply.model, "echo-test");
    }

    #[test]
    fn test_ask_missing_command_errors() {
        let ask = NtModelCliAsk::new().with_command("/nonexistent-nt-xyz");
        let err = ask.ask("hi").unwrap_err();
        assert!(err.to_string().contains("spawn failed"));
    }

    #[cfg(unix)]
    #[test]
    fn test_ask_timeout_kills() {
        // sh -c "sleep 2"：多余的 prompt 参数仅占 $0 位，不影响睡眠
        let ask = NtModelCliAsk::new()
            .with_command("/bin/sh")
            .with_argv_template(vec!["-c".to_string(), "sleep 2".to_string()])
            .with_timeout(Duration::from_millis(400));
        let err = ask.ask("hi").unwrap_err();
        assert!(err.to_string().contains("timed out"), "实际文案: {}", err);
    }

    #[test]
    fn test_parse_stream_event_real_samples() {
        // 实测 `opencode run --format json` 三行样本
        let start = r#"{"type":"step_start","part":{"type":"step-start"}}"#;
        assert_eq!(parse_stream_event(start), NtStreamEvent::Other);
        let text = r#"{"type":"text","part":{"type":"text","text":"好"}}"#;
        assert_eq!(
            parse_stream_event(text),
            NtStreamEvent::Text("好".to_string())
        );
        let finish = r#"{"type":"step_finish","part":{"type":"step-finish","reason":"stop"}}"#;
        assert_eq!(parse_stream_event(finish), NtStreamEvent::Done);
        assert_eq!(parse_stream_event("not json"), NtStreamEvent::Other);
        let err = r#"{"type":"error","message":"boom"}"#;
        assert_eq!(
            parse_stream_event(err),
            NtStreamEvent::RemoteError("boom".to_string())
        );
    }

    #[test]
    fn test_ask_stream_template_falls_back_to_whole() {
        // echo 输出纯文本：宽容路径一次性回调
        let ask = NtModelCliAsk::new()
            .with_command("/bin/echo")
            .with_argv_template(vec![])
            .with_model("echo-test");
        let seen = std::sync::Mutex::new(Vec::new());
        let reply = ask
            .ask_stream("透传内容足够长", &|c| {
                seen.lock().unwrap().push(c.to_string());
                true
            })
            .unwrap();
        assert!(reply.text.contains("透传内容足够长"));
        assert_eq!(seen.lock().unwrap().len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn test_ask_stream_parses_json_chunks() {
        // sh 吐三行 json 事件：逐块回调并拼接
        let script = concat!(
            "printf '%s\\n' ",
            "'{\"type\":\"step_start\",\"part\":{\"type\":\"step-start\"}}' ",
            "'{\"type\":\"text\",\"part\":{\"type\":\"text\",\"text\":\"甲\"}}' ",
            "'{\"type\":\"text\",\"part\":{\"type\":\"text\",\"text\":\"乙\"}}' ",
            "'{\"type\":\"step_finish\",\"part\":{\"type\":\"step-finish\"}}'",
        );
        let ask = NtModelCliAsk::new()
            .with_command("/bin/sh")
            .with_argv_template(vec!["-c".to_string(), script.to_string()]);
        let seen = std::sync::Mutex::new(Vec::new());
        let reply = ask
            .ask_stream("prompt", &|c| {
                seen.lock().unwrap().push(c.to_string());
                true
            })
            .unwrap();
        assert_eq!(reply.text, "甲乙");
        assert_eq!(*seen.lock().unwrap(), vec!["甲".to_string(), "乙".to_string()]);
    }

    #[cfg(unix)]
    #[test]
    fn test_ask_stream_cancel_kills() {
        // 首块即拒收：立刻 kill，返回取消错
        let script = concat!(
            "printf '%s\\n' ",
            "'{\"type\":\"text\",\"part\":{\"type\":\"text\",\"text\":\"甲\"}}' ",
            "'{\"type\":\"text\",\"part\":{\"type\":\"text\",\"text\":\"乙\"}}'",
        );
        let ask = NtModelCliAsk::new()
            .with_command("/bin/sh")
            .with_argv_template(vec!["-c".to_string(), script.to_string()])
            .with_timeout(Duration::from_secs(10));
        let err = ask
            .ask_stream("prompt", &|_| false)
            .unwrap_err();
        assert!(err.to_string().contains("cancelled"));
    }

    #[test]
    fn test_build_stream_argv_shape() {
        let ask = NtModelCliAsk::new().with_model("m/mock");
        let argv = ask.build_stream_argv("p").unwrap();
        assert_eq!(&argv[0..3], &["run", "--format", "json"]);
        assert!(argv.contains(&"-m".to_string()));
        assert_eq!(argv.last().map(String::as_str), Some("p"));
        // 模板模式返回 None（回退整包）
        let t = NtModelCliAsk::new().with_argv_template(vec!["x".to_string()]);
        assert!(t.build_stream_argv("p").is_none());
    }
}

#[cfg(test)]
mod run_capture_regression_tests {
    use super::*;

    /// ⭐⭐⭐ **本改动的核心判据**：`run_capture` 在**大输出**下不得假超时。
    ///
    /// ⭐ **与 `probe.rs:363` 的 `test_large_output_does_not_deadlock` 同源**，
    /// ⭐⭐ 但守的是**另一条路径**（`run_capture` → 免费模型发现）。
    ///
    /// ⛔ 旧实现（轮询 `try_wait`、从不并发读管道）在 >64 KiB 输出下
    /// ⭐ **必然假超时**（macOS/Linux 管道缓冲通常 64 KiB，子进程阻塞在 write
    /// ⇒ `try_wait()` 永远 `Ok(None)` ⇒ 被判超时）。
    #[cfg(unix)]
    #[test]
    fn 大输出不被误判超时() {
        // ⭐ 生成 > 256 KiB（管道缓冲的 4 倍）⇒ 旧实现必挂
        let big = "y".repeat(256 * 1024);
        let script = format!("printf '%s' '{big}'");
        let ask = NtModelCliAsk::new()
            .with_command("/bin/sh")
            .with_argv_template(vec!["-c".to_string(), script])
            .with_timeout(Duration::from_secs(20));
        let got = ask.ask("hi").expect("大输出不应被判超时");
        assert_eq!(got.text.len(), big.len(), "输出应完整取回（未被截断/误判）");
    }

    /// ⭐ 反向护栏：**超时**仍须报「timed out」（⇦ 我第一版替换漏判 `timed_out`
    /// 时，这条正是它暴露的 ⭐ `test_ask_timeout_kills`）。
    #[cfg(unix)]
    #[test]
    fn 超时仍报timed_out而非命令失败() {
        let ask = NtModelCliAsk::new()
            .with_command("/bin/sh")
            .with_argv_template(vec!["-c".to_string(), "sleep 2".to_string()])
            .with_timeout(Duration::from_millis(400));
        let err = ask.ask("hi").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("timed out"),
            "⭐ 超时必须与「命令失败」可区分（实测文案：{msg}）"
        );
    }
}
