use std::collections::HashMap;

/// Rate limit profile for a provider (RPM = requests per minute, TPM = tokens per minute)
#[derive(Debug, Clone, Copy)]
pub struct RateProfile {
    pub rpm: f64,
    pub tpm: f64,
}

impl RateProfile {
    pub const fn new(rpm: f64, tpm: f64) -> Self {
        Self { rpm, tpm }
    }
}

/// Per-provider rate limit database
/// Sources: documented free tiers from OmniRoute, FreeLLMAPI, ProxyGateLLM ecosystem survey
pub fn free_provider_rate_profiles() -> HashMap<&'static str, RateProfile> {
    let mut m = HashMap::new();
    m.insert("openai", RateProfile::new(500.0, 200_000.0));
    m.insert("anthropic", RateProfile::new(200.0, 100_000.0));
    m.insert("gemini", RateProfile::new(15.0, 30_000.0));
    m.insert("groq", RateProfile::new(30.0, 50_000.0));
    m.insert("cerebras", RateProfile::new(30.0, 40_000.0));
    m.insert("sambanova", RateProfile::new(20.0, 30_000.0));
    m.insert("openrouter", RateProfile::new(60.0, 100_000.0));
    m.insert("cloudflare", RateProfile::new(50.0, 100_000.0));
    m.insert("nvidia", RateProfile::new(40.0, 60_000.0));
    m.insert("github-models", RateProfile::new(15.0, 30_000.0));
    m.insert("huggingface", RateProfile::new(30.0, 50_000.0));
    m.insert("together-free", RateProfile::new(30.0, 40_000.0));
    m.insert("siliconflow", RateProfile::new(60.0, 100_000.0));
    m.insert("zai", RateProfile::new(60.0, 100_000.0));
    m.insert("deepseek-free", RateProfile::new(30.0, 50_000.0));
    m.insert("ollama", RateProfile::new(1000.0, 1_000_000.0));
    m.insert("pollinations", RateProfile::new(30.0, 50_000.0));
    m.insert("freetheai", RateProfile::new(20.0, 30_000.0));
    m.insert("llm7", RateProfile::new(20.0, 30_000.0));
    m.insert("kilo", RateProfile::new(20.0, 30_000.0));
    m.insert("opencode-zen", RateProfile::new(30.0, 50_000.0));
    m.insert("ovh", RateProfile::new(20.0, 30_000.0));
    m.insert("modelscope", RateProfile::new(30.0, 50_000.0));
    m.insert("bazaarlink", RateProfile::new(30.0, 50_000.0));
    m.insert("zerolimit", RateProfile::new(30.0, 50_000.0));
    m
}

/// Get rate profile for a provider, or default if not known
pub fn get_rate_profile(provider_name: &str) -> RateProfile {
    let profiles = free_provider_rate_profiles();
    profiles.get(provider_name).copied().unwrap_or(RateProfile::new(30.0, 50_000.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_profiles_have_positive_rates() {
        let profiles = free_provider_rate_profiles();
        assert!(!profiles.is_empty(), "must have at least one profile");
        for (name, profile) in &profiles {
            assert!(profile.rpm > 0.0, "{} RPM must be positive", name);
            assert!(profile.tpm > 0.0, "{} TPM must be positive", name);
        }
    }

    #[test]
    fn test_known_profile_values() {
        let profiles = free_provider_rate_profiles();
        let gemini = profiles.get("gemini").expect("gemini profile exists");
        assert_eq!(gemini.rpm, 15.0);
        assert_eq!(gemini.tpm, 30_000.0);

        let groq = profiles.get("groq").expect("groq profile exists");
        assert_eq!(groq.rpm, 30.0);
        assert_eq!(groq.tpm, 50_000.0);

        let ollama = profiles.get("ollama").expect("ollama profile exists");
        assert_eq!(ollama.rpm, 1000.0);
        assert_eq!(ollama.tpm, 1_000_000.0);
    }

    #[test]
    fn test_get_rate_profile_known() {
        let profile = get_rate_profile("gemini");
        assert_eq!(profile.rpm, 15.0);
        assert_eq!(profile.tpm, 30_000.0);
    }

    #[test]
    fn test_get_rate_profile_unknown_returns_default() {
        let profile = get_rate_profile("nonexistent-provider-xyz");
        assert_eq!(profile.rpm, 30.0);
        assert_eq!(profile.tpm, 50_000.0);
    }

    #[test]
    fn test_all_free_providers_have_entries() {
        let profiles = free_provider_rate_profiles();
        for key in &[
            "openai", "anthropic", "gemini", "groq", "cerebras",
            "sambanova", "openrouter", "cloudflare", "nvidia",
            "github-models", "huggingface", "together-free",
            "siliconflow", "zai", "deepseek-free",
            "ollama", "pollinations", "freetheai", "llm7", "kilo",
            "opencode-zen", "ovh", "modelscope", "bazaarlink", "zerolimit",
        ] {
            assert!(profiles.contains_key(key), "missing profile for {}", key);
        }
    }
}
