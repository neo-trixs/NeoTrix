use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::core::nt_core_llm::{
    UnifiedProvider, LlmRequest, LlmResponse, LlmError,
    ModelCapabilities, HealthStatus, CostEstimate, ProviderMetadata,
};

/// Unified provider registry — wraps GatewayV2's provider management
/// with capability-aware selection and health tracking.
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn UnifiedProvider>>,
    health_cache: RwLock<HashMap<String, HealthStatus>>,
    cost_cache: RwLock<HashMap<String, CostEstimate>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            health_cache: RwLock::new(HashMap::new()),
            cost_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Register a provider.
    pub fn register(&mut self, name: &str, provider: Arc<dyn UnifiedProvider>) {
        self.providers.insert(name.to_string(), provider);
    }

    /// Select best provider for a request — filters by health, capabilities,
    /// then ranks by cost/quality.
    pub async fn select_provider(
        &self,
        request: &LlmRequest,
    ) -> Result<(String, Arc<dyn UnifiedProvider>), LlmError> {
        let health = self.health_cache.read().await;
        
        let mut candidates: Vec<_> = self.providers.iter()
            .filter(|(name, _)| {
                health.get(*name)
                    .map(|h| matches!(h, HealthStatus::Healthy))
                    .unwrap_or(true)
            })
            .filter(|(_, provider)| {
                let caps = provider.capabilities();
                caps.text
            })
            .collect();

        if candidates.is_empty() {
            return Err(LlmError::ProviderError("No healthy providers available".into()));
        }

        // 按成本排序（升序）
        candidates.sort_by(|a, b| {
            let cost_a = a.1.estimate_cost(request).estimated_cost_usd;
            let cost_b = b.1.estimate_cost(request).estimated_cost_usd;
            cost_a.partial_cmp(&cost_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        let (name, provider) = candidates.first().unwrap();
        Ok((name.to_string(), provider.clone()))
    }

    /// Health check all providers — updates cache.
    pub async fn health_check_all(&self) -> HashMap<String, HealthStatus> {
        let mut results = HashMap::new();
        let mut health = self.health_cache.write().await;
        
        for (name, provider) in &self.providers {
            let status = provider.health().await;
            results.insert(name.clone(), status.clone());
            health.insert(name.clone(), status);
        }
        
        results
    }

    /// Cost estimation for all providers — returns sorted by cost.
    pub fn estimate_costs(&self, request: &LlmRequest) -> Vec<(String, CostEstimate)> {
        let mut costs: Vec<_> = self.providers.iter()
            .map(|(name, provider)| {
                let estimate = provider.estimate_cost(request);
                (name.clone(), estimate)
            })
            .collect();
        
        costs.sort_by(|a, b| {
            a.1.estimated_cost_usd.partial_cmp(&b.1.estimated_cost_usd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        costs
    }

    /// List all registered providers.
    pub fn list_providers(&self) -> Vec<ProviderMetadata> {
        self.providers.values()
            .map(|p| p.metadata())
            .collect()
    }
}
