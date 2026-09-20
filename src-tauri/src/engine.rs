use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use tokio::sync::RwLock;

/// Engine identifier — maps to an LLM provider.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineId {
    OpenAI,
    Anthropic,
    XAi,
    Google,
    DeepSeek,
    Ollama,
    Custom(String),
}

impl std::fmt::Display for EngineId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAI => write!(f, "openai"),
            Self::Anthropic => write!(f, "anthropic"),
            Self::XAi => write!(f, "xai"),
            Self::Google => write!(f, "google"),
            Self::DeepSeek => write!(f, "deepseek"),
            Self::Ollama => write!(f, "ollama"),
            Self::Custom(s) => write!(f, "custom:{s}"),
        }
    }
}

impl std::str::FromStr for EngineId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openai" | "gpt" => Ok(Self::OpenAI),
            "anthropic" | "claude" => Ok(Self::Anthropic),
            "xai" | "grok" => Ok(Self::XAi),
            "google" | "gemini" => Ok(Self::Google),
            "deepseek" => Ok(Self::DeepSeek),
            "ollama" => Ok(Self::Ollama),
            other => Ok(Self::Custom(other.to_string())),
        }
    }
}

/// Model tier — Cumora pattern: route to fast/standard/think per use case.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelTier {
    /// Low latency, low cost — for triage, classification, simple Q&A.
    Fast,
    /// Balanced — for normal conversation, code generation.
    Standard,
    /// High reasoning — for complex analysis, deep search, planning.
    Think,
    /// Deep research/search.
    Deep,
}

impl std::fmt::Display for ModelTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fast => write!(f, "Fast"),
            Self::Standard => write!(f, "Standard"),
            Self::Think => write!(f, "Think"),
            Self::Deep => write!(f, "Deep"),
        }
    }
}

impl ModelTier {
    /// Max tokens budgeted for the response body.
    pub fn max_tokens(&self) -> u32 {
        match self {
            Self::Fast => 1_024,
            Self::Standard => 4_096,
            Self::Think => 16_384,
            Self::Deep => 32_768,
        }
    }

    /// Temperature hint (providers may override).
    pub fn temperature(&self) -> f32 {
        match self {
            Self::Fast => 0.3,
            Self::Standard => 0.7,
            Self::Think => 1.0,
            Self::Deep => 0.9,
        }
    }
}

/// Capability flags per engine — what it can actually do.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EngineCaps {
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_audio: bool,
    pub supports_thinking: bool,
    pub max_context_window: u32,
}

/// Request to an engine adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineRequest {
    pub messages: Vec<ChatMessage>,
    pub tier: ModelTier,
    pub system_prompt: Option<String>,
    pub tools: Vec<ToolDefinition>,
    pub temperature_override: Option<f32>,
    pub max_tokens_override: Option<u32>,
}

/// A single chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// Tool definition for function calling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Engine response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineResponse {
    pub content: String,
    pub model: String,
    pub engine: EngineId,
    pub tier: ModelTier,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: FinishReason,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Error(String),
}

/// The engine adapter trait — the Cumora pattern applied to NeoTrix.
///
/// Each engine (OpenAI, Anthropic, xAI, etc.) implements this trait.
/// The trait separates "brain" (LLM provider) from "host" (runtime)
/// from "identity" (persona/memory).
#[async_trait::async_trait]
pub trait EngineAdapter: Send + Sync {
    /// Unique identifier for this engine.
    fn id(&self) -> EngineId;

    /// Human-readable name.
    fn name(&self) -> &str;

    /// Capabilities this engine supports.
    fn caps(&self) -> EngineCaps;

    /// Send a request and get a response.
    async fn run(&self, request: EngineRequest) -> Result<EngineResponse, EngineError>;

    /// Classify a user message locally (triage gate).
    /// Returns (is_complex, suggested_tier). Default: always Standard.
    fn classify(&self, _message: &str) -> (bool, ModelTier) {
        (false, ModelTier::Standard)
    }

    /// Health probe — is this engine reachable and authenticated?
    async fn probe(&self) -> Result<(), EngineError> {
        Ok(())
    }

    /// Seed persona files for this engine in the given home directory.
    fn seed_home(&self, _home: &PathBuf, _persona: &Persona) -> Result<(), EngineError> {
        Ok(())
    }
}

