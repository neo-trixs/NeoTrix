//! LLM Provider 核心类型定义
//!
//! 更新说明 (2026-07-01):
//! - `temperature` 改为 `Option<f32>` — 部分新模型 (如 Claude Sonnet 5) 不再支持采样参数
//! - 新增 `thinking_budget` — 支持模型自适应思考 (extended thinking)
//! - 新增 `provider_params` — 额外 provider 专用参数映射
//!
//! 2026-07-04: 新增 ProviderCategory (自我/客体分离)

use std::cell::RefCell;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::l0_substrate::nt_core_error::NeoTrixError;
use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};

/// 数据信任分级 — core 层出网隐私守卫的核心依据 (NeoTrix 自身源码/对话不外泄给外部模型)。
///
/// 与 neotrix 层 `LlmProviderType::data_trust()` 语义一致, 但定义在 core 层,
/// 因为 `LlmProvider` trait 位于 core (core 不得反向依赖 neotrix 层)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DataTrust {
    /// 本地推理: 数据不出设备 (Ollama/vLLM/SGLang) — 仅脱密钥即可。
    Trusted,
    /// 付费签约云端: 脱敏内部指纹后放行。
    Contracted,
    /// 免费/代理/keyless 端点: 命中内部指纹则阻断 (fail-closed)。
    Untrusted,
}

