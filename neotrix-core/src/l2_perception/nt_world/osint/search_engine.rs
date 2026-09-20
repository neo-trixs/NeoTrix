//! High-level social search engine — orchestrates platform checks and produces a `SearchReport`.

use std::time::Duration;

use reqwest::Client;

use super::report::SearchReport;
use super::social_search::{SocialPlatform, default_platforms};
use super::username_checker::UsernameChecker;

/// Configuration for a social search run.
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Maximum concurrent HTTP requests.
    pub concurrency: usize,
    /// Per-request timeout.
    pub timeout: Duration,
    /// Risk score weight: fraction of platforms found / total (default 1.0).
    pub risk_weight: f64,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            concurrency: 10,
            timeout: Duration::from_secs(8),
            risk_weight: 1.0,
        }
    }
}

/// Orchestrates username presence checks across many social platforms.
pub struct SocialSearchEngine {
    checker: UsernameChecker,
    config: SearchConfig,
}

impl SocialSearchEngine {
    /// Create an engine with default 55+ platforms.
    pub fn new() -> Self {
        Self::with_platforms(default_platforms())
    }

    /// Create an engine with a custom platform list.
    pub fn with_platforms(platforms: Vec<SocialPlatform>) -> Self {
        let config = SearchConfig::default();
        let checker = UsernameChecker::new(platforms)
            .concurrency(config.concurrency)
            .timeout(config.timeout);
        Self { checker, config }
    }

    /// Create an engine with a pre-built client and custom config.
    pub fn with_client(client: Client, platforms: Vec<SocialPlatform>, config: SearchConfig) -> Self {
        let checker = UsernameChecker::with_client(platforms, client)
            .concurrency(config.concurrency)
            .timeout(config.timeout);
        Self { checker, config }
    }

    /// Create an engine from an existing checker with a new config.
    pub fn with_checker(checker: UsernameChecker, config: SearchConfig) -> Self {
        Self { checker, config }
    }

    /// Replace the config on an existing engine (rebuilds concurrency/timeout).
    pub fn with_config(self, config: SearchConfig) -> Self {
        let checker = UsernameChecker::new(self.checker.platforms().to_vec())
            .concurrency(config.concurrency)
            .timeout(config.timeout);
        Self { checker, config }
    }

    /// Search for `username` across all configured platforms.
    ///
    /// Returns a [`SearchReport`] with per-platform results and aggregate stats.
    pub async fn search(&self, username: &str) -> SearchReport {
        let results = self.checker.check(username).await;
        SearchReport::build(username, results, self.config.risk_weight)
    }

    /// Search and return only the platforms where the account was found.
    pub async fn search_found(&self, username: &str) -> SearchReport {
        let results = self.checker.check_found(username).await;
        SearchReport::build(username, results, self.config.risk_weight)
    }

    /// Number of platforms configured.
    pub fn platform_count(&self) -> usize {
        self.checker.platform_count()
    }
}

impl Default for SocialSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_engine_has_many_platforms() {
        let engine = SocialSearchEngine::new();
        assert!(engine.platform_count() >= 55);
    }

    #[test]
    fn search_config_defaults() {
        let cfg = SearchConfig::default();
        assert_eq!(cfg.concurrency, 10);
        assert_eq!(cfg.risk_weight, 1.0);
    }

    #[tokio::test]
    async fn search_returns_report() {
        let engine = SocialSearchEngine::new().with_config(SearchConfig {
            concurrency: 2,
            timeout: Duration::from_secs(5),
            risk_weight: 1.0,
        });
        let report = engine.search("test_user_xyz_99999").await;
        assert_eq!(report.platforms_checked, engine.platform_count());
        assert!(!report.results.is_empty());
    }

    #[tokio::test]
    async fn search_found_returns_subset() {
        let engine = SocialSearchEngine::new().with_config(SearchConfig {
            concurrency: 2,
            timeout: Duration::from_secs(5),
            risk_weight: 1.0,
        });
        let report = engine.search_found("test_user_xyz_99999").await;
        assert!(report.found_count <= report.platforms_checked);
    }
}
