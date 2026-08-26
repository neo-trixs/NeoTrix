//! LLM Provider 核心类型定义
//!
//! 更新说明 (2026-07-01):
//! - `temperature` 改为 `Option<f32>` — 部分新模型 (如 Claude Sonnet 5) 不再支持采样参数
//! - 新增 `thinking_budget` — 支持模型自适应思考 (extended thinking)
//! - 新增 `provider_params` — 额外 provider 专用参数映射
//!
//! 2026-07-04: 新增 ProviderCategory (自我/客体分离)

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError>;
    async fn stream_complete(&self, request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError>;

    /// 将 provider 的 HTTP 客户端切换为代理路由 (子母阵 Proxied/Tor 画像注入)。
    /// 默认 no-op — 不支持代理注入的 provider 保持原客户端不变。
    fn set_proxy(&mut self, _proxy_url: &str) {}
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<Message>,
    /// Optional: None = use model default (Sonnet 5 removes temperature/top_p/top_k)
    pub temperature: Option<f32>,
    pub max_tokens: u32,
    pub tools: Vec<Tool>,
    pub image_data: Option<String>,
    /// Extended thinking budget (tokens). Some(0) = disable, Some(n) = budget, None = model default
    pub thinking_budget: Option<u32>,
    /// Provider-specific extra params (e.g. {"top_p": 0.9, "top_k": 40})
    pub provider_params: HashMap<String, serde_json::Value>,
    /// Optional constrained decoding: JSON-serialized Constraint from nt_io_constrained.
    /// Applied by ConstrainedGateway wrapper at the provider layer.
    pub constraint_json: Option<serde_json::Value>,
    /// Native structured output configuration for provider-native APIs.
    /// When set, providers use their native JSON mode (OpenAI response_format,
    /// Anthropic output_config, Gemini response_mime_type) instead of constrained decoding.
    pub structured_output: Option<StructuredOutputConfig>,
    /// Provider prefix caching hint: 稳定前缀 (system + 工具定义 + 早期历史) 的
    /// token 数。标记后 gateway 将其纳入缓存指纹, 避免不同前缀标记共享缓存;
    /// provider 层可据此对齐其 prefix cache。None = 未标注。
    pub cacheable_prefix_tokens: Option<usize>,
}

/// Provider-native structured output configuration.
///
/// Each major provider has its own native API for enforcing structured output:
/// - OpenAI: `response_format: { type: "json_schema", json_schema: { ... } }`
/// - Anthropic: `output_config: { format: { type: "json_object" } }`
/// - Gemini: `response_mime_type: "application/json"` + `response_schema`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructuredOutputConfig {
    /// Simple JSON object mode (no schema enforcement, just pure JSON)
    JsonObject,
    /// JSON Schema enforcement (provider-native where supported)
    JsonSchema(Value),
}

impl StructuredOutputConfig {
    pub fn json_schema(schema: Value) -> Self {
        StructuredOutputConfig::JsonSchema(schema)
    }

    pub fn is_json_object(&self) -> bool {
        matches!(self, StructuredOutputConfig::JsonObject)
    }
}

impl LlmRequest {
    pub fn new(model: &str, prompt: &str) -> Self {
        Self {
            model: model.to_string(),
            messages: vec![Message::new(Role::User, prompt)],
            temperature: Some(0.7),
            max_tokens: 4096,
            tools: vec![],
            image_data: None,
            thinking_budget: None,
            provider_params: HashMap::new(),
            constraint_json: None,
            structured_output: None,
            cacheable_prefix_tokens: None,
        }
    }

    pub fn with_structured_output(mut self, config: StructuredOutputConfig) -> Self {
        self.structured_output = Some(config);
        self
    }

    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_thinking(mut self, budget: u32) -> Self {
        self.thinking_budget = Some(budget);
        self
    }

    pub fn with_temperature(mut self, temperature: Option<f32>) -> Self {
        self.temperature = temperature;
        self
    }