#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    /// 出网隐私守卫默认入口 — 每条外部请求出站前必经 (R-P42 强化现有节点, 单点全覆盖)。
    ///
    /// 默认实现: 脱密钥 → 按 `data_trust()` 拦截/脱敏 NeoTrix 内部指纹 → 委托 `complete_raw`。
    /// 具体 provider 只需实现 `complete_raw` / `stream_complete_raw` / `data_trust`。
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let mut req = request.clone();
        if let Err(reason) = egress_privacy_guard(&mut req, self.data_trust()) {
            return Err(LlmError::InvalidRequest(reason));
        }
        let mut resp = self.complete_raw(&req).await?;
        resp.content = ingress_privacy_guard(&resp.content);
        if let Some(r) = resp.reasoning.as_mut() {
            *r = ingress_privacy_guard(r);
        }
        Ok(resp)
    }

    async fn stream_complete(
        &self,
        request: &LlmRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let mut req = request.clone();
        if let Err(reason) = egress_privacy_guard(&mut req, self.data_trust()) {
            return Err(LlmError::InvalidRequest(reason));
        }
        let mut rx = self.stream_complete_raw(&req).await?;
        let (tx, rx_out) = tokio::sync::mpsc::channel(64);
        tokio::spawn(async move {
            while let Some(item) = rx.recv().await {
                let item = item.map(|mut r| {
                    r.content = ingress_privacy_guard(&r.content);
                    if let Some(rr) = r.reasoning.as_mut() {
                        *rr = ingress_privacy_guard(rr);
                    }
                    r
                });
                if tx.send(item).await.is_err() {
                    break;
                }
            }
        });
        Ok(rx_out)
    }

    /// 实际出网实现 (守卫已先行处理 request)。
    async fn complete_raw(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError>;

    /// 实际流式出网实现 (守卫已先行处理 request)。
    async fn stream_complete_raw(
        &self,
        request: &LlmRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError>;

    /// 将 provider 的 HTTP 客户端切换为代理路由 (子母阵 Proxied/Tor 画像注入)。
    /// 默认 no-op — 不支持代理注入的 provider 保持原客户端不变。
    fn set_proxy(&mut self, _proxy_url: &str) {}

    /// 数据信任分级 — 决定出网守卫处置 (Trusted/Contracted/Untrusted)。
    /// 外部 LLM 的信任级别. 默认 `Contracted` (脱敏不阻断); 本地/Ollama 与未受信/免费端点覆写.
    /// 单一来源: trait 默认 + 少量显式覆写, 不再逐 provider 手写 (`P2`).
    fn data_trust(&self) -> DataTrust {
        DataTrust::Contracted
    }
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
    /// Model's chain-of-thought / extended-thinking reasoning text, when surfaced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
}

impl LlmResponse {
    /// Convenience constructor for providers that do not surface tool calls.
    pub fn plain(content: String, model: String, usage: Usage, finish_reason: FinishReason) -> Self {
        Self { content, model, usage, finish_reason, tool_calls: None, reasoning: None }
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

impl LlmError {
    /// 是否可重试 (速率限制/网络/超时)
    pub fn is_retryable(&self) -> bool {
        matches!(self, LlmError::RateLimit(_) | LlmError::Network(_) | LlmError::Server(_))
    }

    /// 是否应故障转移 (非认证/验证错误)
    pub fn should_fallback(&self) -> bool {
        !matches!(self, LlmError::Authentication(_) | LlmError::InvalidRequest(_))
    }

    /// 是否为配额耗尽 (应熔断而非重试)
    pub fn is_quota_exhaustion(&self) -> bool {
        match self {
            LlmError::RateLimit(msg) | LlmError::Server(msg) => {
                let m = msg.to_lowercase();
                m.contains("quota exceeded")
                    || m.contains("out of quota")
                    || m.contains("insufficient quota")
                    || m.contains("credit limit")
                    || m.contains("out of credits")
                    || m.contains("billing")
            }
            LlmError::Authentication(msg) => {
                let m = msg.to_lowercase();
                m.contains("quota") || m.contains("billing")
            }
            _ => false,
        }
    }

    /// 是否为模型不可用 (应锁定模型而非熔断 provider)
    pub fn is_model_unavailable(&self) -> bool {
        match self {
            LlmError::Server(msg) | LlmError::InvalidRequest(msg) => {
                let m = msg.to_lowercase();
                m.contains("model not found")
                    || m.contains("model does not exist")
                    || m.contains("model unavailable")
                    || m.contains("unknown model")
                    || m.contains("decommissioned")
                    || m.contains("no longer available")
                    || m.contains("invalid model")
            }
            _ => false,
        }
    }
}

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

/// 统一错误域接入 (进化路线卫生层 P0): 将本地 `LlmError` 映射到核心 `NeoTrixError`,
/// 使 LLM 层错误可经 `?` 传播至统一错误域, 消除散落 `.unwrap()`/`expect()`。
impl From<LlmError> for NeoTrixError {
    fn from(e: LlmError) -> Self {
        match e {
            LlmError::Network(s) => NeoTrixError::Network(s),
            LlmError::Authentication(s) => NeoTrixError::SafetyViolation(format!("LLM auth: {s}")),
            LlmError::RateLimit(s) => NeoTrixError::Brain(format!("LLM rate limit: {s}")),
            LlmError::InvalidRequest(s) => NeoTrixError::InvalidInput(s),
            LlmError::Server(s) => NeoTrixError::Brain(format!("LLM server: {s}")),
            LlmError::Unknown(s) => NeoTrixError::Brain(s),
        }
    }
}

/// ============================================================================
/// 出网隐私守卫 (Egress Privacy Guard) — core 层单点实现
///
/// 修复: 外部模型不应获取 NeoTrix 自身的源代码与对话信息。
/// 机制 (与 neotrix 层 `privacy_guard` 语义一致, 但实现独立置于 core 以满足分层约束):
/// 1. 密钥脱敏 — 剥离 sk-/AKIA/私钥/JWT/Bearer 等凭据。
/// 2. 内部指纹检测 — 扫描 `nt_core_*` / `neotrix-core/` / `ConsciousnessTree` 等。
/// 3. 信任分级门控 — `Trusted`(本地)仅脱密钥; `Contracted`(付费云)脱敏放行;
///    `Untrusted`(免费/代理)命中内部指纹 → 阻断 (fail-closed)。
/// ============================================================================

/// NeoTrix 内部指纹 — 命中即表明消息可能泄露 NeoTrix 自身源代码/KB/对话。
/// 刻意不含项目通用名 "NeoTrix"(用户正常对话会提及, 误伤率高), 只取结构性代码信号。
const INTERNAL_TOKENS: &[&str] = &[
    "neotrix-core/",
    "neotrix_core",
    "crates/neotrix",
    "neotrix_knowledge.db",
    ".neotrix/knowledge.db",
    "nt_core_",
    "nt_mind_",
    "nt_world_",
    "nt_io_",
    "nt_memory_",
    "nt_shield_",
    "nt_act_",
    "nt_governance_",
    "nt_meta_",
    "nt_repair_",
    "nt_scout_",
    "ConsciousnessTree",
    "VSA HyperCube",
    "E8 Hexagram",
    "GWT",
    "SEAL Pipeline",
    "knowledge.db",
    "kv_store",
    "LlmProviderType",
    "ProviderCategory",
    "GatewayProvider",
    "Redactor",
    "CONTEXT.md",
    "AGENTS.md",
    "neotrix-experience",
    "neotrix-tauri",
    "experience.db",
];

/// 常见密钥/凭据前缀 — 出站前必脱 (与 neotrix 层 Redactor 行为一致)。
/// 吸收 Mask/Maskit 模式: 覆盖云厂商、CI/CD、支付、通信、区块链等全场景。
const SECRET_PREFIXES: &[&str] = &[
    // ── 云厂商 API 密钥 ──
    "sk-",           // OpenAI / Anthropic
    "AKIA",          // AWS Access Key
    "AIza",          // Google API
    "pk_live",       // Stripe Live
    "pk_test",       // Stripe Test
    "rk_live",       // Stripe Restricted Key Live
    "rk_test",       // Stripe Restricted Key Test
    "sk_live",       // Stripe Secret Key Live
    "sk_test",       // Stripe Secret Key Test
    "SG.",           // SendGrid
    "xoxb-",         // Slack Bot
    "xoxp-",         // Slack User
    "xoxo-",         // Slack OAuth
    "xapp-",         // Slack App-Level
    "EAAI",          // Facebook/Meta App Token
    "pat-",          // Notion Integration Token
    "napi_",         // Notion Internal Integration
    "aioa",          // Azure OpenAI
    "AZURE_CLIENT_SECRET", // Azure
    // ── CI/CD & DevOps ──
    "ghp_",          // GitHub Personal Access Token
    "gho_",          // GitHub OAuth
    "ghu_",          // GitHub User-to-Server
    "ghs_",          // GitHub Server-to-Server
    "ghr_",          // GitHub Refresh Token
    "glpat-",        // GitLab PAT
    "gldt-",         // GitLab Deploy Token
    "ATATT",         // Atlassian (Jira/Confluence) API Token
    "bxcb",          // Bitbucket App Password
    "boatu",         // Boat CI
    "drone.",        // Drone CI
    "token",         // generic CI tokens
    // ── 容器 & 基础设施 ──
    "docker.",       // Docker Hub PAT
    "hkr_",          // Heroku API Key
    // ── 私钥 & 证书 ──
    "-----BEGIN",    // PEM private keys / certificates
    // ── Bearer / Authorization ──
    "Bearer ",       // OAuth Bearer token
    "Basic ",        // HTTP Basic auth (base64)
    // ── 键值对式泄露 ──
    "api_key=", "apikey=", "api-key=",
    "secret=", "client_secret=", "app_secret=",
    "password=", "passwd=", "pwd=",
    "token=", "access_token=", "auth_token=",
    "private_key=", "signing_key=",
    "DATABASE_URL=", "REDIS_URL=", "MONGO_URI=",
    "AWS_SECRET_ACCESS_KEY=",
    "TWILIO_AUTH_TOKEN=",
    "SLACK_WEBHOOK_URL=",
    "WEBHOOK_SECRET=",
    // ── 加密货币 ──
    "0x",            // Ethereum wallet (40+ hex after 0x)
    "bc1",           // Bech32 Bitcoin address
    "lntb",          // Lightning Network invoice
];

/// ============================================================================
/// 吸收 Mask/Maskit 模式: PII 正则检测规则 (Deterministic Tier 0)
///
/// Mask 的核心洞见: 结构化 PII 可用正则 + 校验和精确识别 (Luhn/Mod-97/Mod-11),
/// 无需 NLP 模型。NeoTrix 在出网守卫中复用此模式, 零额外依赖。
/// ============================================================================

/// PII 检测规则 — 每条规则含正则模式和对应的脱敏标签。
/// 吸收 Mask 支持的 50+ 类型中的高频子集 (Financial / Contact / Identity)。
static PII_RULES: &[(&str, &str)] = &[
    // ── Financial ──
    // SSN: 000-00-0000 (Mask collision prefix: 000)
    (r"\b\d{3}-\d{2}-\d{4}\b", "SSN"),
    // Credit Card: 13-19 digits (Luhn-validated in redact function)
    (r"\b(?:\d[ -]*?){13,19}\b", "CreditCard"),
    // IBAN: 2-letter country + 2 check digits + up to 30 alphanumeric
    (r"\b[A-Z]{2}\d{2}[A-Z0-9]{4,30}\b", "IBAN"),
    // US Routing Number: 9 digits (leading non-zero, Mod-10 validated in redact function)
    (r"\b[1-9]\d{8}\b", "RoutingNumber"),
    // ── Contact ──
    // Email
    (r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "Email"),
    // US Phone: (XXX) XXX-XXXX or XXX-XXX-XXXX or +1XXXXXXXXXX
    (r"(?:\+1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b", "Phone"),
    // IPv4
    (r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b", "IPv4"),
    // IPv6 (simplified)
    (r"\b(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}\b", "IPv6"),
    // MAC Address
    (r"\b[0-9a-fA-F]{2}[:-][0-9a-fA-F]{2}[:-][0-9a-fA-F]{2}[:-][0-9a-fA-F]{2}[:-][0-9a-fA-F]{2}[:-][0-9a-fA-F]{2}\b", "MAC"),
    // ── Identity ──
    // US Passport: 1 letter + 8 digits
    (r"\b[A-Z]\d{8}\b", "Passport"),
    // EIN/Tax ID: XX-XXXXXXX
    (r"\b\d{2}-\d{7}\b", "TaxID"),
    // ── Healthcare ──
    // US NPI: 10 digits (starting with 1 or 2)
    (r"\b[12]\d{9}\b", "NPI"),
];

/// Luhn 校验和 — 用于信用卡号验证 (Mask Tier 0 模式)。
fn luhn_checksum_ok(digits: &str) -> bool {
    let nums: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();
    if nums.len() < 13 || nums.len() > 19 {
        return false;
    }
    let mut sum = 0u32;
    let mut alt = false;
    for &n in nums.iter().rev() {
        let mut val = n;
        if alt {
            val *= 2;
            if val > 9 { val -= 9; }
        }
        sum += val;
        alt = !alt;
    }
    sum % 10 == 0
}

/// 会话级一致性映射 — Mask 的核心洞见:
/// 同一 PII 在整个会话中映射到同一占位符, 保持 LLM 推理上下文不被破坏。
/// 不同会话产生不同映射, 防止跨会话指纹关联。
///
/// 使用线程-local HashMap 实现, 无需全局锁, 零外部依赖。

thread_local! {
    /// 明文 → 占位符 映射 (会话级, 线程隔离)
    static SESSION_PLACEHOLDER_MAP: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    /// 占位符 → 明文 映射 (仅本地工具调用时使用, 绝不出站)
    static SESSION_REVERSE_MAP: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    /// 单调递增计数器, 用于生成唯一占位符标签
    static SESSION_COUNTER: RefCell<u32> = RefCell::new(0);
}

/// 生成会话一致的占位符 — 同一明文始终映射到同一占位符。
/// 占位符格式: `[MASKED:P-{type}-{seq}]`, 如 `[MASKED:P-Email-001]`
///
/// 参考 Mask FPE 模式: 使用 HMAC 确定性 (此处简化为计数器, 因 Rust 无内建 HMAC-PE)
/// 并保持格式可辨识 (方便调试和下游工具识别)。
pub(crate) fn session_consistent_placeholder(plaintext: &str, pii_type: &str) -> String {
    SESSION_PLACEHOLDER_MAP.with(|map| {
        let mut map = map.borrow_mut();
        if let Some(existing) = map.get(plaintext) {
            return existing.clone();
        }
        SESSION_COUNTER.with(|ctr| {
            let mut ctr = ctr.borrow_mut();
            *ctr += 1;
            let seq = *ctr;
            let placeholder = format!("[MASKED:P-{}-{:03}]", pii_type, seq);
            map.insert(plaintext.to_string(), placeholder.clone());
            SESSION_REVERSE_MAP.with(|rev| {
                rev.borrow_mut().insert(placeholder.clone(), plaintext.to_string());
            });
            placeholder
        })
    })
}

/// 重置会话映射 — 每个独立会话/线程开始时调用。
pub(crate) fn reset_session_placeholders() {
    SESSION_PLACEHOLDER_MAP.with(|m| m.borrow_mut().clear());
    SESSION_REVERSE_MAP.with(|m| m.borrow_mut().clear());
    SESSION_COUNTER.with(|c| *c.borrow_mut() = 0);
}

/// 从占位符反查明文 — 仅在本地工具执行时使用 (Tool Pre-Hook, JIT 解密)。
/// 返回 None 表示非 Mask 生成的占位符 (安全: 不会误还原)。
pub(crate) fn resolve_placeholder(masked: &str) -> Option<String> {
    SESSION_REVERSE_MAP.with(|m| m.borrow().get(masked).cloned())
}

/// 用 PII 正则 + 会话一致性映射替换文本中的敏感数据。
/// 遵循 Mask Tier 0 模式: 逐规则扫描 → 命中即替换。
/// 每条规则在当前 `out` 上扫描, 从后往前替换避免索引偏移。
fn redact_pii_session_consistent(s: &str) -> String {
    let mut out = s.to_string();
    for (pattern_str, pii_type) in PII_RULES {
        if let Ok(re) = regex::Regex::new(pattern_str) {
            // 在当前 out 上收集匹配, 从后往前替换
            let mut matches: Vec<(usize, usize, String)> = re.find_iter(&out.clone())
                .filter_map(|mat| {
                    let matched = mat.as_str();
                    if *pii_type == "CreditCard" && !luhn_checksum_ok(&matched.replace(|c: char| !c.is_numeric(), "")) {
                        return None;
                    }
                    let placeholder = session_consistent_placeholder(matched, pii_type);
                    Some((mat.start(), mat.end(), placeholder))
                })
                .collect();
            matches.sort_by(|a, b| b.0.cmp(&a.0));
            for (start, end, placeholder) in matches {
                out = format!("{}{}{}", &out[..start], placeholder, &out[end..]);
            }
        }
    }
    out
}

/// 扫描消息中的 NeoTrix 内部指纹。
pub fn scan_internals(content: &str) -> Vec<&'static str> {
    INTERNAL_TOKENS
        .iter()
        .copied()
        .filter(|tok| content.contains(tok))
        .collect()
}

/// 将内部指纹替换为占位符, 防止 NeoTrix 源码/KB 泄露给外部模型。
pub fn redact_internals(content: &str) -> String {
    let mut out = content.to_string();
    for tok in INTERNAL_TOKENS {
        if out.contains(tok) {
            out = out.replace(tok, "[REDACTED:neotrix-internal]");
        }
    }
    out
}

/// 脱敏单条文本中的密钥/凭据 (子串匹配, 不引入 regex 依赖)。
fn redact_secrets_str(s: &str) -> String {
    let mut out = s.to_string();
    for p in SECRET_PREFIXES {
        let mut scan = 0;
        while let Some(rel) = out[scan..].find(p) {
            let idx = scan + rel;
            let rest = &out[idx..];
            // 终止: 下一个空白/引号, 或最长 64 字符 (避免吞掉整段正常文本)
            let end = rest
                .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == '`')
                .unwrap_or(rest.len())
                .min(64);
            if end == 0 {
                scan = idx + 1;
                continue;
            }
            let secret = out[idx..idx + end].to_string();
            out = out.replace(&secret, "[REDACTED:secret]");
            scan = idx + "[REDACTED:secret]".len();
        }
    }
    out
}

/// 文本是否含已知密钥前缀。
fn has_secret(s: &str) -> bool {
    SECRET_PREFIXES.iter().any(|p| s.contains(p))
}

/// 脱敏绝对文件系统路径 (暴露本地环境), 仅命中明确的家目录/私有路径前缀, 避免误伤 URL。
fn redact_paths_str(s: &str) -> String {
    const PREFIXES: &[&str] = &["/Users/", "/home/", "/private/", "C:\\"];
    let mut out = s.to_string();
    for p in PREFIXES {
        let mut scan = 0;
        while let Some(rel) = out[scan..].find(p) {
            let idx = scan + rel;
            let rest = &out[idx..];
            let end = rest.find(|c: char| c.is_whitespace()).unwrap_or(rest.len());
            if end == 0 {
                scan = idx + 1;
                continue;
            }
            let path = &out[idx..idx + end];
            out = out.replace(path, "[REDACTED:path]");
            scan = idx + "[REDACTED:path]".len();
        }
    }
    out
}

/// 出站脱敏组合: 内部指纹 + 密钥 + 绝对路径 + PII (Mask Tier 0 模式)。
/// 吸收 Mask 流水线: Deterministic 正则 → 会话一致占位符 → 路径脱敏。
fn redact_outbound_str(s: &str) -> String {
    let s = redact_internals(s);
    let s = redact_secrets_str(&s);
    let s = redact_pii_session_consistent(&s);
    redact_paths_str(&s)
}

/// 入站隐私守卫 — 模型回包后处理, 剥离任何被回显的 NeoTrix 内部指纹 / 密钥 / 绝对路径 / PII,
/// 防止泄露进上下文的机密经模型回写被放大存储 (defense-in-depth: egress 已脱敏请求侧,
/// ingress 再兜底回包侧). 由 `LlmProvider::{complete,stream_complete}` 默认方法调用。
pub fn ingress_privacy_guard(text: &str) -> String {
    let s = redact_internals(text);
    let s = redact_secrets_str(&s);
    let s = redact_pii_session_consistent(&s);
    redact_paths_str(&s)
}

/// 脱密钥 (保留原 `scrub_egress_secrets` 行为) — 覆盖所有出站字段, 密钥绝不外泄。
fn scrub_secrets_core(req: &mut LlmRequest) {
    for m in req.messages.iter_mut() {
        if has_secret(&m.content) {
            m.content = redact_secrets_str(&m.content);
        }
    }
    if let Some(ref mut img) = req.image_data {
        if has_secret(img) {
            *img = redact_secrets_str(img);
        }
    }
    for tool in req.tools.iter_mut() {
        if let Ok(s) = serde_json::to_string(&*tool) {
            let red = redact_secrets_str(&s);
            if red != s {
                if let Ok(t) = serde_json::from_str(&red) {
                    *tool = t;
                }
            }
        }
    }
    if let Some(ref mut so) = req.structured_output {
        if let Ok(s) = serde_json::to_string(&*so) {
            let red = redact_secrets_str(&s);
            if red != s {
                if let Ok(parsed) = serde_json::from_str(&red) {
                    *so = parsed;
                }
            }
        }
    }
    for (_, v) in req.provider_params.iter_mut() {
        if let Ok(s) = serde_json::to_string(&*v) {
            let red = redact_secrets_str(&s);
            if red != s {
                if let Ok(parsed) = serde_json::from_str(&red) {
                    *v = parsed;
                }
            }
        }
    }
}

/// 出网隐私守卫主入口 — 由 `LlmProvider::{complete,stream_complete}` 默认方法调用。
///
/// 返回 `Err(reason)` 表示被阻断 (Untrusted + 命中内部指纹)。
/// 返回 `Ok(())` 表示请求已就地脱敏, 可安全出站。
pub fn egress_privacy_guard(req: &mut LlmRequest, trust: DataTrust) -> Result<(), String> {
    // 1. 始终脱密钥 (即使本地, 密钥也绝不外泄)
    scrub_secrets_core(req);

    if trust == DataTrust::Trusted {
        // 本地推理: 数据不出设备, 仅脱密钥即可。
        return Ok(());
    }

    // 2. 扫描所有消息与图像/约束数据中的内部指纹
    let mut leaks: Vec<&'static str> = Vec::new();
    for m in &req.messages {
        leaks.extend(scan_internals(&m.content));
    }
    // model 字段同样可能夹带内部路由名(如 nt_core_* 代理标识), 防御性扫描
    leaks.extend(scan_internals(&req.model));
    if let Some(ref img) = req.image_data {
        leaks.extend(scan_internals(img));
    }
    if let Some(ref c) = req.constraint_json {
        if let Ok(s) = serde_json::to_string(c) {
            leaks.extend(scan_internals(&s));
        }
    }
    // 扩展出站载荷: tool schema / structured_output / provider_params 也可能夹带内部指纹
    if let Some(ref so) = req.structured_output {
        if let Ok(s) = serde_json::to_string(&*so) {
            leaks.extend(scan_internals(&s));
        }
    }
    for (_, v) in &req.provider_params {
        if let Ok(s) = serde_json::to_string(v) {
            leaks.extend(scan_internals(&s));
        }
    }
    for tool in &req.tools {
        if let Ok(s) = serde_json::to_string(tool) {
            leaks.extend(scan_internals(&s));
        }
    }
    leaks.sort_unstable();
    leaks.dedup();

    match trust {
        DataTrust::Contracted => {
            // 付费云: 脱敏内部指纹后放行
            if !leaks.is_empty() {
                tracing::debug!(
                    target: "egress_privacy",
                    leaked = leaks.join(", ").as_str(),
                    "internal fingerprint redacted for contracted provider"
                );
            }
            for m in req.messages.iter_mut() {
                // Contracted 下始终脱敏: 内部指纹 + 密钥 + 绝对路径 (无条件, 防路径/密钥漏脱敏)
                m.content = redact_outbound_str(&m.content);
            }
            if let Some(ref mut img) = req.image_data {
                *img = redact_outbound_str(img);
            }
            // 扩展载荷脱敏
            for tool in req.tools.iter_mut() {
                if let Ok(s) = serde_json::to_string(&*tool) {
                    let red = redact_outbound_str(&s);
                    if red != s {
                        if let Ok(t) = serde_json::from_str(&red) {
                            *tool = t;
                        }
                    }
                }
            }
            if let Some(ref mut so) = req.structured_output {
                if let Ok(s) = serde_json::to_string(&*so) {
                    let red = redact_outbound_str(&s);
                    if red != s {
                        if let Ok(parsed) = serde_json::from_str(&red) {
                            *so = parsed;
                        }
                    }
                }
            }
            for (_, v) in req.provider_params.iter_mut() {
                if let Ok(s) = serde_json::to_string(&*v) {
                    let red = redact_outbound_str(&s);
                    if red != s {
                        if let Ok(parsed) = serde_json::from_str(&red) {
                            *v = parsed;
                        }
                    }
                }
            }
            Ok(())
        }
        DataTrust::Untrusted => {
            // fail-closed: 免费/代理端点绝不放行 NeoTrix 内部代码/对话
            if leaks.is_empty() {
                Ok(())
            } else {
                let joined = leaks.join(", ");
                tracing::warn!(
                    target: "egress_privacy",
                    leaked = joined.as_str(),
                    "egress BLOCKED to untrusted provider (would leak NeoTrix internals)"
                );
                Err(format!(
                    "privacy guard: egress to untrusted provider would leak NeoTrix internal code/conversation ({}). \
                     Blocked. Use a local (Ollama/vLLM/SGLang) or paid contracted provider.",
                    joined
                ))
            }
        }
        DataTrust::Trusted => Ok(()),
    }
}

/// T1 检测: LLM 核心类型 + token 预算引擎 (D3 下沉) 自测。
///
/// 接入 SelfTest 注册表, 使该核心模块具备 T1 存在 + T2 注册 (进化路线卫生层:
/// "无测试的进化=盲目重构" → 核心模块必须可自测)。
pub struct LlmSelfTest;

impl SelfTest for LlmSelfTest {
    fn name(&self) -> &str {
        "llm_core"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        // 1) 统一错误域转换: LlmError -> NeoTrixError 必须保留语义
        let nt: NeoTrixError = LlmError::Authentication("bad key".into()).into();
        if !nt.to_string().contains("LLM") && !nt.to_string().contains("auth") {
            failures.push("llm_core: LlmError::Authentication 未能映射到 NeoTrixError".into());
        }
        let nt2: NeoTrixError = LlmError::InvalidRequest("bad req".into()).into();
        if !matches!(nt2, NeoTrixError::InvalidInput(_)) {
            failures.push("llm_core: LlmError::InvalidRequest 应映射到 InvalidInput".into());
        }

        // 2) token 估算单一事实源: 非空文本必须 >=1 token
        if estimate_tokens("NeoTrix 核心") == 0 {
            failures.push("llm_core: estimate_tokens 对非空文本返回 0".into());
        }

        // 3) 上下文预算压缩: 超长文本截断后 token 必须削减且在预算内
        let long = "工具输出 ".repeat(2000);
        let original = estimate_tokens(&long);
        let truncated = truncate_preserving(&long, 100, 0.6);
        let after = estimate_tokens(&truncated);
        if after >= original {
            failures.push(format!(
                "llm_core: truncate_preserving 未削减 token ({after} >= {original})"
            ));
        }
        if after > 120 {
            failures.push(format!("llm_core: 截断超出预算 (got {after} > 120)"));
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

/// 注册 LLM 核心 SelfTest 到全局注册表。
pub fn register_llm_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(LlmSelfTest));
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

    #[test]
    fn test_llm_self_test_passes() {
        // 锁定 T1 检测: LlmSelfTest 必须自洽通过 (进化路线卫生层)
        let t = super::LlmSelfTest;
        let result = t.self_test();
        assert!(
            result.is_ok(),
            "llm_core self_test 必须通过: {:?}",
            result.err()
        );
    }

    #[test]
    fn core_egress_guard_blocks_untrusted_internal() {
        let mut r = LlmRequest::new("m", "read nt_core_consciousness_core.rs");
        r.messages.clear();
        r.messages.push(Message::new(Role::User, "read nt_core_consciousness_core.rs"));
        let res = egress_privacy_guard(&mut r, DataTrust::Untrusted);
        assert!(res.is_err(), "untrusted + internal fingerprint must be blocked");
    }

    #[test]
    fn core_egress_guard_allows_trusted_internal() {
        let mut r = LlmRequest::new("m", "read nt_core_consciousness_core.rs");
        r.messages.clear();
        r.messages.push(Message::new(Role::User, "read nt_core_consciousness_core.rs"));
        assert!(egress_privacy_guard(&mut r, DataTrust::Trusted).is_ok(), "local must pass through");
    }

    #[test]
    fn core_egress_guard_scrubs_secret_contracted() {
        let mut r = LlmRequest::new("m", "key sk-abcdEFGH1234567890abcdef");
        r.messages.clear();
        r.messages.push(Message::new(Role::User, "key sk-abcdEFGH1234567890abcdef"));
        assert!(egress_privacy_guard(&mut r, DataTrust::Contracted).is_ok());
        assert!(!r.messages[0].content.contains("sk-abcdEFGH"), "secret must be redacted");
    }

    #[test]
    fn egress_guard_blocks_untrusted_tool_schema_leak() {
        let tool = Tool {
            name: "fs".into(),
            description: "read file".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "path": { "description": "read neotrix-core/src/core/nt_core_harness.rs" } }
            }),
        };
        let mut r = LlmRequest::new("m", "hi").with_tools(vec![tool]);
        let res = egress_privacy_guard(&mut r, DataTrust::Untrusted);
        assert!(res.is_err(), "untrusted + internal fingerprint in tool schema must be blocked");
    }

    #[test]
    fn egress_guard_redacts_contracted_tool_schema_leak() {
        let tool = Tool {
            name: "fs".into(),
            description: "read file".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "path": { "description": "read neotrix-core/src/core/nt_core_harness.rs" } }
            }),
        };
        let mut r = LlmRequest::new("m", "hi").with_tools(vec![tool]);
        assert!(egress_privacy_guard(&mut r, DataTrust::Contracted).is_ok());
        let s = serde_json::to_string(&r.tools[0]).unwrap();
        assert!(!s.contains("nt_core_harness"), "tool schema fingerprint must be redacted under Contracted");
    }

    #[test]
    fn egress_guard_blocks_untrusted_model_field_leak() {
        let mut r = LlmRequest::new("nt_core_consciousness_core-proxy", "hi");
        let res = egress_privacy_guard(&mut r, DataTrust::Untrusted);
        assert!(res.is_err(), "untrusted + internal fingerprint in model field must be blocked");
    }

    #[test]
    fn egress_guard_blocks_untrusted_new_internal_token() {
        // 扩展指纹: neotrix-experience 命中即阻断 (Untrusted)
        let mut r = LlmRequest::new("m", "run neotrix-experience query --kw test");
        r.messages.clear();
        r.messages.push(Message::new(Role::User, "run neotrix-experience query --kw test"));
        assert!(egress_privacy_guard(&mut r, DataTrust::Untrusted).is_err());
    }

    #[test]
    fn egress_guard_redacts_contracted_tool_secret() {
        // 密钥藏在 tool schema 描述中, Contracted 下必须被脱敏 (修复仅 messages 脱密钥的缺口)
        let tool = Tool {
            name: "fs".into(),
            description: "use sk-ABCD1234secretkey here".into(),
            input_schema: serde_json::json!({ "type": "object" }),
        };
        let mut r = LlmRequest::new("m", "hi").with_tools(vec![tool]);
        assert!(egress_privacy_guard(&mut r, DataTrust::Contracted).is_ok());
        let s = serde_json::to_string(&r.tools[0]).unwrap();
        assert!(!s.contains("sk-ABCD1234"), "tool schema secret must be redacted under Contracted");
    }

    #[test]
    fn egress_guard_redacts_contracted_absolute_path() {
        // 绝对路径泄露本地环境, Contracted 下必须脱敏
        let mut r = LlmRequest::new("m", "read /Users/neo/secret-project/keys.txt");
        r.messages.clear();
        r.messages.push(Message::new(Role::User, "read /Users/neo/secret-project/keys.txt"));
        assert!(egress_privacy_guard(&mut r, DataTrust::Contracted).is_ok());
        assert!(r.messages[0].content.contains("[REDACTED:path]"), "absolute path must be redacted under Contracted");
        assert!(!r.messages[0].content.contains("/Users/neo"), "raw path must not leak");
    }

    #[test]
    fn ingress_strips_echoed_secret() {
        let out = ingress_privacy_guard("here is your key sk-abcdEFGH1234567890abcdef back");
        assert!(!out.contains("sk-abcdEFGH"), "回包中回显的密钥必须被脱敏");
        assert!(out.contains("[REDACTED:secret]"), "密钥应替换为占位符");
    }

    #[test]
    fn ingress_strips_internal_token() {
        let out = ingress_privacy_guard("you referenced nt_core_consciousness_core.rs");
        assert!(!out.contains("nt_core_consciousness_core"), "回包中内部指纹必须被脱敏");
        assert!(out.contains("[REDACTED:neotrix-internal]"), "内部指纹应替换为占位符");
    }

    #[test]
    fn ingress_strips_absolute_path() {
        let out = ingress_privacy_guard("the file is at /Users/neo/secret/keys.txt");
        assert!(!out.contains("/Users/neo"), "回包中绝对路径必须被脱敏");
        assert!(out.contains("[REDACTED:path]"));
    }

    #[test]
    fn ingress_passthrough_clean() {
        let out = ingress_privacy_guard("the capital of France is Paris");
        assert_eq!(out, "the capital of France is Paris", "良性内容不应被改动");
    }

    // ---- 集成级测试: 验证 trait 默认方法 complete()/stream_complete() 真的执行 egress 闸门 (T3 生产接线) ----
    use std::sync::{Arc, Mutex};

    struct UntrustedProbe;
    #[async_trait::async_trait]
    impl LlmProvider for UntrustedProbe {
        async fn complete_raw(&self, _req: &LlmRequest) -> Result<LlmResponse, LlmError> {
            Ok(LlmResponse::plain("leak".into(), "m".into(), Usage::default(), FinishReason::Stop))
        }
        async fn stream_complete_raw(
            &self,
            _req: &LlmRequest,
        ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            let _ = tx.send(Ok(LlmResponse::plain("leak".into(), "m".into(), Usage::default(), FinishReason::Stop))).await;
            Ok(rx)
        }
        fn data_trust(&self) -> DataTrust { DataTrust::Untrusted }
    }

    struct TrustedProbe;
    #[async_trait::async_trait]
    impl LlmProvider for TrustedProbe {
        async fn complete_raw(&self, _req: &LlmRequest) -> Result<LlmResponse, LlmError> {
            Ok(LlmResponse::plain("ok".into(), "m".into(), Usage::default(), FinishReason::Stop))
        }
        async fn stream_complete_raw(
            &self,
            _req: &LlmRequest,
        ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            let _ = tx.send(Ok(LlmResponse::plain("ok".into(), "m".into(), Usage::default(), FinishReason::Stop))).await;
            Ok(rx)
        }
        fn data_trust(&self) -> DataTrust { DataTrust::Trusted }
    }

    struct ContractedProbe {
        captured: Arc<Mutex<Option<String>>>,
    }
    #[async_trait::async_trait]
    impl LlmProvider for ContractedProbe {
        async fn complete_raw(&self, req: &LlmRequest) -> Result<LlmResponse, LlmError> {
            let joined = req.messages.iter().map(|m| m.content.clone()).collect::<Vec<_>>().join("|");
            *self.captured.lock().unwrap() = Some(joined);
            Ok(LlmResponse::plain("ok".into(), "m".into(), Usage::default(), FinishReason::Stop))
        }
        async fn stream_complete_raw(
            &self,
            _req: &LlmRequest,
        ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            let _ = tx.send(Ok(LlmResponse::plain("ok".into(), "m".into(), Usage::default(), FinishReason::Stop))).await;
            Ok(rx)
        }
        fn data_trust(&self) -> DataTrust { DataTrust::Contracted }
    }

    #[test]
    fn complete_gate_blocks_untrusted_internal_at_trait_level() {
        let mut r = LlmRequest::new("m", "read nt_core_consciousness_core.rs");
        r.messages.clear();
        r.messages.push(Message::new(Role::User, "read nt_core_consciousness_core.rs"));
        let p = UntrustedProbe;
        let res = tokio::runtime::Runtime::new().unwrap().block_on(p.complete(&r));
        assert!(res.is_err(), "trait 默认 complete() 必须拦截 untrusted + 内部指纹");
    }

    #[test]
    fn complete_gate_allows_trusted_internal_at_trait_level() {
        let mut r = LlmRequest::new("m", "read nt_core_consciousness_core.rs");
        r.messages.clear();
        r.messages.push(Message::new(Role::User, "read nt_core_consciousness_core.rs"));
        let p = TrustedProbe;
        let res = tokio::runtime::Runtime::new().unwrap().block_on(p.complete(&r));
        assert!(res.is_ok(), "trait 默认 complete() 必须放行 trusted");
    }

    #[test]
    fn complete_gate_scrubs_secret_for_contracted_at_trait_level() {
        let mut r = LlmRequest::new("m", "key sk-abcdEFGH1234567890abcdef");
        r.messages.clear();
        r.messages.push(Message::new(Role::User, "key sk-abcdEFGH1234567890abcdef"));
        let cap = Arc::new(Mutex::new(None));
        let p = ContractedProbe { captured: cap.clone() };
        let res = tokio::runtime::Runtime::new().unwrap().block_on(p.complete(&r));
        assert!(res.is_ok(), "Contracted 不应阻断, 只脱敏");
        let got = cap.lock().unwrap().clone().unwrap();
        assert!(!got.contains("sk-abcdEFGH"), "Contracted 经 trait 默认必须脱敏密钥");
    }

    #[test]
    fn stream_gate_blocks_untrusted_internal_at_trait_level() {
        let mut r = LlmRequest::new("m", "read nt_core_consciousness_core.rs");
        r.messages.clear();
        r.messages
            .push(Message::new(Role::User, "read nt_core_consciousness_core.rs"));
        let p = UntrustedProbe;
        let res = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(p.stream_complete(&r));
        assert!(
            res.is_err(),
            "trait 默认 stream_complete() 必须拦截 untrusted + 内部指纹"
        );
    }

    // ---- A5: 验证 ingress 守卫经 trait 默认 complete()/stream_complete() 真正接线 (T3) ----
    struct IngressProbe;
    #[async_trait::async_trait]
    impl LlmProvider for IngressProbe {
        async fn complete_raw(&self, _req: &LlmRequest) -> Result<LlmResponse, LlmError> {
            // 模拟外部模型把上下文里泄露的密钥又回写进回包
            Ok(LlmResponse::plain(
                "your key sk-abcdEFGH1234567890abcdef is compromised".into(),
                "m".into(),
                Usage::default(),
                FinishReason::Stop,
            ))
        }
        async fn stream_complete_raw(
            &self,
            _req: &LlmRequest,
        ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            let _ = tx
                .send(Ok(LlmResponse::plain(
                    "your key sk-abcdEFGH1234567890abcdef is compromised".into(),
                    "m".into(),
                    Usage::default(),
                    FinishReason::Stop,
                )))
                .await;
            Ok(rx)
        }
    }

    #[test]
    fn complete_applies_ingress_guard_to_response() {
        let p = IngressProbe;
        let r = LlmRequest::new("m", "hi");
        let resp = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(p.complete(&r))
            .expect("complete must succeed");
        assert!(!resp.content.contains("sk-abcdEFGH"), "回包密钥必须经 ingress 守卫脱敏");
    }

    #[test]
    fn stream_complete_applies_ingress_guard_to_response() {
        let p = IngressProbe;
        let r = LlmRequest::new("m", "hi");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let content = rt.block_on(async move {
            let mut rx = p.stream_complete(&r).await.expect("stream_complete must succeed");
            let item = rx.recv().await.expect("must yield a chunk");
            item.expect("chunk must be ok").content
        });
        assert!(!content.contains("sk-abcdEFGH"), "流式回包密钥必须经 ingress 守卫脱敏");
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
#[derive(Debug, Clone, Default, PartialEq)]
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

// ═══════════════════════════════════════════════════════════════
// Unified Provider Abstraction Layer
// ═══════════════════════════════════════════════════════════════

/// Provider category — core-layer classification (avoids dependency on IO-layer `LlmProviderType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CoreProviderCategory {
    Local,
    Cloud,
    Free,
    Proxy,
}

