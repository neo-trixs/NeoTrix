//! 请求边 payload 塑形钩子 (ProviderPayloadTransform) — dscode 吸收 B-4
//!
//! 吸收来源 (notes/absorption-dscode-1.md 条目 1 + Q2, 2026-08-13):
//! - deepseek.ts:9-53   请求前单一纯函数变换点 `optimizeDeepSeekResponsesPayload`
//! - deepseek.ts:20-23  删除不支持的 OpenAI 字段 (prompt_cache_key/prompt_cache_retention/prompt_cache_options/include)
//! - deepseek.ts:24-27  reasoning 收窄为 `{effort}`, 连带删除 temperature/top_p (thinking 下采样无效)
//! - deepseek.ts:28-30  apply_patch 从 function tool 改写为 `type:"custom"`
//! - deepseek.ts:31-33  按开关服务端注入 web_search tool
//! - model.ts:22-46     Model.compat 旗标契约 (supportsDeveloperRole/supportsLongCacheRetention/
//!                      supportsStrictMode/supportsOpenAIGrammarTools/requiresReasoningContentOnAssistantMessages/
//!                      thinkingFormat/sessionAffinityFormat)
//! - dscode-extension.ts:224-227 挂载点 `pi.on("before_provider_request")`, 条件 provider=="deepseek" && transport=="responses"
//!
//! 接口边界 (吸收 Q2, absorption-dscode-1.md:398-401): Model.compat + 一个纯函数 + 注册表数据,
//! 无任何 provider 特判散落业务代码。
//!
//! 接线点 (BLOCKED — 并行会话已改 nt_io_provider/mod.rs / gateway.rs):
//! 生产接线应在 `nt_io_provider/gateway.rs::call_provider` (行 919 `provider.complete(&req)` 之前)
//! 与 `gateway.rs::call_provider_stream` (行 1446 `provider.stream_complete(&req)` 之前) 调用
//! `PayloadTransformRegistry::apply(provider_id, &mut request)`; 模块声明应迁移到
//! `nt_io_provider/mod.rs` 的 `pub mod payload_transform;`。当前临时接线在 rate_limiter.rs。
#![allow(dead_code)] // 临时接线: 模块已编译但未进生产请求路径 (合并 mod.rs 后可移除)

use crate::neotrix::nt_io_provider::types::{LlmRequest, Tool};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

/// OpenAI-only 参数名 — deepseek.ts:20-23 删除的不支持字段。
pub const OPENAI_ONLY_PARAMS: [&str; 4] = [
    "prompt_cache_key",
    "prompt_cache_retention",
    "prompt_cache_options",
    "include",
];

/// `apply_patch` 工具名 — deepseek.ts:28-30 改写为 custom tool 的对象。
pub const APPLY_PATCH_TOOL: &str = "apply_patch";

/// web_search 注入工具名 — deepseek.ts:31-33。
pub const WEB_SEARCH_TOOL: &str = "web_search";

/// provider_params 内 custom-tool kind 标记 key。
/// NeoTrix `Tool` 结构体无 wire 级 `type` 字段 (types.rs:223-227), 请求边以该 key 记录
/// 应序列化为 `type:"custom"` 的工具名数组; provider `build_body` 序列化时消费之。
pub const CUSTOM_TOOL_KINDS_KEY: &str = "_custom_tool_kinds";

/// Model.compat 旗标契约 — model.ts:22-46 (responses compat) + model.ts:41-46 (chat compat)。
///
/// 通用请求层据此决定行为, 不写 provider 特判 (吸收 Q2 接口边界)。
#[derive(Debug, Clone, Default)]
pub struct ProviderCompat {
    /// supportsDeveloperRole (model.ts:22)
    pub supports_developer_role: bool,
    /// supportsLongCacheRetention: false for deepseek (model.ts:23)
    pub supports_long_cache_retention: bool,
    /// supportsStrictMode: false for deepseek (model.ts:24)
    pub supports_strict_mode: bool,
    /// supportsOpenAIGrammarTools (model.ts:25)
    pub supports_openai_grammar_tools: bool,
    /// requiresReasoningContentOnAssistantMessages (model.ts:41)
    pub requires_reasoning_content_on_assistant_messages: bool,
    /// thinkingFormat: "deepseek" (model.ts:42)
    pub thinking_format: Option<String>,
    /// sessionAffinityFormat: "openai-nosession" (model.ts:27)
    pub session_affinity_format: Option<String>,
}

