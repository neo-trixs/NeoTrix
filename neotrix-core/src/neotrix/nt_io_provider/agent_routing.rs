use std::collections::{HashMap, HashSet};

/// Routes different agent types to different provider/models
pub struct AgentRoutingTable {
    /// Map of agent type -> (provider, model)
    routes: HashMap<String, (String, String)>,
    /// Fallback provider/model when no route matches
    default_route: (String, String),
}

impl AgentRoutingTable {
    pub fn new(default_provider: &str, default_model: &str) -> Self {
        Self {
            routes: HashMap::new(),
            default_route: (default_provider.to_string(), default_model.to_string()),
        }
    }

    /// Add a route: when agent type matches, use this provider/model
    pub fn add_route(&mut self, agent_type: &str, provider: &str, model: &str) {
        self.routes.insert(
            agent_type.to_string(),
            (provider.to_string(), model.to_string()),
        );
    }

    /// Remove a route
    pub fn remove_route(&mut self, agent_type: &str) -> Option<(String, String)> {
        self.routes.remove(agent_type)
    }

    /// Get the provider/model for a given agent type (falls back to default)
    pub fn route_for(&self, agent_type: &str) -> &(String, String) {
        self.routes.get(agent_type).unwrap_or(&self.default_route)
    }

    /// Get all routes
    pub fn all_routes(&self) -> &HashMap<String, (String, String)> {
        &self.routes
    }

    /// Number of configured routes
    pub fn len(&self) -> usize {
        self.routes.len()
    }

    /// Whether the table has no custom routes
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }

    /// Load routes from a settings map (e.g., from config file)
    pub fn load_from_map(&mut self, settings: &HashMap<String, serde_json::Value>) {
        for (key, val) in settings {
            if let Some(obj) = val.as_object() {
                let provider = obj
                    .get("provider")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let model = obj.get("model").and_then(|v| v.as_str()).unwrap_or("");
                if !provider.is_empty() && !model.is_empty() {
                    self.routes
                        .insert(key.clone(), (provider.to_string(), model.to_string()));
                }
            }
        }
    }

    /// Serialize to a settings map
    pub fn to_settings_map(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        for (key, (provider, model)) in &self.routes {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "provider".to_string(),
                serde_json::Value::String(provider.clone()),
            );
            obj.insert(
                "model".to_string(),
                serde_json::Value::String(model.clone()),
            );
            map.insert(key.clone(), serde_json::Value::Object(obj));
        }
        map
    }
}

/// Model tier classification — HashCortX getModelTier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModelTier {
    Frontier = 5,
    Strong = 4,
    Capable = 3,
    Moderate = 2,
    Small = 1,
}

impl ModelTier {
    pub fn parse(model_name: &str) -> Self {
        let lower = model_name.to_lowercase();
        // Frontier: 300B+ models
        if lower.contains("claude-4")
            || lower.contains("gpt-5")
            || lower.contains("gemini-3")
            || lower.contains("claude-3.5-opus")
            || lower.contains("gpt-4.5")
        {
            return Self::Frontier;
        }
        // Capable: 30B-70B (must check BEFORE Strong for overlapping names)
        if lower.contains("gpt-4o-mini") || lower.contains("claude-3-haiku") || lower.contains("claude-haiku")
        {
            return Self::Capable;
        }
        // Strong: 70B-300B
        if lower.contains("claude-3")
            || lower.contains("claude-sonnet-4")
            || lower.contains("gpt-4")
            || lower.contains("gpt-4o")
            || lower.contains("llama-405")
            || lower.contains("llama3.1-405")
            || lower.contains("gemini-2.0-ultra")
            || lower.contains("gemini-2.5")
            || lower.contains("deepseek-v3")
            || lower.contains("deepseek-r1")
        {
            return Self::Strong;
        }
        // Capable: 30B-70B
        if lower.contains("llama-70b")
            || lower.contains("llama3.1-70")
            || lower.contains("mixtral-8x22")
            || lower.contains("qwen-72")
            || lower.contains("gemma-2-27")
            || lower.contains("mistral-large")
        {
            return Self::Capable;
        }
        // Moderate: 8B-30B
        if lower.contains("llama-8b")
            || lower.contains("llama3.1-8")
            || lower.contains("gemma-2-9")
            || lower.contains("mistral-8b")
            || lower.contains("qwen-14")
            || lower.contains("deepseek-16")
            || lower.contains("mixtral-8x7")
        {
            return Self::Moderate;
        }
        // Small: everything else (1.5B-8B)
        Self::Small
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Frontier => "frontier",
            Self::Strong => "strong",
            Self::Capable => "capable",
            Self::Moderate => "moderate",
            Self::Small => "small",
        }
    }
}

