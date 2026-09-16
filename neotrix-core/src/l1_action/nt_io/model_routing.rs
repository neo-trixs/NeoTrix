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
pub struct RoutingModelConfig {
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
    /// 复杂度画像（可选，由 ComplexityClassifier 填充）
    pub complexity: Option<ComplexityProfile>,
}

/// 复杂度画像 — 14维请求分类器输出
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplexityProfile {
    /// 任务类型复杂度 (0.0-1.0)
    pub task_type_complexity: f32,
    /// 上下文长度占比 (0.0-1.0)
    pub context_length_pct: f32,
    /// 工具调用数量
    pub tool_call_count: u32,
    /// 推理深度 (0-5)
    pub reasoning_depth: u32,
    /// 并行操作数
    pub parallel_ops: u32,
    /// 输出大小估计 (tokens)
    pub output_size_estimate: u32,
    /// 交叉引用计数
    pub cross_ref_count: u32,
    /// 错误恢复复杂度 (0.0-1.0)
    pub error_recovery_complexity: f32,
    /// 安全敏感度 (0.0-1.0)
    pub security_sensitivity: f32,
    /// 延迟容忍度 (0.0=严格, 1.0=宽松)
    pub latency_tolerance: f32,
    /// 成本敏感度 (0.0=不敏感, 1.0=极敏感)
    pub cost_sensitivity: f32,
    /// 模型能力需求位掩码
    pub model_capability_reqs: u32,
    /// 格式要求位掩码
    pub format_requirements: u32,
    /// 历史成功率 (0.0-1.0)
    pub historical_success_rate: f32,
}

/// 复杂度分类器 — 14维请求分类
#[derive(Debug, Clone)]
pub struct ComplexityClassifier;

impl ComplexityClassifier {
    /// 根据请求特征计算复杂度画像
    pub fn classify_request(request: &RoutingRequest) -> ComplexityProfile {
        let task_type_complexity = match request.task_type {
            ModelType::TextGeneration => 0.3,
            ModelType::Embedding => 0.2,
            ModelType::ImageGeneration => 0.6,
            ModelType::AudioGeneration => 0.65,
            ModelType::VideoGeneration => 0.9,
        };

        let prompt_len = request.prompt.len();
        let context_length_pct = (prompt_len as f32 / 128_000.0).min(1.0);

        let tool_call_count = request.params.get("tool_calls")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32)
            .unwrap_or(0);

        let reasoning_depth = request.params.get("reasoning_depth")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32)
            .unwrap_or(0)
            .min(5);

        let parallel_ops = request.params.get("parallel_ops")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32)
            .unwrap_or(1);

        let output_size_estimate = request.params.get("max_tokens")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32)
            .unwrap_or(1024);

        let cross_ref_count = request.params.get("cross_refs")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32)
            .unwrap_or(0);

        let error_recovery_complexity = if tool_call_count > 5 || reasoning_depth > 3 {
            0.8
        } else if tool_call_count > 2 || reasoning_depth > 1 {
            0.5
        } else {
            0.2
        };

        let security_sensitivity = if request.required_tags.iter().any(|t| t.contains("secure") || t.contains("auth")) {
            0.9
        } else if security_sensitive_keywords(&request.prompt) {
            0.6
        } else {
            0.1
        };

        let latency_tolerance = request.params.get("latency_tolerance")
            .and_then(|v| v.as_f64())
            .map(|v| v.min(1.0).max(0.0) as f32)
            .unwrap_or(0.5);

        let cost_sensitivity = request.params.get("cost_sensitivity")
            .and_then(|v| v.as_f64())
            .map(|v| v.min(1.0).max(0.0) as f32)
            .unwrap_or(0.5);

        let model_capability_reqs = Self::compute_capability_reqs(&request.task_type);
        let format_requirements = Self::compute_format_reqs(&request.params);

        let historical_success_rate = 0.85;

        ComplexityProfile {
            task_type_complexity,
            context_length_pct,
            tool_call_count,
            reasoning_depth,
            parallel_ops,
            output_size_estimate,
            cross_ref_count,
            error_recovery_complexity,
            security_sensitivity,
            latency_tolerance,
            cost_sensitivity,
            model_capability_reqs,
            format_requirements,
            historical_success_rate,
        }
    }

    /// 计算复杂度综合分数 (0.0-1.0, 越高越复杂)
    pub fn complexity_score(profile: &ComplexityProfile) -> f64 {
        let task_weight = profile.task_type_complexity as f64 * 0.15;
        let context_weight = profile.context_length_pct as f64 * 0.15;
        let tool_weight = (profile.tool_call_count as f64 / 10.0).min(1.0) * 0.1;
        let reasoning_weight = (profile.reasoning_depth as f64 / 5.0).min(1.0) * 0.15;
        let parallel_weight = (profile.parallel_ops as f64 / 8.0).min(1.0) * 0.05;
        let output_weight = (profile.output_size_estimate as f64 / 8192.0).min(1.0) * 0.1;
        let cross_ref_weight = (profile.cross_ref_count as f64 / 20.0).min(1.0) * 0.05;
        let error_weight = profile.error_recovery_complexity as f64 * 0.1;
        let security_weight = profile.security_sensitivity as f64 * 0.05;

        task_weight + context_weight + tool_weight + reasoning_weight
            + parallel_weight + output_weight + cross_ref_weight + error_weight + security_weight
    }

    fn compute_capability_reqs(task_type: &ModelType) -> u32 {
        match task_type {
            ModelType::TextGeneration => 0b001,
            ModelType::ImageGeneration => 0b010,
            ModelType::VideoGeneration => 0b100,
            ModelType::AudioGeneration => 0b1000,
            ModelType::Embedding => 0b10000,
        }
    }

    fn compute_format_reqs(params: &HashMap<String, serde_json::Value>) -> u32 {
        let mut reqs = 0u32;
        if params.contains_key("json_output") {
            reqs |= 0b001;
        }
        if params.contains_key("streaming") {
            reqs |= 0b010;
        }
        if params.contains_key("structured_output") {
            reqs |= 0b100;
        }
        reqs
    }
}

