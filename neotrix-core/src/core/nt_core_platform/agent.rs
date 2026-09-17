#![forbid(unsafe_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::core::nt_core_capability::{UnifiedCapability, Layer, Domain};

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("初始化失败: {0}")]
    InitializationFailed(String),
    #[error("启动失败: {0}")]
    StartFailed(String),
    #[error("停止失败: {0}")]
    StopFailed(String),
    #[error("执行失败: {0}")]
    ExecutionFailed(String),
    #[error("配置错误: {0}")]
    ConfigError(String),
    #[error("依赖缺失: {0}")]
    DependencyMissing(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Uninitialized,
    Initializing,
    Running,
    Stopping,
    Stopped,
    Error(String),
}

/// Agent 指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub uptime_secs: u64,
    pub last_request_time: Option<u64>,
    pub error_rate: f64,
    pub throughput_per_sec: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHealth {
    pub healthy: bool,
    pub message: String,
    pub status: AgentStatus,
    pub metrics: AgentMetrics,
}

#[async_trait]
pub trait Agent: UnifiedCapability + Send + Sync {
    fn agent_id(&self) -> &str;

    fn agent_name(&self) -> &str;

    fn agent_layer(&self) -> Layer;

    fn agent_domain(&self) -> Domain;

    async fn initialize(&mut self) -> Result<(), AgentError>;

    async fn start(&self) -> Result<(), AgentError>;

    async fn stop(&self) -> Result<(), AgentError>;

    fn status(&self) -> AgentStatus;

    fn metrics(&self) -> AgentMetrics;

    async fn health_check(&self) -> AgentHealth {
        AgentHealth {
            healthy: matches!(self.status(), AgentStatus::Running),
            message: format!("Agent {} is {:?}", self.agent_name(), self.status()),
            status: self.status(),
            metrics: self.metrics(),
        }
    }

    fn record_request(&self, _success: bool, _duration_ms: u64) {}

    /// 获取错误率 (有默认实现)
    fn error_rate(&self) -> f64 {
        let metrics = self.metrics();
        if metrics.total_requests == 0 {
            0.0
        } else {
            metrics.failed_requests as f64 / metrics.total_requests as f64
        }
    }

    /// 获取吞吐量 (有默认实现)
    fn throughput(&self) -> f64 {
        let metrics = self.metrics();
        if metrics.uptime_secs == 0 {
            0.0
        } else {
            metrics.total_requests as f64 / metrics.uptime_secs as f64
        }
    }

    /// 重置指标 (有默认实现)
    fn reset_metrics(&mut self) {
        // 默认空实现，子类可覆盖
    }
}
