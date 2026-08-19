// ── Mode System (from Kimi Code: Agent + Shell dual-mode) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum NeoCodexMode {
    #[default]
    Agent,
    Shell,
    Plan,
}

// ── Provider Catalog (from Kimi Code kosong: capability-based) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelCapability {
    Code,
    Reasoning,
    Vision,
    Video,
    Thinking,
    FunctionCalling,
    ParallelToolUse,
    LongContext,
}

#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub name: String,
    pub model: String,
    pub capabilities: Vec<ModelCapability>,
    pub context_limit: usize,
    pub cost_per_m_input: f64,
    pub cost_per_m_output: f64,
}

impl Default for ProviderInfo {
    fn default() -> Self {
        Self {
            name: "opencode".into(),
            model: "default".into(),
            capabilities: vec![
                ModelCapability::Code,
                ModelCapability::Reasoning,
                ModelCapability::FunctionCalling,
                ModelCapability::LongContext,
            ],
            context_limit: 100_000,
            cost_per_m_input: 0.0,
            cost_per_m_output: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProviderCatalog {
    pub providers: Vec<ProviderInfo>,
    pub active: usize,
}

impl Default for ProviderCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderCatalog {
    pub fn new() -> Self {
        Self {
            providers: vec![ProviderInfo::default()],
            active: 0,
        }
    }

    pub fn has_capability(&self, cap: ModelCapability) -> bool {
        self.providers
            .get(self.active)
            .map(|p| p.capabilities.contains(&cap))
            .unwrap_or(false)
    }

    pub fn add_provider(&mut self, info: ProviderInfo) {
        self.providers.push(info);
    }

    /// Populate catalog from the real nt_io_provider layer
    pub fn sync_from_real(&mut self) {
        use crate::neotrix::nt_io_provider::provider_catalog;
        self.providers.clear();
        for entry in provider_catalog::PROVIDER_CATALOG.iter() {
            let is_code = entry
                .models
                .iter()
                .any(|m| m.to_lowercase().contains("code") || m.to_lowercase().contains("coder"));
            let mut capabilities = vec![ModelCapability::Reasoning];
            if is_code {
                capabilities.push(ModelCapability::Code);
            }
            if entry.models.iter().any(|m| {
                m.to_lowercase().contains("vision")
                    || m.to_lowercase().contains("vl")
                    || m.to_lowercase().contains("4o")
            }) {
                capabilities.push(ModelCapability::Vision);
            }
            if entry.default_model.to_lowercase().contains("thinking")
                || entry
                    .models
                    .iter()
                    .any(|m| m.to_lowercase().contains("think"))
            {
                capabilities.push(ModelCapability::Thinking);
            }
            capabilities.push(ModelCapability::FunctionCalling);
            capabilities.push(ModelCapability::LongContext);
            self.providers.push(ProviderInfo {
                name: entry.name.to_string(),
                model: entry.default_model.to_string(),
                capabilities,
                context_limit: 100_000,
                cost_per_m_input: if entry.is_free { 0.0 } else { 0.5 },
                cost_per_m_output: if entry.is_free { 0.0 } else { 2.0 },
            });
        }
        if self.providers.is_empty() {
            self.providers.push(ProviderInfo::default());
        }
    }

    /// Map a provider name to a real LlmProviderType (shared by selection helpers).
    fn provider_type_of(name: &str) -> Option<crate::neotrix::nt_io_provider::LlmProviderType> {
        match name {
            "openai" | "gpt" => Some(crate::neotrix::nt_io_provider::LlmProviderType::OpenAI),
            "anthropic" | "claude" => {
                Some(crate::neotrix::nt_io_provider::LlmProviderType::Anthropic)
            }
            "gemini" | "google" => Some(crate::neotrix::nt_io_provider::LlmProviderType::Gemini),
            "ollama" => Some(crate::neotrix::nt_io_provider::LlmProviderType::Ollama),
            "openrouter" => Some(crate::neotrix::nt_io_provider::LlmProviderType::OpenRouter),
            "groq" => Some(crate::neotrix::nt_io_provider::LlmProviderType::Groq),
            "cerebras" => Some(crate::neotrix::nt_io_provider::LlmProviderType::Cerebras),
            "sambanova" => Some(crate::neotrix::nt_io_provider::LlmProviderType::SambaNova),
            "pollinations" => Some(crate::neotrix::nt_io_provider::LlmProviderType::Pollinations),
            "bazaarlink" => Some(crate::neotrix::nt_io_provider::LlmProviderType::BazaarLink),
            "nvidia" => Some(crate::neotrix::nt_io_provider::LlmProviderType::Nvidia),
            "github-models" | "github_models" => {
                Some(crate::neotrix::nt_io_provider::LlmProviderType::GitHubModels)
            }
            "huggingface" | "hf" => {
                Some(crate::neotrix::nt_io_provider::LlmProviderType::HuggingFace)
            }
            "cohere" => Some(crate::neotrix::nt_io_provider::LlmProviderType::Cohere),
            "siliconflow" => Some(crate::neotrix::nt_io_provider::LlmProviderType::SiliconFlow),
            "deepseek-free" | "deepseek_free" => {
                Some(crate::neotrix::nt_io_provider::LlmProviderType::DeepSeekFree)
            }
            "lm-studio" | "llamacpp" | "local" => {
                Some(crate::neotrix::nt_io_provider::LlmProviderType::Ollama)
            }
            _ => None,
        }
    }

    /// True if the active provider name maps to a real LlmProvider type.
    pub fn is_resolvable(&self) -> bool {
        self.providers
            .get(self.active)
            .map(|p| Self::provider_type_of(&p.name).is_some())
            .unwrap_or(false)
    }

    /// True if the given provider name maps to a real LlmProvider type.
    pub fn is_resolvable_for(&self, name: &str) -> bool {
        Self::provider_type_of(name).is_some()
    }

    /// Set the active provider by name. Returns true if found.
    pub fn set_active_provider(&mut self, name: &str) -> bool {
        if let Some(idx) = self.providers.iter().position(|p| p.name == name) {
            self.active = idx;
            true
        } else {
            false
        }
    }

    /// Pick the active provider's concrete model id
    pub fn active_model(&self) -> String {
        self.providers
            .get(self.active)
            .map(|p| p.model.clone())
            .unwrap_or_else(|| "default".to_string())
    }

    /// Sync from the real provider catalog and select a usable provider.
    /// Honors `NEOTRIX_PROVIDER` env if set; otherwise picks the first
    /// resolvable (non-stub) provider. Fixes the Cycle 159 gap where the
    /// default "opencode" stub was never replaced, leaving the ReAct loop
    /// unreachable in production.
    pub fn ensure_production_provider(&mut self) {
        self.sync_from_real();
        if let Ok(name) = std::env::var("NEOTRIX_PROVIDER") {
            if let Some(idx) = self.providers.iter().position(|p| p.name == name) {
                self.active = idx;
                return;
            }
        }
        // Restore the user's last provider choice (persisted by set_active_provider).
        self.load_persisted();
        if !self.is_resolvable() {
            if let Some(idx) = self
                .providers
                .iter()
                .position(|p| Self::provider_type_of(&p.name).is_some())
            {
                self.active = idx;
            }
        }
    }

    /// Path to the persisted active-provider file (~/.neocodex/provider.json).
    fn persist_path() -> std::path::PathBuf {
        let base = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from(".neocodex"))
            .join("neocodex");
        base.join("provider.json")
    }

    /// Re-apply the persisted provider choice after a catalog sync.
    /// Safe to call on every agent boot: no-ops when no file exists.
    pub fn load_persisted(&mut self) {
        let path = Self::persist_path();
        let Ok(text) = std::fs::read_to_string(&path) else {
            return;
        };
        let Ok(saved) = serde_json::from_str::<serde_json::Value>(&text) else {
            return;
        };
        if let Some(name) = saved.get("active").and_then(|v| v.as_str()) {
            if let Some(idx) = self.providers.iter().position(|p| p.name == name) {
                self.active = idx;
            }
        }
    }

    /// Persist the active provider name so it survives app restarts.
    pub fn save_persisted(&self) {
        let path = Self::persist_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let name = self
            .providers
            .get(self.active)
            .map(|p| p.name.clone())
            .unwrap_or_default();
        if let Ok(text) = serde_json::to_string(&serde_json::json!({ "active": name })) {
            let _ = std::fs::write(path, text);
        }
    }

    /// Create a LlmProvider from real layer (if matching provider type)
    pub fn to_llm_provider(&self) -> Option<Box<dyn crate::neotrix::nt_io_provider::LlmProvider>> {
        let info = self.providers.get(self.active)?;
        let provider_type = Self::provider_type_of(&info.name)?;
        let mut config = crate::neotrix::nt_io_provider::ProviderConfig::from_env();
        config.provider_type = provider_type;
        config.model = Some(info.model.clone());
        if info.name == "anthropic" || info.name == "claude" {
            config.api_key = std::env::var("ANTHROPIC_API_KEY")
                .ok()
                .or_else(|| std::env::var("NEOTRIX_API_KEY").ok());
        }
        Some(crate::neotrix::nt_io_provider::create_provider(config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_catalog() {
        let catalog = ProviderCatalog::new();
        assert!(catalog.has_capability(ModelCapability::Code));
        assert!(!catalog.has_capability(ModelCapability::Video));
    }

    #[test]
    fn test_sync_from_real_populates_catalog() {
        let mut catalog = ProviderCatalog::new();
        catalog.sync_from_real();
        assert!(!catalog.providers.is_empty());
        // Real catalog has ollama + many cloud providers
        assert!(catalog.providers.iter().any(|p| p.name == "ollama"));
        // All entries carry a concrete model id
        for p in &catalog.providers {
            assert!(!p.model.is_empty());
        }
    }

    #[test]
    fn test_active_model_after_sync() {
        let mut catalog = ProviderCatalog::new();
        catalog.sync_from_real();
        assert!(!catalog.active_model().is_empty());
    }

    #[test]
    fn test_provider_add() {
        let mut catalog = ProviderCatalog::new();
        catalog.add_provider(ProviderInfo {
            name: "anthropic".into(),
            model: "claude-3-5-sonnet".into(),
            capabilities: vec![ModelCapability::Reasoning, ModelCapability::Vision],
            context_limit: 200_000,
            cost_per_m_input: 3.0,
            cost_per_m_output: 15.0,
        });
        assert_eq!(catalog.providers.len(), 2);
    }

    #[test]
    fn test_provider_capability_check() {
        let catalog = ProviderCatalog::new();
        assert!(catalog.has_capability(ModelCapability::Code));
    }

    #[test]
    fn test_provider_capability_after_sync() {
        let mut catalog = ProviderCatalog::new();
        catalog.sync_from_real();
        // At least one provider offers code capability
        assert!(catalog
            .providers
            .iter()
            .any(|p| p.capabilities.contains(&ModelCapability::Code)));
    }

    #[test]
    fn test_provider_persist_roundtrip() {
        // Isolate the persisted provider file to a temp data dir.
        let tmp =
            std::env::temp_dir().join(format!("neocodex-provider-test-{}", std::process::id()));
        let old_data = std::env::var("XDG_DATA_HOME").ok();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("XDG_DATA_HOME", &tmp);
        std::env::set_var("HOME", &tmp);

        // Save a persisted choice.
        let mut catalog = ProviderCatalog::new();
        catalog.sync_from_real();
        assert!(
            catalog.set_active_provider("ollama"),
            "ollama should exist in real catalog"
        );
        catalog.save_persisted();
        let persist_file = ProviderCatalog::persist_path();
        assert!(
            persist_file.exists(),
            "provider.json should be written at {}",
            persist_file.display()
        );

        // Fresh catalog must restore the saved choice after sync.
        let mut restored = ProviderCatalog::new();
        restored.sync_from_real();
        restored.load_persisted();
        assert_eq!(restored.active_model(), catalog.active_model());

        // ensure_production_provider also honors the persisted choice.
        let mut via_ensure = ProviderCatalog::new();
        via_ensure.ensure_production_provider();
        assert_eq!(via_ensure.active_model(), catalog.active_model());

        // Cleanup.
        std::fs::remove_dir_all(&tmp).ok();
        match old_data {
            Some(v) => std::env::set_var("XDG_DATA_HOME", v),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }
        match old_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }
}