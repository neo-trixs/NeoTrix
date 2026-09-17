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