    /// 干净的 temperature 值（f64，四舍五入到 2 位小数）。
    ///
    /// f32 序列化会产生精度噪声（如 0.7 → 0.699999988079071），
    /// 部分 provider（Pollinations 实测 2026-08-09）会拒绝非标准采样值。
    /// 所有 provider 序列化 temperature 时必须用此方法而非 `json!(f32)`。
    pub fn temperature_clean(&self) -> Option<f64> {
        self.temperature.map(|t| {
            // 必须在 f64 域运算：f32 域 round 后仍是 f32 近似值
            let t64 = t as f64;
            (t64 * 100.0).round() / 100.0
        })
    }

    /// Attach a base64 image payload as `image_data` (data URI) for providers
    /// that support vision. `image_b64` is the raw base64 of the image bytes;
    /// the data-URI prefix is inferred as png/jpeg/jpeg by presence of the
    /// PNG magic bytes.
    pub fn with_image_b64(mut self, image_b64: &str) -> Self {
        let prefix = if image_b64.starts_with("data:image/") {
            String::new()
        } else if image_b64.starts_with("/9j/") {
            "data:image/jpeg;base64,".to_string()
        } else {
            "data:image/png;base64,".to_string()
        };
        self.image_data = Some(format!("{}{}", prefix, image_b64));
        self
    }

    pub fn with_provider_param(mut self, key: &str, value: serde_json::Value) -> Self {
        self.provider_params.insert(key.to_string(), value);
        self
    }

    /// Set a constrained decoding constraint (serialized JSON).
    /// The ConstrainedGateway will parse and apply this at inference time.
    pub fn with_constraint(mut self, constraint: serde_json::Value) -> Self {
        self.constraint_json = Some(constraint);
        self
    }