/// Provider kind for free-tier ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    Groq,
    Gemini,
    Cerebras,
    SambaNova,
    OpenRouter,
    OpenAI,
    Anthropic,
    DeepSeek,
    Mistral,
    Moonshot,
    Ollama,
    Custom,
}

impl ProviderKind {
    /// Free-tier order: groq > gemini > cerebras > samba > openrouter > others
    fn free_tier_order(self) -> u8 {
        match self {
            Self::Groq => 0,
            Self::Gemini => 1,
            Self::Cerebras => 2,
            Self::SambaNova => 3,
            Self::OpenRouter => 4,
            _ => 10,
        }
    }
}

/// Tier-aware failover strategy — HashCortX failover algorithm
#[derive(Debug, Clone)]
pub struct FailoverStrategy {
    #[allow(dead_code)]
    free_tier_providers: HashSet<ProviderKind>,
}

impl FailoverStrategy {
    pub fn new() -> Self {
        Self {
            free_tier_providers: HashSet::from([
                ProviderKind::Groq,
                ProviderKind::Gemini,
                ProviderKind::Cerebras,
                ProviderKind::SambaNova,
                ProviderKind::OpenRouter,
            ]),
        }
    }

    /// Find best failover provider+model with deterministic ranking
    ///
    /// Algorithm (HashCortX):
    /// 1. Parse failed model tier
    /// 2. Sort candidates: same-tier > one-up > one-down > free-tier > any
    /// 3. Return best match with failover marker
    pub fn find_failover<'a>(
        &self,
        failed_model: &str,
        available: &'a [ProviderProfile],
        excluded: &HashSet<String>,
        failover_count: u32,
    ) -> Option<(&'a ProviderProfile, String)> {
        let current_tier = ModelTier::parse(failed_model);

        let mut candidates: Vec<&ProviderProfile> = available
            .iter()
            .filter(|p| !excluded.contains(&p.name))
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // 确定性排序
        candidates.sort_by(|a, b| {
            let tier_a = a.effective_tier();
            let tier_b = b.effective_tier();

            // 同tier优先
            let a_same = if tier_a == current_tier { 0 } else { 1 };
            let b_same = if tier_b == current_tier { 0 } else { 1 };
            if a_same != b_same {
                return a_same.cmp(&b_same);
            }

            // 然后高一档
            let a_up = if tier_a > current_tier { 0 } else { 1 };
            let b_up = if tier_b > current_tier { 0 } else { 1 };
            if a_up != b_up {
                return a_up.cmp(&b_up);
            }

            // free-tier 优先
            let kind_a = ProviderKind::from_type(&a.provider_type);
            let kind_b = ProviderKind::from_type(&b.provider_type);
            let a_free = kind_a.free_tier_order();
            let b_free = kind_b.free_tier_order();
            if a_free != b_free {
                return a_free.cmp(&b_free);
            }

            // 最后按名字稳定排序
            a.name.cmp(&b.name)
        });

        let best = candidates.first()?;
        let marker = format!("_(Failover {}: {} → {})", failover_count, failed_model, best.default_model.as_deref().unwrap_or(&best.name));

        Some((best, marker))
    }
}

impl Default for FailoverStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderKind {
    pub fn from_type(provider_type: &str) -> Self {
        match provider_type.to_lowercase().as_str() {
            "groq" => Self::Groq,
            "gemini" | "google" => Self::Gemini,
            "cerebras" => Self::Cerebras,
            "sambanova" => Self::SambaNova,
            "openrouter" => Self::OpenRouter,
            "openai" => Self::OpenAI,
            "anthropic" => Self::Anthropic,
            "deepseek" => Self::DeepSeek,
            "mistral" => Self::Mistral,
            "moonshot" | "kimi" => Self::Moonshot,
            "ollama" | "local" => Self::Ollama,
            _ => Self::Custom,
        }
    }
}

