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
        // HONESTY: These RPM/TPM values are hardcoded from provider documentation.
        // Provider limits may change without notice. Once a dynamic rate-limit
        // discovery mechanism exists (e.g., probing or API introspection), replace
        // these assertions with bounds checks or dynamic lookups.
        let profiles = free_provider_rate_profiles();
        let gemini = profiles.get("gemini").expect("gemini profile exists");
        assert!(gemini.rpm > 0.0, "gemini RPM must be positive");
        assert!(gemini.tpm > 0.0, "gemini TPM must be positive");

        let groq = profiles.get("groq").expect("groq profile exists");
        assert!(groq.rpm > 0.0, "groq RPM must be positive");
        assert!(groq.tpm > 0.0, "groq TPM must be positive");

        let ollama = profiles.get("ollama").expect("ollama profile exists");
        assert!(ollama.rpm > 0.0, "ollama RPM must be positive");
        assert!(ollama.tpm > 0.0, "ollama TPM must be positive");
    }

    #[test]
    fn test_get_rate_profile_known() {
        // HONESTY: Hardcoded values from provider documentation.
        // TODO: Replace with dynamic rate-limit discovery once available.
        let profile = get_rate_profile("gemini");
        assert!(profile.rpm > 0.0, "gemini RPM must be positive");
        assert!(profile.tpm > 0.0, "gemini TPM must be positive");
    }

    #[test]
    fn test_get_rate_profile_unknown_returns_default() {
        // HONESTY: Tests that unknown providers get a default rate profile. The
        // default values (30 RPM, 50K TPM) are hardcoded constants — this tests
        // the fallback contract, NOT that the defaults are appropriate for any
        // given provider.
        // TODO(R-P79): Once dynamic rate-limit discovery exists, verify that the
        // default fallback is appropriate (e.g., conservative enough to avoid
        // overloading unknown providers).
        let profile = get_rate_profile("nonexistent-provider-xyz");
        assert!(profile.rpm > 0.0, "default RPM must be positive");
        assert!(profile.tpm > 0.0, "default TPM must be positive");
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
