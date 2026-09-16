//! Anti-Detection Bridge — connects social_access HTTP pool with NT-SHIELD stealth_net
//!
//! Wraps `StealthHttpClient` (L3 Embodiment) and exposes it through the social_access
//! interface, providing browser fingerprint rotation, proxy support, request header
//! randomization, and persona camouflage for social media access.

use std::sync::Arc;
use std::time::Duration;

use crate::l3_embodiment::nt_shield::nt_shield_stealth_net::{Response, StealthHttpClient};

/// Anti-detect configuration for social media access.
///
/// Maps social_access concepts to NT-SHIELD `StealthHttpClient` features.
#[derive(Debug, Clone)]
pub struct AntiDetectConfig {
    /// Enable TLS fingerprint rotation (JA3/JA4 variant switching).
    pub enable_fingerprint_rotation: bool,
    /// Proxy URI — supports http, https, socks5, and "tor" keyword.
    pub proxy_uri: Option<String>,
    /// Interval between fingerprint rotations in seconds.
    pub rotate_interval_secs: u64,
    /// Enable persona camouflage (randomized real-browser headers).
    pub enable_persona: bool,
    /// Specific persona key; empty string = weighted random selection.
    pub persona_key: String,
    /// Enable tracker/ads domain blocking.
    pub enable_tracker_check: bool,
    /// TLS certificate verification skip (dev/testing only).
    pub tls_insecure: bool,
}

impl Default for AntiDetectConfig {
    fn default() -> Self {
        Self {
            enable_fingerprint_rotation: true,
            proxy_uri: None,
            rotate_interval_secs: 300,
            enable_persona: true,
            persona_key: String::new(),
            enable_tracker_check: true,
            tls_insecure: false,
        }
    }
}

/// Anti-detection aware HTTP client backed by NT-SHIELD `StealthHttpClient`.
///
/// Wraps the full stealth pipeline (fingerprint rotation, proxy chains,
/// persona camouflage, tracker blocking) behind a simple `fetch` interface.
pub struct AntiDetectHttpPool {
    inner: Arc<StealthHttpClient>,
    config: AntiDetectConfig,
}

impl AntiDetectHttpPool {
    /// Build a new pool from config. Applies builder settings before wrapping in Arc.
    pub fn new(config: AntiDetectConfig) -> Self {
        let client = match config.proxy_uri.as_deref() {
            Some("tor") | Some("tor://") => StealthHttpClient::with_tor()
                .with_tracker_check(config.enable_tracker_check)
                .with_tls_insecure(config.tls_insecure),
            Some(url) => StealthHttpClient::with_proxy(Some(url))
                .with_tracker_check(config.enable_tracker_check)
                .with_tls_insecure(config.tls_insecure),
            None => StealthHttpClient::new()
                .with_tracker_check(config.enable_tracker_check)
                .with_tls_insecure(config.tls_insecure),
        };

        Self {
            inner: Arc::new(client),
            config,
        }
    }

    /// Wrap an existing `StealthHttpClient` (e.g. shared from NT-SHIELD manager).
    pub fn from_stealth_client(client: Arc<StealthHttpClient>, config: AntiDetectConfig) -> Self {
        Self {
            inner: client,
            config,
        }
    }

    /// Initialize persona camouflage. Call after construction if `enable_persona` is set.
    pub async fn init(&self) {
        if self.config.enable_persona {
            self.inner.enable_persona(&self.config.persona_key).await;
        }
    }

    /// Get a reference to the underlying `StealthHttpClient`.
    pub fn stealth_client(&self) -> &StealthHttpClient {
        &self.inner
    }

    /// Fetch a URL through the stealth pipeline (auto-rotates fingerprint, applies proxy).
    pub async fn fetch(&self, url: &str) -> Result<Response, String> {
        self.inner.fetch(url).await
    }

    /// Trigger an immediate fingerprint rotation.
    pub async fn rotate_fingerprint(&self) {
        self.inner.rotate_fingerprint().await;
    }

    /// Trigger an immediate TLS variant rotation.
    pub async fn rotate_tls(&self) {
        self.inner.rotate_tls_variant().await;
    }

    /// Replace persona camouflage at runtime.
    pub async fn set_persona(&self, persona_key: &str) {
        self.inner.enable_persona(persona_key).await;
    }

    /// Disable persona camouflage, restoring system fingerprint headers.
    pub async fn disable_persona(&self) {
        self.inner.disable_persona().await;
    }

    /// Replace extra headers at runtime.
    pub async fn set_extra_headers(&self, headers: std::collections::HashMap<String, String>) {
        self.inner.set_extra_headers(headers).await;
    }

    /// Toggle TLS certificate verification (takes effect on next client rebuild).
    pub async fn set_tls_insecure(&self, insecure: bool) {
        self.inner.set_tls_insecure(insecure).await;
    }

    /// Run the auto-rotation cycle (fingerprint + TLS) — call periodically.
    pub async fn auto_rotate(&self) {
        self.inner.auto_rotate_fingerprint().await;
    }

    /// Compute the rotation interval as a `Duration`.
    pub fn rotation_interval(&self) -> Duration {
        Duration::from_secs(self.config.rotate_interval_secs)
    }
}

impl std::fmt::Debug for AntiDetectHttpPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AntiDetectHttpPool")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = AntiDetectConfig::default();
        assert!(cfg.enable_fingerprint_rotation);
        assert!(cfg.proxy_uri.is_none());
        assert_eq!(cfg.rotate_interval_secs, 300);
        assert!(cfg.enable_persona);
        assert!(cfg.enable_tracker_check);
        assert!(!cfg.tls_insecure);
    }

    #[test]
    fn test_pool_creation_no_proxy() {
        let pool = AntiDetectHttpPool::new(AntiDetectConfig::default());
        assert!(pool.config.proxy_uri.is_none());
    }

    #[test]
    fn test_pool_creation_with_proxy() {
        let cfg = AntiDetectConfig {
            proxy_uri: Some("http://proxy:8080".to_string()),
            ..Default::default()
        };
        let pool = AntiDetectHttpPool::new(cfg);
        assert_eq!(pool.config.proxy_uri.as_deref(), Some("http://proxy:8080"));
    }

    #[test]
    fn test_rotation_interval() {
        let cfg = AntiDetectConfig {
            rotate_interval_secs: 600,
            ..Default::default()
        };
        let pool = AntiDetectHttpPool::new(cfg);
        assert_eq!(pool.rotation_interval(), Duration::from_secs(600));
    }
}
