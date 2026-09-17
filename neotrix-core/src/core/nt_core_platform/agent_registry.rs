//! Agent Registry — 全局 Agent 注册表
//!
//! 提供统一的 Agent 注册、发现、生命周期管理功能。
//! 所有 NeoTrix Agent 必须注册到此注册表才能被全局发现。

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::agent::{Agent, AgentError, AgentHealth, AgentMetrics, AgentStatus};
use super::error::{PlatformError, PlatformResult};
use super::health::HealthChecker;
use super::metrics::MetricsCollector;
use crate::core::nt_core_capability::{Domain, Layer};

// ============================================================
// 1. AgentRegistry — 全局 Agent 注册表
// ============================================================

/// 全局 Agent 注册表
///
/// 提供按 ID/层级/域 索引的 Agent 注册、发现、生命周期管理。
pub struct AgentRegistry {
    /// 按 ID 索引的 Agent
    agents: RwLock<HashMap<String, Arc<tokio::sync::Mutex<Box<dyn Agent>>>>>,
    /// 按层级索引
    by_layer: RwLock<HashMap<Layer, Vec<String>>>,
    /// 按域索引
    by_domain: RwLock<HashMap<Domain, Vec<String>>>,
    /// 健康检查器
    #[allow(dead_code)]
    health_checker: RwLock<HealthChecker>,
    /// 指标收集器
    metrics: MetricsCollector,
}

