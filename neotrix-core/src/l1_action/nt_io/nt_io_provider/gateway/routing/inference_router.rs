//! Inference Router — UnifiedInference trait 的 GatewayV2 实现
//!
//! Facade 模式: 包装 GatewayV2, 复用其:
//! - 熔断器 (CircuitBreaker)
//! - 限流器 (RateLimiter)
//! - 自适应节流 (AdaptivePacer/TieredSemaphore)
//! - 响应缓存 (ResponseCache)
//! - 响应修复 (ResponseHealer)
//! - 账户池 (AccountPool)
//! - 生成分类 (GenerationClassifier)

use std::sync::Arc;
use std::time::Instant;

use crate::l1_action::nt_io::nt_io_provider::gateway::execution::unified_inference::{UnifiedInference, InferenceRequest, InferenceResponse, InferenceError, StreamHandle, RouterHealth, InferenceCapabilities, ResponseMetadata};
use crate::l1_action::nt_io::nt_io_provider::gateway::GatewayV2;
use crate::l1_action::nt_io::nt_io_provider::common::types::{LlmRequest, LlmResponse, LlmError};

/// 路由器配置
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// 最大重试次数
    pub max_retries: u32,
    /// 最大故障转移链长度
    pub fallback_chain_length: u32,
    /// 是否启用成本预算
    pub enable_cost_budget: bool,
    /// 默认成本预算 (USD)
    pub default_cost_budget: f64,
    /// 是否启用延迟预算
    pub enable_latency_budget: bool,
    /// 默认延迟预算 (ms)
    pub default_latency_budget: u64,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            fallback_chain_length: 3,
            enable_cost_budget: true,
            default_cost_budget: 0.02,
            enable_latency_budget: true,
            default_latency_budget: 30000,
        }
    }
}

/// 推理路由器 — 统一 LLM 调用入口
pub struct InferenceRouter {
    gateway: Arc<GatewayV2>,
    config: RouterConfig,
}

impl InferenceRouter {
    /// 创建路由器
    pub fn new(gateway: Arc<GatewayV2>) -> Self {
        Self {
            gateway,
            config: RouterConfig::default(),
        }
    }

    /// 创建路由器 (自定义配置)
    pub fn with_config(gateway: Arc<GatewayV2>, config: RouterConfig) -> Self {
        Self { gateway, config }
    }

    /// 获取 Gateway 引用
    pub fn gateway(&self) -> &GatewayV2 {
        &self.gateway
    }

    /// 获取配置
    pub fn config(&self) -> &RouterConfig {
        &self.config
    }

    /// 预算检查
    fn check_budget(&self, request: &InferenceRequest) -> Result<(), InferenceError> {
        if self.config.enable_cost_budget {
            let estimate = self.estimate_cost(request);
            let budget = request.metadata.cost_budget.unwrap_or(self.config.default_cost_budget);
            if estimate.estimated_cost_usd > budget {
                return Err(InferenceError::BudgetExceeded {
                    estimated: estimate.estimated_cost_usd,
                    budget,
                });
            }
        }

        if self.config.enable_latency_budget {
            let budget = request.metadata.latency_budget.unwrap_or(self.config.default_latency_budget);
            if budget == 0 {
                return Err(InferenceError::ValidationError("Latency budget is zero".into()));
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl UnifiedInference for InferenceRouter {
    async fn complete(&self, request: &InferenceRequest) -> Result<InferenceResponse, InferenceError> {
        // 1. 预算检查
        self.check_budget(request)?;

        // 2. 转换请求
        let llm_request = request.to_llm_request();

        // 3. 委托 Gateway (处理熔断/限流/故障转移)
        let start = Instant::now();
        let result = self.gateway.complete_with_selection(&llm_request).await;
        let latency = start.elapsed().as_millis() as u64;

        match result {
            Ok(selection) => {
                Ok(InferenceResponse::from_llm_response(selection.response, &selection.provider, latency, 0))
            }
            Err(e) => Err(InferenceError::from(e)),
        }
    }

    async fn stream(&self, request: &InferenceRequest) -> Result<StreamHandle, InferenceError> {
        // 1. 预算检查
        self.check_budget(request)?;

        // 2. 转换请求
        let llm_request = request.to_llm_request();

        // 3. 委托 Gateway 流式
        let rx = self.gateway.stream_complete_with_selection(&llm_request).await
            .map_err(InferenceError::from)?;

        // 4. 包装响应流
        let (tx_out, rx_out) = tokio::sync::mpsc::channel(64);
        let provider_status = self.gateway.provider_status();
        let default_provider = provider_status.first()
            .and_then(|s| s.get("name").and_then(|v| v.as_str()))
            .unwrap_or("unknown")
            .to_string();
        let stream_provider = default_provider.clone();
        tokio::spawn(async move {
            while let Some(item) = rx.recv().await {
                let mapped: Result<InferenceResponse, InferenceError> = match item {
                    Ok(resp) => Ok(InferenceResponse {
                        content: resp.content,
                        model: resp.model,
                        provider: stream_provider.clone(),
                        usage: resp.usage,
                        finish_reason: resp.finish_reason,
                        tool_calls: resp.tool_calls,
                        reasoning: resp.reasoning,
                        metadata: ResponseMetadata {
                            provider_selected: stream_provider.clone(),
                            ..Default::default()
                        },
                    }),
                    Err(e) => Err(InferenceError::from(e)),
                };
                if tx_out.send(mapped).await.is_err() {
                    break;
                }
            }
        });

        Ok(StreamHandle {
            rx: rx_out,
            provider: default_provider,
        })
    }

    fn capabilities(&self) -> InferenceCapabilities {
        // 从 Gateway 状态推断能力
        InferenceCapabilities {
            providers: Vec::new(),
            total_free: 0,
            total_paid: 0,
            total_local: 0,
            supports_vision: true,
            supports_tools: true,
            supports_streaming: true,
        }
    }

    async fn health(&self) -> RouterHealth {
        let status = self.gateway.provider_status();
        let total = status.len();
        let healthy = status.iter()
            .filter(|s| s.get("available").and_then(|v| v.as_bool()).unwrap_or(false))
            .count();

        RouterHealth {
            total_providers: total,
            healthy,
            degraded: 0,
            circuit_open: total.saturating_sub(healthy),
            pool_sufficient: self.gateway.is_pool_sufficient(2),
        }
    }

    fn estimate_cost(&self, request: &InferenceRequest) -> CostEstimate {
        let prompt = request.messages.iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let prompt_tokens = crate::l1_action::nt_io::nt_io_provider::health::context_budget::estimate_tokens(&prompt);
        let completion_tokens = request.max_tokens.unwrap_or(4096) as usize;

        // 简单估算: $0.002/1K tokens (可替换为 per-provider 定价)
        let cost = (prompt_tokens as f64 / 1000.0) * 0.002
            + (completion_tokens as f64 / 1000.0) * 0.002;

        CostEstimate {
            input_tokens: prompt_tokens,
            output_tokens: completion_tokens,
            estimated_cost_usd: cost,
            model: request.model.clone().unwrap_or_default(),
        }
    }
}

impl std::fmt::Debug for InferenceRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InferenceRouter")
            .field("config", &self.config)
            .finish()
    }
}
