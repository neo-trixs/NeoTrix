//! CLI session-reuse inference backend.
//!
//! Absorbed from grok-bot-0.18-reconstructed's inference router pattern:
//! "Claude Code and Codex do not require separate API keys when their local
//! clients are already authenticated." This backend routes inference through
//! a locally-authenticated CLI agent by spawning it in non-interactive mode —
//! the CLI's existing login/session provides authentication; no API key is
//! stored or transmitted.
//!
//! Opt-in only via env (packaging-boundary privacy default): set
//! `NEOTRIX_CLI_BACKEND_CMD` to the full backend command (e.g.
//! `claude -p --output-format json`). Unset/blank = feature off.
//!
//! Honesty note (absorbed usage-ledger semantics): CLI backends do not return
//! token accounting usable for billing reconciliation — `Usage` is zeroed and
//! must be treated as an activity record, never an authoritative invoice.

use async_trait::async_trait;

use super::types::{FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, Role, Usage};

pub const ENV_BACKEND_CMD: &str = "NEOTRIX_CLI_BACKEND_CMD";
pub const ENV_MODEL_LABEL: &str = "NEOTRIX_CLI_BACKEND_MODEL";

pub struct CliSessionProvider {
    command: Vec<String>,
    model_label: String,
}

impl CliSessionProvider {
    /// Build from env. Returns `None` (feature off) when
    /// `NEOTRIX_CLI_BACKEND_CMD` is absent or blank — fail-closed default.
    pub fn from_env() -> Option<Self> {
        let raw = std::env::var(ENV_BACKEND_CMD).ok()?;
        let command = split_command(&raw)?;
        let model_label =
            std::env::var(ENV_MODEL_LABEL).unwrap_or_else(|_| "cli-session".to_string());
        Some(Self { command, model_label })
    }

    /// 测试专用直构器: 绕开进程级 env (多线程测试下 set_var/remove_var 是
    /// UB 温床, KNOWN FLAKY 根因)。生产路径仍唯一走 from_env。
    #[cfg(test)]
    fn from_command_for_test(cmd: &str, model_label: &str) -> Self {
        Self {
            command: split_command(cmd).expect("valid test command"),
            model_label: model_label.to_string(),
        }
    }

    /// Flatten a multi-turn request into one single-shot prompt. CLI agent
    /// backends are stateless per invocation; roles are tagged so the model
    /// can reconstruct the conversation shape.
    fn build_prompt(request: &LlmRequest) -> String {
        let mut sections: Vec<String> = Vec::with_capacity(request.messages.len());
        for m in &request.messages {
            let tag = match m.role {
                Role::System => "System",
                Role::User => "User",
                Role::Assistant => "Assistant",
                Role::Tool => "Tool",
            };
            sections.push(format!("[{}] {}", tag, m.content));
        }
        sections.join("\n\n")
    }

    /// Best-effort output extraction: if stdout is a JSON object carrying a
    /// text field (`result` — Claude Code `-p --output-format json`, or
    /// `text` / `content`), unwrap it; otherwise treat stdout as plain text.
    /// Never fails — malformed output degrades to raw text.
    fn parse_backend_output(stdout: &str) -> String {
        let trimmed = stdout.trim();
        if !trimmed.starts_with('{') {
            return trimmed.to_string();
        }
        match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(v) => ["result", "text", "content"]
                .iter()
                .find_map(|k| v.get(*k).and_then(|x| x.as_str()))
                .map(str::to_string)
                .unwrap_or_else(|| trimmed.to_string()),
            Err(_) => trimmed.to_string(),
        }
    }

    fn spawn_err(context: &str, e: std::io::Error) -> LlmError {
        LlmError::Network(format!("cli backend {}: {}", context, e))
    }
}

/// 引号感知的命令分词 (单/双引号内空白不切分) — 修复 grok-bot 吸收时
/// `sh -c 'cat >/dev/null; ...'` 被裸 split_whitespace 撕碎的缺陷。
fn split_command(raw: &str) -> Option<Vec<String>> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut has_token = false;
    for c in raw.chars() {
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                } else {
                    cur.push(c);
                }
            }
            None => match c {
                '\'' | '"' => {
                    quote = Some(c);
                    has_token = true;
                }
                c if c.is_whitespace() => {
                    if has_token {
                        parts.push(std::mem::take(&mut cur));
                        has_token = false;
                    }
                }
                _ => {
                    cur.push(c);
                    has_token = true;
                }
            },
        }
    }
    if has_token {
        parts.push(cur);
    }
    if parts.is_empty() { None } else { Some(parts) }
}

#[async_trait]
impl LlmProvider for CliSessionProvider {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        use tokio::io::AsyncWriteExt;

