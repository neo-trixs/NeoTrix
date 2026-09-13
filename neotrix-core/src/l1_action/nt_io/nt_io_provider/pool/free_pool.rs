//! FreePool — Token budget tracker for free LLM providers.
//!
//! Tracks per-provider monthly token caps, daily request caps,
//! and estimates total USD savings from using free tiers.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FreeTokenBudget {
    pub provider_name: String,
    pub monthly_token_cap: u64,
    pub tokens_used: u64,
    pub daily_request_cap: u64,
    pub requests_used: u64,
    pub is_keyless: bool,
    pub is_active: bool,
}

pub struct FreePool {
    budgets: RwLock<HashMap<String, FreeTokenBudget>>,
    total_saved: RwLock<f64>,
}

impl FreePool {
    pub fn new() -> Self {
        let mut budgets = HashMap::new();

        budgets.insert(
            "openrouter".to_string(),
            FreeTokenBudget {
                provider_name: "openrouter".into(),
                monthly_token_cap: 200_000,
                tokens_used: 0,
                daily_request_cap: 100,
                requests_used: 0,
                is_keyless: false,
                is_active: true,
            },
        );
        budgets.insert(
            "groq".to_string(),
            FreeTokenBudget {
                provider_name: "groq".into(),
                monthly_token_cap: 500_000,
                tokens_used: 0,
                daily_request_cap: 100,
                requests_used: 0,
                is_keyless: false,
                is_active: true,
            },
        );
        budgets.insert(
            "cloudflare".to_string(),
            FreeTokenBudget {
                provider_name: "cloudflare".into(),
                monthly_token_cap: 10_000_000,
                tokens_used: 0,
                daily_request_cap: 10_000,
                requests_used: 0,
                is_keyless: false,
                is_active: true,
            },
        );
        budgets.insert(
            "nvidia".to_string(),
            FreeTokenBudget {
                provider_name: "nvidia".into(),
                monthly_token_cap: 1_000_000,
                tokens_used: 0,
                daily_request_cap: 1000,
                requests_used: 0,
                is_keyless: false,
                is_active: true,
            },
        );
        budgets.insert(
            "github-models".to_string(),
            FreeTokenBudget {
                provider_name: "github-models".into(),
                monthly_token_cap: 500_000,
                tokens_used: 0,
                daily_request_cap: 300,
                requests_used: 0,
                is_keyless: false,
                is_active: true,
            },
        );
        budgets.insert(
            "huggingface".to_string(),
            FreeTokenBudget {
                provider_name: "huggingface".into(),
                monthly_token_cap: 200_000,
                tokens_used: 0,
                daily_request_cap: 100,
                requests_used: 0,
                is_keyless: false,
                is_active: true,
            },
        );
        budgets.insert(
            "llm7".to_string(),
            FreeTokenBudget {
                provider_name: "llm7".into(),
                monthly_token_cap: 1_000_000,
                tokens_used: 0,
                daily_request_cap: 0,
                requests_used: 0,
                is_keyless: true,
                is_active: true,
            },
        );
        budgets.insert(
            "opencode-zen".to_string(),
            FreeTokenBudget {
                provider_name: "opencode-zen".into(),
                monthly_token_cap: 2_000_000,
                tokens_used: 0,
                daily_request_cap: 0,
                requests_used: 0,
                is_keyless: true,
                is_active: true,
            },
        );
        budgets.insert(
            "together-free".to_string(),
            FreeTokenBudget {
                provider_name: "together-free".into(),
                monthly_token_cap: 500_000,
                tokens_used: 0,
                daily_request_cap: 100,
                requests_used: 0,
                is_keyless: false,
                is_active: true,
            },
        );
        budgets.insert(
            "siliconflow".to_string(),
            FreeTokenBudget {
                provider_name: "siliconflow".into(),
                monthly_token_cap: 2_000_000,
                tokens_used: 0,
                daily_request_cap: 500,
                requests_used: 0,
                is_keyless: false,
                is_active: true,
            },
        );
        budgets.insert(
            "zai".to_string(),
            FreeTokenBudget {
                provider_name: "zai".into(),
                monthly_token_cap: 1_000_000,
                tokens_used: 0,
                daily_request_cap: 300,
                requests_used: 0,
                is_keyless: false,
                is_active: true,
            },
        );
        budgets.insert(
            "api-airforce".to_string(),
            FreeTokenBudget {
                provider_name: "api-airforce".into(),
                monthly_token_cap: 1_000_000,
                tokens_used: 0,
                daily_request_cap: 0,
                requests_used: 0,
                is_keyless: true,
                is_active: true,
            },
        );

        Self {
            budgets: RwLock::new(budgets),
            total_saved: RwLock::new(0.0),
        }
    }

