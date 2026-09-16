#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::Arc;

/// Supported authentication methods for platform adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthMethod {
    Cookie,
    Token,
    OAuth2,
    ApiKey,
}

/// Features a platform adapter may support.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformFeature {
    CustomerSync,
    EmailSync,
    WhatsAppSync,
    ContactSync,
}

/// Platform-specific configuration.
#[derive(Debug, Clone)]
pub struct PlatformConfig {
    pub base_url: String,
    pub auth_method: AuthMethod,
    pub rate_limit: u32,
    pub timeout_secs: u64,
}

/// Trait for platform adapters providing identity and feature declarations.
pub trait PlatformAdapter: Send + Sync {
    /// Unique platform identifier (e.g. "alibaba", "amazon").
    fn platform_id(&self) -> &str;

    /// Human-readable platform name.
    fn platform_name(&self) -> &str;

    /// List of features this adapter supports.
    fn supported_features(&self) -> Vec<PlatformFeature>;
}

/// Registry of platform adapters with associated configurations.
pub struct PlatformRegistry {
    platforms: HashMap<String, Arc<dyn PlatformAdapter>>,
    configs: HashMap<String, PlatformConfig>,
}

impl Default for PlatformRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            platforms: HashMap::new(),
            configs: HashMap::new(),
        }
    }

    /// Register a platform adapter with its configuration.
    /// Overwrites any previous adapter for the same platform id.
    pub fn register(&mut self, adapter: Arc<dyn PlatformAdapter>, config: PlatformConfig) {
        let id = adapter.platform_id().to_string();
        self.platforms.insert(id.clone(), adapter);
        self.configs.insert(id, config);
    }

    /// Get a platform adapter by id.
    pub fn get(&self, platform: &str) -> Option<Arc<dyn PlatformAdapter>> {
        self.platforms.get(platform).cloned()
    }

    /// Get platform config by id.
    pub fn get_config(&self, platform: &str) -> Option<&PlatformConfig> {
        self.configs.get(platform)
    }

    /// List all registered platform ids.
    pub fn list_platforms(&self) -> Vec<String> {
        self.platforms.keys().cloned().collect()
    }

    /// Check if a platform supports a given feature.
    pub fn supports_feature(&self, platform: &str, feature: PlatformFeature) -> bool {
        self.platforms
            .get(platform)
            .map(|a| a.supported_features().contains(&feature))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestAdapter {
        id: String,
        name: String,
        features: Vec<PlatformFeature>,
    }

    impl PlatformAdapter for TestAdapter {
        fn platform_id(&self) -> &str {
            &self.id
        }
        fn platform_name(&self) -> &str {
            &self.name
        }
        fn supported_features(&self) -> Vec<PlatformFeature> {
            self.features.clone()
        }
    }

    fn test_config() -> PlatformConfig {
        PlatformConfig {
            base_url: "https://example.com".into(),
            auth_method: AuthMethod::Token,
            rate_limit: 100,
            timeout_secs: 30,
        }
    }

    #[test]
    fn test_empty_registry() {
        let reg = PlatformRegistry::new();
        assert!(reg.list_platforms().is_empty());
        assert!(reg.get("nope").is_none());
    }

    #[test]
    fn test_register_and_get() {
        let mut reg = PlatformRegistry::new();
        let adapter = Arc::new(TestAdapter {
            id: "alibaba".into(),
            name: "Alibaba".into(),
            features: vec![PlatformFeature::CustomerSync, PlatformFeature::EmailSync],
        });
        reg.register(adapter, test_config());

        assert_eq!(reg.list_platforms().len(), 1);
        assert!(reg.get("alibaba").is_some());
        assert_eq!(reg.get("alibaba").unwrap().platform_name(), "Alibaba");
    }

    #[test]
    fn test_get_config() {
        let mut reg = PlatformRegistry::new();
        let adapter = Arc::new(TestAdapter {
            id: "amazon".into(),
            name: "Amazon".into(),
            features: vec![],
        });
        reg.register(adapter, test_config());

        let cfg = reg.get_config("amazon").unwrap();
        assert_eq!(cfg.base_url, "https://example.com");
        assert_eq!(cfg.auth_method, AuthMethod::Token);
        assert_eq!(cfg.rate_limit, 100);
        assert_eq!(cfg.timeout_secs, 30);
    }

    #[test]
    fn test_supports_feature() {
        let mut reg = PlatformRegistry::new();
        let adapter = Arc::new(TestAdapter {
            id: "whatsapp".into(),
            name: "WhatsApp".into(),
            features: vec![PlatformFeature::WhatsAppSync, PlatformFeature::ContactSync],
        });
        reg.register(adapter, test_config());

        assert!(reg.supports_feature("whatsapp", PlatformFeature::WhatsAppSync));
        assert!(reg.supports_feature("whatsapp", PlatformFeature::ContactSync));
        assert!(!reg.supports_feature("whatsapp", PlatformFeature::EmailSync));
        assert!(!reg.supports_feature("unknown", PlatformFeature::WhatsAppSync));
    }

    #[test]
    fn test_overwrite_registration() {
        let mut reg = PlatformRegistry::new();
        let adapter1 = Arc::new(TestAdapter {
            id: "x".into(),
            name: "X v1".into(),
            features: vec![],
        });
        let adapter2 = Arc::new(TestAdapter {
            id: "x".into(),
            name: "X v2".into(),
            features: vec![],
        });

        reg.register(adapter1, test_config());
        reg.register(adapter2, test_config());

        assert_eq!(reg.list_platforms().len(), 1);
        assert_eq!(reg.get("x").unwrap().platform_name(), "X v2");
    }

    #[test]
    fn test_list_platforms() {
        let mut reg = PlatformRegistry::new();
        for id in &["a", "b", "c"] {
            let adapter = Arc::new(TestAdapter {
                id: id.to_string(),
                name: id.to_string(),
                features: vec![],
            });
            reg.register(adapter, test_config());
        }

        let mut platforms = reg.list_platforms();
        platforms.sort();
        assert_eq!(platforms, vec!["a", "b", "c"]);
    }
}