impl AgentRegistry {
    /// 创建空的 Agent 注册表
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
            by_layer: RwLock::new(HashMap::new()),
            by_domain: RwLock::new(HashMap::new()),
            health_checker: RwLock::new(HealthChecker::new()),
            metrics: MetricsCollector::new(),
        }
    }

    /// 注册 Agent
    pub async fn register(
        &self,
        agent: Box<dyn Agent>,
    ) -> Result<(), AgentError> {
        let agent_id = agent.agent_id().to_string();
        let layer = agent.agent_layer();
        let domain = agent.agent_domain();

        // 注册到主表
        let mut agents = self.agents.write().await;
        if agents.contains_key(&agent_id) {
            return Err(AgentError::InitializationFailed(
                format!("Agent {} already registered", agent_id),
            ));
        }
        agents.insert(agent_id.clone(), Arc::new(tokio::sync::Mutex::new(agent)));

        // 索引到层级表
        let mut by_layer = self.by_layer.write().await;
        by_layer.entry(layer).or_default().push(agent_id.clone());

        // 索引到域表
        let mut by_domain = self.by_domain.write().await;
        by_domain.entry(domain).or_default().push(agent_id.clone());

        // 更新指标
        self.metrics.increment_counter("agents.registered", 1).await;

        Ok(())
    }

    /// 获取 Agent Arc (克隆后返回，调用方可自行 lock)
    pub async fn get_agent_arc(
        &self,
        agent_id: &str,
    ) -> Option<Arc<tokio::sync::Mutex<Box<dyn Agent>>>> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// 获取 Agent ID 列表
    pub async fn list_ids(&self) -> Vec<String> {
        let agents = self.agents.read().await;
        agents.keys().cloned().collect()
    }

    /// 获取 Agent 数量
    pub async fn len(&self) -> usize {
        let agents = self.agents.read().await;
        agents.len()
    }

    /// 检查注册表是否为空
    pub async fn is_empty(&self) -> bool {
        let agents = self.agents.read().await;
        agents.is_empty()
    }

    /// 检查 Agent 是否存在
    pub async fn has(&self, agent_id: &str) -> bool {
        let agents = self.agents.read().await;
        agents.contains_key(agent_id)
    }

    /// 获取按层级索引的 Agent ID
    pub async fn get_by_layer(&self, layer: Layer) -> Vec<String> {
        let by_layer = self.by_layer.read().await;
        by_layer.get(&layer).cloned().unwrap_or_default()
    }

    /// 获取按域索引的 Agent ID
    pub async fn get_by_domain(&self, domain: Domain) -> Vec<String> {
        let by_domain = self.by_domain.read().await;
        by_domain.get(&domain).cloned().unwrap_or_default()
    }

    /// 初始化所有 Agent
    pub async fn initialize_all(&self) -> PlatformResult<()> {
        let ids = self.list_ids().await;
        for id in &ids {
            let agent_arc = self.get_agent_arc(id).await
                .ok_or_else(|| PlatformError::Agent(format!("Agent {} not found", id)))?;
            let mut agent = agent_arc.lock().await;
            agent.initialize().await.map_err(|e| {
                PlatformError::Agent(format!("Failed to initialize {}: {}", id, e))
            })?;
        }
        Ok(())
    }

    /// 启动所有 Agent
    pub async fn start_all(&self) -> PlatformResult<()> {
        let ids = self.list_ids().await;
        for id in &ids {
            let agent_arc = self.get_agent_arc(id).await
                .ok_or_else(|| PlatformError::Agent(format!("Agent {} not found", id)))?;
            let agent = agent_arc.lock().await;
            agent.start().await.map_err(|e| {
                PlatformError::Agent(format!("Failed to start {}: {}", id, e))
            })?;
            self.metrics.increment_counter("agents.started", 1).await;
        }
        Ok(())
    }

    /// 停止所有 Agent
    pub async fn stop_all(&self) -> PlatformResult<()> {
        let ids = self.list_ids().await;
        for id in &ids {
            let agent_arc = self.get_agent_arc(id).await
                .ok_or_else(|| PlatformError::Agent(format!("Agent {} not found", id)))?;
            let agent = agent_arc.lock().await;
            agent.stop().await.map_err(|e| {
                PlatformError::Agent(format!("Failed to stop {}: {}", id, e))
            })?;
            self.metrics.increment_counter("agents.stopped", 1).await;
        }
        Ok(())
    }

    /// 健康检查所有 Agent
    pub async fn health_check_all(&self) -> HashMap<String, AgentHealth> {
        let ids = self.list_ids().await;
        let mut results = HashMap::new();

        for id in &ids {
            if let Some(agent_arc) = self.get_agent_arc(id).await {
                let agent = agent_arc.lock().await;
                let health = agent.health_check().await;
                results.insert(id.clone(), health);
            }
        }

        results
    }

    /// 获取所有 Agent 状态摘要
    pub async fn status_summary(&self) -> HashMap<String, AgentStatus> {
        let ids = self.list_ids().await;
        let mut results = HashMap::new();

        for id in &ids {
            if let Some(agent_arc) = self.get_agent_arc(id).await {
                let agent = agent_arc.lock().await;
                results.insert(id.clone(), agent.status());
            }
        }

        results
    }

    /// 获取所有 Agent 指标摘要
    pub async fn metrics_summary(&self) -> HashMap<String, AgentMetrics> {
        let ids = self.list_ids().await;
        let mut results = HashMap::new();

        for id in &ids {
            if let Some(agent_arc) = self.get_agent_arc(id).await {
                let agent = agent_arc.lock().await;
                results.insert(id.clone(), agent.metrics());
            }
        }

        results
    }

    /// 获取全局指标
    pub async fn global_metrics(&self) -> HashMap<String, super::metrics::MetricValue> {
        self.metrics.snapshot().await
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 2. AgentRegistryBuilder — 构建器模式
// ============================================================

/// AgentRegistry 构建器
pub struct AgentRegistryBuilder {
    agents: Vec<Box<dyn Agent>>,
}

impl AgentRegistryBuilder {
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
        }
    }

    /// 添加 Agent
    pub fn add_agent(mut self, agent: Box<dyn Agent>) -> Self {
        self.agents.push(agent);
        self
    }

    /// 构建并注册所有 Agent
    pub async fn build(self) -> PlatformResult<AgentRegistry> {
        let registry = AgentRegistry::new();
        for agent in self.agents {
            let id = agent.agent_id().to_string();
            registry.register(agent).await.map_err(|e| {
                PlatformError::Agent(format!("Failed to register {}: {}", id, e))
            })?;
        }
        Ok(registry)
    }
}

impl Default for AgentRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
