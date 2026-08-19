// ── Subagent System (from Claude Code: fork/async/sync/teammate 4 paths) ──

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use crate::neotrix::nt_io_provider::factory::create_gateway;
use crate::neotrix::nt_io_provider::types::{LlmProvider, LlmRequest, Message, Role};

use super::context::ContextTurn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubagentKind {
    Coder,
    Explorer,
    Planner,
}

#[derive(Debug, Clone)]
pub struct SubagentResult {
    pub kind: SubagentKind,
    pub output: String,
    pub tool_calls: u64,
    pub duration_ms: u64,
    pub success: bool,
}

/// 默认 subagent 模型 (与 entry/mod.rs 中 opencode 默认模型同级别)。
const DEFAULT_SUBAGENT_MODEL: &str = "gpt-4o-mini";
/// 可通过环境变量覆盖 subagent 模型。
const SUBAGENT_MODEL_ENV: &str = "NEOTRIX_SUBAGENT_MODEL";
const SUBAGENT_MAX_TOKENS: u32 = 2048;

/// 进程级共享 gateway 缓存 (UCN Phase 3: opencode 去依赖)。
///
/// `GatewayV2` 实现 `LlmProvider` (trait 要求 `Send + Sync`)，因此可以
/// 安全地以 `Arc<dyn LlmProvider>` 静态缓存 — 每个 subagent 都重建 gateway
/// (15s 超时 + provider 探测) 代价过高，只首次懒加载。
static SUBAGENT_GATEWAY: OnceLock<Arc<dyn LlmProvider>> = OnceLock::new();

/// 各 kind 的 system prompt — 保持与旧 `sub_prompt` 前缀语义一致
/// (Coder 直接执行 / Explorer 探索总结 / Planner 制定计划)。
fn subagent_system_prompt(kind: SubagentKind) -> &'static str {
    match kind {
        SubagentKind::Coder => {
            "You are a coding subagent. Write or modify code to accomplish the given task. \
             Return the concrete result: the code or files changed plus a concise summary."
        }
        SubagentKind::Explorer => {
            "You are an exploration subagent. Explore and summarize the given topic: \
             investigate the codebase or subject, then return a concise, structured summary of findings."
        }
        SubagentKind::Planner => {
            "You are a planning subagent. Create a plan for the given task: \
             return a step-by-step plan with clear phases, dependencies, and acceptance criteria."
        }
    }
}

pub struct SubagentDispatch;

impl SubagentDispatch {
    /// 根据 kind 构建 LLM 请求：system prompt (角色语义) + user message (原始任务)。
    /// 模型读 `NEOTRIX_SUBAGENT_MODEL`，缺省 `gpt-4o-mini`；Coder 用低温采样，
    /// Explorer/Planner 交给模型默认采样 (None)。
    pub fn build_request(kind: &SubagentKind, task: &str) -> LlmRequest {
        Self::build_request_with_context(kind, task, None)
    }

    /// P2-C2: 同 `build_request`，但允许注入父代理的压缩上下文摘要
    /// (subagent-as-context-management)。摘要以独立 System 消息下发，标记为
    /// "仅参考、勿复述"——子代理获得父对话的定向信息，又不会被任务语义污染。
    pub fn build_request_with_context(
        kind: &SubagentKind,
        task: &str,
        context_hint: Option<&str>,
    ) -> LlmRequest {
        let model = std::env::var(SUBAGENT_MODEL_ENV)
            .unwrap_or_else(|_| DEFAULT_SUBAGENT_MODEL.to_string());
        let temperature = match kind {
            SubagentKind::Coder => Some(0.2),
            SubagentKind::Explorer | SubagentKind::Planner => None,
        };
        let mut messages = vec![Message::new(Role::System, subagent_system_prompt(*kind))];
        if let Some(ctx) = context_hint {
            let ctx = ctx.trim();
            if !ctx.is_empty() {
                let digest = format!(
                    "Parent conversation digest (reference only, do not restate it):\n{}",
                    ctx
                );
                messages.push(Message::new(Role::System, &digest));
            }
        }
        messages.push(Message::new(Role::User, task));
        LlmRequest {
            model,
            messages,
            temperature,
            max_tokens: SUBAGENT_MAX_TOKENS,
            tools: vec![],
            image_data: None,
            thinking_budget: None,
            provider_params: HashMap::new(),
            constraint_json: None,
            structured_output: None,
            cacheable_prefix_tokens: None,
        }
    }

