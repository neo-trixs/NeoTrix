#![forbid(unsafe_code)]

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub message: String,
    pub components: HashMap<String, ComponentHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub healthy: bool,
    pub message: String,
    pub latency_ms: Option<f64>,
}

pub struct HealthChecker {
    components: HashMap<String, Box<dyn HealthCheck>>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, checker: Box<dyn HealthCheck>) {
        self.components.insert(name, checker);
    }

    pub async fn check_all(&self) -> HealthStatus {
        let mut components = HashMap::new();
        let mut all_healthy = true;

        for (name, checker) in &self.components {
            let result = checker.check().await;
            if !result.healthy {
                all_healthy = false;
            }
            components.insert(name.clone(), result);
        }

        HealthStatus {
            healthy: all_healthy,
            message: if all_healthy {
                "All components healthy".into()
            } else {
                "Some components unhealthy".into()
            },
            components,
        }
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> ComponentHealth;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysHealthy;

    #[async_trait::async_trait]
    impl HealthCheck for AlwaysHealthy {
        async fn check(&self) -> ComponentHealth {
            ComponentHealth {
                name: "healthy".into(),
                healthy: true,
                message: "ok".into(),
                latency_ms: Some(1.0),
            }
        }
    }

    struct UnhealthyChecker;

    #[async_trait::async_trait]
    impl HealthCheck for UnhealthyChecker {
        async fn check(&self) -> ComponentHealth {
            ComponentHealth {
                name: "bad".into(),
                healthy: false,
                message: "broken".into(),
                latency_ms: None,
            }
        }
    }

    #[tokio::test]
    async fn health_checker_empty() {
        let checker = HealthChecker::new();
        let status = checker.check_all().await;
        assert!(status.healthy);
        assert!(status.components.is_empty());
    }

    #[tokio::test]
    async fn health_checker_all_healthy() {
        let mut checker = HealthChecker::new();
        checker.register("svc1".into(), Box::new(AlwaysHealthy));
        let status = checker.check_all().await;
        assert!(status.healthy);
        assert!(status.components.contains_key("svc1"));
    }

    #[tokio::test]
    async fn health_checker_has_unhealthy() {
        let mut checker = HealthChecker::new();
        checker.register("good".into(), Box::new(AlwaysHealthy));
        checker.register("bad".into(), Box::new(UnhealthyChecker));
        let status = checker.check_all().await;
        assert!(!status.healthy);
        assert!(status.components.len() == 2);
    }

    #[tokio::test]
    async fn health_checker_default() {
        let checker = HealthChecker::default();
        let status = checker.check_all().await;
        assert!(status.healthy);
    }
}
