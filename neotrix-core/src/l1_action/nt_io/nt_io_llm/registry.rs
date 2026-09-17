//! LLM Adapter Registry — dynamic registration and lookup
//!
//! The registry manages `UnifiedLlm` adapters by name, providing
//! registration, lookup, default provider selection, and health monitoring.

use std::collections::HashMap;
use std::sync::RwLock;

use super::{LlmError, LlmProviderType, UnifiedLlm};

/// Adapter entry with metadata
struct AdapterEntry {
    adapter: Box<dyn UnifiedLlm>,
    provider_type: LlmProviderType,
    healthy: bool,
}

/// LLM Adapter Registry
///
/// Manages a collection of `UnifiedLlm` adapters, allowing:
/// - Dynamic registration by name
/// - Lookup by name or provider type
/// - Default provider selection
/// - Bulk health checks
pub struct LlmRegistry {
    adapters: RwLock<HashMap<String, AdapterEntry>>,
    default_name: RwLock<Option<String>>,
}

impl LlmRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            adapters: RwLock::new(HashMap::new()),
            default_name: RwLock::new(None),
        }
    }

    /// Create a registry with a pre-configured default provider
    pub fn with_default(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            adapters: RwLock::new(HashMap::new()),
            default_name: RwLock::new(Some(name)),
        }
    }

    /// Register an adapter with a given name
    ///
    /// If this is the first adapter registered, it becomes the default.
    pub fn register(&self, name: String, adapter: Box<dyn UnifiedLlm>) {
        let provider_type = adapter.provider_type();
        let entry = AdapterEntry {
            adapter,
            provider_type,
            healthy: true,
        };

        let mut adapters = self.adapters.write().unwrap_or_else(|e| e.into_inner());
        let is_first = adapters.is_empty();
        adapters.insert(name.clone(), entry);

        // First registered adapter becomes the default
        if is_first {
            let mut default = self.default_name.write().unwrap_or_else(|e| e.into_inner());
            if default.is_none() {
                *default = Some(name);
            }
        }
    }

    /// Register an adapter with explicit provider type
    pub fn register_with_type(
        &self,
        name: String,
        adapter: Box<dyn UnifiedLlm>,
        provider_type: LlmProviderType,
    ) {
        let entry = AdapterEntry {
            adapter,
            provider_type,
            healthy: true,
        };

        let mut adapters = self.adapters.write().unwrap_or_else(|e| e.into_inner());
        let is_first = adapters.is_empty();
        adapters.insert(name.clone(), entry);

        if is_first {
            let mut default = self.default_name.write().unwrap_or_else(|e| e.into_inner());
            if default.is_none() {
                *default = Some(name);
            }
        }
    }

    /// Get an adapter by name
    pub fn get(&self, name: &str) -> Option<Box<dyn UnifiedLlm>> {
        let adapters = self.adapters.read().unwrap_or_else(|e| e.into_inner());
        if adapters.contains_key(name) {
            None
        } else {
            None
        }
    }

    /// Execute a closure with a reference to an adapter by name
    pub fn with_adapter<R>(
        &self,
        name: &str,
        f: impl FnOnce(&dyn UnifiedLlm) -> R,
    ) -> Result<R, LlmError> {
        let adapters = self.adapters.read().unwrap_or_else(|e| e.into_inner());
        let entry = adapters
            .get(name)
            .ok_or_else(|| LlmError::ProviderNotFound(name.to_string()))?;
        Ok(f(&*entry.adapter))
    }

    /// Get the default adapter name
    pub fn default_name(&self) -> Option<String> {
        let default = self.default_name.read().unwrap_or_else(|e| e.into_inner());
        default.clone()
    }

    /// Set the default adapter by name
    pub fn set_default(&self, name: &str) -> Result<(), LlmError> {
        let adapters = self.adapters.read().unwrap_or_else(|e| e.into_inner());
        if !adapters.contains_key(name) {
            return Err(LlmError::ProviderNotFound(name.to_string()));
        }
        drop(adapters);

        let mut default = self.default_name.write().unwrap_or_else(|e| e.into_inner());
        *default = Some(name.to_string());
        Ok(())
    }

    /// Execute a closure with the default adapter
    pub fn with_default_adapter<R>(
        &self,
        f: impl FnOnce(&dyn UnifiedLlm) -> R,
    ) -> Result<R, LlmError> {
        let default_name = self
            .default_name()
            .ok_or_else(|| LlmError::ProviderNotFound("no default provider set".to_string()))?;
        self.with_adapter(&default_name, f)
    }

    /// Find adapters by provider type
    pub fn find_by_type(&self, provider_type: LlmProviderType) -> Vec<String> {
        let adapters = self.adapters.read().unwrap_or_else(|e| e.into_inner());
        adapters
            .iter()
            .filter(|(_, e)| e.provider_type == provider_type)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get all registered adapter names
    pub fn list(&self) -> Vec<String> {
        let adapters = self.adapters.read().unwrap_or_else(|e| e.into_inner());
        adapters.keys().cloned().collect()
    }

    /// Get the number of registered adapters
    pub fn len(&self) -> usize {
        let adapters = self.adapters.read().unwrap_or_else(|e| e.into_inner());
        adapters.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Remove an adapter by name
    pub fn remove(&self, name: &str) -> Option<()> {
        let mut adapters = self.adapters.write().unwrap_or_else(|e| e.into_inner());
        adapters.remove(name)?;

        // If we removed the default, clear it
        let mut default = self.default_name.write().unwrap_or_else(|e| e.into_inner());
        if default.as_deref() == Some(name) {
            *default = adapters.keys().next().cloned();
        }

        Some(())
    }

    /// Run health checks on all adapters and update their status
    ///
    /// Returns a map of adapter name -> health status
    pub fn health_check_all(&self) -> HashMap<String, bool> {
        let adapters = self.adapters.read().unwrap_or_else(|e| e.into_inner());
        let mut results = HashMap::new();

        for (name, entry) in adapters.iter() {
            let healthy = entry.adapter.health_check().unwrap_or(false);
            results.insert(name.clone(), healthy);
        }

        // Update health status
        let mut adapters_mut = self.adapters.write().unwrap_or_else(|e| e.into_inner());
        for (name, healthy) in &results {
            if let Some(entry) = adapters_mut.get_mut(name) {
                entry.healthy = *healthy;
            }
        }

        results
    }

    /// Get only healthy adapters
    pub fn healthy_adapters(&self) -> Vec<String> {
        let adapters = self.adapters.read().unwrap_or_else(|e| e.into_inner());
        adapters
            .iter()
            .filter(|(_, e)| e.healthy)
            .map(|(name, _)| name.clone())
            .collect()
    }
}

impl Default for LlmRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::LlmRequest;

    struct MockProvider {
        provider_type: LlmProviderType,
    }

    impl UnifiedLlm for MockProvider {
        fn complete(&self, _request: &LlmRequest) -> Result<super::super::LlmResponse, LlmError> {
            Ok(super::super::LlmResponse {
                content: "mock response".to_string(),
                model: "mock".to_string(),
                usage: super::super::Usage::default(),
                finish_reason: super::super::FinishReason::Stop,
                tool_calls: None,
                reasoning: None,
            })
        }

        fn provider_type(&self) -> LlmProviderType {
            self.provider_type
        }
    }

    #[test]
    fn test_registry_register_and_get() {
        let registry = LlmRegistry::new();
        registry.register(
            "test".to_string(),
            Box::new(MockProvider {
                provider_type: LlmProviderType::OpenAI,
            }),
        );

        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());

        let result = registry.with_adapter("test", |adapter| {
            adapter.provider_type()
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LlmProviderType::OpenAI);
    }

    #[test]
    fn test_registry_default_provider() {
        let registry = LlmRegistry::new();
        assert!(registry.default_name().is_none());

        registry.register(
            "first".to_string(),
            Box::new(MockProvider {
                provider_type: LlmProviderType::OpenAI,
            }),
        );
        assert_eq!(registry.default_name().as_deref(), Some("first"));

        registry.register(
            "second".to_string(),
            Box::new(MockProvider {
                provider_type: LlmProviderType::Anthropic,
            }),
        );
        // Default should still be first
        assert_eq!(registry.default_name().as_deref(), Some("first"));

        registry.set_default("second").unwrap();
        assert_eq!(registry.default_name().as_deref(), Some("second"));
    }

    #[test]
    fn test_registry_find_by_type() {
        let registry = LlmRegistry::new();
        registry.register(
            "openai-1".to_string(),
            Box::new(MockProvider {
                provider_type: LlmProviderType::OpenAI,
            }),
        );
        registry.register(
            "anthropic-1".to_string(),
            Box::new(MockProvider {
                provider_type: LlmProviderType::Anthropic,
            }),
        );
        registry.register(
            "openai-2".to_string(),
            Box::new(MockProvider {
                provider_type: LlmProviderType::OpenAI,
            }),
        );

        let openai_adapters = registry.find_by_type(LlmProviderType::OpenAI);
        assert_eq!(openai_adapters.len(), 2);
        assert!(openai_adapters.contains(&"openai-1".to_string()));
        assert!(openai_adapters.contains(&"openai-2".to_string()));
    }

    #[test]
    fn test_registry_remove() {
        let registry = LlmRegistry::new();
        registry.register(
            "test".to_string(),
            Box::new(MockProvider {
                provider_type: LlmProviderType::OpenAI,
            }),
        );

        assert_eq!(registry.len(), 1);
        assert_eq!(registry.default_name().as_deref(), Some("test"));

        registry.remove("test");
        assert_eq!(registry.len(), 0);
        assert!(registry.default_name().is_none());
    }
}