/// Saved provider profile (for /provider-style switching)
#[derive(Debug, Clone)]
pub struct ProviderProfile {
    pub name: String,
    pub provider_type: String, // "openai", "anthropic", "gemini", "ollama"
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub default_model: Option<String>,
    pub is_active: bool,
}

pub struct ProviderProfileManager {
    profiles: Vec<ProviderProfile>,
    active_profile: Option<String>,
}

impl ProviderProfile {
    /// 从 provider_type + default_model 推断 tier
    pub fn effective_tier(&self) -> ModelTier {
        if let Some(ref model) = self.default_model {
            ModelTier::parse(model)
        } else {
            // Provider-level tier heuristic
            match ProviderKind::from_type(&self.provider_type) {
                ProviderKind::Groq => ModelTier::Capable,
                ProviderKind::Gemini => ModelTier::Frontier,
                ProviderKind::OpenAI => ModelTier::Frontier,
                ProviderKind::Anthropic => ModelTier::Frontier,
                ProviderKind::Cerebras => ModelTier::Capable,
                ProviderKind::SambaNova => ModelTier::Capable,
                ProviderKind::DeepSeek => ModelTier::Strong,
                ProviderKind::Mistral => ModelTier::Capable,
                ProviderKind::Moonshot => ModelTier::Capable,
                ProviderKind::OpenRouter => ModelTier::Capable,
                ProviderKind::Ollama => ModelTier::Moderate,
                ProviderKind::Custom => ModelTier::Moderate,
            }
        }
    }
}

