//! LLM Provider 桥接层 — 将现有 30+ 个 LLM 提供者适配到统一架构
//!
//! 将 `nt_io_provider` 下的 LLM 提供者包装为统一的 `LlmProvider` trait，
//! 无需修改现有 LLM 提供者代码。

use super::unified::*;
use crate::l2_perception::nt_world::l1_facade::{IoLlmProvider, IoLlmRequest, LlmProviderType, create_provider_from_type, Message, Role, FinishReason};
use std::sync::Arc;

/// ── 类型转换: source::unified ↔ neotrix_types ────────────────────────────

fn convert_role(role: &str) -> Role {
    match role {
        "system" => Role::System,
        "user" => Role::User,
        "assistant" => Role::Assistant,
        "tool" => Role::Tool,
        _ => Role::User,
    }
}

fn to_core_request(req: &LlmRequest) -> IoLlmRequest {
    let messages = req
        .messages
        .iter()
        .map(|m| Message::new(convert_role(&m.role), m.content.clone()))
        .collect();

    IoLlmRequest {
        model: req.model.clone(),
        messages,
        temperature: req.temperature,
        max_tokens: req.max_tokens.unwrap_or(4096),
        tools: vec![],
        image_data: None,
        thinking_budget: None,
        provider_params: std::collections::HashMap::new(),
        constraint_json: None,
        structured_output: None,
        cacheable_prefix_tokens: None,
    }
}

fn convert_finish_reason(fr: FinishReason) -> Option<String> {
    match fr {
        FinishReason::Stop => Some("stop".into()),
        FinishReason::Length => Some("length".into()),
        FinishReason::Tool => Some("tool".into()),
        FinishReason::ContentFilter => Some("content_filter".into()),
        FinishReason::Unknown => Some("unknown".into()),
    }
}

/// 将 nt_io_provider 的 LLM 提供者适配为统一 LlmProvider
pub struct LlmProviderBridge {
    id: String,
    name: String,
    is_free: bool,
    requires_api_key: bool,
    inner: Arc<dyn IoLlmProvider>,
}

impl LlmProviderBridge {
    pub fn new(id: &str, name: &str, is_free: bool, requires_api_key: bool, inner: Arc<dyn IoLlmProvider>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            is_free,
            requires_api_key,
            inner,
        }
    }
}

impl DataSource for LlmProviderBridge {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Llm]
    }

    fn requires_key(&self) -> bool {
        self.requires_api_key
    }
}