/// DeepSeek responses compat 旗标 — model.ts:22-28 + 41-46。
pub fn deepseek_compat() -> ProviderCompat {
    ProviderCompat {
        supports_developer_role: true,
        supports_long_cache_retention: false,
        supports_strict_mode: false,
        supports_openai_grammar_tools: false,
        requires_reasoning_content_on_assistant_messages: true,
        thinking_format: Some("deepseek".to_string()),
        session_affinity_format: Some("openai-nosession".to_string()),
    }
}

/// 变换选项 — deepseek.ts:31-33 `webSearch` 注入开关。
#[derive(Debug, Clone, Default)]
pub struct PayloadTransformOptions {
    pub web_search: bool,
}

/// ProviderPayloadTransform — 请求边单一纯函数变换点 (deepseek.ts:9-53)。
///
/// 语义与 dscode `optimizeDeepSeekResponsesPayload(payload, options)` 对齐:
/// 一个 provider 一条纯函数, 由注册表在请求发出前统一调用。
pub trait ProviderPayloadTransform: Send + Sync {
    fn provider_id(&self) -> &str;
    fn compat(&self) -> &ProviderCompat;
    fn transform(&self, request: &mut LlmRequest);
}

/// 按 provider_id 注册的变换注册表 (dscode-extension.ts:224-227 挂载点语义)。
pub struct PayloadTransformRegistry {
    transforms: RwLock<HashMap<String, Box<dyn ProviderPayloadTransform>>>,
}

impl PayloadTransformRegistry {
    pub fn new() -> Self {
        Self {
            transforms: RwLock::new(HashMap::new()),
        }
    }

    /// 注册变换, 以 `transform.provider_id()` 为键。
    pub fn register(&self, transform: Box<dyn ProviderPayloadTransform>) {
        let id = transform.provider_id().to_string();
        let mut guard = self
            .transforms
            .write()
            .unwrap_or_else(|e| e.into_inner());
        guard.insert(id, transform);
    }

    /// 是否存在该 provider 的变换。
    pub fn has(&self, provider_id: &str) -> bool {
        let guard = self
            .transforms
            .read()
            .unwrap_or_else(|e| e.into_inner());
        guard.contains_key(provider_id)
    }

    /// 查询 provider 的 compat 旗标 (consumer: nt_core_self 模型选择路由)。
    pub fn compat(&self, provider_id: &str) -> Option<ProviderCompat> {
        let guard = self
            .transforms
            .read()
            .unwrap_or_else(|e| e.into_inner());
        guard.get(provider_id).map(|t| t.compat().clone())
    }

    /// 应用变换。返回是否命中注册表 (请求是否被塑形)。
    pub fn apply(&self, provider_id: &str, request: &mut LlmRequest) -> bool {
        let guard = self
            .transforms
            .read()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(t) = guard.get(provider_id) {
            t.transform(request);
            true
        } else {
            false
        }
    }
}

impl Default for PayloadTransformRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// DeepSeek responses 变换示例 — deepseek.ts:20-33 四步塑形。
#[derive(Debug, Clone)]
pub struct DeepSeekResponsesTransform {
    pub compat: ProviderCompat,
    pub web_search: bool,
}

impl Default for DeepSeekResponsesTransform {
    fn default() -> Self {
        Self {
            compat: deepseek_compat(),
            web_search: false,
        }
    }
}

impl ProviderPayloadTransform for DeepSeekResponsesTransform {
    fn provider_id(&self) -> &str {
        "deepseek"
    }

    fn compat(&self) -> &ProviderCompat {
        &self.compat
    }

