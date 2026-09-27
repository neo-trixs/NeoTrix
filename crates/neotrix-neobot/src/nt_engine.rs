//! `nt_engine` — 本地引擎适配（同步 trait，无额外 `async-trait` 依赖）:
//! 默认 `Echo` 零模型可跑；`Cli` 透传本机 `claude/codex/opencode` 等.

use serde::{Deserialize, Serialize};

use crate::nt_cli::SideEffect;
use crate::nt_error::NtBotError;
use crate::nt_types::{TokenUsage, ToolCall, ToolName, TranscriptItem, TurnStatus};

/// CLI 子进程 stdout 上限（防模型刷屏打爆内存；超限截断并注记）。
pub const CLI_OUTPUT_CAP: usize = 64 * 1024;
/// CLI 默认超时秒（有界但宽松；`NEOBOT_CLI_TIMEOUT_SECS` 可调，
/// 0=不等即炸→用 30s 保守值）。
pub const CLI_DEFAULT_TIMEOUT_SECS: u64 = 300;

/// 引擎一轮产出.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineTurn {
    pub assistant_text: String,
    pub status: TurnStatus,
    pub tool_calls: Vec<ToolCall>,
    /// 有则由 `nt_agent` 落 `ledger`（调用账本）。
    #[serde(default)]
    pub usage: Option<TokenUsage>,
    /// CLI 子进程经 `NEOBOT_RESULT_PATH` 回传的结构化副作用
    ///（`nt_agent` 落 step 行）。
    #[serde(default)]
    pub side_effects: Vec<SideEffect>,
}

/// 本地引擎接口.
pub trait EngineAdapter {
    fn engine_id(&self) -> &str;
    /// 模型名 (ledger 用; 无模型概念返回空串).
    fn model_name(&self) -> &str {
        ""
    }
    fn probe(&self) -> Result<String, NtBotError>;
    fn run_turn(&self, prompt: &str, inbox: &[String]) -> Result<EngineTurn, NtBotError>;
    /// 带历史的多跳入口 — 默认忽略历史 (Echo/CLI 保持原语义).
    fn run_turn_with_history(
        &self,
        prompt: &str,
        history: &[TranscriptItem],
    ) -> Result<EngineTurn, NtBotError> {
        let inbox: Vec<String> = history
            .iter()
            .map(|item| item.content.clone())
            .collect();
        self.run_turn(prompt, &inbox)
    }
    /// 流式回合 — 默认退化为整段一次回调, 真流式引擎覆盖.
    fn run_turn_stream(
        &self,
        prompt: &str,
        history: &[TranscriptItem],
        on_delta: &mut dyn FnMut(&str),
    ) -> Result<EngineTurn, NtBotError> {
        let turn = self.run_turn_with_history(prompt, history)?;
        on_delta(&turn.assistant_text);
        Ok(turn)
    }
}

/// 零模型回显引擎：不调任何外部模型，把用户文本回显为 `reply`，首轮即 `done`.
#[derive(Debug, Default)]
pub struct LocalEchoEngine;

impl EngineAdapter for LocalEchoEngine {
    fn engine_id(&self) -> &str {
        "echo"
    }

    fn probe(&self) -> Result<String, NtBotError> {
        Ok("echo ready (no model required)".to_owned())
    }

    fn run_turn(&self, prompt: &str, _inbox: &[String]) -> Result<EngineTurn, NtBotError> {
        let text = format!("neobot(echo): {prompt}");
        Ok(EngineTurn {
            assistant_text: text,
            status: TurnStatus::Done,
            tool_calls: Vec::new(),
            usage: None,
            side_effects: Vec::new(),
        })
    }
}

/// 本机 CLI 引擎: 经 PATH 启动指定命令 (`--version` 探活,
/// prompt 经 stdin 传入, stdout 即回复, 首轮即 `done`).
/// 安全默认：不透传任何 token/URL（文件 IPC 零密钥）。
/// 有界执行：超时杀（整棵进程等价物：kill 主进程；子进程树由操作系统回收，
/// 单机可接受）+ stdout 上限 + `NEOBOT_RESULT_PATH` 副作用回传。
#[derive(Debug)]
pub struct CliEngine {
    command: String,
    timeout: std::time::Duration,
}