    /// Whether this model uses adaptive thinking (no temperature/top_p/top_k)
    pub fn has_no_sampling_params(&self) -> bool {
        self.temperature.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn new(role: Role, content: &str) -> Self {
        Self {
            role,
            content: content.to_string(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn tool(content: &str, call_id: &str) -> Self {
        Self {
            role: Role::Tool,
            content: content.to_string(),
            tool_calls: None,
            tool_call_id: Some(call_id.to_string()),
        }
    }

    pub fn assistant_with_calls(content: &str, calls: Vec<ToolCallInfo>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.to_string(),
            tool_calls: Some(calls),
            tool_call_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub usage: Usage,
    pub finish_reason: FinishReason,
    /// Tool calls requested by the model (when finish_reason == Tool).
    /// Populated by providers that parse `tool_calls` from the raw response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallInfo>>,
}

impl LlmResponse {
    /// Convenience constructor for providers that do not surface tool calls.
    pub fn plain(content: String, model: String, usage: Usage, finish_reason: FinishReason) -> Self {
        Self { content, model, usage, finish_reason, tool_calls: None }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    Length,
    Tool,
    ContentFilter,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum LlmError {
    Network(String),
    Authentication(String),
    RateLimit(String),
    InvalidRequest(String),
    Server(String),
    Unknown(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::Network(e) => write!(f, "Network: {}", e),
            LlmError::Authentication(e) => write!(f, "Auth: {}", e),
            LlmError::RateLimit(e) => write!(f, "RateLimit: {}", e),
            LlmError::InvalidRequest(e) => write!(f, "Invalid: {}", e),
            LlmError::Server(e) => write!(f, "Server: {}", e),
            LlmError::Unknown(e) => write!(f, "Unknown: {}", e),
        }
    }
}

impl std::error::Error for LlmError {}

impl From<String> for LlmError {
    fn from(s: String) -> Self {
        LlmError::Unknown(s)
    }
}

impl From<&str> for LlmError {
    fn from(s: &str) -> Self {
        LlmError::Unknown(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_image_b64_plain_png_gets_data_uri() {
        let req = LlmRequest::new("gpt-4o", "describe").with_image_b64("iVBORw0KGgo=");
        let uri = req.image_data.expect("image_data set");
        assert!(uri.starts_with("data:image/png;base64,iVBORw0KGgo="), "got {uri}");
    }

    #[test]
    fn test_with_image_b64_jpeg_magic_gets_jpeg_uri() {
        let req = LlmRequest::new("gpt-4o", "describe").with_image_b64("/9j/4AAQSkZJRg==");
        let uri = req.image_data.expect("image_data set");
        assert!(uri.starts_with("data:image/jpeg;base64,/9j/"), "got {uri}");
    }

    #[test]
    fn test_with_image_b64_already_data_uri_unchanged() {
        let req = LlmRequest::new("gpt-4o", "describe").with_image_b64("data:image/webp;base64,UklGRg==");
        let uri = req.image_data.expect("image_data set");
        assert_eq!(uri, "data:image/webp;base64,UklGRg==");
    }

    #[test]
    fn test_temperature_clean_removes_f32_noise() {
        // f32 0.7 序列化为 0.699999988079071，clean 后必须是干净的 0.7
        let req = LlmRequest::new("openai", "hi");
        let clean = req.temperature_clean().expect("temperature set");
        assert_eq!(clean, 0.7, "f32 noise must be removed, got {clean}");
        // 序列化验证：json!(f64) 输出干净值
        let json = serde_json::json!(clean);
        assert_eq!(json.to_string(), "0.7", "json serialization must be clean, got {json}");
    }

    #[test]
    fn test_temperature_clean_none_returns_none() {
        let req = LlmRequest::new("gpt-4o", "describe").with_temperature(None);
        assert_eq!(req.temperature_clean(), None);
    }

    #[test]
    fn test_temperature_clean_preserves_common_values() {
        // 常见采样值必须原样保留（f64 域四舍五入到 2 位小数）
        for (input, expected) in [
            (0.0f32, 0.0f64),
            (1.0f32, 1.0f64),
            (0.5f32, 0.5f64),
            (1.5f32, 1.5f64),
            (0.25f32, 0.25f64),
        ] {
            let req = LlmRequest::new("gpt-4o", "describe").with_temperature(Some(input));
            let clean = req.temperature_clean().expect("temperature set");
            assert_eq!(clean, expected, "input {input} -> {clean}, expected {expected}");
        }
    }

    #[test]
    fn test_temperature_clean_rounds_to_two_decimals() {
        // 超过 2 位小数的值应四舍五入到 2 位
        let req = LlmRequest::new("gpt-4o", "describe").with_temperature(Some(0.3333f32));
        let clean = req.temperature_clean().expect("temperature set");
        assert_eq!(clean, 0.33, "got {clean}");
    }
}

// ────────────────────────────────────────────────────────
// Token 预算引擎 (D3 下沉: 原 nt_io_provider::context_budget)
// ────────────────────────────────────────────────────────
// Token 预算引擎 — LLM 调用链路上下文压缩的共享实现。
//
// 文献依据 (2026 token optimization 主线):
// - 工具输出在进入 LLM 上下文前压缩可省 60-95% token (Headroom / RTK)
// - agent 重发上下文占推理账单 ~62% (Cockroach Labs, 2026)
// - 选择性压缩/裁剪历史省 20-40% 且不损连贯性 (Adaline, 2026)
//
// 策略 (确定性、无损质量, 不做 LLM 重写以避免引入额外调用):
// 1. 单条工具输出超限 → 头/尾保留折叠 (head 60% / tail 40%)
// 2. 总上下文超预算 → 丢弃最旧非 System 轮次, 保留末条 (当前请求)
//
// 与 `budget_react_messages` (neocodex) / `resume_session` 的 token 估算口径一致。
// 本估算器是**单一事实源** (P0-7): tiktoken cl100k_base 精确计数优先;
// tiktoken 不可用 (如离线首跑) 时回退到 CJK 感知逐字符估算 (非 CJK ≈ 4 chars/token,
// CJK ≈ 1 token/char — 字节口径会低估 CJK 4x, 导致上下文溢出)。

use std::sync::OnceLock;

/// CJK 相关 Unicode 区间: 汉字/假名/谚文/全角。
fn is_cjk(c: char) -> bool {
    matches!(
        c,
        '\u{3000}'..='\u{303F}'   // CJK 标点
        | '\u{3040}'..='\u{30FF}' // 假名
        | '\u{3400}'..='\u{4DBF}' // CJK Ext A
        | '\u{4E00}'..='\u{9FFF}' // CJK 统一表意
        | '\u{AC00}'..='\u{D7AF}' // 谚文
        | '\u{FF00}'..='\u{FFEF}' // 全角/半角
    )
}

/// 单字符 token 成本: CJK ≈ 1 token, 其余 ≈ 1/4 token (4 chars/token)。
fn char_token_cost(c: char) -> f64 {
    if is_cjk(c) {
        1.0
    } else {
        0.25
    }
}

/// 进程级 tiktoken BPE 单例 (cl100k_base)。构建失败 (如离线首跑) 时为 `None`,
/// 此时回退到 CJK 感知逐字符估算。与 neocodex `count_tokens` 共享同一口径。
static TIKTOKEN_BPE: OnceLock<Option<tiktoken_rs::CoreBPE>> = OnceLock::new();

/// 估算一段文本的 token 数。
///
/// **单一事实源 (P0-7)**: 若 tiktoken 可用, 用 cl100k_base 精确计数
/// (`encode_with_special_tokens`); 否则回退到 CJK 感知逐字符估算 (保守上界, 最小 1)。
pub fn estimate_tokens(text: &str) -> usize {
    let bpe = TIKTOKEN_BPE.get_or_init(|| tiktoken_rs::cl100k_base().ok());
    if let Some(bpe) = bpe {
        bpe.encode_with_special_tokens(text).len().max(1)
    } else {
        let mut tokens = 0.0;
        for c in text.chars() {
            tokens += char_token_cost(c);
        }
        (tokens.ceil() as usize).max(1)
    }
}

/// 估算一组消息的 token 数 (含每消息协议开销 ~4 token)。
pub fn estimate_messages_tokens(messages: &[Message]) -> usize {
    let mut total = 0usize;
    for m in messages {
        total += estimate_tokens(&m.content);
        // role 标签 + 可能的 tool_calls/tool_call_id 协议开销
        total += 4;
    }
    total
}

/// 按 token 预算截断字符串, 保留头部 `head_ratio` 与尾部其余, 中段折叠。
/// 字符边界安全 (按 char 扫描, 不会切坏 UTF-8 / CJK)。
pub fn truncate_preserving(text: &str, max_tokens: usize, head_ratio: f64) -> String {
    if estimate_tokens(text) <= max_tokens {
        return text.to_string();
    }
    let head_budget = ((max_tokens as f64) * head_ratio.clamp(0.0, 1.0)).floor() as usize;
    let tail_budget = max_tokens.saturating_sub(head_budget);
    let head = take_until_tokens(text, head_budget, true);
    let tail = take_until_tokens(text, tail_budget, false);
    match (head.is_empty(), tail.is_empty()) {
        (true, true) => "…[truncated]…".to_string(),
        (true, false) => format!("…[truncated]…\n{tail}"),
        (false, true) => format!("{head}\n…[truncated]…"),
        (false, false) => format!("{head}\n…[truncated]…\n{tail}"),
    }
}

/// 从头部或尾部累进 token 消耗, 返回不超过预算的字符数。
fn take_until_tokens(text: &str, budget: usize, from_start: bool) -> String {
    if budget == 0 {
        return String::new();
    }
    let mut consumed = 0.0;
    let mut out = String::new();
    if from_start {
        for c in text.chars() {
            consumed += char_token_cost(c);
            if consumed > budget as f64 {
                break;
            }
            out.push(c);
        }
    } else {
        for c in text.chars().rev() {
            consumed += char_token_cost(c);
            if consumed > budget as f64 {
                break;
            }
            out.push(c);
        }
        // 尾部反向收集 → 恢复正序
        out = out.chars().rev().collect();
    }
    out
}

/// 上下文预算压缩结果 (观测杠杆: 每次 apply 后可见具体削减量)。
///
/// W1.1 (batch3 2026-08-26, 源: arxiv 2608.22752 Compaction Cliff):
/// 长会话压缩过猛 → 下游任务成功率断崖式衰减。本结构新增观测字段,
/// 使调用方可在压缩事件发生时感知断崖风险 (retention_ratio / is_cliff)。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BudgetResult {
    pub original_tokens: usize,
    pub final_tokens: usize,
    pub tool_outputs_truncated: usize,
    pub messages_evicted: usize,
    /// 压缩保留率, 万分比 bp (final*10000/original; original=0 时记 10000)。
    /// 用整数存储保持结构体 Eq 可用 (f64 不实现 Eq)。
    pub retention_ratio_bp: u32,
    /// 断崖事件: 上下文规模 ≥ `COMPACTION_CLIFF_MIN_TOKENS` 且保留率 < `COMPACTION_CLIFF_RATIO_BP`
    pub is_cliff: bool,
}

impl BudgetResult {
    pub fn saved_tokens(&self) -> usize {
        self.original_tokens.saturating_sub(self.final_tokens)
    }

    /// 压缩保留率 (0.0-1.0)。内部以 bp (万分比) 存储保持 Eq 可导出。
    pub fn retention_ratio(&self) -> f64 {
        self.retention_ratio_bp as f64 / 10_000.0
    }
}

/// 断崖判定: 保留率低于此值 (35%) 视为激进压缩。
pub const COMPACTION_CLIFF_RATIO_BP: u32 = 3_500;
/// 断崖判定的最小上下文规模 — 微型对话天然高压缩率, 不构成断崖。
pub const COMPACTION_CLIFF_MIN_TOKENS: usize = 2_048;

/// 对消息序列应用 token 预算:
/// 1. 单条工具输出 > `per_tool_output_tokens` 时截断 (0 = 禁用截断)
/// 2. 总量仍超 `max_tokens` 时, 丢弃最旧非 System 轮次 (保留末条 = 当前请求)
/// 3. 锚点保护 (W1.1): 首条 User 消息 (任务定义) 不被驱逐 — 压缩断崖研究中
///    丢失任务定义是下游性能坍缩的主因之一; 仅当除锚点外无可弃消息时放行驱逐
///
/// 保留不变量: 索引 0 若为 System 永不丢弃; 末条 (当前 user 请求/最新 tool 结果)
/// 永不被驱逐 — 与 neocodex `budget_react_messages` 语义对齐。
pub fn apply_context_budget(
    messages: &mut Vec<Message>,
    max_tokens: usize,
    per_tool_output_tokens: usize,
) -> BudgetResult {
    let original_tokens = estimate_messages_tokens(messages);
    let mut result = BudgetResult {
        original_tokens,
        final_tokens: original_tokens,
        tool_outputs_truncated: 0,
        messages_evicted: 0,
        retention_ratio_bp: 10_000,
        is_cliff: false,
    };

    // Pass 1: 截断超大工具输出 (最省且不丢历史轮次)
    if per_tool_output_tokens > 0 {
        for m in messages.iter_mut() {
            if m.role == Role::Tool && estimate_tokens(&m.content) > per_tool_output_tokens {
                m.content = truncate_preserving(&m.content, per_tool_output_tokens, 0.6);
                result.tool_outputs_truncated += 1;
            }
        }
    }

    // 锚点定位: System 头之后的首条 User 消息 = 任务定义锚点。
    let anchor_idx: Option<usize> = {
        let start = if messages.first().map(|m| m.role) == Some(Role::System) {
            1
        } else {
            0
        };
        messages[start..]
            .iter()
            .position(|m| m.role == Role::User)
            .map(|p| start + p)
    };

    // Pass 2: 超预算则逐条驱逐最旧可弃消息 (跳过 System 首条 / 任务锚点 / 末条)
    loop {
        let total = estimate_messages_tokens(messages);
        result.final_tokens = total;
        if total <= max_tokens || messages.len() <= 2 {
            break;
        }
        let last_idx = messages.len() - 1;
        let mut evict_at: Option<usize> = None;
        for (idx, m) in messages.iter().enumerate() {
            let is_system_head = idx == 0 && m.role == Role::System;
            let is_anchor = Some(idx) == anchor_idx;
            let is_last = idx == last_idx;
            if !is_system_head && !is_anchor && !is_last {
                evict_at = Some(idx);
                break;
            }
        }
        match evict_at {
            Some(idx) => {
                messages.remove(idx);
                result.messages_evicted += 1;
            }
            None => break,
        }
    }
    result.final_tokens = estimate_messages_tokens(messages);

    // W1.1 断崖判定: 大上下文 + 激进压缩 → 标记断崖事件, 调用方据此告警/降级。
    result.retention_ratio_bp = if original_tokens > 0 {
        ((result.final_tokens as u64 * 10_000) / original_tokens as u64) as u32
    } else {
        10_000
    };
    result.is_cliff = original_tokens >= COMPACTION_CLIFF_MIN_TOKENS
        && result.retention_ratio_bp < COMPACTION_CLIFF_RATIO_BP;
    result
}

/// W1.1 (batch3 2026-08-26, 源: arxiv 2608.22752 Compaction Cliff) 验收测试。
#[cfg(test)]
mod compaction_cliff_tests {
    use super::*;

    fn msg(role: Role, text: &str) -> Message {
        Message::new(role, text)
    }

    /// 构造 CJK 长文 (fallback/tiktoken 双口径下都 ≈1 token/字, 口径无关)。
    fn cjk_blob(chars: usize) -> String {
        "压缩断崖研究用长文本。".repeat(chars / 11 + 1)
    }

    #[test]
    fn task_anchor_survives_aggressive_eviction() {
        let mut messages = vec![
            msg(Role::System, "You are a helpful assistant."),
            msg(Role::User, "任务定义锚点：分析季度财报并输出要点"),
            msg(Role::Assistant, &cjk_blob(300)),
            msg(Role::User, &cjk_blob(300)),
            msg(Role::Assistant, &cjk_blob(300)),
            msg(Role::User, "当前请求"),
        ];
        let r = apply_context_budget(&mut messages, 800, 0);
        assert!(r.messages_evicted > 0, "expected evictions");
        // 锚点 (首条 User = 任务定义) 必须存活
        assert!(
            messages.iter().any(|m| m.content.contains("任务定义锚点")),
            "task anchor was evicted!"
        );
        // System 头与末条不变量
        assert_eq!(messages.first().unwrap().role, Role::System);
        assert_eq!(messages.last().unwrap().content, "当前请求");
    }

    #[test]
    fn cliff_flag_fires_on_aggressive_compaction() {
        let mut messages = vec![
            msg(Role::System, "sys"),
            msg(Role::User, "任务定义锚点"),
            msg(Role::Assistant, &cjk_blob(4_000)),
            msg(Role::User, &cjk_blob(4_000)),
            msg(Role::Assistant, &cjk_blob(4_000)),
            msg(Role::User, "当前请求"),
        ];
        let r = apply_context_budget(&mut messages, 3_000, 0);
        assert!(r.original_tokens >= COMPACTION_CLIFF_MIN_TOKENS);
        assert!(r.is_cliff, "retention={}bp", r.retention_ratio_bp);
        assert!(r.retention_ratio() < 0.35);
    }

    #[test]
    fn no_cliff_on_light_compaction() {
        let mut messages = vec![
            msg(Role::System, "sys head"),
            msg(Role::User, "普通短对话"),
            msg(Role::Assistant, "简短回答"),
            msg(Role::User, "当前请求"),
        ];
        let r = apply_context_budget(&mut messages, 50, 0);
        assert!(!r.is_cliff);
        // 微型上下文即使高压缩率也不触发 (MIN_TOKENS 门)
        assert!(r.retention_ratio_bp < COMPACTION_CLIFF_RATIO_BP || r.final_tokens <= 50);
    }

    #[test]
    fn no_cliff_when_budget_not_exceeded() {
        let mut messages = vec![msg(Role::User, &cjk_blob(500)), msg(Role::Assistant, "ok")];
        let before = estimate_messages_tokens(&messages);
        let r = apply_context_budget(&mut messages, before * 10, 0);
        assert!(!r.is_cliff);
        assert_eq!(r.retention_ratio_bp, 10_000);
        assert_eq!(r.messages_evicted, 0);
    }
}