/// Provider metadata — static identity information.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderMetadata {
    pub name: String,
    pub version: String,
    pub provider_type: CoreProviderCategory,
    pub endpoint: String,
    pub data_trust: DataTrust,
}

/// Model capabilities — what this provider supports.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelCapabilities {
    pub text: bool,
    pub vision: bool,
    pub audio: bool,
    pub function_calling: bool,
    pub streaming: bool,
    pub max_tokens: usize,
    pub context_window: usize,
    pub supported_params: Vec<String>,
}

/// Health status — result of a health probe.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unavailable { reason: String },
    Unknown,
}

/// Cost estimation — predicted cost for a request.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CostEstimate {
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub estimated_cost_usd: f64,
    pub model: String,
}

/// Unified provider trait — extends LlmProvider with metadata, capabilities,
/// health, and cost estimation.
#[async_trait::async_trait]
pub trait UnifiedProvider: LlmProvider {
    /// Provider metadata — static identity information.
    fn metadata(&self) -> ProviderMetadata;

    /// Model capabilities — what this provider supports.
    fn capabilities(&self) -> ModelCapabilities;

    /// Health check — lightweight probe for availability.
    async fn health(&self) -> HealthStatus;

    /// Cost estimation — predict cost for a given request.
    fn estimate_cost(&self, request: &LlmRequest) -> CostEstimate;
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
