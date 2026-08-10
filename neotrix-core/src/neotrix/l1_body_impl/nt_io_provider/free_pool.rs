//! FreePool — Token budget tracker for free LLM providers.
//!
//! Tracks per-provider monthly token caps, daily request caps,
//! and estimates total USD savings from using free tiers.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeTokenBudget {
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
            "pollinations".to_string(),
            FreeTokenBudget {
                provider_name: "pollinations".into(),
                monthly_token_cap: 0,
                tokens_used: 0,
                daily_request_cap: 0,
                requests_used: 0,
                is_keyless: true,
                is_active: true,
            },
        );
        budgets.insert(
            "llm7".to_string(),
            FreeTokenBudget {
                provider_name: "llm7".into(),
                monthly_token_cap: 0,
                tokens_used: 0,
                daily_request_cap: 0,
                requests_used: 0,
                is_keyless: true,
                is_active: true,
            },
        );
        budgets.insert(
            "kilo".to_string(),
            FreeTokenBudget {
                provider_name: "kilo".into(),
                monthly_token_cap: 0,
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
                monthly_token_cap: 0,
                tokens_used: 0,
                daily_request_cap: 0,
                requests_used: 0,
                is_keyless: true,
                is_active: true,
            },
        );
        budgets.insert(
            "ovh".to_string(),
            FreeTokenBudget {
                provider_name: "ovh".into(),
                monthly_token_cap: 0,
                tokens_used: 0,
                daily_request_cap: 0,
                requests_used: 0,
                is_keyless: true,
                is_active: true,
            },
        );
        budgets.insert(
            "freetheai".to_string(),
            FreeTokenBudget {
                provider_name: "freetheai".into(),
                monthly_token_cap: 0,
                tokens_used: 0,
                daily_request_cap: 0,
                requests_used: 0,
                is_keyless: true,
                is_active: true,
            },
        );
        budgets.insert(
            "modelscope".to_string(),
            FreeTokenBudget {
                provider_name: "modelscope".into(),
                monthly_token_cap: 0,
                tokens_used: 0,
                daily_request_cap: 0,
                requests_used: 0,
                is_keyless: true,
                is_active: true,
            },
        );

        budgets.insert(
            "gemini".to_string(),
            FreeTokenBudget {
                provider_name: "gemini".into(),
                monthly_token_cap: 1_000_000,
                tokens_used: 0,
                daily_request_cap: 1500,
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
            "cerebras".to_string(),
            FreeTokenBudget {
                provider_name: "cerebras".into(),
                monthly_token_cap: 1_000_000,
                tokens_used: 0,
                daily_request_cap: 100,
                requests_used: 0,
                is_keyless: false,
                is_active: true,
            },
        );
        budgets.insert(
            "sambanova".to_string(),
            FreeTokenBudget {
                provider_name: "sambanova".into(),
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
            "deepseek-free".to_string(),
            FreeTokenBudget {
                provider_name: "deepseek-free".into(),
                monthly_token_cap: 500_000,
                tokens_used: 0,
                daily_request_cap: 100,
                requests_used: 0,
                is_keyless: false,
                is_active: true,
            },
        );
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
        assert!(budgets.len() >= 18, "should have at least 18 free provider budgets, got {}", budgets.len());
    }

    #[test]
    fn test_record_usage_updates_tokens() {
        let pool = FreePool::new();
        pool.record_usage("gemini", 1000);
        pool.record_usage("gemini", 500);
        let budget = pool.get_budget("gemini").expect("gemini budget exists");
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
        // Should sum all provider caps (gemini=1M, groq=500K, etc) - at least 5M
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
        pool.record_usage("gemini", 250_000);
        let after = pool.total_free_tokens_remaining();
        assert_eq!(before - after, 250_000);
    }

    #[test]
    fn test_total_savings_increases() {
        let pool = FreePool::new();
        assert_eq!(pool.total_savings(), 0.0);
        pool.record_usage("gemini", 1000);
        let savings = pool.total_savings();
        assert!(savings > 0.0, "savings should be positive after usage, got {}", savings);
    }

    #[test]
    fn test_keyless_providers_have_unlimited_budget() {
        let pool = FreePool::new();
        // 预算层契约 (与 gateway 注册解耦): 预算存在 ≠ 端点当前可用。
        // 2026-08-06 走代理实测: pollinations(匿名层关)/llm7(可用)/kilo(端点死)/
        // opencode-zen(需key)/ovh+freetheai+modelscope(DNS不可达)。
        // 保留 budget 使端点恢复时预算就绪; 可用性由 gateway 注册/探测决定。
        for name in &["pollinations", "llm7", "kilo", "opencode-zen", "ovh", "freetheai", "modelscope"] {
            let budget = pool.get_budget(name).unwrap_or_else(|| panic!("{} should have a budget", name));
            assert!(budget.is_keyless, "{} should be keyless", name);
            assert_eq!(budget.monthly_token_cap, 0, "{} should have unlimited cap", name);
        }
    }
}