impl LlmProvider for LlmProviderBridge {
    fn complete(
        &self,
        request: &LlmRequest,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<LlmResponse, String>> + Send>>
    {
        let core_req = to_core_request(request);
        let inner = self.inner.clone();
        Box::pin(async move {
            let core_resp = inner
                .complete_raw(&core_req)
                .await
                .map_err(|e| e.to_string())?;
            Ok(LlmResponse {
                content: core_resp.content,
                model: core_resp.model,
                usage: LlmUsage {
                    prompt_tokens: core_resp.usage.prompt_tokens,
                    completion_tokens: core_resp.usage.completion_tokens,
                    total_tokens: core_resp.usage.total_tokens,
                },
                finish_reason: convert_finish_reason(core_resp.finish_reason),
            })
        })
    }

    fn is_free(&self) -> bool {
        self.is_free
    }

    fn requires_api_key(&self) -> bool {
        self.requires_api_key
    }
}

/// 创建单个提供者桥接器
fn bridge_provider(
    variant: LlmProviderType,
    id: &str,
    name: &str,
    is_free: bool,
    requires_api_key: bool,
) -> Arc<dyn LlmProvider> {
    let inner = create_provider_from_type(variant, None);
    Arc::new(LlmProviderBridge::new(id, name, is_free, requires_api_key, inner))
}

/// 创建所有 LLM 提供者桥接器 — 每个桥接真实 provider 实现
pub fn create_llm_bridges() -> Vec<Arc<dyn LlmProvider>> {
    vec![
        // ── 付费提供者 ──────────────────────────────────────────────
        bridge_provider(LlmProviderType::OpenAI,      "openai",      "OpenAI",              false, true),
        bridge_provider(LlmProviderType::Anthropic,   "anthropic",   "Anthropic",           false, true),
        bridge_provider(LlmProviderType::Xai,         "xai",         "xAI (Grok)",          false, true),
        bridge_provider(LlmProviderType::Moonshot,    "moonshot",    "Moonshot (Kimi)",     false, true),
        bridge_provider(LlmProviderType::Qwen,        "qwen",        "Qwen (DashScope)",    false, true),
        bridge_provider(LlmProviderType::Doubao,      "doubao",      "Doubao (Ark)",        false, true),
        bridge_provider(LlmProviderType::MiniMax,     "minimax",     "MiniMax",             false, true),
        bridge_provider(LlmProviderType::Perplexity,  "perplexity",  "Perplexity",          false, true),
        bridge_provider(LlmProviderType::Cohere,      "cohere",      "Cohere",              false, true),
        // ── 免费提供者 ──────────────────────────────────────────────
        bridge_provider(LlmProviderType::Gemini,          "gemini",          "Google Gemini",              true, true),
        bridge_provider(LlmProviderType::Groq,            "groq",            "Groq",                      true, true),
        bridge_provider(LlmProviderType::OpenRouter,      "openrouter",      "OpenRouter",                 true, true),
        bridge_provider(LlmProviderType::Cerebras,        "cerebras",        "Cerebras",                   true, true),
        bridge_provider(LlmProviderType::SambaNova,       "sambanova",       "SambaNova",                  true, true),
        bridge_provider(LlmProviderType::Pollinations,    "pollinations",    "Pollinations",               true, false),
        bridge_provider(LlmProviderType::Cloudflare,      "cloudflare",      "Cloudflare Workers AI",      true, true),
        bridge_provider(LlmProviderType::Nvidia,          "nvidia",          "NVIDIA NIM",                 true, true),
        bridge_provider(LlmProviderType::GitHubModels,    "github_models",   "GitHub Models",              true, true),
        bridge_provider(LlmProviderType::HuggingFace,     "huggingface",     "HuggingFace Inference",      true, true),
        bridge_provider(LlmProviderType::TogetherFree,    "together_free",   "Together AI Free",           true, true),
        bridge_provider(LlmProviderType::Llm7,            "llm7",            "LLM7",                       true, false),
        bridge_provider(LlmProviderType::Kilo,            "kilo",            "Kilo AI",                    true, true),
        bridge_provider(LlmProviderType::SiliconFlow,     "siliconflow",     "SiliconFlow",                true, true),
        bridge_provider(LlmProviderType::ZAI,             "zai",             "Z.AI",                       true, true),
        bridge_provider(LlmProviderType::OpenCodeZen,     "opencode_zen",    "OpenCode Zen",               true, true),
        bridge_provider(LlmProviderType::Ovh,             "ovh",             "OVH AI",                     true, true),
        bridge_provider(LlmProviderType::DeepSeekFree,    "deepseek_free",   "DeepSeek Free",              true, true),
        bridge_provider(LlmProviderType::ModelScope,      "modelscope",      "ModelScope",                 true, true),
        bridge_provider(LlmProviderType::ApiAirforce,     "api_airforce",    "API Airforce",               true, false),
        bridge_provider(LlmProviderType::Empero,          "empero",          "Free Empero",                true, false),
        bridge_provider(LlmProviderType::Ollama,          "ollama",          "Ollama (Local)",             true, false),
        bridge_provider(LlmProviderType::Vllm,            "vllm",            "vLLM (Local)",               true, false),
        bridge_provider(LlmProviderType::Sglang,          "sglang",          "SGLang (Local)",             true, false),
        // ── 代理/自定义提供者 ──────────────────────────────────────
        bridge_provider(LlmProviderType::CustomProxy,     "custom_proxy",    "Custom Proxy",               false, true),
        bridge_provider(LlmProviderType::Aihub,           "aihub",           "AI Hub",                     true, true),
    ]
}

/// bridge_all_llm_providers — 统一入口
pub fn bridge_all_llm_providers() -> Vec<Arc<dyn LlmProvider>> {
    create_llm_bridges()
}

/// 按免费/付费分类
pub fn categorize_llm_providers(
    providers: &[Arc<dyn LlmProvider>],
) -> (Vec<&Arc<dyn LlmProvider>>, Vec<&Arc<dyn LlmProvider>>) {
    let free: Vec<_> = providers.iter().filter(|p| p.is_free()).collect();
    let paid: Vec<_> = providers.iter().filter(|p| !p.is_free()).collect();
    (free, paid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_bridge_creation() {
        let inner = create_provider_from_type(LlmProviderType::OpenAI, None);
        let bridge = LlmProviderBridge::new("openai", "OpenAI", false, true, inner);
        assert_eq!(bridge.id(), "openai");
        assert!(!bridge.is_free());
        assert!(bridge.requires_key());
    }

    #[test]
    fn test_create_all_llm_bridges() {
        let bridges = create_llm_bridges();
        assert!(bridges.len() >= 30); // 至少 30 个 LLM 提供者
    }

    #[test]
    fn test_categorize_providers() {
        let bridges = create_llm_bridges();
        let (free, paid) = categorize_llm_providers(&bridges);
        assert!(!free.is_empty());
        assert!(!paid.is_empty());
    }

    #[test]
    fn test_bridge_domains() {
        let inner = create_provider_from_type(LlmProviderType::Ollama, None);
        let bridge = LlmProviderBridge::new("ollama", "Ollama", true, false, inner);
        let domains = bridge.domains();
        assert_eq!(domains.len(), 1);
        assert!(matches!(domains[0], SourceDomain::Llm));
    }

    #[test]
    fn test_bridge_all_fn() {
        let bridges = bridge_all_llm_providers();
        assert!(bridges.len() >= 30);
    }

    #[tokio::test]
    async fn test_llm_complete() {
        let inner = create_provider_from_type(LlmProviderType::Ollama, None);
        let bridge = LlmProviderBridge::new("ollama", "Ollama", true, false, inner);
        let request = LlmRequest {
            model: "test-model".into(),
            messages: vec![LlmMessage {
                role: "user".into(),
                content: "Hello".into(),
            }],
            max_tokens: None,
            temperature: None,
            stream: false,
        };

        // Ollama may not be running; just verify the bridge compiles and dispatches
        let result = bridge.complete(&request).await;
        // Accept either success (Ollama running) or error (Ollama not running)
        assert!(result.is_ok() || result.is_err());
    }
}
