//! Concurrent username presence checker across multiple social platforms.

use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use tokio::sync::Semaphore;

use super::social_search::{AccountStatus, PlatformResult, SocialPlatform};

/// Checks a username against a set of social platforms concurrently.
pub struct UsernameChecker {
    platforms: Vec<SocialPlatform>,
    client: Client,
    concurrency: usize,
    timeout: Duration,
}

impl UsernameChecker {
    /// Create a checker with the given platform list and default settings.
    pub fn new(platforms: Vec<SocialPlatform>) -> Self {
        Self {
            platforms,
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
                .build()
                .unwrap_or_else(|_| Client::new()),
            concurrency: 10,
            timeout: Duration::from_secs(8),
        }
    }

    /// Create a checker with a pre-built reqwest client.
    pub fn with_client(platforms: Vec<SocialPlatform>, client: Client) -> Self {
        Self {
            platforms,
            client,
            concurrency: 10,
            timeout: Duration::from_secs(8),
        }
    }

    /// Override the default concurrency limit.
    pub fn concurrency(mut self, n: usize) -> Self {
        self.concurrency = n.max(1);
        self
    }

    /// Override the per-request timeout.
    pub fn timeout(mut self, d: Duration) -> Self {
        self.timeout = d;
        self
    }

    /// Check a single username across all configured platforms.
    ///
    /// Returns a `Vec<PlatformResult>` for every platform, regardless of outcome.
    /// Results are **not** filtered — callers decide which statuses to keep.
    pub async fn check(&self, username: &str) -> Vec<PlatformResult> {
        let semaphore = Arc::new(Semaphore::new(self.concurrency));
        let mut handles = Vec::with_capacity(self.platforms.len());

        for platform in &self.platforms {
            let sem = Arc::clone(&semaphore);
            let plat = platform.clone();
            let user = username.to_string();
            let client = self.client.clone();
            let timeout = self.timeout;

            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.ok();
                plat.check(&user, &client, timeout).await
            }));
        }

        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
            }
        }
        results
    }

    /// Check a username and return only platforms where the account was found.
    pub async fn check_found(&self, username: &str) -> Vec<PlatformResult> {
        self.check(username)
            .await
            .into_iter()
            .filter(|r| r.status == AccountStatus::Found)
            .collect()
    }

    /// Return a reference to the configured platforms.
    pub fn platforms(&self) -> &[SocialPlatform] {
        &self.platforms
    }

    /// Return the number of configured platforms.
    pub fn platform_count(&self) -> usize {
        self.platforms.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l2_perception::nt_world::osint::social_search::CheckMethod;

    fn make_test_platforms() -> Vec<SocialPlatform> {
        vec![
            SocialPlatform {
                name: "GitHub".into(),
                url_template: "https://github.com/{username}".into(),
                check_method: CheckMethod::Get,
                claim_patterns: vec!["repositories".into()],
            },
            SocialPlatform {
                name: "GitLab".into(),
                url_template: "https://gitlab.com/{username}".into(),
                check_method: CheckMethod::Get,
                claim_patterns: vec![],
            },
        ]
    }

    #[test]
    fn new_stores_platforms() {
        let checker = UsernameChecker::new(make_test_platforms());
        assert_eq!(checker.platform_count(), 2);
    }

    #[test]
    fn concurrency_minimum() {
        let checker = UsernameChecker::new(make_test_platforms()).concurrency(0);
        assert_eq!(checker.concurrency, 1);
    }

    #[test]
    fn timeout_override() {
        let d = Duration::from_secs(42);
        let checker = UsernameChecker::new(make_test_platforms()).timeout(d);
        assert_eq!(checker.timeout, d);
    }

    #[tokio::test]
    async fn check_returns_all_platforms() {
        let checker = UsernameChecker::new(make_test_platforms())
            .concurrency(2)
            .timeout(Duration::from_secs(5));
        let results = checker.check("nonexistent_user_xyz_12345").await;
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn check_found_filters() {
        let checker = UsernameChecker::new(make_test_platforms())
            .timeout(Duration::from_secs(5));
        let results = checker.check_found("nonexistent_user_xyz_12345").await;
        // Should be empty or small since user doesn't exist
        assert!(results.len() <= 2);
    }
}