impl Default for ProviderProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderProfileManager {
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
            active_profile: None,
        }
    }

    pub fn add_profile(&mut self, profile: ProviderProfile) {
        let is_active = profile.is_active;
        if is_active {
            for p in &mut self.profiles {
                p.is_active = false;
            }
            self.active_profile = Some(profile.name.clone());
        }
        self.profiles.push(profile);
    }

    pub fn remove_profile(&mut self, name: &str) -> bool {
        if let Some(pos) = self.profiles.iter().position(|p| p.name == name) {
            let _removed = self.profiles.remove(pos);
            if self.active_profile.as_deref() == Some(name) {
                self.active_profile = None;
            }
            true
        } else {
            false
        }
    }

    pub fn activate(&mut self, name: &str) -> bool {
        for p in &mut self.profiles {
            if p.name == name {
                p.is_active = true;
                self.active_profile = Some(name.to_string());
            } else {
                p.is_active = false;
            }
        }
        self.active_profile.as_deref() == Some(name)
    }

    pub fn get_active(&self) -> Option<&ProviderProfile> {
        self.active_profile
            .as_ref()
            .and_then(|name| self.profiles.iter().find(|p| p.name == *name))
    }

    pub fn get_profile(&self, name: &str) -> Option<&ProviderProfile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    pub fn list_profiles(&self) -> &[ProviderProfile] {
        &self.profiles
    }

    pub fn active_name(&self) -> Option<&str> {
        self.active_profile.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_default_fallback() {
        let table = AgentRoutingTable::new("openai", "gpt-4");
        let route = table.route_for("unknown_agent");
        assert_eq!(route.0, "openai");
        assert_eq!(route.1, "gpt-4");
    }

    #[test]
    fn test_routing_custom_routes() {
        let mut table = AgentRoutingTable::new("openai", "gpt-4");
        table.add_route("code", "anthropic", "claude-3");
        table.add_route("search", "gemini", "gemini-pro");

        let code = table.route_for("code");
        assert_eq!(code.0, "anthropic");
        assert_eq!(code.1, "claude-3");

        let search = table.route_for("search");
        assert_eq!(search.0, "gemini");
        assert_eq!(search.1, "gemini-pro");
    }

    #[test]
    fn test_remove_route() {
        let mut table = AgentRoutingTable::new("openai", "gpt-4");
        table.add_route("code", "anthropic", "claude-3");
        assert_eq!(table.len(), 1);

        let removed = table.remove_route("code");
        assert!(removed.is_some());
        assert_eq!(table.len(), 0);
        assert!(table.is_empty());

        let route = table.route_for("code");
        assert_eq!(route.0, "openai");
    }

    #[test]
    fn test_load_from_json_map() {
        let mut table = AgentRoutingTable::new("default", "default");
        let mut settings = HashMap::new();

        let mut obj = serde_json::Map::new();
        obj.insert(
            "provider".to_string(),
            serde_json::Value::String("ollama".to_string()),
        );
        obj.insert(
            "model".to_string(),
            serde_json::Value::String("llama3".to_string()),
        );
        settings.insert("local".to_string(), serde_json::Value::Object(obj));

        table.load_from_map(&settings);
        assert_eq!(table.len(), 1);

        let route = table.route_for("local");
        assert_eq!(route.0, "ollama");
        assert_eq!(route.1, "llama3");
    }

    #[test]
    fn test_to_settings_map() {
        let mut table = AgentRoutingTable::new("default", "default");
        table.add_route("code", "anthropic", "claude-opus");

        let map = table.to_settings_map();
        assert_eq!(map.len(), 1);

        let entry = map.get("code").expect("code route should exist in map");
        assert_eq!(entry["provider"], "anthropic");
        assert_eq!(entry["model"], "claude-opus");
    }

    #[test]
    fn test_provider_profile_lifecycle() {
        let mut mgr = ProviderProfileManager::new();
        assert!(mgr.get_active().is_none());

        let profile = ProviderProfile {
            name: "work".to_string(),
            provider_type: "openai".to_string(),
            api_key: Some("sk-...".to_string()),
            base_url: None,
            default_model: Some("gpt-4".to_string()),
            is_active: true,
        };
        mgr.add_profile(profile);
        assert_eq!(mgr.active_name(), Some("work"));

        let profile2 = ProviderProfile {
            name: "local".to_string(),
            provider_type: "ollama".to_string(),
            api_key: None,
            base_url: Some("http://localhost:11434".to_string()),
            default_model: Some("llama3".to_string()),
            is_active: false,
        };
        mgr.add_profile(profile2);
        assert_eq!(mgr.list_profiles().len(), 2);

        assert!(mgr.activate("local"));
        assert_eq!(mgr.active_name(), Some("local"));

        let active = mgr.get_active().expect("active profile should exist");
        assert_eq!(active.provider_type, "ollama");

        assert!(mgr.remove_profile("local"));
        assert!(mgr.get_active().is_none());
    }

    #[test]
    fn test_remove_nonexistent_profile() {
        let mut mgr = ProviderProfileManager::new();
        assert!(!mgr.remove_profile("nonexistent"));
    }

    #[test]
    fn test_model_tier_parse_frontier() {
        assert_eq!(ModelTier::parse("claude-4-opus"), ModelTier::Frontier);
        assert_eq!(ModelTier::parse("gpt-5"), ModelTier::Frontier);
    }

    #[test]
    fn test_model_tier_parse_strong() {
        assert_eq!(ModelTier::parse("claude-sonnet-4"), ModelTier::Strong);
        assert_eq!(ModelTier::parse("gpt-4o"), ModelTier::Strong);
        assert_eq!(ModelTier::parse("deepseek-v3"), ModelTier::Strong);
    }

    #[test]
    fn test_model_tier_parse_capable() {
        assert_eq!(ModelTier::parse("claude-haiku"), ModelTier::Capable);
        assert_eq!(ModelTier::parse("gpt-4o-mini"), ModelTier::Capable);
        assert_eq!(ModelTier::parse("llama-70b"), ModelTier::Capable);
    }

    #[test]
    fn test_model_tier_parse_small() {
        assert_eq!(ModelTier::parse("phi-3"), ModelTier::Small);
        assert_eq!(ModelTier::parse("tiny-llama"), ModelTier::Small);
    }

    #[test]
    fn test_failover_deterministic_order() {
        let strategy = FailoverStrategy::new();
        let profiles = vec![
            ProviderProfile {
                name: "slow-llama".into(), provider_type: "ollama".into(),
                api_key: None, base_url: None, default_model: Some("llama3.1-8b".into()), is_active: false,
            },
            ProviderProfile {
                name: "fast-groq".into(), provider_type: "groq".into(),
                api_key: Some("key".into()), base_url: None, default_model: Some("llama-3.3-70b".into()), is_active: true,
            },
            ProviderProfile {
                name: "gemini-free".into(), provider_type: "google".into(),
                api_key: Some("key".into()), base_url: None, default_model: Some("gemini-2.0-flash".into()), is_active: false,
            },
        ];
        let excluded = HashSet::new();
        let result = strategy.find_failover("gpt-4o", &profiles, &excluded, 1);
        assert!(result.is_some());
        let (profile, marker) = result.expect("failover should find a candidate");
        // groq should be preferred (free-tier + capable model)
        assert_eq!(profile.name, "fast-groq");
        assert!(marker.contains("Failover 1"));
    }

    #[test]
    fn test_failover_excludes_failed() {
        let strategy = FailoverStrategy::new();
        let profiles = vec![
            ProviderProfile {
                name: "groq-1".into(), provider_type: "groq".into(),
                api_key: None, base_url: None, default_model: Some("llama-70b".into()), is_active: false,
            },
            ProviderProfile {
                name: "gemini-1".into(), provider_type: "google".into(),
                api_key: None, base_url: None, default_model: Some("gemini-2.0".into()), is_active: false,
            },
        ];
        let mut excluded: HashSet<String> = HashSet::new();
        excluded.insert("groq-1".to_string());
        let result = strategy.find_failover("gpt-4o", &profiles, &excluded, 2);
        assert!(result.is_some());
        assert_eq!(result.expect("failover should find gemini-1").0.name, "gemini-1");
    }

    #[test]
    fn test_failover_no_available() {
        let strategy = FailoverStrategy::new();
        let profiles = vec![];
        let result = strategy.find_failover("gpt-4", &profiles, &HashSet::new(), 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_failover_marker_format() {
        let strategy = FailoverStrategy::new();
        let profiles = vec![
            ProviderProfile {
                name: "backup".into(), provider_type: "ollama".into(),
                api_key: None, base_url: None, default_model: Some("llama3".into()), is_active: false,
            },
        ];
        let result = strategy.find_failover("gpt-4", &profiles, &HashSet::new(), 3);
        assert!(result.is_some());
        let marker = result.expect("failover should find a candidate for marker").1;
        assert_eq!(marker, "_(Failover 3: gpt-4 → llama3)");
    }

    #[test]
    fn test_provider_tier_heuristic() {
        let profile = ProviderProfile {
            name: "test".into(), provider_type: "ollama".into(),
            api_key: None, base_url: None, default_model: None, is_active: false,
        };
        assert_eq!(profile.effective_tier(), ModelTier::Moderate);

        let profile2 = ProviderProfile {
            name: "test".into(), provider_type: "openai".into(),
            api_key: None, base_url: None, default_model: None, is_active: false,
        };
        assert_eq!(profile2.effective_tier(), ModelTier::Frontier);
    }

    #[test]
    fn test_model_tier_label() {
        assert_eq!(ModelTier::Frontier.label(), "frontier");
        assert_eq!(ModelTier::Small.label(), "small");
    }

    #[test]
    fn test_provider_kind_free_tier_order() {
        assert!(ProviderKind::Groq.free_tier_order() < ProviderKind::Gemini.free_tier_order());
        assert!(ProviderKind::Gemini.free_tier_order() < ProviderKind::Cerebras.free_tier_order());
        assert!(ProviderKind::Ollama.free_tier_order() > ProviderKind::OpenRouter.free_tier_order());
    }

    #[test]
    fn test_failover_tier_preference() {
        let strategy = FailoverStrategy::new();
        let profiles = vec![
            ProviderProfile {
                name: "small".into(), provider_type: "ollama".into(),
                api_key: None, base_url: None, default_model: Some("phi-3".into()), is_active: false,
            },
            ProviderProfile {
                name: "same-tier".into(), provider_type: "openai".into(),
                api_key: None, base_url: None, default_model: Some("gpt-4o-mini".into()), is_active: false,
            },
        ];
        // gpt-4o-mini is Capable, llama-70b is also Capable
        let result = strategy.find_failover("llama-70b", &profiles, &HashSet::new(), 1);
        assert!(result.is_some());
        // same-tier (capable) should be preferred over small
        assert_eq!(result.expect("failover should prefer same-tier").0.name, "same-tier");
    }
}
