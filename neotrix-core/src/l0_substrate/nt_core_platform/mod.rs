//! NeoTrix Unified Platform — 跨层统一基础设施
//!
//! 定义所有 Agent/Orchestrator/Pipeline/Registry 的统一 trait，
//! 实现能力生态统一，消除重复构建。

#![forbid(unsafe_code)]

pub mod agent;
pub mod agent_registry;
pub mod config;
pub mod error;
pub mod health;
pub mod init;
pub mod metrics;
pub mod orchestrator;
pub mod pipeline;
pub mod pipeline_registry;
pub mod registry;

pub use agent::{Agent, AgentError, AgentHealth, AgentMetrics, AgentStatus};
pub use agent_registry::AgentRegistry;
pub use config::Configurable;
pub use error::{PlatformError, PlatformResult};
pub use health::{ComponentHealth, HealthCheck, HealthChecker, HealthStatus};
pub use init::{init_agent_registry, init_all, init_monitor, init_pipeline_registry, PlatformMonitor};
pub use metrics::{MetricType, MetricValue, MetricsCollector};
pub use orchestrator::{Orchestrator, OrchestratorConfig, OrchestratorStats, OrchestratorStatus};
pub use pipeline::{Pipeline, PipelineConfig, PipelineResult, PipelineStage};
pub use pipeline_registry::{PipelineRegistry, PipelineRegistryBuilder};
pub use registry::{DomainRegistry, RegistryEntry};