        let mut child = tokio::process::Command::new(&self.command[0])
            .args(&self.command[1..])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| Self::spawn_err("spawn", e))?;

        // Prompt goes over stdin — never argv (argv leaks via `ps`).
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(Self::build_prompt(request).as_bytes())
                .await
                .map_err(|e| Self::spawn_err("stdin", e))?;
            stdin.shutdown().await.map_err(|e| Self::spawn_err("stdin close", e))?;
            drop(stdin);
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| Self::spawn_err("wait", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LlmError::Server(format!(
                "cli backend exit {}: {}",
                output.status,
                stderr.trim()
            )));
        }

        let content = Self::parse_backend_output(&String::from_utf8_lossy(&output.stdout));
        Ok(LlmResponse::plain(
            content,
            self.model_label.clone(),
            Usage::default(),
            FinishReason::Stop,
        ))
    }

    /// Single-shot backend: run `complete`, yield exactly one chunk.
    async fn stream_complete(
        &self,
        request: &LlmRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let result = self.complete(request).await;
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        tokio::spawn(async move {
            let _ = tx.send(result).await;
        });
        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::Message;
    use crate::core::nt_core_self_test::TEST_ENV_LOCK;

    fn clear_env() {
        std::env::remove_var(ENV_BACKEND_CMD);
        std::env::remove_var(ENV_MODEL_LABEL);
    }

    /// 进程级 env 是全局的, 并行测试会互相清掉对方的变量 (实测 flaky) —
    /// 所有触碰 env 的测试必须持 TEST_ENV_LOCK 串行化。
    fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        TEST_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn test_from_env_absent_is_none() {
        let _g = env_guard();
        clear_env();
        assert!(CliSessionProvider::from_env().is_none(), "unset env = feature off");
    }

    #[test]
    fn test_from_env_blank_is_none() {
        let _g = env_guard();
        clear_env();
        std::env::set_var(ENV_BACKEND_CMD, "   ");
        assert!(CliSessionProvider::from_env().is_none(), "blank cmd = fail-closed off");
        clear_env();
    }

    #[test]
    fn test_from_env_parses_command_and_label() {
        let _g = env_guard();
        clear_env();
        std::env::set_var(ENV_BACKEND_CMD, "claude -p --output-format json");
        std::env::set_var(ENV_MODEL_LABEL, "claude-code-local");
        let p = CliSessionProvider::from_env().expect("env set");
        assert_eq!(p.command, vec!["claude", "-p", "--output-format", "json"]);
        assert_eq!(p.model_label, "claude-code-local");
        clear_env();
    }

    #[test]
    fn test_build_prompt_flattens_roles_in_order() {
        let mut request = LlmRequest::new("m", "ignored");
        request.messages = vec![
            Message::new(Role::System, "be terse"),
            Message::new(Role::User, "hi"),
            Message::new(Role::Assistant, "hello"),
            Message::new(Role::User, "bye"),
        ];
        let prompt = CliSessionProvider::build_prompt(&request);
        assert_eq!(prompt, "[System] be terse\n\n[User] hi\n\n[Assistant] hello\n\n[User] bye");
    }

    #[test]
    fn test_parse_backend_output_json_fields() {
        assert_eq!(
            CliSessionProvider::parse_backend_output(r#"{"result": "the answer"}"#),
            "the answer"
        );
        assert_eq!(
            CliSessionProvider::parse_backend_output(r#"{"text": "alt"}"#),
            "alt"
        );
    }

    #[test]
    fn test_parse_backend_output_plain_passthrough() {
        assert_eq!(CliSessionProvider::parse_backend_output("plain text"), "plain text");
        // trim() 语义: 破损 JSON 降级为去首尾空白的原文
        assert_eq!(
            CliSessionProvider::parse_backend_output("{\"broken\": "),
            "{\"broken\":"
        );
    }

    /// 端到端: cat 后端原样回显 stdin → complete 返回 prompt 全文。
    #[test]
    /// 经 from_command_for_test 直构 — 不触碰进程级 env, 根治 KNOWN FLAKY
    fn test_complete_echo_backend() {
        let p = CliSessionProvider::from_command_for_test("sh -c cat", "cli-session");
        let request = LlmRequest::new("m", "echo-me");
        let rt = tokio::runtime::Runtime::new().expect("rt");
        let resp = rt.block_on(p.complete(&request)).expect("complete ok");
        assert_eq!(resp.content, "[User] echo-me");
        assert_eq!(resp.usage.total_tokens, 0, "activity record only — zeroed usage");
    }

    /// 负例: 后端非零退出 → Err(Server), stderr 进错误信息。
    #[test]
    /// 经直构器绕开 env 竞态 (同上)
    fn test_complete_failing_backend_is_server_error() {
        let p = CliSessionProvider::from_command_for_test(
            "sh -c 'cat >/dev/null; echo boom >&2; exit 3'",
            "cli-session",
        );
        let request = LlmRequest::new("m", "hi");
        let rt = tokio::runtime::Runtime::new().expect("rt");
        match rt.block_on(p.complete(&request)) {
            Err(LlmError::Server(msg)) => assert!(msg.contains("boom"), "stderr surfaced: {}", msg),
            other => panic!("expected Server error, got {:?}", other.map(|_| ())),
        }
    }
}
