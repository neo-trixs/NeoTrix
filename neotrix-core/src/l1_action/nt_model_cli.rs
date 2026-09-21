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

use crate::neotrix::nt_crystal_core::{NtLlmAsk, NtLlmReply, NtTaskFusionError};
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
}

impl Default for NtModelCliAsk {
    fn default() -> Self {
        Self::new()
    }
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

/// 带超时的同步执行：轮询收割，超时 kill，不留僵尸。
fn run_with_timeout(cmd: &mut Command, timeout: Duration) -> Result<String, String> {
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("spawn failed: {e}"))?;
    let start = std::time::Instant::now();
    loop {
        match child
            .try_wait()
            .map_err(|e| format!("wait failed: {e}"))?
        {
            Some(_) => {
                let output = child
                    .wait_with_output()
                    .map_err(|e| format!("collect failed: {e}"))?;
                if output.status.success() {
                    let raw = String::from_utf8_lossy(&output.stdout);
                    let text = strip_ansi(&raw).trim().to_string();
                    if text.is_empty() {
                        return Err("empty stdout from model command".to_string());
                    }
                    return Ok(text);
                }
                let raw = String::from_utf8_lossy(&output.stderr);
                let err = strip_ansi(&raw).trim().to_string();
                return Err(format!("model command failed: {err}"));
            }
            None => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "model command timed out after {}s",
                        timeout.as_secs()
                    ));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
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
        assert!(err.to_string().contains("timed out"));
    }
}