    /// P2-C2: 将父代理最近的对话 turns 压缩为有界摘要。
    /// 只取最近 N 条 turn，每条按预算截断，总量封顶 `max_total_chars`——
    /// 子代理只看到压缩后的父上下文，不复制全量历史 (token 节省核心)。
    pub fn compress_context(turns: &VecDeque<ContextTurn>, max_total_chars: usize) -> String {
        const MAX_TURNS: usize = 8;
        let tail: Vec<&ContextTurn> = turns.iter().rev().take(MAX_TURNS).collect();
        let mut budget = max_total_chars;
        let mut parts: Vec<String> = Vec::new();
        for t in tail.iter().rev() {
            let text = t.content.trim();
            if text.is_empty() {
                continue;
            }
            let take: String = text.chars().take(budget).collect();
            if take.is_empty() {
                break;
            }
            budget = budget.saturating_sub(take.chars().count());
            parts.push(format!("[{}] {}", t.role, take));
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("(recent conversation digest)\n{}", parts.join("\n"))
        }
    }

    /// 复用项目内原生 LLM 通道 (GatewayV2) 派发 subagent，不再 spawn
    /// 外部 `opencode exec` 子进程。`cwd` 保留签名兼容 — 原生通道无需进程 cwd。
    pub async fn run(kind: SubagentKind, task: &str, cwd: &str) -> SubagentResult {
        Self::run_with_provider(kind, task, cwd, Self::shared_gateway().await).await
    }

    /// 解析 (并懒构建一次) 进程级共享 gateway provider。
    async fn shared_gateway() -> Arc<dyn LlmProvider> {
        if let Some(provider) = SUBAGENT_GATEWAY.get() {
            return provider.clone();
        }
        // create_gateway 同步阻塞 (最多 15s: 探测本地端点 + 注册云端 provider)，
        // 放到 blocking 线程执行，避免卡住 async worker。
        let gateway = tokio::task::spawn_blocking(create_gateway)
            .await
            .unwrap_or_else(|e| {
                log::warn!("[subagent] gateway build task failed: {e}; using empty gateway");
                crate::neotrix::nt_io_provider::gateway::GatewayV2::new()
            });
        let gateway = Arc::new(gateway);
        // R-P79: 注册参与 Auto Exacto 周期重估 — 后台循环 5min cadence 统一 tick。
        // 即使并发下本 Arc 输给 get_or_init 竞争而被丢弃, Weak 注册也会在
        // 下一次 tick 自动剔除, 无泄漏。
        crate::neotrix::l1_body_impl::nt_io_provider::gateway::register_gateway_for_re_evaluation(&gateway);
        SUBAGENT_GATEWAY.get_or_init(|| gateway.clone()).clone()
    }

    /// 核心派发逻辑 — 对注入的 provider 执行，测试可注入 fake。
    async fn run_with_provider(
        kind: SubagentKind,
        task: &str,
        _cwd: &str,
        provider: Arc<dyn LlmProvider>,
    ) -> SubagentResult {
        Self::run_with_provider_ctx(kind, task, None, _cwd, provider).await
    }

    /// P2-C2: 带压缩上下文摘要的派发核心 (供 `run_with_context` 与并行派发复用)。
    async fn run_with_provider_ctx(
        kind: SubagentKind,
        task: &str,
        context_hint: Option<&str>,
        _cwd: &str,
        provider: Arc<dyn LlmProvider>,
    ) -> SubagentResult {
        let start = Instant::now();
        let request = Self::build_request_with_context(&kind, task, context_hint);
        match provider.complete(&request).await {
            Ok(resp) => SubagentResult {
                kind,
                output: resp.content,
                tool_calls: resp
                    .tool_calls
                    .as_ref()
                    .map(|c| c.len() as u64)
                    .unwrap_or(0),
                duration_ms: start.elapsed().as_millis() as u64,
                success: true,
            },
            Err(e) => SubagentResult {
                kind,
                output: format!("Subagent error: {}", e),
                tool_calls: 0,
                duration_ms: start.elapsed().as_millis() as u64,
                success: false,
            },
        }
    }

