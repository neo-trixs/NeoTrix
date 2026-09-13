//! Fallback 链路由器 — 当首选模型失败时自动切换到备选模型
//!
//! 设计原则:
//! 1. 按优先级顺序尝试模型
//! 2. 支持熔断状态感知 (跳过 circuit breaker open 的模型)
//! 3. 支持能力过滤 (只尝试满足需求的模型)
//! 4. 记录每次调用的延迟和错误，用于动态调优

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::l2_perception::nt_core_llm::{LlmRequest, LlmResponse};

use super::traits::{
    ModelError, ModelHealth, ModelIdentifier, TaskType, UniversalModel,
};

/// Fallback 链配置
#[derive(Debug, Clone)]
pub struct FallbackConfig {
    /// 最大重试次数 (含首次调用)
    pub max_attempts: usize,
    /// 是否跳过熔断模型
    pub skip_open_circuit: bool,
    /// 最小健康延迟阈值 (ms)，高于此值视为降级
    pub healthy_latency_ms: f64,
    /// 最大错误率阈值 (0.0-1.0)，高于此值视为不健康
    pub max_error_rate: f32,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            skip_open_circuit: true,
            healthy_latency_ms: 5000.0,
            max_error_rate: 0.5,
        }
    }
}

/// Fallback 链路由器
pub struct FallbackRouter {
    /// 按优先级排序的模型列表 (index 0 = 最高优先级)
    models: Vec<Arc<RwLock<Box<dyn UniversalModel>>>>,
    /// 每个模型的运行时状态
    states: RwLock<HashMap<String, ModelState>>,
    /// 配置
    config: FallbackConfig,
}

/// 模型运行时状态
#[derive(Debug, Clone)]
struct ModelState {
    health: ModelHealth,
    total_calls: u64,
    failed_calls: u64,
}

impl FallbackRouter {
    /// 创建空路由器
    pub fn new(config: FallbackConfig) -> Self {
        Self {
            models: Vec::new(),
            states: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// 创建默认配置路由器
    pub fn default_config() -> Self {
        Self::new(FallbackConfig::default())
    }

    /// 添加模型到链尾
    pub fn add_model(&mut self, model: Arc<RwLock<Box<dyn UniversalModel>>>) {
        let id = {
            let m = model.blocking_read();
            m.full_id()
        };
        self.states.blocking_write().insert(
            id,
            ModelState {
                health: ModelHealth::default(),
                total_calls: 0,
                failed_calls: 0,
            },
        );
        self.models.push(model);
    }

    /// 获取模型数量
    pub fn model_count(&self) -> usize {
        self.models.len()
    }

    /// 获取所有模型 ID
    pub async fn model_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        for model in &self.models {
            let m = model.read().await;
            ids.push(m.full_id());
        }
        ids
    }

    /// 筛选满足能力要求且健康的模型
    async fn eligible_models(
        &self,
        required_task: TaskType,
    ) -> Vec<(usize, Arc<RwLock<Box<dyn UniversalModel>>>)> {
        let states = self.states.read().await;
        let mut eligible = Vec::new();

        for (i, model) in self.models.iter().enumerate() {
            let m = model.read().await;
            let id = m.full_id();

            // 检查能力
            let caps = m.capabilities();
            if !caps.task_types.contains(&required_task) {
                continue;
            }

            // 检查熔断
            if self.config.skip_open_circuit {
                if let Some(state) = states.get(&id) {
                    if state.health.circuit_breaker_open {
                        continue;
                    }
                }
            }

            // 检查错误率
            if let Some(state) = states.get(&id) {
                if state.total_calls > 10 {
                    let error_rate = state.failed_calls as f32 / state.total_calls as f32;
                    if error_rate > self.config.max_error_rate {
                        continue;
                    }
                }
            }

            eligible.push((i, model.clone()));
        }

        eligible
    }

    /// 通过 fallback 链执行请求
    pub async fn complete(
        &self,
        request: &LlmRequest,
    ) -> Result<LlmResponse, ModelError> {
        // 确定任务类型
        let task_type = if request.messages.iter().any(|m| m.role == crate::core::l2_perception::nt_core_llm::Role::System) {
            TaskType::Chat
        } else {
            TaskType::Chat
        };

        let eligible = self.eligible_models(task_type).await;
        let mut last_error = None;

        for (attempt, (idx, model)) in eligible.iter().enumerate() {
            if attempt >= self.config.max_attempts {
                break;
            }

            let model_id = {
                let m = model.read().await;
                m.full_id()
            };

            match model.read().await.complete(request).await {
                Ok(response) => {
                    // 记录成功
                    let mut states = self.states.write().await;
                    if let Some(state) = states.get_mut(&model_id) {
                        state.total_calls += 1;
                        state.health.last_success = Some(
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                        );
                        state.health.available = true;
                        state.health.circuit_breaker_open = false;
                    }
                    return Ok(response);
                }
                Err(e) => {
                    // 记录失败
                    let mut states = self.states.write().await;
                    if let Some(state) = states.get_mut(&model_id) {
                        state.total_calls += 1;
                        state.failed_calls += 1;
                        state.health.error_rate = state.failed_calls as f32 / state.total_calls as f32;
                    }

                    if !e.should_fallback() {
                        return Err(ModelError::Llm(e));
                    }

                    last_error = Some(ModelError::Llm(e));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ModelError::Unavailable {
            model: "fallback_chain".to_string(),
            reason: "No eligible models in fallback chain".to_string(),
        }))
    }

    /// 获取所有模型的健康状态
    pub async fn health_report(&self) -> Vec<(String, ModelHealth)> {
        let mut report = Vec::new();
        for model in &self.models {
            let m = model.read().await;
            let id = m.full_id();
            let health = m.health();
            report.push((id, health));
        }
        report
    }
}
