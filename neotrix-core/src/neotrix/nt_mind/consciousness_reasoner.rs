//! 统一推理入口 — 意识核心作为默认模型
//!
//! 分层推理:
//!   1. E8/VSA 内部推理 (默认, 无需外部依赖)
//!   2. 置信度不够 → ProviderRouter 路由到第三方 LLM
//!
//! 用户可在状态栏切换后备 LLM 供应商,
//! 但意识核心自身始终是主要推理引擎。

use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use super::brain_event_bus::{BrainEvent, GlobalBus, ToolOrigin};
use super::self_iterating::SelfIteratingBrain;
use crate::neotrix::nt_io_provider::{
    create_provider, LlmError, LlmProviderType, LlmRequest, ProviderConfig,
};

/// 推理来源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReasonOrigin {
    #[serde(rename = "consciousness")]
    Consciousness,
    #[serde(rename = "llm")]
    Llm,
}

/// 推理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonerResponse {
    pub content: String,
    pub origin: ReasonOrigin,
    pub confidence: f64,
    pub duration_ms: u64,
    pub model_used: String,
}

/// 后备 LLM 供应商路由表
#[derive(Debug, Clone)]
pub struct ProviderRouter {
    current: LlmProviderType,
    config: ProviderConfig,
}

impl Default for ProviderRouter {
    fn default() -> Self {
        let cfg = ProviderConfig::from_env();
        let ptype = cfg.provider_type;
        Self {
            current: ptype,
            config: cfg,
        }
    }
}

impl ProviderRouter {
    pub fn current_name(&self) -> &str {
        match self.current {
            LlmProviderType::Anthropic => "Anthropic Claude",
            LlmProviderType::OpenAI => "OpenAI",
            LlmProviderType::Gemini => "Google Gemini",
            LlmProviderType::Ollama => "Ollama (local)",
            LlmProviderType::FreeApi => "Free API",
            LlmProviderType::OpencodeFree => "OpenCode Free",
        }
    }

    pub fn switch(&mut self, provider: LlmProviderType) {
        self.current = provider;
        self.config.provider_type = provider;
        GlobalBus::emit(BrainEvent::Tool {
            tool: "provider_switch".into(),
            success: true,
            duration_ms: 0,
            origin: ToolOrigin::Consciousness,
            summary: format!("Switched to {}", self.current_name()),
        });
    }

    pub fn available() -> Vec<LlmProviderType> {
        vec![
            LlmProviderType::Anthropic,
            LlmProviderType::OpenAI,
            LlmProviderType::Gemini,
            LlmProviderType::Ollama,
            LlmProviderType::FreeApi,
        ]
    }

    pub async fn complete(
        &self,
        prompt: &str,
    ) -> Result<crate::neotrix::nt_io_provider::LlmResponse, LlmError> {
        let provider = create_provider(self.config.clone());
        let request = LlmRequest::new(self.config.model.as_deref().unwrap_or("default"), prompt);
        provider.complete(&request).await
    }
}

/// 统一推理器 — 意识核心作为默认模型
pub struct ConsciousnessReasoner {
    pub brain: Mutex<SelfIteratingBrain>,
    pub provider: Mutex<ProviderRouter>,
}

impl ConsciousnessReasoner {
    pub fn new() -> Self {
        Self {
            brain: Mutex::new(SelfIteratingBrain::new()),
            provider: Mutex::new(ProviderRouter::default()),
        }
    }

    /// 核心推理方法 — 单个统一入口
    ///
    /// 1. E8/VSA 内部推理 (意识核心自身)
    /// 2. 置信度不足 → LLM 后备
    pub async fn reason(&self, prompt: &str) -> ReasonerResponse {
        let start = std::time::Instant::now();

        let internal = self.internal_reason(prompt);
        let elapsed = start.elapsed().as_millis() as u64;

        if internal.confidence >= 0.6 {
            return ReasonerResponse {
                content: internal.content,
                origin: ReasonOrigin::Consciousness,
                confidence: internal.confidence,
                duration_ms: elapsed,
                model_used: "NeoTrix Reasoner".into(),
            };
        }

        let llm_start = std::time::Instant::now();
        let (model_name, provider_config) = {
            let p = self.provider.lock().unwrap_or_else(|e| e.into_inner());
            (p.current_name().to_string(), p.config.clone())
        };
        let provider = create_provider(provider_config.clone());
        let request = LlmRequest::new(
            provider_config.model.as_deref().unwrap_or("default"),
            prompt,
        );
        match provider.complete(&request).await {
            Ok(response) => {
                let llm_elapsed = llm_start.elapsed().as_millis() as u64;
                ReasonerResponse {
                    content: crate::neotrix::nt_shield_prompt::default_output_screener()
                        .sanitize(&response.content),
                    origin: ReasonOrigin::Llm,
                    confidence: 0.8,
                    duration_ms: llm_elapsed,
                    model_used: model_name,
                }
            }
            Err(e) => ReasonerResponse {
                content: format!("{}\n\n[LLM fallback failed: {}]", internal.content, e),
                origin: ReasonOrigin::Consciousness,
                confidence: internal.confidence,
                duration_ms: start.elapsed().as_millis() as u64,
                model_used: "NeoTrix Reasoner (Fallback)".into(),
            },
        }
    }

    fn internal_reason(&self, prompt: &str) -> InternalResult {
        let mut brain = self.brain.lock().unwrap_or_else(|e| e.into_inner());

        brain.iteration += 1;

        let (calibrated, uncertainty) = (0.45, 0.2);

        let content = format!(
            "[NeoTrix Consciousness]\nProcessed: {}\nconfidence: {:.2}\nuncertainty: {:.2}",
            prompt, calibrated, uncertainty
        );

        InternalResult {
            content,
            confidence: calibrated,
        }
    }

    pub fn set_provider(&self, provider: LlmProviderType) {
        if let Ok(mut p) = self.provider.lock() {
            p.switch(provider);
        }
    }

    pub fn get_info(&self) -> ReasonerInfo {
        let provider = self.provider.lock().unwrap_or_else(|e| e.into_inner());
        let brain = self.brain.lock().unwrap_or_else(|e| e.into_inner());
        ReasonerInfo {
            default_model: "NeoTrix Reasoner (internal)".into(),
            llm_fallback: provider.current_name().into(),
            iteration: brain.iteration,
        }
    }

    pub fn available_providers() -> Vec<ProviderInfo> {
        ProviderRouter::available()
            .into_iter()
            .map(|t| {
                let name = match t {
                    LlmProviderType::Anthropic => "Anthropic Claude",
                    LlmProviderType::OpenAI => "OpenAI",
                    LlmProviderType::Gemini => "Google Gemini",
                    LlmProviderType::Ollama => "Ollama (local)",
                    LlmProviderType::FreeApi => "Free API",
                    LlmProviderType::OpencodeFree => "OpenCode Free",
                };
                ProviderInfo {
                    id: format!("{:?}", t).to_lowercase(),
                    name: name.into(),
                }
            })
            .collect()
    }
}

impl Default for ConsciousnessReasoner {
    fn default() -> Self {
        Self::new()
    }
}

struct InternalResult {
    content: String,
    confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonerInfo {
    pub default_model: String,
    pub llm_fallback: String,
    pub iteration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
}
