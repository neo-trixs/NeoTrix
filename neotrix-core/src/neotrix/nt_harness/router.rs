#![forbid(unsafe_code)]
//! Inference Router — 对标 grok-bot-0.18-reconstructed Router + harness/cli profile
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InferenceProvider {
    #[default]
    Cursor,
    ClaudeCode,
    Codex,
    OpenRouter,
    Local,
}

impl InferenceProvider {
    pub fn all() -> &'static [Self] {
        &[Self::Cursor, Self::ClaudeCode, Self::Codex, Self::OpenRouter, Self::Local]
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::Cursor => "Cursor",
            Self::ClaudeCode => "Claude Code",
            Self::Codex => "Codex",
            Self::OpenRouter => "OpenRouter",
            Self::Local => "Local",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessRouterConfig {
    pub default_provider: InferenceProvider,
    pub fallback_order: Vec<InferenceProvider>,
    pub auto_routing: bool,
}

impl Default for HarnessRouterConfig {
    fn default() -> Self {
        Self {
            default_provider: InferenceProvider::Cursor,
            fallback_order: vec![InferenceProvider::Cursor, InferenceProvider::ClaudeCode, InferenceProvider::Codex],
            auto_routing: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct InferenceRouter {
    pub config: HarnessRouterConfig,
    usage: std::collections::HashMap<String, u64>,
}

impl InferenceRouter {
    pub fn with_config(config: HarnessRouterConfig) -> Self {
        Self { config, usage: std::collections::HashMap::new() }
    }
    pub fn select(&self, hint: Option<InferenceProvider>) -> InferenceProvider {
        hint.unwrap_or(self.config.default_provider)
    }
    pub fn record_usage(&mut self, provider: InferenceProvider, tokens: u64) {
        *self.usage.entry(provider.label().to_string()).or_insert(0) += tokens;
    }
    pub fn usage(&self) -> &std::collections::HashMap<String, u64> {
        &self.usage
    }
    pub fn set_default(&mut self, p: InferenceProvider) {
        self.config.default_provider = p;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_is_cursor() {
        let r = InferenceRouter::default();
        assert_eq!(r.select(None), InferenceProvider::Cursor);
        assert_eq!(r.select(Some(InferenceProvider::Codex)), InferenceProvider::Codex);
    }
    #[test]
    fn record_usage() {
        let mut r = InferenceRouter::default();
        r.record_usage(InferenceProvider::Cursor, 100);
        assert_eq!(*r.usage().get("Cursor").unwrap(), 100);
    }
}
