//! Backward-compatible re-export: types migrated to l1_action::nt_io::nt_io_llm
//!
//! Provides `LlmProvider` trait (old interface) alongside `UnifiedLlm` (new interface).
//! Note: `LlmProviderType` is defined in `nt_io_provider::common::factory` (comprehensive 30+ variants).

pub use crate::l1_action::nt_io::nt_io_llm::{
    LlmError, LlmRequest, LlmResponse, Message, Role, Usage, FinishReason,
    Tool, ToolCallInfo, StructuredOutputConfig, DataTrust,
    UnifiedLlm, LlmRegistry, LlmProviderType,
};

use async_trait::async_trait;
use tokio::sync::mpsc;

/// Backward-compatible LlmProvider trait (old interface).
///
/// The new interface is `UnifiedLlm`. This trait exists so that existing
/// implementations (`OpenAiProvider`, `AnthropicProvider`, etc.) continue to compile.
/// Providers should implement both traits for full compatibility.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Set proxy URL for HTTP requests.
    fn set_proxy(&mut self, proxy_url: &str);

    /// Raw completion: send request, get full response.
    async fn complete_raw(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError>;

    /// Streaming completion: returns a channel receiver of response chunks.
    async fn stream_complete_raw(
        &self,
        request: &LlmRequest,
    ) -> Result<mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError>;

    /// Data trust level for this provider.
    fn data_trust(&self) -> DataTrust;

    /// Complete (non-raw): delegates to complete_raw by default.
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        self.complete_raw(request).await
    }

    /// Stream complete: delegates to stream_complete_raw by default.
    async fn stream_complete(
        &self,
        request: &LlmRequest,
    ) -> Result<mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        self.stream_complete_raw(request).await
    }
}

/// CJK / 全角 字符判定 (1 token/char 口径)。
///
/// 2026-09-29：本地副本已删，改用唯一事实源。
/// 本处是**计量**（1 token/char 估算）⇒ 用宽口径 `is_cjk_wide`。
/// ⛔ 勿改用窄口径 `is_cjk_han` —— 那会让假名/谚文/全角被错判为
/// 「英文 1/4 char」，直接低估 token 数。
fn is_cjk_char(c: char) -> bool {
    neotrix_types::core::nt_cjk::is_cjk_wide(c)
}

/// Backward-compatible: estimate_tokens helper
///
/// CJK 感知: CJK/全角 1 token/char, 其余 4 chars/token (向下取整)。
/// 纯字节数 `len()/4` 会把中文高估 3 倍 (一个汉字 3 字节), 预算推导随之失真。
/// 非空输入至少 1 token; 空串 0 token。
///
/// 2026-09-27: ASCII 段由向上取整改为向下取整 ——
/// `nt_forecast::test_estimate_tokens_cjk_aware` 把本函数标为"单一事实源
/// (P0-7)", 要求 11 个 ASCII 字符 = 2 token (与 tiktoken cl100k 一致);
/// 向上取整会得 3。非空最小 1 的约定保留, 预算不足时由调用方按需加严。
pub fn estimate_tokens(text: &str) -> usize {
    // 空串同样走 `.max(1)`: nt_forecast 的"单一事实源"契约要求空串 = 1
    // (保守上界, 预算侧不为空串开口子)。
    let mut cjk = 0usize;
    let mut rest = 0usize;
    for c in text.chars() {
        if is_cjk_char(c) {
            cjk += 1;
        } else {
            rest += 1;
        }
    }
    (cjk + rest / 4).max(1)
}

/// Backward-compatible: truncate_preserving helper
pub fn truncate_preserving(text: &str, max_tokens: usize) -> &str {
    // 2026-09-27 修复: 原按**字节**切 (`&text[..max_chars]`), 对中文会切在
    // 多字节字符中间 → 直接 panic (违反禁 panic 铁律)。改为按 char 边界单调回退,
    // token 口径与 estimate_tokens 一致 (CJK 1 字 1 token, ASCII 4 字符 1 token)。
    if estimate_tokens(text) <= max_tokens {
        return text;
    }
    let mut cjk = 0usize;
    let mut ascii = 0usize;
    for (i, c) in text.char_indices() {
        if cjk + ascii / 4 >= max_tokens {
            return &text[..i];
        }
        if is_cjk_char(c) {
            cjk += 1;
        } else {
            ascii += 1;
        }
    }
    text
}

// ─── Additional backward-compatible stubs ──────────────────────────

/// Context budget calculation result
#[derive(Debug, Clone, Default)]
pub struct BudgetResult {
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub within_budget: bool,
    pub is_cliff: bool,
    pub messages_evicted: usize,
    pub tool_outputs_truncated: usize,
}

impl BudgetResult {
    pub fn retention_ratio(&self) -> f64 {
        if self.input_tokens + self.output_tokens == 0 {
            1.0
        } else {
            (self.input_tokens + self.output_tokens - self.messages_evicted) as f64
                / (self.input_tokens + self.output_tokens) as f64
        }
    }
}