impl CliEngine {
    pub fn new(command: &str) -> Result<Self, NtBotError> {
        let trimmed = command.trim();
        if trimmed.is_empty() || trimmed.contains('/') || trimmed.contains('\\') {
            return Err(NtBotError::Invalid(
                "cli engine must be a bare command name on PATH".to_owned(),
            ));
        }
        Ok(Self {
            command: trimmed.to_owned(),
            timeout: cli_timeout_from_env(),
        })
    }

    /// 覆盖超时（测试/特殊长任务用）。
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

fn cli_timeout_from_env() -> std::time::Duration {
    let secs = std::env::var("NEOBOT_CLI_TIMEOUT_SECS")
        .ok()
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .filter(|secs| *secs > 0 && *secs <= 3600)
        .unwrap_or(CLI_DEFAULT_TIMEOUT_SECS);
    std::time::Duration::from_secs(secs)
}

fn result_path_for_child() -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("neobot-result-{}-{n}.jsonl", std::process::id()))
}

impl EngineAdapter for CliEngine {
    fn engine_id(&self) -> &str {
        &self.command
    }

    fn probe(&self) -> Result<String, NtBotError> {
        let output = std::process::Command::new(&self.command)
            .arg("--version")
            .output()
            .map_err(|err| NtBotError::Engine {
                engine: self.command.clone(),
                reason: format!("not found on PATH: {err}"),
            })?;
        let mut version = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if version.is_empty() {
            version = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        }
        if version.is_empty() {
            version = "unknown version".to_owned();
        }
        Ok(version)
    }

