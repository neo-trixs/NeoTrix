#![allow(dead_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgiFramework {
    SingularityNet,
    Bittensor,
    FetchAi,
    Custom(String),
}

impl AgiFramework {
    pub fn name(&self) -> &str {
        match self {
            Self::SingularityNet => "singularitynet",
            Self::Bittensor => "bittensor",
            Self::FetchAi => "fetchai",
            Self::Custom(s) => s,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgiBridgeConfig {
    pub supported_frameworks: Vec<AgiFramework>,
    pub endpoints: HashMap<String, String>,
}

impl Default for AgiBridgeConfig {
    fn default() -> Self {
        Self {
            supported_frameworks: vec![
                AgiFramework::SingularityNet,
                AgiFramework::Bittensor,
                AgiFramework::FetchAi,
            ],
            endpoints: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgiService {
    pub service_id: String,
    pub framework: AgiFramework,
    pub endpoint: String,
    pub description: String,
    pub price_per_call: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ReputationRecord {
    pub service_id: String,
    pub calls_count: u64,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
}

#[derive(Debug, Clone)]
pub struct TokenEconomics {
    pub token_symbol: String,
    pub balance: f64,
    pub staking_amount: f64,
    pub rewards: f64,
}

impl Default for TokenEconomics {
    fn default() -> Self {
        Self {
            token_symbol: "NTX".to_string(),
            balance: 1000.0,
            staking_amount: 100.0,
            rewards: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DecentralizedMarketplace {
    services: Vec<AgiService>,
    reputation: Vec<ReputationRecord>,
    economics: TokenEconomics,
}

impl DecentralizedMarketplace {
    pub fn new(economics: TokenEconomics) -> Self {
        Self {
            services: Vec::new(),
            reputation: Vec::new(),
            economics,
        }
    }

    pub fn register_service(&mut self, service: AgiService) {
        self.reputation.push(ReputationRecord {
            service_id: service.service_id.clone(),
            calls_count: 0,
            success_rate: 1.0,
            avg_latency_ms: 0.0,
        });
        self.services.push(service);
    }

    pub fn discover_services(&self) -> &[AgiService] {
        &self.services
    }

    pub fn get_service(&self, service_id: &str) -> Option<&AgiService> {
        self.services.iter().find(|s| s.service_id == service_id)
    }

    pub fn record_call(&mut self, service_id: &str, success: bool, latency_ms: f64) {
        if let Some(rec) = self
            .reputation
            .iter_mut()
            .find(|r| r.service_id == service_id)
        {
            let old_total = rec.calls_count as f64 * rec.avg_latency_ms;
            rec.calls_count += 1;
            rec.success_rate = ((rec.success_rate * (rec.calls_count as f64 - 1.0))
                + if success { 1.0 } else { 0.0 })
                / rec.calls_count as f64;
            rec.avg_latency_ms = (old_total + latency_ms) / rec.calls_count as f64;
        }
    }

    pub fn economics(&self) -> &TokenEconomics {
        &self.economics
    }

    pub fn economics_mut(&mut self) -> &mut TokenEconomics {
        &mut self.economics
    }
}

pub struct AgiBridge {
    config: AgiBridgeConfig,
    marketplace: DecentralizedMarketplace,
}

impl AgiBridge {
    pub fn new(config: AgiBridgeConfig) -> Self {
        let economics = TokenEconomics::default();
        Self {
            config,
            marketplace: DecentralizedMarketplace::new(economics),
        }
    }

    pub fn with_marketplace(
        config: AgiBridgeConfig,
        marketplace: DecentralizedMarketplace,
    ) -> Self {
        Self {
            config,
            marketplace,
        }
    }

    pub fn config(&self) -> &AgiBridgeConfig {
        &self.config
    }

    pub fn marketplace(&self) -> &DecentralizedMarketplace {
        &self.marketplace
    }

    pub fn marketplace_mut(&mut self) -> &mut DecentralizedMarketplace {
        &mut self.marketplace
    }

    pub fn discover_services(&self) -> Vec<AgiService> {
        self.marketplace.services.clone()
    }

    pub fn call_service(&self, service_id: &str, payload: &str) -> Result<String, String> {
        let service = self
            .marketplace
            .services
            .iter()
            .find(|s| s.service_id == service_id)
            .ok_or_else(|| format!("service '{}' not found", service_id))?;
        Ok(format!(
            "Called {} via {} payload={}: response_ok",
            service.description,
            service.framework.name(),
            payload
        ))
    }

    pub fn register_capability(
        &mut self,
        name: &str,
        description: &str,
        framework: AgiFramework,
    ) -> Result<String, String> {
        let service_id = format!("{}_{}", framework.name(), name);
        let service = AgiService {
            service_id: service_id.clone(),
            framework,
            endpoint: format!("https://api.neotrix.ai/{}", service_id),
            description: description.to_string(),
            price_per_call: Some(0.01),
        };
        self.marketplace.register_service(service);
        Ok(service_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_enum() {
        let frameworks = vec![
            AgiFramework::SingularityNet,
            AgiFramework::Bittensor,
            AgiFramework::FetchAi,
        ];
        assert_eq!(frameworks.len(), 3);
    }

    #[test]
    fn test_bridge_config() {
        let config = AgiBridgeConfig::default();
        assert_eq!(config.supported_frameworks.len(), 3);
    }

    #[test]
    fn test_service_discovery() {
        let bridge = AgiBridge::new(AgiBridgeConfig::default());
        let services = bridge.discover_services();
        assert!(services.is_empty());
    }

    #[test]
    fn test_register_and_call_service() {
        let mut bridge = AgiBridge::new(AgiBridgeConfig::default());
        let id = bridge
            .register_capability("test_skill", "test service", AgiFramework::SingularityNet)
            .unwrap();
        assert!(id.contains("singularitynet_test_skill"));

        let services = bridge.discover_services();
        assert_eq!(services.len(), 1);

        let result = bridge.call_service(&id, "test_payload");
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_nonexistent_service() {
        let bridge = AgiBridge::new(AgiBridgeConfig::default());
        let result = bridge.call_service("nonexistent", "payload");
        assert!(result.is_err());
    }

    #[test]
    fn test_reputation_tracking() {
        let mut marketplace = DecentralizedMarketplace::new(TokenEconomics {
            token_symbol: "NTX".to_string(),
            balance: 5000.0,
            staking_amount: 500.0,
            rewards: 10.0,
        });
        marketplace.register_service(AgiService {
            service_id: "test_svc".to_string(),
            framework: AgiFramework::Bittensor,
            endpoint: "https://test.endpoint".to_string(),
            description: "test".to_string(),
            price_per_call: None,
        });
        marketplace.record_call("test_svc", true, 150.0);
        marketplace.record_call("test_svc", true, 120.0);
        marketplace.record_call("test_svc", false, 500.0);
        let rec = marketplace
            .reputation
            .iter()
            .find(|r| r.service_id == "test_svc")
            .unwrap();
        assert_eq!(rec.calls_count, 3);
        assert!((rec.success_rate - 2.0 / 3.0).abs() < 1e-6);
        assert_eq!(marketplace.economics().balance, 5000.0);
    }
}
