//! `nt_engine` — 本地引擎适配.
//!
//! 移植 cumora `EngineAdapter{seedHome/startSession/run/classify/probe}`
//! 思想, 但收敛为同步 trait (无额外 `async-trait` 依赖):
//! 默认 `Echo` 零模型可跑; `Cli` 透传本机 `claude/codex/opencode` 等.

use serde::{Deserialize, Serialize};

use crate::nt_error::NtBotError;
use crate::nt_types::{TokenUsage, ToolCall, ToolName, TranscriptItem, TurnStatus};

/// 引擎一轮产出.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineTurn {
    pub assistant_text: String,
    pub status: TurnStatus,
    pub tool_calls: Vec<ToolCall>,
    /// 有则由 `nt_agent` 落 `ledger` (cumora `llm_calls` 本地子集).
    #[serde(default)]
    pub usage: Option<TokenUsage>,
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

/// 零模型回显引擎 (OpenMuse `sample` 对应物): 不调任何外部模型,
/// 把用户文本回显为 `reply`, 首轮即 `done`.
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
        })
    }
}

/// 本机 CLI 引擎: 经 PATH 启动指定命令 (`--version` 探活,
/// prompt 经 stdin 传入, stdout 即回复, 首轮即 `done`).
/// 安全默认: 不透传任何 token/URL (cumora file-IPC 零密钥思想的简化版).
#[derive(Debug)]
pub struct CliEngine {
    command: String,
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
        })
    }
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
        let mut child = std::process::Command::new(&self.command)
            .arg("-p")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|err| NtBotError::Engine {
                engine: self.command.clone(),
                reason: format!("spawn failed: {err}"),
            })?;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(prompt.as_bytes())?;
        }
        let output = child.wait_with_output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            return Err(NtBotError::Engine {
                engine: self.command.clone(),
                reason: if stderr.is_empty() {
                    "cli exited non-zero".to_owned()
                } else {
                    stderr
                },
            });
        }
        let text = String::from_utf8_lossy(&output.stdout).trim().to_owned();
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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{CliEngine, EngineAdapter, LocalEchoEngine};

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
}