    fn run_turn(&self, prompt: &str, _inbox: &[String]) -> Result<EngineTurn, NtBotError> {
        use std::io::Write as _;
        let result_path = result_path_for_child();
        let result_env = result_path.to_string_lossy().into_owned();
        let mut child = std::process::Command::new(&self.command)
            .arg("-p")
            .env(crate::nt_cli::RESULT_PATH_ENV, &result_env)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|err| NtBotError::Engine {
                engine: self.command.clone(),
                reason: format!("spawn failed: {err}"),
            })?;
        if let Some(stdin) = child.stdin.take() {
            let mut stdin = stdin;
            // stdin 写失败（子进程早退）不炸：后面等退出码说话。
            let _stdin_written: Result<(), std::io::Error> =
                stdin.write_all(prompt.as_bytes());
        }
        // 并发读 stdout（防管道撑满死锁）+ 超时轮询杀。
        let mut stdout = child.stdout.take();
        let reader = std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(out) = stdout.as_mut() {
                use std::io::Read as _;
                let mut chunk = [0u8; 8192];
                loop {
                    match out.read(&mut chunk) {
                        Ok(0) => break,
                        Ok(n) => {
                            let room = CLI_OUTPUT_CAP.saturating_sub(buf.len());
                            if room == 0 {
                                // 已满：继续消费防死锁，多余丢弃。
                                continue;
                            }
                            let take = n.min(room);
                            if let Some(slice) = chunk.get(..take) {
                                buf.extend_from_slice(slice);
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
            buf
        });
        let start = std::time::Instant::now();
        let status = loop {
            match child.try_wait()? {
                Some(status) => break status,
                None => {
                    if start.elapsed() >= self.timeout {
                        let _killed = child.kill();
                        let _waited = child.wait();
                        let _joined = reader.join();
                        let _cleaned = std::fs::remove_file(&result_path);
                        return Err(NtBotError::Engine {
                            engine: self.command.clone(),
                            reason: format!(
                                "timeout after {}s (NEOBOT_CLI_TIMEOUT_SECS)",
                                self.timeout.as_secs()
                            ),
                        });
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        };
        let raw = reader.join().map_err(|err| NtBotError::Engine {
            engine: self.command.clone(),
            reason: format!("stdout reader failed: {err:?}"),
        })?;
        // 副作用回传文件（子进程是 `neobot` CLI 时写；无文件即无副作用）。
        let side_effects = std::fs::read_to_string(&result_path)
            .map(|raw| crate::nt_cli::parse_side_effects_jsonl(&raw).0)
            .unwrap_or_default();
        let _cleaned = std::fs::remove_file(&result_path);
        // stderr 随子进程退出后读（短输出场景；长输出截断保错）。
        let stderr = child
            .wait_with_output()
            .map(|output| String::from_utf8_lossy(&output.stderr).trim().to_owned())
            .unwrap_or_default();
        if !status.success() {
            return Err(NtBotError::Engine {
                engine: self.command.clone(),
                reason: if stderr.is_empty() {
                    format!("cli exited with {status}")
                } else {
                    stderr
                },
            });
        }
        let mut text = String::from_utf8_lossy(&raw).trim().to_owned();
        if raw.len() >= CLI_OUTPUT_CAP {
            text.push_str("…[stdout-truncated]");
        }
        // CLI 引擎不直接给 tool_calls (工具统一走本地网关, 防模型直调).
        let call = ToolCall {
            id: "cli-status-1".to_owned(),
            name: ToolName::SetTurnStatus,
            args: serde_json::json!({"status": "done"}),
        };
        Ok(EngineTurn {
            assistant_text: if text.is_empty() {
                "(empty reply)".to_owned()
            } else {
                text
            },
            status: TurnStatus::Done,
            tool_calls: vec![call],
            usage: None,
            side_effects,
        })
    }
}

/// 默认 Zen 免费模型（实测可用；`opencode models` 有更多 `-free` 可换）。
pub const DEFAULT_ZEN_MODEL: &str = "opencode/space-bunny-free";

/// opencode CLI 引擎（Zen 免费档直驱：本机已 `opencode auth login`，免 key）。
/// prompt 走 argv（`opencode run -m <model> <prompt>`，stdin 协议不兼容故不用）；
/// stdout 去 ANSI + 去 `>` 状态行，尾部即回复。无工具调用，终态 done。
#[derive(Debug)]
pub struct OpencodeEngine {
    model: String,
    timeout: std::time::Duration,
}

impl OpencodeEngine {
    pub fn new(model: &str) -> Result<Self, NtBotError> {
        let trimmed = model.trim();
        if trimmed.is_empty() {
            return Err(NtBotError::Invalid("opencode model is empty".to_owned()));
        }
        Ok(Self {
            model: trimmed.to_owned(),
            timeout: cli_timeout_from_env(),
        })
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    /// 本机 zen 免费表（`opencode models` 输出解析 `opencode/<id>` 行；未安装/未登录回空）。
    pub fn zen_models() -> Vec<(String, String)> {
        let output = std::process::Command::new("opencode")
            .arg("models")
            .output();
        let Ok(output) = output else {
            return Vec::new();
        };
        if !output.status.success() {
            return Vec::new();
        }
        parse_opencode_models(&String::from_utf8_lossy(&output.stdout))
    }
}

/// 解析 `opencode models` 输出（纯函数）：`opencode/<id>` 行 → (provider, model)。
pub fn parse_opencode_models(output: &str) -> Vec<(String, String)> {
    output
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .filter_map(|l| {
            let (provider, model) = l.split_once('/')?;
            if provider.trim().is_empty() || model.trim().is_empty() {
                return None;
            }
            if provider.contains(' ') || model.contains(' ') {
                return None;
            }
            Some((provider.to_owned(), model.to_owned()))
        })
        .collect()
}

/// 去 ANSI 转义（`\x1b[...m` 裸机实现，无 regex 依赖）。
fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.next() == Some('[') {
                for c2 in chars.by_ref() {
                    if c2.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

impl EngineAdapter for OpencodeEngine {
    fn engine_id(&self) -> &str {
        "opencode"
    }

    fn probe(&self) -> Result<String, NtBotError> {
        let output = std::process::Command::new("opencode")
            .arg("--version")
            .output()
            .map_err(|err| NtBotError::Engine {
                engine: "opencode".to_owned(),
                reason: format!("not found on PATH: {err}"),
            })?;
        let mut version = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if version.is_empty() {
            version = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        }
        Ok(version)
    }

    fn run_turn(&self, prompt: &str, _inbox: &[String]) -> Result<EngineTurn, NtBotError> {
        let mut child = std::process::Command::new("opencode")
            .args(["run", "-m", &self.model])
            .arg(prompt)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|err| NtBotError::Engine {
                engine: "opencode".to_owned(),
                reason: format!("spawn failed: {err}"),
            })?;
        let mut stdout = child.stdout.take();
        let reader = std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(out) = stdout.as_mut() {
                use std::io::Read as _;
                let mut chunk = [0u8; 8192];
                loop {
                    match out.read(&mut chunk) {
                        Ok(0) => break,
                        Ok(n) => {
                            let room = CLI_OUTPUT_CAP.saturating_sub(buf.len());
                            if room == 0 {
                                continue;
                            }
                            let take = n.min(room);
                            if let Some(slice) = chunk.get(..take) {
                                buf.extend_from_slice(slice);
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
            buf
        });
        let start = std::time::Instant::now();
        let status = loop {
            match child.try_wait().map_err(|err| NtBotError::Engine {
                engine: "opencode".to_owned(),
                reason: format!("wait failed: {err}"),
            })? {
                Some(status) => break status,
                None => {
                    if start.elapsed() >= self.timeout {
                        let _killed = child.kill();
                        let _waited = child.wait();
                        let _joined = reader.join();
                        return Err(NtBotError::Engine {
                            engine: "opencode".to_owned(),
                            reason: format!("timeout after {}s", self.timeout.as_secs()),
                        });
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        };
        let raw = reader.join().map_err(|e| NtBotError::Engine {
            engine: "opencode".to_owned(),
            reason: format!("reader thread failed: {e:?}"),
        })?;
        if !status.success() {
            let stderr = String::from_utf8_lossy(
                &child.stderr.take().map(|mut s| {
                    use std::io::Read as _;
                    let mut b = Vec::new();
                    let _read: Result<usize, std::io::Error> = s.read_to_end(&mut b);
                    b
                }).unwrap_or_default(),
            )
            .trim()
            .to_owned();
            return Err(NtBotError::Engine {
                engine: "opencode".to_owned(),
                reason: format!("exited {status}: {}", strip_ansi(&stderr).chars().take(300).collect::<String>()),
            });
        }
        let text = strip_ansi(&String::from_utf8_lossy(&raw))
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('>'))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_owned();
        if text.is_empty() {
            return Err(NtBotError::Engine {
                engine: "opencode".to_owned(),
                reason: "empty reply".to_owned(),
            });
        }
        Ok(EngineTurn {
            assistant_text: text,
            status: TurnStatus::Done,
            tool_calls: Vec::new(),
            usage: None,
            side_effects: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{CliEngine, EngineAdapter, LocalEchoEngine, OpencodeEngine, parse_opencode_models};

    #[test]
    fn echo_turn_is_done() {
        let turn = LocalEchoEngine.run_turn("hi", &[]).expect("turn");
        assert_eq!(turn.status, super::TurnStatus::Done);
    }

    #[test]
    fn cli_rejects_paths() {
        assert!(CliEngine::new("../evil").is_err());
        assert!(CliEngine::new("").is_err());
    }

    #[test]
    fn opencode_models_parse_filters_noise() {
        let got = parse_opencode_models(
            "opencode/big-pickle\nopencode/mimo-v2.5-free\n\nbroken line here\n/opencode\n",
        );
        assert_eq!(
            got,
            vec![
                ("opencode".to_owned(), "big-pickle".to_owned()),
                ("opencode".to_owned(), "mimo-v2.5-free".to_owned()),
            ]
        );
        assert!(OpencodeEngine::new("").is_err());
        assert!(OpencodeEngine::new("opencode/space-bunny-free").is_ok());
    }

    #[test]
    fn strip_ansi_drops_escapes() {
        assert_eq!(super::strip_ansi("\x1b[91m\x1b[1mError:\x1b[0m hi"), "Error: hi");
    }

    #[test]
    fn cli_timeout_kills_hung_child() {
        // `yes` 无视 `-p` 参数永久输出：300ms 超时必须杀掉并报 timeout。
        let engine = CliEngine::new("yes")
            .expect("engine")
            .with_timeout(std::time::Duration::from_millis(300));
        let err = engine.run_turn("hi", &[]).expect_err("yes must be killed");
        assert!(err.to_string().contains("timeout"), "{err}");
        // 缺失命令 → spawn 失败（Engine 错误，非 panic）。
        let err = CliEngine::new("definitely-not-a-real-binary-xyz")
            .expect("engine")
            .run_turn("hi", &[])
            .expect_err("missing binary must fail");
        assert!(err.to_string().contains("spawn failed"), "{err}");
        // 非零退出 → 退出码可见。
        let err = CliEngine::new("false")
            .expect("engine")
            .run_turn("hi", &[])
            .expect_err("false must fail");
        assert!(err.to_string().contains("exited"), "{err}");
    }

    #[test]
    fn cli_picks_up_result_file_side_effects() {
        // 本机 `sh` 不在 bare-name 白名单外（无斜杠即可）：用 `true` 跑通，
        // 副作用走 result 文件需要子进程配合——此处验证无文件即空副作用。
        let engine = CliEngine::new("true").expect("engine");
        let turn = engine.run_turn("hi", &[]).expect("turn");
        assert!(turn.side_effects.is_empty());
        assert_eq!(turn.status, super::TurnStatus::Done);
    }
}
