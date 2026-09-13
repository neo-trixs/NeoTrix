//! 多模型路由模块
//!
//! 统一接口适配多模型，支持负载均衡、故障转移
//! 支持模型切换、提供商管理、速率限制

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 模型路由定义
// ============================================================================

/// 模型提供商
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ModelProvider {
    /// OpenAI
    OpenAI,
    /// Anthropic
    Anthropic,
    /// Stability AI
    StabilityAI,
    /// Runway
    Runway,
    /// Kling
    Kling,
    /// Pika
    Pika,
    /// Luma
    Luma,
    /// 本地模型
    Local,
    /// 自定义
    Custom,
}

/// 模型类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ModelType {
    /// 文本生成
    TextGeneration,
    /// 图像生成
    ImageGeneration,
    /// 视频生成
    VideoGeneration,
    /// 音频生成
    AudioGeneration,
    /// 嵌入
    Embedding,
}

/// 模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// 模型ID
    pub id: String,
    /// 模型名称
    pub name: String,
    /// 提供商
    pub provider: ModelProvider,
    /// 模型类型
    pub model_type: ModelType,
    /// API 端点
    pub api_endpoint: String,
    /// API 密钥 (可选)
    pub api_key: Option<String>,
    /// 最大并发请求数
    pub max_concurrent: u32,
    /// 速率限制 (请求/分钟)
    pub rate_limit: u32,
    /// 每秒价格 (美元)
    pub price_per_second: f64,
    /// 最大分辨率
    pub max_resolution: Option<(u32, u32)>,
    /// 最大时长 (秒)
    pub max_duration: Option<f32>,
    /// 是否启用
    pub enabled: bool,
    /// 优先级
    pub priority: u32,
    /// 标签
    pub tags: Vec<String>,
}

/// 路由策略
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RoutingStrategy {
    /// 优先级路由
    Priority,
    /// 轮询路由
    RoundRobin,
    /// 最低成本路由
    LowestCost,
    /// 最高质量路由
    HighestQuality,
    /// 负载均衡
    LoadBalanced,
}

/// 路由配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// 路由策略
    pub strategy: RoutingStrategy,
    /// 是否启用故障转移
    pub enable_failover: bool,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试间隔 (毫秒)
    pub retry_interval_ms: u64,
    /// 超时时间 (秒)
    pub timeout_secs: u32,
    /// 是否启用缓存
    pub enable_cache: bool,
}

/// 路由请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRequest {
    /// 任务类型
    pub task_type: ModelType,
    /// 提示词
    pub prompt: String,
    /// 参数
    pub params: HashMap<String, serde_json::Value>,
    /// 首选模型ID
    pub preferred_model: Option<String>,
    /// 标签过滤
    pub required_tags: Vec<String>,
}

/// 路由响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingResponse {
    /// 是否成功
    pub success: bool,
    /// 选择的模型ID
    pub selected_model: String,
    /// 输出数据
    pub output: Option<serde_json::Value>,
    /// 成本 (美元)
    pub cost: f64,
    /// 耗时 (毫秒)
    pub latency_ms: u64,
    /// 重试次数
    pub retries: u32,
    /// 错误信息
    pub error: Option<String>,
}

/// 模型状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelState {
    /// 当前并发数
    pub current_concurrent: u32,
    /// 当前速率 (请求/分钟)
    pub current_rate: u32,
    /// 总请求数
    pub total_requests: u64,
    /// 总成本 (美元)
    pub total_cost: f64,
    /// 平均延迟 (毫秒)
    pub avg_latency_ms: f64,
    /// 成功率
    pub success_rate: f32,
    /// 是否可用
    pub available: bool,
}

// ============================================================================
// 多模型路由器
// ============================================================================

/// 多模型路由器
/// 统一接口适配多模型
#[derive(Debug)]
pub struct ModelRoutingLayer {
    /// 路由配置
    config: RoutingConfig,
    /// 模型配置
    models: HashMap<String, ModelConfig>,
    /// 模型状态
    states: HashMap<String, ModelState>,
    /// 轮询索引
    round_robin_index: usize,
    /// 请求历史
    history: Vec<RoutingResponse>,
}

impl ModelRoutingLayer {
    /// 创建路由器
    pub fn new() -> Self {
        Self {
            config: RoutingConfig {
                strategy: RoutingStrategy::Priority,
                enable_failover: true,
                max_retries: 3,
                retry_interval_ms: 1000,
                timeout_secs: 300,
                enable_cache: true,
            },
            models: HashMap::new(),
            states: HashMap::new(),
            round_robin_index: 0,
            history: vec![],
        }
    }
    
    /// 使用配置创建
    pub fn with_config(config: RoutingConfig) -> Self {
        Self {
            config,
            models: HashMap::new(),
            states: HashMap::new(),
            round_robin_index: 0,
            history: vec![],
        }
    }
    
    /// 注册模型
    pub fn register_model(&mut self, model: ModelConfig) {
        let id = model.id.clone();
        self.states.insert(id.clone(), ModelState {
            current_concurrent: 0,
            current_rate: 0,
            total_requests: 0,
            total_cost: 0.0,
            avg_latency_ms: 0.0,
            success_rate: 1.0,
            available: true,
        });
        self.models.insert(id, model);
    }
    