/// Agent persona — the identity attached to an engine session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub name: String,
    pub role: String,
    pub system_prompt: String,
    pub avatar_color: Option<String>,
    pub skills: Vec<String>,
}

/// Errors from engine operations.
#[derive(Debug, Clone, Error)]
pub enum EngineError {
    #[error("auth failed: {0}")]
    AuthFailed(String),
    #[error("rate limited, retry in {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    #[error("provider error: {0}")]
    ProviderError(String),
    #[error("network error: {0}")]
    NetworkError(String),
    #[error("context too long: {actual} > {max}")]
    ContextTooLong { max: u32, actual: u32 },
    #[error("content filtered: {0}")]
    ContentFiltered(String),
    #[error("timeout after {0}ms")]
    Timeout(u64),
    #[error("engine error: {0}")]
    Unknown(String),
}

/// The engine registry — manages all available adapters.
pub struct EngineRegistry {
    adapters: RwLock<HashMap<EngineId, std::sync::Arc<dyn EngineAdapter>>>,
    /// Default engine for each tier: tier → engine_id.
    tier_map: RwLock<HashMap<ModelTier, EngineId>>,
}

impl EngineRegistry {
    pub fn new() -> Self {
        Self {
            adapters: RwLock::new(HashMap::new()),
            tier_map: RwLock::new(HashMap::new()),
        }
    }

    /// Register an engine adapter.
    pub async fn register(&self, adapter: Box<dyn EngineAdapter>) {
        let id = adapter.id();
        self.adapters
            .write()
            .await
            .insert(id.clone(), std::sync::Arc::from(adapter));
        tracing::info!("registered engine adapter: {id}");
    }

    /// Set the default engine for a tier.
    pub async fn set_tier_default(&self, tier: ModelTier, engine_id: EngineId) {
        self.tier_map.write().await.insert(tier, engine_id);
    }

    /// Get an adapter by engine ID.
    pub async fn get(&self, id: &EngineId) -> Option<std::sync::Arc<dyn EngineAdapter>> {
        let adapters = self.adapters.read().await;
        adapters.get(id).cloned()
    }

    /// Run a request using the default engine for the given tier.
    pub async fn run_tier(
        &self,
        tier: ModelTier,
        request: EngineRequest,
    ) -> Result<EngineResponse, EngineError> {
        let engine_id = {
            let map = self.tier_map.read().await;
            map.get(&tier)
                .cloned()
                .ok_or_else(|| EngineError::Unknown(format!("no engine for tier {tier:?}")))?
        };

        // In production, look up the adapter and call run().
        // This is a skeleton — the real implementation clones the Arc<dyn EngineAdapter>.
        tracing::debug!("running request on engine {engine_id} tier {tier:?}");
        Err(EngineError::Unknown(
            "registry.run_tier not yet wired".into(),
        ))
    }

    /// Probe all registered engines for health.
    pub async fn probe_all(&self) -> HashMap<EngineId, Result<(), EngineError>> {
        let adapters = self.adapters.read().await;
        let mut results = HashMap::new();
        for (id, adapter) in adapters.iter() {
            results.insert(id.clone(), adapter.probe().await);
        }
        results
    }
}

impl Default for EngineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_id_roundtrip() {
        let cases = vec![
            ("openai", EngineId::OpenAI),
            ("anthropic", EngineId::Anthropic),
            ("xai", EngineId::XAi),
            ("google", EngineId::Google),
            ("deepseek", EngineId::DeepSeek),
            ("ollama", EngineId::Ollama),
            ("custom:myengine", EngineId::Custom("myengine".into())),
        ];
        for (s, expected) in cases {
            let parsed: EngineId = s.parse().expect("test parse should succeed");
            assert_eq!(parsed, expected);
            assert_eq!(parsed.to_string(), s);
        }
    }

    #[test]
    fn model_tier_defaults() {
        assert_eq!(ModelTier::Fast.max_tokens(), 1_024);
        assert_eq!(ModelTier::Standard.max_tokens(), 4_096);
        assert_eq!(ModelTier::Think.max_tokens(), 16_384);

        assert!(ModelTier::Fast.temperature() < ModelTier::Standard.temperature());
        assert!(ModelTier::Standard.temperature() < ModelTier::Think.temperature());
    }
}
