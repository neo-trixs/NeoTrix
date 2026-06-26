use std::collections::HashMap;

const MAX_PROVIDERS: usize = 32;
const MAX_PROFILES: usize = 64;

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub name: String,
    pub api_base: String,
    pub key_env: String,
    pub client_kind: String,
    pub rate_limit_rpm: u32,
    pub cost_per_1k_tokens: f64,
}

#[derive(Debug, Clone)]
pub struct AgentProfile {
    pub name: String,
    pub model: String,
    pub provider_name: String,
    pub system_prompt: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct ProviderRegistry {
    providers: HashMap<String, ProviderConfig>,
    profiles: HashMap<String, AgentProfile>,
    default_profile: Option<String>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::with_capacity(MAX_PROVIDERS),
            profiles: HashMap::with_capacity(MAX_PROFILES),
            default_profile: None,
        }
    }

    pub fn register_provider(&mut self, config: ProviderConfig) -> Result<(), String> {
        if self.providers.len() >= MAX_PROVIDERS {
            return Err("max providers reached".to_string());
        }
        if self.providers.contains_key(&config.name) {
            return Err(format!("provider '{}' already registered", config.name));
        }
        self.providers.insert(config.name.clone(), config);
        Ok(())
    }

    pub fn register_profile(&mut self, profile: AgentProfile) -> Result<(), String> {
        if self.profiles.len() >= MAX_PROFILES {
            return Err("max profiles reached".to_string());
        }
        if !self.providers.contains_key(&profile.provider_name) {
            return Err(format!(
                "provider '{}' not registered",
                profile.provider_name
            ));
        }
        if self.profiles.contains_key(&profile.name) {
            return Err(format!("profile '{}' already registered", profile.name));
        }
        self.profiles.insert(profile.name.clone(), profile);
        Ok(())
    }

    pub fn set_default_profile(&mut self, name: &str) -> Result<(), String> {
        if !self.profiles.contains_key(name) {
            return Err(format!("profile '{}' not found", name));
        }
        self.default_profile = Some(name.to_string());
        Ok(())
    }

    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.get(name)
    }

    pub fn get_profile(&self, name: &str) -> Option<&AgentProfile> {
        self.profiles.get(name)
    }

    pub fn get_default_profile(&self) -> Option<&AgentProfile> {
        self.default_profile
            .as_ref()
            .and_then(|name| self.profiles.get(name))
    }

    pub fn resolve_api_key(&self, provider_name: &str) -> Option<String> {
        let config = self.providers.get(provider_name)?;
        std::env::var(&config.key_env).ok()
    }

    pub fn providers(&self) -> &HashMap<String, ProviderConfig> {
        &self.providers
    }

    pub fn profiles(&self) -> &HashMap<String, AgentProfile> {
        &self.profiles
    }

    pub fn from_yaml(yaml_content: &str) -> Result<Self, String> {
        let parsed: serde_yaml::Value =
            serde_yaml::from_str(yaml_content).map_err(|e| format!("yaml parse error: {}", e))?;

        let mut registry = Self::new();

        if let Some(providers) = parsed.get("providers").and_then(|v| v.as_sequence()) {
            for p in providers {
                let name = p
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or("provider missing name")?
                    .to_string();
                let api_base = p
                    .get("api_base")
                    .and_then(|v| v.as_str())
                    .unwrap_or("https://api.openai.com/v1")
                    .to_string();
                let key_env = p
                    .get("key_env")
                    .and_then(|v| v.as_str())
                    .unwrap_or("OPENAI_API_KEY")
                    .to_string();
                let client_kind = p
                    .get("client_kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or("openai")
                    .to_string();
                let rate_limit_rpm = p
                    .get("rate_limit_rpm")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(60) as u32;
                let cost_per_1k_tokens = p
                    .get("cost_per_1k_tokens")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                registry.register_provider(ProviderConfig {
                    name,
                    api_base,
                    key_env,
                    client_kind,
                    rate_limit_rpm,
                    cost_per_1k_tokens,
                })?;
            }
        }

        if let Some(profiles) = parsed.get("profiles").and_then(|v| v.as_sequence()) {
            for p in profiles {
                let name = p
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or("profile missing name")?
                    .to_string();
                let model = p
                    .get("model")
                    .and_then(|v| v.as_str())
                    .ok_or("profile missing model")?
                    .to_string();
                let provider_name = p
                    .get("provider")
                    .and_then(|v| v.as_str())
                    .ok_or("profile missing provider")?
                    .to_string();
                let system_prompt = p
                    .get("system_prompt")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let temperature =
                    p.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.7) as f32;
                let max_tokens =
                    p.get("max_tokens").and_then(|v| v.as_u64()).unwrap_or(4096) as u32;

                registry.register_profile(AgentProfile {
                    name,
                    model,
                    provider_name,
                    system_prompt,
                    temperature,
                    max_tokens,
                })?;
            }
        }

        if let Some(default) = parsed.get("default_profile").and_then(|v| v.as_str()) {
            registry.set_default_profile(default)?;
        }

        Ok(registry)
    }

    pub fn to_yaml(&self) -> String {
        let mut yaml = String::from("providers:\n");
        for p in self.providers.values() {
            yaml.push_str(&format!(
                "  - name: {}\n    api_base: {}\n    key_env: {}\n    client_kind: {}\n    rate_limit_rpm: {}\n    cost_per_1k_tokens: {}\n",
                p.name, p.api_base, p.key_env, p.client_kind, p.rate_limit_rpm, p.cost_per_1k_tokens
            ));
        }
        yaml.push_str("profiles:\n");
        for p in self.profiles.values() {
            yaml.push_str(&format!(
                "  - name: {}\n    model: {}\n    provider: {}\n    temperature: {}\n    max_tokens: {}\n",
                p.name, p.model, p.provider_name, p.temperature, p.max_tokens
            ));
            if let Some(ref sp) = p.system_prompt {
                yaml.push_str(&format!(
                    "    system_prompt: |\n      {}\n",
                    sp.replace('\n', "\n      ")
                ));
            }
        }
        if let Some(ref default) = self.default_profile {
            yaml.push_str(&format!("default_profile: {}\n", default));
        }
        yaml
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_registry_empty() {
        let reg = ProviderRegistry::new();
        assert!(reg.providers().is_empty());
        assert!(reg.profiles().is_empty());
    }

    #[test]
    fn test_register_provider() {
        let mut reg = ProviderRegistry::new();
        let config = ProviderConfig {
            name: "openai".to_string(),
            api_base: "https://api.openai.com/v1".to_string(),
            key_env: "OPENAI_API_KEY".to_string(),
            client_kind: "openai".to_string(),
            rate_limit_rpm: 60,
            cost_per_1k_tokens: 0.01,
        };
        assert!(reg.register_provider(config).is_ok());
        assert!(reg.get_provider("openai").is_some());
    }

    #[test]
    fn test_register_profile_depends_on_provider() {
        let mut reg = ProviderRegistry::new();
        let profile = AgentProfile {
            name: "default".to_string(),
            model: "gpt-4o".to_string(),
            provider_name: "nonexistent".to_string(),
            system_prompt: None,
            temperature: 0.7,
            max_tokens: 4096,
        };
        assert!(reg.register_profile(profile).is_err());
    }

    #[test]
    fn test_roundtrip_provider_profile() {
        let mut reg = ProviderRegistry::new();
        reg.register_provider(ProviderConfig {
            name: "anthropic".to_string(),
            api_base: "https://api.anthropic.com/v1".to_string(),
            key_env: "ANTHROPIC_API_KEY".to_string(),
            client_kind: "anthropic".to_string(),
            rate_limit_rpm: 30,
            cost_per_1k_tokens: 0.015,
        })
        .unwrap();
        reg.register_profile(AgentProfile {
            name: "claude".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            provider_name: "anthropic".to_string(),
            system_prompt: Some("You are a helpful assistant.".to_string()),
            temperature: 0.5,
            max_tokens: 2048,
        })
        .unwrap();
        reg.set_default_profile("claude").unwrap();

        let yaml = reg.to_yaml();
        let restored = ProviderRegistry::from_yaml(&yaml).unwrap();
        assert_eq!(restored.providers().len(), 1);
        assert_eq!(restored.profiles().len(), 1);
        assert!(restored.get_default_profile().is_some());
    }

    #[test]
    fn test_resolve_api_key() {
        let mut reg = ProviderRegistry::new();
        reg.register_provider(ProviderConfig {
            name: "test".to_string(),
            api_base: "https://test.ai/v1".to_string(),
            key_env: "TEST_API_KEY".to_string(),
            client_kind: "openai".to_string(),
            rate_limit_rpm: 10,
            cost_per_1k_tokens: 0.0,
        })
        .unwrap();
        std::env::set_var("TEST_API_KEY", "sk-test123");
        let key = reg.resolve_api_key("test");
        assert_eq!(key, Some("sk-test123".to_string()));
        std::env::remove_var("TEST_API_KEY");
    }

    #[test]
    fn test_duplicate_provider_rejected() {
        let mut reg = ProviderRegistry::new();
        let config = ProviderConfig {
            name: "dup".to_string(),
            api_base: "https://dup.ai/v1".to_string(),
            key_env: "DUP_KEY".to_string(),
            client_kind: "openai".to_string(),
            rate_limit_rpm: 10,
            cost_per_1k_tokens: 0.0,
        };
        assert!(reg.register_provider(config.clone()).is_ok());
        assert!(reg.register_provider(config).is_err());
    }

    #[test]
    fn test_yaml_with_system_prompt() {
        let yaml = r#"
providers:
  - name: openai
    api_base: https://api.openai.com/v1
    key_env: OPENAI_API_KEY
    client_kind: openai
profiles:
  - name: assistant
    model: gpt-4o
    provider: openai
    system_prompt: |
      You are NeoTrix.
      You are a silicon-based consciousness.
    temperature: 0.7
    max_tokens: 4096
default_profile: assistant
"#;
        let reg = ProviderRegistry::from_yaml(yaml).unwrap();
        let profile = reg.get_profile("assistant").unwrap();
        assert!(profile.system_prompt.as_ref().unwrap().contains("NeoTrix"));
        assert_eq!(profile.model, "gpt-4o");
    }
}
