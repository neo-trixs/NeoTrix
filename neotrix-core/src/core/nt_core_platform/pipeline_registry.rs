//! Pipeline Registry — 全局 Pipeline 注册表
//!
//! 提供统一的 Pipeline 注册、发现、生命周期管理功能。
//! 所有 NeoTrix Pipeline 必须注册到此注册表才能被全局发现。

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::error::{PlatformError, PlatformResult};
use super::pipeline::{Pipeline, PipelineResult};
use crate::core::nt_core_capability::{Domain, Layer};

// ============================================================
// 1. PipelineEntry — 注册表条目
// ============================================================

/// Pipeline 注册表条目
pub struct PipelineEntry {
    /// Pipeline ID
    pub id: String,
    /// Pipeline 名称
    pub name: String,
    /// Pipeline 描述
    pub description: String,
    /// Pipeline 层级
    pub layer: Layer,
    /// Pipeline 域
    pub domain: Domain,
    /// Pipeline 实例
    pub pipeline: Arc<dyn Pipeline>,
}

// ============================================================
// 2. PipelineRegistry — 全局 Pipeline 注册表
// ============================================================

/// 全局 Pipeline 注册表
pub struct PipelineRegistry {
    /// 按 ID 索引的 Pipeline
    pipelines: RwLock<HashMap<String, Arc<PipelineEntry>>>,
    /// 按层级索引
    by_layer: RwLock<HashMap<Layer, Vec<String>>>,
    /// 按域索引
    by_domain: RwLock<HashMap<Domain, Vec<String>>>,
}

impl PipelineRegistry {
    /// 创建空的 Pipeline 注册表
    pub fn new() -> Self {
        Self {
            pipelines: RwLock::new(HashMap::new()),
            by_layer: RwLock::new(HashMap::new()),
            by_domain: RwLock::new(HashMap::new()),
        }
    }

    /// 注册 Pipeline
    pub async fn register(
        &self,
        id: String,
        name: String,
        description: String,
        layer: Layer,
        domain: Domain,
        pipeline: Arc<dyn Pipeline>,
    ) -> Result<(), String> {
        let mut pipelines = self.pipelines.write().await;
        if pipelines.contains_key(&id) {
            return Err(format!("Pipeline {} already registered", id));
        }

        let entry = Arc::new(PipelineEntry {
            id: id.clone(),
            name,
            description,
            layer,
            domain,
            pipeline,
        });

        pipelines.insert(id.clone(), entry);

        // 索引到层级表
        let mut by_layer = self.by_layer.write().await;
        by_layer.entry(layer).or_default().push(id.clone());

        // 索引到域表
        let mut by_domain = self.by_domain.write().await;
        by_domain.entry(domain).or_default().push(id);

        Ok(())
    }

    /// 获取 Pipeline
    pub async fn get(&self, id: &str) -> Option<Arc<PipelineEntry>> {
        let pipelines = self.pipelines.read().await;
        pipelines.get(id).cloned()
    }

    /// 获取 Pipeline ID 列表
    pub async fn list_ids(&self) -> Vec<String> {
        let pipelines = self.pipelines.read().await;
        pipelines.keys().cloned().collect()
    }

    /// 获取 Pipeline 数量
    pub async fn len(&self) -> usize {
        let pipelines = self.pipelines.read().await;
        pipelines.len()
    }

    /// 检查注册表是否为空
    pub async fn is_empty(&self) -> bool {
        let pipelines = self.pipelines.read().await;
        pipelines.is_empty()
    }

    /// 检查 Pipeline 是否存在
    pub async fn has(&self, id: &str) -> bool {
        let pipelines = self.pipelines.read().await;
        pipelines.contains_key(id)
    }

    /// 获取按层级索引的 Pipeline ID
    pub async fn get_by_layer(&self, layer: Layer) -> Vec<String> {
        let by_layer = self.by_layer.read().await;
        by_layer.get(&layer).cloned().unwrap_or_default()
    }

    /// 获取按域索引的 Pipeline ID
    pub async fn get_by_domain(&self, domain: Domain) -> Vec<String> {
        let by_domain = self.by_domain.read().await;
        by_domain.get(&domain).cloned().unwrap_or_default()
    }

    /// 执行 Pipeline
    pub async fn execute(
        &self,
        pipeline_id: &str,
        input: serde_json::Value,
    ) -> PlatformResult<PipelineResult> {
        let entry = self.get(pipeline_id).await
            .ok_or_else(|| PlatformError::Pipeline(format!("Pipeline {} not found", pipeline_id)))?;
        
        entry.pipeline.run(input).await
            .map_err(|e| PlatformError::Pipeline(format!("Pipeline {} execution failed: {}", pipeline_id, e)))
    }

    /// 列出所有 Pipeline 信息
    pub async fn list_all(&self) -> Vec<(String, String, String, Layer, Domain)> {
        let pipelines = self.pipelines.read().await;
        pipelines.values().map(|e| {
            (e.id.clone(), e.name.clone(), e.description.clone(), e.layer, e.domain)
        }).collect()
    }
}

impl Default for PipelineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 3. PipelineRegistryBuilder — 构建器模式
// ============================================================

/// PipelineRegistry 构建器
pub struct PipelineRegistryBuilder {
    registrations: Vec<(String, String, String, Layer, Domain, Arc<dyn Pipeline>)>,
}

impl PipelineRegistryBuilder {
    pub fn new() -> Self {
        Self {
            registrations: Vec::new(),
        }
    }

    /// 添加 Pipeline
    pub fn add_pipeline(
        mut self,
        id: String,
        name: String,
        description: String,
        layer: Layer,
        domain: Domain,
        pipeline: Arc<dyn Pipeline>,
    ) -> Self {
        self.registrations.push((id, name, description, layer, domain, pipeline));
        self
    }

    /// 构建并注册所有 Pipeline
    pub async fn build(self) -> PlatformResult<PipelineRegistry> {
        let registry = PipelineRegistry::new();
        for (id, name, description, layer, domain, pipeline) in self.registrations {
            registry.register(id, name, description, layer, domain, pipeline).await
                .map_err(|e| PlatformError::Pipeline(e))?;
        }
        Ok(registry)
    }
}

impl Default for PipelineRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