    fn transform(&self, request: &mut LlmRequest) {
        strip_openai_only_params(request);
        narrow_reasoning(request);
        rewrite_apply_patch_as_custom(request);
        inject_web_search(request, self.web_search);
    }
}

/// 进程内全局注册表 (dscode 单一挂载点语义)。
static GLOBAL_REGISTRY: LazyLock<PayloadTransformRegistry> =
    LazyLock::new(PayloadTransformRegistry::new);

/// 全局注册表引用。
pub fn global_registry() -> &'static PayloadTransformRegistry {
    &GLOBAL_REGISTRY
}

/// 在全局注册表注册默认 DeepSeek 变换。
pub fn register_deepseek_default(web_search: bool) {
    GLOBAL_REGISTRY.register(Box::new(DeepSeekResponsesTransform {
        compat: deepseek_compat(),
        web_search,
    }));
}

/// 请求边统一变换入口 — 生产接线后由 gateway.rs::call_provider / call_provider_stream 调用。
pub fn apply_payload_transform(provider_id: &str, request: &mut LlmRequest) -> bool {
    GLOBAL_REGISTRY.apply(provider_id, request)
}

// ---------------------------------------------------------------------------
// 纯函数变换原语 (deepseek.ts 各步的 typed 等价, 可单测)
// ---------------------------------------------------------------------------

/// 删除不支持的 OpenAI 字段 — deepseek.ts:20-23。
pub fn strip_openai_only_params(request: &mut LlmRequest) {
    for key in OPENAI_ONLY_PARAMS {
        request.provider_params.remove(key);
    }
}

/// reasoning 收窄 — deepseek.ts:24-27。
///
/// typed 等价: 思考开启 (thinking_budget > 0) 时删除 temperature/top_p/top_k
/// (thinking 下采样无效, 保留采样参数会导致 DeepSeek 拒绝/忽略)。
pub fn narrow_reasoning(request: &mut LlmRequest) {
    let thinking_active = request.thinking_budget.map_or(false, |b| b > 0);
    if !thinking_active {
        return;
    }
    request.temperature = None;
    request.provider_params.remove("top_p");
    request.provider_params.remove("top_k");
}

/// apply_patch 改写为 custom tool — deepseek.ts:28-30。
///
/// typed 等价: 在 provider_params[CUSTOM_TOOL_KINDS_KEY] 记录 custom kind,
/// 供 provider 序列化时输出 `type:"custom"`。
pub fn rewrite_apply_patch_as_custom(request: &mut LlmRequest) {
    if !has_tool(request, APPLY_PATCH_TOOL) {
        return;
    }
    let mut kinds = request
        .provider_params
        .get(CUSTOM_TOOL_KINDS_KEY)
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if !kinds.iter().any(|v| v.as_str() == Some(APPLY_PATCH_TOOL)) {
        kinds.push(Value::String(APPLY_PATCH_TOOL.to_string()));
    }
    request
        .provider_params
        .insert(CUSTOM_TOOL_KINDS_KEY.to_string(), Value::Array(kinds));
}

/// web_search 注入 — deepseek.ts:31-33。
///
/// 仅当开关开启且尚无 web_search 工具时注入 `{type:"web_search"}`。
pub fn inject_web_search(request: &mut LlmRequest, enabled: bool) {
    if !enabled || has_tool(request, WEB_SEARCH_TOOL) {
        return;
    }
    request.tools.push(Tool {
        name: WEB_SEARCH_TOOL.to_string(),
        description: "Perform a web search and return up-to-date results.".to_string(),
        input_schema: serde_json::json!({}),
    });
}

/// 请求是否包含同名工具。
pub fn has_tool(request: &LlmRequest, name: &str) -> bool {
    request.tools.iter().any(|t| t.name == name)
}