fn security_sensitive_keywords(prompt: &str) -> bool {
    let keywords = ["password", "secret", "token", "api_key", "credential", "private key"];
    let lower = prompt.to_lowercase();
    keywords.iter().any(|kw| lower.contains(kw))
}

/// 投机解码配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpeculativeDecodingConfig {
    /// 置信度阈值，低于此值触发完整模型验证
    pub confidence_threshold: f64,
    /// 草稿模型ID
    pub draft_model_id: String,
    /// 验证模型ID
    pub verify_model_id: String,
    /// N-gram重叠度阈值
    pub n_gram_overlap: f64,
}

impl Default for SpeculativeDecodingConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.85,
            draft_model_id: "draft_small".to_string(),
            verify_model_id: "verify_large".to_string(),
            n_gram_overlap: 0.9,
        }
    }
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
    models: HashMap<String, RoutingModelConfig>,
    /// 模型状态
    states: HashMap<String, ModelState>,
    /// 轮询索引
    round_robin_index: usize,
    /// 请求历史
    history: Vec<RoutingResponse>,
    /// 投机解码配置
    speculative_config: Option<SpeculativeDecodingConfig>,
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
            speculative_config: None,
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
            speculative_config: None,
        }
    }

    /// 设置投机解码配置
    pub fn with_speculative_decoding(&mut self, config: SpeculativeDecodingConfig) {
        self.speculative_config = Some(config);
    }
    
    /// 注册模型
    pub fn register_model(&mut self, model: RoutingModelConfig) {
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
    
    /// 选择模型（综合成本、质量、复杂度信号）
    pub fn select_model(&self, request: &RoutingRequest) -> Option<&RoutingModelConfig> {
        let mut candidates: Vec<&RoutingModelConfig> = self.models.values()
            .filter(|m| {
                m.enabled
                    && m.model_type == request.task_type
                    && self.states.get(&m.id).map_or(false, |s| s.available)
                    && request.required_tags.iter().all(|t| m.tags.contains(t))
            })
            .collect();

        let complexity_score = request.complexity.as_ref()
            .map(|c| ComplexityClassifier::complexity_score(c))
            .unwrap_or(0.5);

        match self.config.strategy {
            RoutingStrategy::Priority => {
                candidates.sort_by(|a, b| b.priority.cmp(&a.priority));
                if complexity_score > 0.7 {
                    candidates.sort_by(|a, b| {
                        let a_cap = a.max_concurrent as u32;
                        let b_cap = b.max_concurrent as u32;
                        b_cap.cmp(&a_cap)
                    });
                }
            }
            RoutingStrategy::LowestCost => {
                candidates.sort_by(|a, b| a.price_per_second.partial_cmp(&b.price_per_second).unwrap_or(std::cmp::Ordering::Equal));
                if complexity_score > 0.6 {
                    candidates.sort_by(|a, b| {
                        let a_quality = a.rate_limit as f64 / (a.price_per_second as f64 + 0.01);
                        let b_quality = b.rate_limit as f64 / (b.price_per_second as f64 + 0.01);
                        b_quality.partial_cmp(&a_quality).unwrap_or(std::cmp::Ordering::Equal)
                    });
                }
            }
            RoutingStrategy::HighestQuality => {
                candidates.sort_by(|a, b| {
                    let a_quality = a.rate_limit as f64 / (a.price_per_second as f64 + 0.01);
                    let b_quality = b.rate_limit as f64 / (b.price_per_second as f64 + 0.01);
                    b_quality.partial_cmp(&a_quality).unwrap_or(std::cmp::Ordering::Equal)
                });
                if complexity_score > 0.5 {
                    candidates.sort_by(|a, b| {
                        b.max_concurrent.cmp(&a.max_concurrent)
                    });
                }
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
                if complexity_score > 0.6 {
                    candidates.sort_by(|a, b| {
                        let a_cap = a.max_concurrent as f32;
                        let b_cap = b.max_concurrent as f32;
                        a_cap.partial_cmp(&b_cap).unwrap_or(std::cmp::Ordering::Equal)
                    });
                }
            }
        }

        candidates.into_iter().next()
    }

    /// 分类并路由 — 使用复杂度画像选择模型层级
    pub fn classify_and_route(&mut self, mut request: RoutingRequest) -> RoutingResponse {
        let profile = ComplexityClassifier::classify_request(&request);
        request.complexity = Some(profile);
        self.route(request)
    }

    /// 投机解码 — 草稿模型快速推理，高置信度则返回，否则用完整模型验证
    pub fn speculative_decoding(&mut self, request: RoutingRequest) -> RoutingResponse {
        let config = match &self.speculative_config {
            Some(c) => c.clone(),
            None => return self.route(request),
        };

        let draft_model_id = config.draft_model_id.clone();
        let verify_model_id = config.verify_model_id.clone();

        let draft_request = {
            let mut req = request.clone();
            req.preferred_model = Some(draft_model_id.clone());
            req
        };

        let draft_response = self.route(draft_request);
        if !draft_response.success {
            return self.route(request);
        }

        let confidence = self.estimate_confidence(&draft_response);
        if confidence >= config.confidence_threshold {
            return draft_response;
        }

        let mut verify_request = request;
        verify_request.preferred_model = Some(verify_model_id);
        verify_request.params.insert("n_gram_overlap".to_string(), serde_json::Value::from(config.n_gram_overlap));
        self.route(verify_request)
    }

    /// 估计草稿模型输出的置信度
    fn estimate_confidence(&self, draft_response: &RoutingResponse) -> f64 {
        if let Some(history_entry) = self.history.last() {
            let success_rate = history_entry.latency_ms as f64 / 1000.0;
            let base_conf = 1.0 - success_rate.min(1.0);
            (base_conf * 0.7 + draft_response.cost * 0.3).min(1.0).max(0.0)
        } else {
            0.5
        }
    }
    
    /// 路由请求
    pub fn route(&mut self, request: RoutingRequest) -> RoutingResponse {
        let mut retries = 0;
        let mut last_error = None;

        let model_id = if let Some(model) = self.select_model(&request) {
            model.id.clone()
        } else {
            retries += 1;
            last_error = Some("无可用模型".to_string());
            String::new()
        };

        if model_id.is_empty() {
            return RoutingResponse {
                success: false,
                selected_model: String::new(),
                output: None,
                cost: 0.0,
                latency_ms: 0,
                retries,
                error: last_error,
            };
        }

        let cost = self.states.get(&model_id)
            .map(|s| s.total_cost)
            .unwrap_or(0.0);
        let response = RoutingResponse {
            success: true,
            selected_model: model_id.clone(),
            output: None,
            cost: cost,
            latency_ms: 0,
            retries,
            error: None,
        };

        if let Some(state) = self.states.get_mut(&model_id) {
            state.total_requests += 1;
            state.total_cost += response.cost;
        }

        self.history.push(response.clone());
        response
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

    fn setup_router() -> ModelRoutingLayer {
        let mut router = ModelRoutingLayer::new();
        router.register_model(RoutingModelConfig {
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
        router.register_model(RoutingModelConfig {
            id: "text_small".to_string(),
            name: "Text Small".to_string(),
            provider: ModelProvider::OpenAI,
            model_type: ModelType::TextGeneration,
            api_endpoint: "https://api.openai.com".to_string(),
            api_key: None,
            max_concurrent: 50,
            rate_limit: 100,
            price_per_second: 0.001,
            max_resolution: None,
            max_duration: None,
            enabled: true,
            priority: 2,
            tags: vec!["text".to_string()],
        });
        router.register_model(RoutingModelConfig {
            id: "text_large".to_string(),
            name: "Text Large".to_string(),
            provider: ModelProvider::Anthropic,
            model_type: ModelType::TextGeneration,
            api_endpoint: "https://api.anthropic.com".to_string(),
            api_key: None,
            max_concurrent: 5,
            rate_limit: 30,
            price_per_second: 0.01,
            max_resolution: None,
            max_duration: None,
            enabled: true,
            priority: 1,
            tags: vec!["text".to_string()],
        });
        router
    }

    fn build_request(task_type: ModelType) -> RoutingRequest {
        RoutingRequest {
            task_type,
            prompt: "test prompt".to_string(),
            params: HashMap::new(),
            preferred_model: None,
            required_tags: vec![],
            complexity: None,
        }
    }

    #[test]
    fn test_model_routing() {
        let mut router = setup_router();
        let request = build_request(ModelType::VideoGeneration);
        let response = router.route(request);
        assert!(response.success);
        assert_eq!(response.selected_model, "kling_001");
    }

    #[test]
    fn test_complexity_classifier() {
        let mut request = build_request(ModelType::VideoGeneration);
        request.params.insert("tool_calls".to_string(), serde_json::Value::from(5));
        request.params.insert("reasoning_depth".to_string(), serde_json::Value::from(3));
        request.params.insert("max_tokens".to_string(), serde_json::Value::from(4096));

        let profile = ComplexityClassifier::classify_request(&request);
        assert_eq!(profile.task_type_complexity, 0.9);
        assert_eq!(profile.tool_call_count, 5);
        assert_eq!(profile.reasoning_depth, 3);
        assert_eq!(profile.output_size_estimate, 4096);
    }

    #[test]
    fn test_complexity_score_range() {
        let profile = ComplexityProfile {
            task_type_complexity: 0.9,
            context_length_pct: 0.5,
            tool_call_count: 3,
            reasoning_depth: 2,
            parallel_ops: 2,
            output_size_estimate: 2048,
            cross_ref_count: 1,
            error_recovery_complexity: 0.5,
            security_sensitivity: 0.3,
            latency_tolerance: 0.5,
            cost_sensitivity: 0.5,
            model_capability_reqs: 0b010,
            format_requirements: 0b001,
            historical_success_rate: 0.85,
        };
        let score = ComplexityClassifier::complexity_score(&profile);
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_classify_and_route() {
        let mut router = setup_router();
        let request = build_request(ModelType::TextGeneration);
        let response = router.classify_and_route(request);
        assert!(response.success);
        assert!(!response.selected_model.is_empty());
        assert!(router.history.len() >= 1);
    }

    #[test]
    fn test_complexity_field_on_request() {
        let request = RoutingRequest {
            task_type: ModelType::TextGeneration,
            prompt: "test".to_string(),
            params: HashMap::new(),
            preferred_model: None,
            required_tags: vec![],
            complexity: Some(ComplexityProfile {
                task_type_complexity: 0.5,
                context_length_pct: 0.3,
                tool_call_count: 0,
                reasoning_depth: 0,
                parallel_ops: 1,
                output_size_estimate: 1024,
                cross_ref_count: 0,
                error_recovery_complexity: 0.2,
                security_sensitivity: 0.1,
                latency_tolerance: 0.5,
                cost_sensitivity: 0.5,
                model_capability_reqs: 0b001,
                format_requirements: 0,
                historical_success_rate: 0.85,
            }),
        };
        assert!(request.complexity.is_some());
        assert_eq!(request.complexity.as_ref().unwrap().task_type_complexity, 0.5);
    }

    #[test]
    fn test_select_model_with_complexity() {
        let mut router = setup_router();
        let mut request = build_request(ModelType::TextGeneration);
        request.complexity = Some(ComplexityProfile {
            task_type_complexity: 0.9,
            context_length_pct: 0.8,
            tool_call_count: 10,
            reasoning_depth: 5,
            parallel_ops: 8,
            output_size_estimate: 8192,
            cross_ref_count: 20,
            error_recovery_complexity: 0.9,
            security_sensitivity: 0.5,
            latency_tolerance: 0.3,
            cost_sensitivity: 0.2,
            model_capability_reqs: 0b001,
            format_requirements: 0b001,
            historical_success_rate: 0.9,
        });
        let model = router.select_model(&request);
        assert!(model.is_some());
    }

    #[test]
    fn test_speculative_decoding_config_default() {
        let config = SpeculativeDecodingConfig::default();
        assert_eq!(config.confidence_threshold, 0.85);
        assert_eq!(config.draft_model_id, "draft_small");
        assert_eq!(config.verify_model_id, "verify_large");
        assert_eq!(config.n_gram_overlap, 0.9);
    }

    #[test]
    fn test_speculative_decoding_without_config() {
        let mut router = setup_router();
        let request = build_request(ModelType::TextGeneration);
        let response = router.speculative_decoding(request);
        assert!(response.success);
    }

    #[test]
    fn test_speculative_decoding_with_config() {
        let mut router = setup_router();
        router.register_model(RoutingModelConfig {
            id: "draft_small".to_string(),
            name: "Draft Small".to_string(),
            provider: ModelProvider::OpenAI,
            model_type: ModelType::TextGeneration,
            api_endpoint: "https://api.openai.com".to_string(),
            api_key: None,
            max_concurrent: 100,
            rate_limit: 200,
            price_per_second: 0.0001,
            max_resolution: None,
            max_duration: None,
            enabled: true,
            priority: 3,
            tags: vec!["text".to_string()],
        });
        router.register_model(RoutingModelConfig {
            id: "verify_large".to_string(),
            name: "Verify Large".to_string(),
            provider: ModelProvider::Anthropic,
            model_type: ModelType::TextGeneration,
            api_endpoint: "https://api.anthropic.com".to_string(),
            api_key: None,
            max_concurrent: 5,
            rate_limit: 30,
            price_per_second: 0.01,
            max_resolution: None,
            max_duration: None,
            enabled: true,
            priority: 1,
            tags: vec!["text".to_string()],
        });

        let config = SpeculativeDecodingConfig {
            confidence_threshold: 0.3,
            draft_model_id: "draft_small".to_string(),
            verify_model_id: "verify_large".to_string(),
            n_gram_overlap: 0.9,
        };
        router.with_speculative_decoding(config);

        let request = build_request(ModelType::TextGeneration);
        let response = router.speculative_decoding(request);
        assert!(response.success);
    }

    #[test]
    fn test_speculative_decoding_high_confidence_skips_verify() {
        let mut router = setup_router();
        router.register_model(RoutingModelConfig {
            id: "draft_small".to_string(),
            name: "Draft Small".to_string(),
            provider: ModelProvider::OpenAI,
            model_type: ModelType::TextGeneration,
            api_endpoint: "https://api.openai.com".to_string(),
            api_key: None,
            max_concurrent: 100,
            rate_limit: 200,
            price_per_second: 0.0001,
            max_resolution: None,
            max_duration: None,
            enabled: true,
            priority: 3,
            tags: vec!["text".to_string()],
        });
        router.register_model(RoutingModelConfig {
            id: "verify_large".to_string(),
            name: "Verify Large".to_string(),
            provider: ModelProvider::Anthropic,
            model_type: ModelType::TextGeneration,
            api_endpoint: "https://api.anthropic.com".to_string(),
            api_key: None,
            max_concurrent: 5,
            rate_limit: 30,
            price_per_second: 0.01,
            max_resolution: None,
            max_duration: None,
            enabled: true,
            priority: 1,
            tags: vec!["text".to_string()],
        });

        let config = SpeculativeDecodingConfig {
            confidence_threshold: 0.0,
            draft_model_id: "draft_small".to_string(),
            verify_model_id: "verify_large".to_string(),
            n_gram_overlap: 0.9,
        };
        router.with_speculative_decoding(config);

        let request = build_request(ModelType::TextGeneration);
        let response = router.speculative_decoding(request);
        assert!(response.success);
        assert_eq!(response.selected_model, "draft_small");
    }

    #[test]
    fn test_security_sensitive_classification() {
        let mut request = build_request(ModelType::TextGeneration);
        request.prompt = "Give me the password and secret token".to_string();
        let profile = ComplexityClassifier::classify_request(&request);
        assert!(profile.security_sensitivity > 0.5);
    }

    #[test]
    fn test_model_capability_reqs() {
        let text_reqs = ComplexityClassifier::compute_capability_reqs(&ModelType::TextGeneration);
        assert_eq!(text_reqs, 0b001);
        let video_reqs = ComplexityClassifier::compute_capability_reqs(&ModelType::VideoGeneration);
        assert_eq!(video_reqs, 0b100);
    }
}