use std::collections::HashMap;
use super::{LlmProvider, config::ProviderConfig};

pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn LlmProvider>>,
    configs: HashMap<String, ProviderConfig>,
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            configs: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, provider: Box<dyn LlmProvider>, config: ProviderConfig) {
        self.configs.insert(name.clone(), config);
        self.providers.insert(name, provider);
    }

    pub fn get(&self, name: &str) -> Option<&dyn LlmProvider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    pub fn get_config(&self, name: &str) -> Option<&ProviderConfig> {
        self.configs.get(name)
    }

    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub fn cheapest_for_task(&self, _task_type: &str) -> Option<(&str, f64)> {
        self.configs
            .iter()
            .min_by(|a, b| a.1.cost_per_1k_tokens.partial_cmp(&b.1.cost_per_1k_tokens).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, cfg)| (name.as_str(), cfg.cost_per_1k_tokens))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::provider::{CompletionRequest, CompletionResponse, Message};

    struct StubProvider {
        name: String,
    }

    impl LlmProvider for StubProvider {
        fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, String> {
            Ok(CompletionResponse { content: "ok".into(), tokens_used: 1, model: "stub".into() })
        }
        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn register_and_get() {
        let mut reg = ProviderRegistry::new();
        reg.register(
            "openai".into(),
            Box::new(StubProvider { name: "openai".into() }),
            ProviderConfig { name: "openai".into(), api_base: "".into(), api_key_env: "".into(), models: vec![], cost_per_1k_tokens: 0.03 },
        );
        assert!(reg.get("openai").is_some());
        assert!(reg.get("anthropic").is_none());
    }

    #[test]
    fn list_providers() {
        let mut reg = ProviderRegistry::new();
        reg.register("a".into(), Box::new(StubProvider { name: "a".into() }), ProviderConfig { name: "a".into(), api_base: "".into(), api_key_env: "".into(), models: vec![], cost_per_1k_tokens: 0.01 });
        reg.register("b".into(), Box::new(StubProvider { name: "b".into() }), ProviderConfig { name: "b".into(), api_base: "".into(), api_key_env: "".into(), models: vec![], cost_per_1k_tokens: 0.02 });
        let mut names = reg.list_providers();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn cheapest_for_task() {
        let mut reg = ProviderRegistry::new();
        reg.register("expensive".into(), Box::new(StubProvider { name: "expensive".into() }), ProviderConfig { name: "expensive".into(), api_base: "".into(), api_key_env: "".into(), models: vec![], cost_per_1k_tokens: 0.1 });
        reg.register("cheap".into(), Box::new(StubProvider { name: "cheap".into() }), ProviderConfig { name: "cheap".into(), api_base: "".into(), api_key_env: "".into(), models: vec![], cost_per_1k_tokens: 0.001 });
        let (name, cost) = reg.cheapest_for_task("any").unwrap();
        assert_eq!(name, "cheap");
        assert!((cost - 0.001).abs() < f64::EPSILON);
    }
}