/// 请求中某工具是否已标记为 custom kind。
pub fn is_custom_tool(request: &LlmRequest, name: &str) -> bool {
    request
        .provider_params
        .get(CUSTOM_TOOL_KINDS_KEY)
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().any(|v| v.as_str() == Some(name)))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deepseek_request() -> LlmRequest {
        let mut req = LlmRequest::new("deepseek/deepseek-chat", "hi");
        req.provider_params.insert(
            "prompt_cache_key".to_string(),
            Value::String("k".to_string()),
        );
        req.provider_params
            .insert("prompt_cache_retention".to_string(), Value::from(3600));
        req.provider_params.insert("include".to_string(), Value::Null);
        req.temperature = Some(0.3);
        req.provider_params
            .insert("top_p".to_string(), Value::from(0.9));
        req.thinking_budget = Some(2048);
        req.tools.push(Tool {
            name: APPLY_PATCH_TOOL.to_string(),
            description: "apply patch".to_string(),
            input_schema: serde_json::json!({}),
        });
        req
    }

    #[test]
    fn payload_transform_strips_openai_only_params() {
        let mut req = deepseek_request();
        strip_openai_only_params(&mut req);
        for key in OPENAI_ONLY_PARAMS {
            assert!(
                !req.provider_params.contains_key(key),
                "key {key} must be removed"
            );
        }
        // 无关参数保留
        assert!(req.provider_params.contains_key("top_p"));
    }

    #[test]
    fn payload_transform_narrows_reasoning_strips_sampling() {
        let mut req = deepseek_request();
        narrow_reasoning(&mut req);
        assert_eq!(req.temperature, None, "thinking 下采样无效, temperature 必须删除");
        assert!(!req.provider_params.contains_key("top_p"));
        assert!(!req.provider_params.contains_key("top_k"));
    }

    #[test]
    fn payload_transform_narrow_reasoning_noop_when_thinking_off() {
        let mut req = LlmRequest::new("deepseek/deepseek-chat", "hi");
        req.temperature = Some(0.7);
        req.provider_params
            .insert("top_p".to_string(), Value::from(0.9));
        narrow_reasoning(&mut req);
        assert_eq!(req.temperature, Some(0.7), "思考关闭时采样参数必须保留");
    }

    #[test]
    fn payload_transform_rewrites_apply_patch_as_custom() {
        let mut req = deepseek_request();
        rewrite_apply_patch_as_custom(&mut req);
        assert!(is_custom_tool(&req, APPLY_PATCH_TOOL));
        assert!(!is_custom_tool(&req, "other_tool"));
    }

    #[test]
    fn payload_transform_injects_web_search_once() {
        let mut req = deepseek_request();
        inject_web_search(&mut req, true);
        assert!(has_tool(&req, WEB_SEARCH_TOOL));
        inject_web_search(&mut req, true);
        let count = req.tools.iter().filter(|t| t.name == WEB_SEARCH_TOOL).count();
        assert_eq!(count, 1, "web_search 只能注入一次");
    }

    #[test]
    fn payload_transform_registry_applies_by_provider_id() {
        let registry = PayloadTransformRegistry::new();
        registry.register(Box::new(DeepSeekResponsesTransform {
            compat: deepseek_compat(),
            web_search: true,
        }));
        assert!(registry.has("deepseek"));
        assert!(!registry.has("anthropic"));

        let mut req = deepseek_request();
        let applied = registry.apply("deepseek", &mut req);
        assert!(applied, "deepseek 变换必须命中注册表");
        assert!(!req.provider_params.contains_key("prompt_cache_key"));
        assert_eq!(req.temperature, None);
        assert!(is_custom_tool(&req, APPLY_PATCH_TOOL));
        assert!(has_tool(&req, WEB_SEARCH_TOOL));

        let compat = registry.compat("deepseek").expect("compat present");
        assert_eq!(compat.session_affinity_format.as_deref(), Some("openai-nosession"));
        assert!(!compat.supports_long_cache_retention);
    }

    #[test]
    fn payload_transform_registry_noop_for_unknown_provider() {
        let registry = PayloadTransformRegistry::new();
        let mut req = LlmRequest::new("anthropic/claude", "hi");
        assert!(!registry.apply("anthropic", &mut req));
    }
}