    /// 选择模型
    pub fn select_model(&self, request: &RoutingRequest) -> Option<&ModelConfig> {
        let mut candidates: Vec<&ModelConfig> = self.models.values()
            .filter(|m| {
                m.enabled
                    && m.model_type == request.task_type
                    && self.states.get(&m.id).map_or(false, |s| s.available)
                    && request.required_tags.iter().all(|t| m.tags.contains(t))
            })
            .collect();
        
        match self.config.strategy {
            RoutingStrategy::Priority => {
                candidates.sort_by(|a, b| b.priority.cmp(&a.priority));
            }
            RoutingStrategy::LowestCost => {
                candidates.sort_by(|a, b| a.price_per_second.partial_cmp(&b.price_per_second).unwrap_or(std::cmp::Ordering::Equal));
            }
            RoutingStrategy::HighestQuality => {
                // 基于质量分数排序：优先选择 rate_limit 高、价格适中的模型
                candidates.sort_by(|a, b| {
                    // 质量分数 = rate_limit / (price + 0.01) — 高容量低成本优先
                    let a_quality = a.rate_limit as f64 / (a.price_per_second as f64 + 0.01);
                    let b_quality = b.rate_limit as f64 / (b.price_per_second as f64 + 0.01);
                    b_quality.partial_cmp(&a_quality).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            RoutingStrategy::RoundRobin => {
                if !candidates.is_empty() {
                    let idx = self.round_robin_index % candidates.len();
                    return candidates.into_iter().nth(idx);
                }
            }
            RoutingStrategy::LoadBalanced => {
                candidates.sort_by(|a, b| {
                    let a_load = self.states.get(&a.id).map_or(0.0, |s| {
                        s.current_concurrent as f32 / a.max_concurrent as f32
                    });
                    let b_load = self.states.get(&b.id).map_or(0.0, |s| {
                        s.current_concurrent as f32 / b.max_concurrent as f32
                    });
                    a_load.partial_cmp(&b_load).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }
        
        candidates.into_iter().next()
    }
    
    /// 路由请求
    pub fn route(&mut self, request: RoutingRequest) -> RoutingResponse {
        let mut retries = 0;
        let mut last_error = None;
        
        while retries <= self.config.max_retries {
            let model_id = if let Some(model) = self.select_model(&request) {
                model.id.clone()
            } else {
                retries += 1;
                last_error = Some("无可用模型".to_string());
                continue;
            };

            // 返回路由结果，但不实际调用模型 API
            // 实际调用应通过 nt_io_provider 的具体实现（OpenAI/Gemini/Ollama 等）
            let cost = self.states.get(&model_id)
                .map(|s| s.total_cost)
                .unwrap_or(0.0);
            let response = RoutingResponse {
                success: true,
                selected_model: model_id.clone(),
                output: None, // 输出由具体 provider 填充
                cost: cost,
                latency_ms: 0, // 实际延迟由 provider 测量
                retries,
                error: None,
            };

            // 更新状态
            if let Some(state) = self.states.get_mut(&model_id) {
                state.total_requests += 1;
                state.total_cost += response.cost;
            }

            self.history.push(response.clone());
            return response;
        }
        
        RoutingResponse {
            success: false,
            selected_model: String::new(),
            output: None,
            cost: 0.0,
            latency_ms: 0,
            retries,
            error: last_error,
        }
    }
    
    /// 获取模型状态
    pub(crate) fn _get_model_state(&self, model_id: &str) -> Option<&ModelState> {
        self.states.get(model_id)
    }
    
    /// 获取统计信息
    pub fn statistics(&self) -> RoutingStats {
        let total_models = self.models.len();
        let available = self.states.values().filter(|s| s.available).count();
        let total_requests: u64 = self.states.values().map(|s| s.total_requests).sum();
        let total_cost: f64 = self.states.values().map(|s| s.total_cost).sum();
        let avg_latency = if !self.history.is_empty() {
            self.history.iter().map(|r| r.latency_ms as f64).sum::<f64>() / self.history.len() as f64
        } else {
            0.0
        };
        
        RoutingStats {
            total_models,
            available_models: available,
            total_requests,
            total_cost,
            avg_latency_ms: avg_latency,
        }
    }
}

/// 路由统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingStats {
    /// 总模型数
    pub total_models: usize,
    /// 可用模型数
    pub available_models: usize,
    /// 总请求数
    pub total_requests: u64,
    /// 总成本 (美元)
    pub total_cost: f64,
    /// 平均延迟 (毫秒)
    pub avg_latency_ms: f64,
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_routing() {
        let mut router = ModelRoutingLayer::new();
        
        router.register_model(ModelConfig {
            id: "kling_001".to_string(),
            name: "Kling 2.6".to_string(),
            provider: ModelProvider::Kling,
            model_type: ModelType::VideoGeneration,
            api_endpoint: "https://api.kling.com".to_string(),
            api_key: None,
            max_concurrent: 10,
            rate_limit: 60,
            price_per_second: 0.1,
            max_resolution: Some((1920, 1080)),
            max_duration: Some(10.0),
            enabled: true,
            priority: 1,
            tags: vec!["video".to_string()],
        });
        
        let request = RoutingRequest {
            task_type: ModelType::VideoGeneration,
            prompt: "测试视频".to_string(),
            params: HashMap::new(),
            preferred_model: None,
            required_tags: vec![],
        };
        
        let response = router.route(request);
        assert!(response.success);
        assert_eq!(response.selected_model, "kling_001");
    }
}