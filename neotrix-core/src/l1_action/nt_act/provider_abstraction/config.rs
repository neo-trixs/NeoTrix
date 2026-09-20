#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub name: String,
    pub api_base: String,
    pub api_key_env: String,
    pub models: Vec<String>,
    pub cost_per_1k_tokens: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_config_creation() {
        let cfg = ProviderConfig {
            name: "openai".into(),
            api_base: "https://api.openai.com/v1".into(),
            api_key_env: "OPENAI_API_KEY".into(),
            models: vec!["gpt-4".into(), "gpt-3.5-turbo".into()],
            cost_per_1k_tokens: 0.03,
        };
        assert_eq!(cfg.name, "openai");
        assert_eq!(cfg.models.len(), 2);
        assert!((cfg.cost_per_1k_tokens - 0.03).abs() < f64::EPSILON);
    }

    #[test]
    fn provider_config_clone() {
        let cfg = ProviderConfig {
            name: "anthropic".into(),
            api_base: "https://api.anthropic.com".into(),
            api_key_env: "ANTHROPIC_API_KEY".into(),
            models: vec!["claude-3-opus".into()],
            cost_per_1k_tokens: 0.015,
        };
        let cloned = cfg.clone();
        assert_eq!(cloned.name, cfg.name);
        assert_eq!(cloned.cost_per_1k_tokens, cfg.cost_per_1k_tokens);
    }
}