/// Apply the context budget in place: evict oldest turns until the estimated
/// token count fits `max_tokens`.
///
/// Eviction order = oldest first, and the load-bearing anchors are never
/// dropped: System messages (task definition) and the trailing message (the
/// current request). `max_tokens == 0` means "budget disabled" → no eviction.
///
/// `tool_outputs_truncated` stays 0: this pass evicts whole turns only; tool
/// output truncation belongs to ContextPipeline layer 3
/// (`nt_agent_exec::budget_react_messages` documents the same contract).
pub fn apply_context_budget(messages: &mut Vec<Message>, max_tokens: usize) -> BudgetResult {
    let input_tokens = estimate_messages_tokens(messages);
    let mut total = input_tokens;
    let mut messages_evicted = 0usize;

    if max_tokens > 0 {
        while total > max_tokens {
            // 末条 (当前请求) 不可驱逐, System 不可驱逐。
            let evictable = messages.len().saturating_sub(1);
            match messages[..evictable].iter().position(|m| m.role != Role::System) {
                Some(idx) => {
                    // 同 estimate_messages_tokens 口径: content tokens + 4 协议开销
                    total = total.saturating_sub(estimate_tokens(&messages[idx].content) + 4);
                    messages.remove(idx);
                    messages_evicted += 1;
                }
                None => break,
            }
        }
    }

    BudgetResult {
        input_tokens,
        output_tokens: 0,
        within_budget: max_tokens == 0 || total <= max_tokens,
        // W1.1 compaction cliff: 上下文被压到保留率 < 35% = 任务成功率坍缩前兆。
        is_cliff: messages_evicted > 0
            && total.saturating_mul(100) < input_tokens.saturating_mul(35),
        messages_evicted,
        tool_outputs_truncated: 0,
    }
}

/// Estimate tokens in a list of messages.
///
/// 口径与下沉前 `core::nt_core_llm` 一致 (e2d73656): 每条 `estimate_tokens(content) + 4`
/// (role 标签 + tool 协议开销)。636d379c 误留 stub 返回 0, 导致 compaction
/// 预算推导为 0 并命中 `budget == 0` 早返, 测试 `test_compaction_*` 漂移, 故恢复真实现。
pub fn estimate_messages_tokens(messages: &[Message]) -> usize {
    let mut total = 0usize;
    for m in messages {
        total += estimate_tokens(&m.content);
        // role 标签 + 可能的 tool_calls/tool_call_id 协议开销
        total += 4;
    }
    total
}

/// Unified provider metadata (stub)
#[derive(Debug, Clone, Default)]
pub struct ProviderMetadata {
    pub name: String,
    pub version: String,
}

/// Provider capability flags
#[derive(Debug, Clone, Default)]
pub struct ProviderCapabilities {
    pub text: bool,
    pub vision: bool,
    pub tools: bool,
    pub streaming: bool,
}

/// Unified provider trait (stub, distinct from LlmProvider)
pub trait UnifiedProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;

    fn estimate_cost(&self, _request: &LlmRequest) -> CostEstimate {
        CostEstimate::default()
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities::default()
    }

    fn health(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = HealthStatus> + Send + '_>> {
        Box::pin(async { HealthStatus::healthy() })
    }
}

/// Health status (stub)
#[derive(Debug, Clone, Default)]
pub struct HealthStatus {
    /// ⛔⚠️ **伪信号**（审计裁定 2026-10-07，`check-fake-signal` R1 命中）。
    ///
    /// **实测证据**：
    ///  1. 本 struct **零外部消费**（`nt_core_llm::HealthStatus` 全仓 0 处引用）；
    ///  2. `healthy: true` 的**唯一**赋值点是 `healthy()` 构造器（L233）
    ///     ⇒ 字面量取值集合 = `{true}`（**没有**任何 `false` 路径）
    ///     ⇒ ⛔ **不存在**「不健康」这个状态；
    ///  3. 唯一调用点 L220 `Box::pin(async { HealthStatus::healthy() })`
    ///     ⛔ **不检查任何东西**，直接返回「健康」；
    ///  4. 类型自称 `Health status (stub)` ⇒ 作者已标注它是桩。
    ///
    /// ⚠️ 后果：任何调用它做健康判断的代码，
    ///   得到的**永远是「健康 + OK」** ⇒ 比「没有健康 API」更危险。
    ///
    /// ⚠️ ⭐ **同名异型 ×13**：全仓至少 **8 个** `HealthStatus` 定义
    ///   （`ffi/types.rs`、`nt_core_platform/health.rs`、`runtime_monitor.rs`、
    ///   `healing/self_healing/health_monitor.rs` 等）
    ///   外加 `neotrix-types/shared_types.rs:64` 的**枚举**版本
    ///   （`Healthy/Degraded/Unhealthy` —— ⭐ **那个才有真正的三态**）。
    ///   ⇒ 只按名字查找**必然**取到错的那个。
    ///
    /// ⭐ 正解（需 owner 决策）：
    ///  (a) 让 L220 **真的检查**（它已有 `&self`/provider 列表可查）
    ///      并按结果构造 `healthy: true/false`；
    ///  (b) 或改用 `neotrix-types::shared_types::HealthStatus`（**已有三态**）
    ///      ⇒ ⛔ 别再定义第 9 个同名类型；
    ///  (c) ⛔ 或删除本 stub（零消费 ⇒ 删除无破坏）。
    /// ⛔ 我**不擅自删除**：属公开 API（L1 trait 的一部分）。
    pub healthy: bool,
    pub message: String,
}

impl HealthStatus {
    /// ⚠️ 返回**恒定**「健康」⇒ ⛔ 不是检查结果（见字段文档的裁定）
    pub fn healthy() -> Self {
        Self { healthy: true, message: "OK".into() }
    }
}

/// Cost estimate (stub)
#[derive(Debug, Clone, Default)]
pub struct CostEstimate {
    pub input_cost: f64,
    pub output_cost: f64,
    pub total_cost: f64,
    pub estimated_cost_usd: f64,
}

/// Register LLM self tests (stub)
pub fn register_llm_self_tests() -> Vec<String> {
    vec![]
}

/// Redact internal content (stub)
pub fn redact_internals(content: &str) -> String {
    content.to_string()
}

/// Egress privacy guard (stub)
pub fn egress_privacy_guard(content: &str) -> Result<String, LlmError> {
    Ok(content.to_string())
}
