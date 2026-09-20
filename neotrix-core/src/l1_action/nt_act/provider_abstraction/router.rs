use super::registry::ProviderRegistry;

pub struct CostAwareRouter {
    pub quality_threshold: f64,
}

impl CostAwareRouter {
    pub fn select_provider(&self, registry: &ProviderRegistry, task_complexity: f64) -> Option<String> {
        if registry.list_providers().is_empty() {
            return None;
        }

        let providers = registry.list_providers();
        let mut best: Option<(String, f64)> = None;

        for name in &providers {
            let config = match registry.get_config(name) {
                Some(c) => c,
                None => continue,
            };

            let score = if task_complexity < self.quality_threshold {
                1.0 - config.cost_per_1k_tokens
            } else {
                config.models.len() as f64 - config.cost_per_1k_tokens
            };

            match &best {
                None => best = Some((name.clone(), score)),
                Some((_, best_score)) if score > *best_score => best = Some((name.clone(), score)),
                _ => {}
            }
        }

        best.map(|(name, _)| name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{ProviderConfig, provider::{LlmProvider, CompletionRequest, CompletionResponse}};

    struct Stub { name: String }
    impl LlmProvider for Stub {
        fn complete(&self, _: &CompletionRequest) -> Result<CompletionResponse, String> {
            Ok(CompletionResponse { content: "".into(), tokens_used: 0, model: self.name.clone() })
        }
        fn name(&self) -> &str { &self.name }
    }

    fn make_reg(entries: Vec<(&str, f64, usize)>) -> ProviderRegistry {
        let mut reg = ProviderRegistry::new();
        for (name, cost, model_count) in entries {
            let models: Vec<String> = (0..model_count).map(|i| format!("m{i}")).collect();
            reg.register(
                name.into(),
                Box::new(Stub { name: name.into() }),
                ProviderConfig { name: name.into(), api_base: "".into(), api_key_env: "".into(), models, cost_per_1k_tokens: cost },
            );
        }
        reg
    }

    #[test]
    fn low_complexity_picks_cheapest() {
        let reg = make_reg(vec![("expensive", 0.1, 5), ("cheap", 0.001, 1)]);
        let router = CostAwareRouter { quality_threshold: 0.5 };
        let selected = router.select_provider(&reg, 0.1).unwrap();
        assert_eq!(selected, "cheap");
    }

    #[test]
    fn high_complexity_picks_most_models() {
        let reg = make_reg(vec![("small", 0.001, 1), ("large", 0.05, 10)]);
        let router = CostAwareRouter { quality_threshold: 0.5 };
        let selected = router.select_provider(&reg, 0.9).unwrap();
        assert_eq!(selected, "large");
    }

    #[test]
    fn empty_registry_returns_none() {
        let reg = ProviderRegistry::new();
        let router = CostAwareRouter { quality_threshold: 0.5 };
        assert!(router.select_provider(&reg, 0.5).is_none());
    }
}