    pub fn record_usage(&self, provider_name: &str, tokens: u64) {
        if let Ok(mut budgets) = self.budgets.write() {
            if let Some(budget) = budgets.get_mut(provider_name) {
                budget.tokens_used = budget.tokens_used.saturating_add(tokens);
                budget.requests_used = budget.requests_used.saturating_add(1);
            }
        }
        if let Ok(mut saved) = self.total_saved.write() {
            *saved += (tokens as f64 / 1000.0) * 0.01;
        }
    }

    pub fn get_budget(&self, provider_name: &str) -> Option<FreeTokenBudget> {
        self.budgets
            .read()
            .ok()
            .and_then(|b| b.get(provider_name).cloned())
    }

    pub fn all_budgets(&self) -> Vec<FreeTokenBudget> {
        self.budgets
            .read()
            .ok()
            .map(|b| b.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn total_savings(&self) -> f64 {
        self.total_saved.read().map(|s| *s).unwrap_or(0.0)
    }

    pub fn total_free_tokens_remaining(&self) -> u64 {
        self.budgets
            .read()
            .ok()
            .map(|b| {
                b.values()
                    .filter(|b| b.monthly_token_cap > 0)
                    .map(|b| b.monthly_token_cap.saturating_sub(b.tokens_used))
                    .sum()
            })
            .unwrap_or(0)
    }
}

/// Global shared FreePool singleton
pub fn global_free_pool() -> &'static FreePool {
    static POOL: OnceLock<FreePool> = OnceLock::new();
    POOL.get_or_init(|| {
        log::info!("[free_pool] Initialized global FreePool singleton");
        FreePool::new()
    })
}

impl Default for FreePool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_pool_initializes_all_budgets() {
        let pool = FreePool::new();
        let budgets = pool.all_budgets();
        assert_eq!(budgets.len(), 12, "should have 12 free provider budgets (matching FreeModelCatalog), got {}", budgets.len());
    }

    #[test]
    fn test_record_usage_updates_tokens() {
        let pool = FreePool::new();
        pool.record_usage("groq", 1000);
        pool.record_usage("groq", 500);
        let budget = pool.get_budget("groq").expect("groq budget exists");
        assert_eq!(budget.tokens_used, 1500);
        assert_eq!(budget.requests_used, 2);
    }

    #[test]
    fn test_record_usage_unknown_provider_no_panic() {
        let pool = FreePool::new();
        pool.record_usage("nonexistent-provider", 100);
        // should not panic, just no-op
        assert!(pool.get_budget("nonexistent-provider").is_none());
    }

    #[test]
    fn test_total_free_tokens_remaining() {
        let pool = FreePool::new();
        let total = pool.total_free_tokens_remaining();
        // Sum of caps: groq=500K + cloudflare=10M + nvidia=1M + github=500K + huggingface=200K + together=500K + siliconflow=2M + zai=1M + openrouter=200K = ~15.9M
        assert!(
            total >= 5_000_000,
            "total free tokens should be >= 5M, got {}",
            total
        );
    }

    #[test]
    fn test_total_free_tokens_decreases_after_usage() {
        let pool = FreePool::new();
        let before = pool.total_free_tokens_remaining();
        pool.record_usage("cloudflare", 250_000);
        let after = pool.total_free_tokens_remaining();
        assert_eq!(before - after, 250_000);
    }

    #[test]
    fn test_total_savings_increases() {
        let pool = FreePool::new();
        assert_eq!(pool.total_savings(), 0.0);
        pool.record_usage("groq", 1000);
        let savings = pool.total_savings();
        assert!(savings > 0.0, "savings should be positive after usage, got {}", savings);
    }

    #[test]
    fn test_keyless_providers_have_budget_caps() {
        let pool = FreePool::new();
        let expected_caps = [("llm7", 1_000_000), ("opencode-zen", 2_000_000), ("api-airforce", 1_000_000)];
        for (name, expected_cap) in &expected_caps {
            let budget = pool.get_budget(name).unwrap_or_else(|| panic!("{} should have a budget", name));
            assert!(budget.is_keyless, "{} should be keyless", name);
            assert_eq!(budget.monthly_token_cap, *expected_cap, "{} should have cap {}", name, expected_cap);
        }
    }
}