    /// P2-C2: 派发单个 subagent 并注入压缩的父上下文摘要。
    pub async fn run_with_context(
        kind: SubagentKind,
        task: &str,
        context_hint: &str,
        cwd: &str,
    ) -> SubagentResult {
        Self::run_with_provider_ctx(kind, task, Some(context_hint), cwd, Self::shared_gateway().await)
            .await
    }

    pub async fn run_parallel(
        tasks: Vec<(SubagentKind, String)>,
        cwd: &str,
    ) -> Vec<SubagentResult> {
        Self::run_parallel_with_context(tasks, None, cwd).await
    }

    /// P2-C2: 并行派发多个 subagent，共享同一份父上下文摘要。
    pub async fn run_parallel_with_context(
        tasks: Vec<(SubagentKind, String)>,
        context_hint: Option<&str>,
        cwd: &str,
    ) -> Vec<SubagentResult> {
        let handles: Vec<_> = tasks
            .into_iter()
            .map(|(kind, task)| {
                let cwd = cwd.to_string();
                let ctx = context_hint.map(|s| s.to_string());
                tokio::spawn(async move {
                    Self::run_with_provider_ctx(
                        kind,
                        &task,
                        ctx.as_deref(),
                        &cwd,
                        Self::shared_gateway().await,
                    )
                    .await
                })
            })
            .collect();
        let mut results = Vec::new();
        for h in handles {
            results.push(h.await.unwrap_or_else(|e| SubagentResult {
                kind: SubagentKind::Coder,
                output: format!("Join error: {}", e),
                tool_calls: 0,
                duration_ms: 0,
                success: false,
            }));
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_io_provider::types::{FinishReason, LlmError, LlmResponse, Usage};

    use super::super::context::ContextPipeline;

    /// Deterministic fake provider: returns the canned result for `complete`,
    /// so dispatch behavior is testable without any network or gateway build.
    struct FakeProvider {
        result: Result<LlmResponse, LlmError>,
    }

    #[async_trait::async_trait]
    impl LlmProvider for FakeProvider {
        async fn complete(&self, _req: &LlmRequest) -> Result<LlmResponse, LlmError> {
            self.result.clone()
        }

        async fn stream_complete(
            &self,
            _req: &LlmRequest,
        ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
            Err(LlmError::Unknown(
                "stream_complete not used in subagent tests".into(),
            ))
        }
    }

    fn fake_ok() -> Arc<FakeProvider> {
        Arc::new(FakeProvider {
            result: Ok(LlmResponse::plain(
                "fake-ok".into(),
                "fake-model".into(),
                Usage::default(),
                FinishReason::Stop,
            )),
        })
    }

    #[tokio::test]
    async fn test_subagent_dispatch_uses_provider() {
        let result = SubagentDispatch::run_with_provider(
            SubagentKind::Coder,
            "write a test",
            "/tmp",
            fake_ok(),
        )
        .await;
        assert!(result.success, "output: {}", result.output);
        assert!(
            result.output.contains("fake-ok"),
            "output: {}",
            result.output
        );
        assert_eq!(result.kind, SubagentKind::Coder);
        assert_eq!(result.tool_calls, 0);
    }

    #[tokio::test]
    async fn test_subagent_dispatch_provider_error() {
        let provider = Arc::new(FakeProvider {
            result: Err(LlmError::Server("boom".into())),
        });
        let result = SubagentDispatch::run_with_provider(
            SubagentKind::Explorer,
            "find something",
            "/tmp",
            provider,
        )
        .await;
        assert!(!result.success, "must fail on provider error");
        assert!(
            result.output.contains("Subagent error"),
            "output: {}",
            result.output
        );
        assert!(
            result.output.contains("boom"),
            "error detail must surface: {}",
            result.output
        );
        assert_eq!(result.tool_calls, 0);
    }

    #[test]
    fn test_subagent_build_request_shapes() {
        // If the env pins a model, the default-value assertion is meaningless — skip.
        if std::env::var(SUBAGENT_MODEL_ENV).is_ok() {
            return;
        }
        let req = SubagentDispatch::build_request(&SubagentKind::Coder, "do it");
        assert_eq!(req.model, DEFAULT_SUBAGENT_MODEL, "default model must hold");
        assert_eq!(req.max_tokens, SUBAGENT_MAX_TOKENS);
        assert_eq!(req.messages.len(), 2, "system + user");
        assert_eq!(req.messages[0].role, Role::System);
        assert!(
            req.messages[0].content.contains("coding subagent"),
            "coder system prompt: {}",
            req.messages[0].content
        );
        assert_eq!(req.messages[1].role, Role::User);
        assert_eq!(req.messages[1].content, "do it");
        assert_eq!(req.temperature, Some(0.2), "coder uses low temperature");
        assert!(req.tools.is_empty());

        // Explorer/Planner: model-default sampling (None), prefix semantics preserved.
        let explorer = SubagentDispatch::build_request(&SubagentKind::Explorer, "x");
        assert_eq!(explorer.temperature, None);
        assert!(
            explorer.messages[0]
                .content
                .contains("Explore and summarize"),
            "explorer system prompt: {}",
            explorer.messages[0].content
        );
        let planner = SubagentDispatch::build_request(&SubagentKind::Planner, "x");
        assert_eq!(planner.temperature, None);
        assert!(
            planner.messages[0].content.contains("Create a plan"),
            "planner system prompt: {}",
            planner.messages[0].content
        );
    }

    #[test]
    fn test_build_request_with_context_injects_digest() {
        let req = SubagentDispatch::build_request_with_context(
            &SubagentKind::Coder,
            "write x",
            Some("  [user] recent turn  "),
        );
        assert_eq!(req.messages.len(), 3, "system + digest + user");
        assert_eq!(req.messages[0].role, Role::System);
        assert_eq!(req.messages[1].role, Role::System, "digest as separate system msg");
        assert!(
            req.messages[1].content.contains("Parent conversation digest"),
            "digest marker: {}",
            req.messages[1].content
        );
        assert!(
            req.messages[1].content.contains("recent turn"),
            "digest body: {}",
            req.messages[1].content
        );
        assert_eq!(req.messages[2].role, Role::User);
        assert_eq!(req.messages[2].content, "write x", "task message unpolluted");

        let no_ctx = SubagentDispatch::build_request_with_context(&SubagentKind::Coder, "t", None);
        assert_eq!(no_ctx.messages.len(), 2, "None hint adds nothing");
    }

    #[test]
    fn test_compress_context_bounded_digest() {
        let mut pipeline = ContextPipeline::new(64_000);
        pipeline.push("user", "hello world".into(), 0);
        pipeline.push("assistant", "hi there".into(), 0);
        pipeline.push("user", "long line ".repeat(50).into(), 0);
        pipeline.push("tool", "".into(), 0);

        let digest = SubagentDispatch::compress_context(&pipeline.turns, 64);
        assert!(
            digest.chars().count() <= 64 + 64,
            "digest bounded (header/tag slack): {digest}"
        );
        assert!(digest.contains("hello world"), "newest tail includes early turns: {digest}");
        assert!(digest.contains("[user]") && digest.contains("[assistant]"), "role tags: {digest}");
        assert!(
            !digest.contains(&"long line ".repeat(5)),
            "over-budget content truncated, not copied whole: {digest}"
        );

        let empty = SubagentDispatch::compress_context(&VecDeque::new(), 64);
        assert!(empty.is_empty(), "no turns -> empty digest");
    }
}