//! 平台网关模块 (通用)
//!
//! 统一接口适配多平台
//! 适用于：所有需要多平台集成的场景

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 平台定义
// ============================================================================

/// 平台类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub(crate) enum PlatformType {
    /// ComfyUI
    ComfyUI,
    /// SD WebUI
    SDWebUI,
    /// Runway
    Runway,
    /// Pika
    Pika,
    /// Kling
    Kling,
    /// Luma
    Luma,
    /// Stable Video
    StableVideo,
    /// 自定义平台
    Custom,
}

/// 平台状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub(crate) enum PlatformStatus {
    /// 在线
    Online,
    /// 离线
    Offline,
    /// 维护中
    Maintenance,
    /// 限流
    RateLimited,
}

/// 平台配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PlatformConfig {
    /// 平台ID
    pub id: String,
    /// 平台名称
    pub name: String,
    /// 平台类型
    pub platform_type: PlatformType,
    /// API端点
    pub api_endpoint: String,
    /// API密钥
    pub api_key: Option<String>,
    /// 是否启用
    pub enabled: bool,
    /// 优先级
    pub priority: u32,
    /// 速率限制 (请求/分钟)
    pub rate_limit: u32,
    /// 自定义参数
    pub params: HashMap<String, serde_json::Value>,
}

/// 平台能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PlatformCapability {
    /// 支持的任务类型
    pub task_types: Vec<String>,
    /// 支持的输入格式
    pub input_formats: Vec<String>,
    /// 支持的输出格式
    pub output_formats: Vec<String>,
    /// 最大分辨率
    pub max_resolution: (u32, u32),
    /// 最大帧数
    pub max_frames: u32,
    /// 是否支持批量
    pub supports_batch: bool,
}

/// 平台请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PlatformRequest {
    /// 任务类型
    pub task_type: String,
    /// 输入文件路径
    pub input_path: String,
    /// 输出格式
    pub output_format: String,
    /// 参数
    pub params: HashMap<String, serde_json::Value>,
    /// 超时时间 (秒)
    pub timeout_secs: u32,
    /// 回调URL
    pub callback_url: Option<String>,
}

/// 平台响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PlatformResponse {
    /// 是否成功
    pub success: bool,
    /// 任务ID
    pub task_id: Option<String>,
    /// 输出文件路径
    pub output_path: Option<String>,
    /// 处理时间 (毫秒)
    pub processing_time_ms: u64,
    /// 错误信息
    pub error: Option<String>,
    /// 平台元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 平台网关配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PlatformGatewayConfig {
    /// 默认超时时间 (秒)
    pub default_timeout_secs: u32,
    /// 是否启用负载均衡
    pub enable_load_balancing: bool,
    /// 是否启用故障转移
    pub enable_failover: bool,
    /// 最大重试次数
    pub max_retries: u32,
    /// 是否启用缓存
    pub enable_cache: bool,
    /// 缓存过期时间 (秒)
    pub cache_ttl_secs: u32,
}

// ============================================================================
// 平台网关
// ============================================================================

/// 平台网关
/// 统一管理多平台集成
#[derive(Debug)]
pub(crate) struct PlatformGateway {
    /// 配置
    #[allow(dead_code)]
    config: PlatformGatewayConfig,
    /// 平台配置列表
    platforms: HashMap<String, PlatformConfig>,
    /// 平台状态
    statuses: HashMap<String, PlatformStatus>,
    /// 平台能力
    capabilities: HashMap<String, PlatformCapability>,
    /// 请求历史
    history: Vec<(String, PlatformResponse)>,
}

impl PlatformGateway {
    /// 创建网关
    pub fn new() -> Self {
        Self {
            config: PlatformGatewayConfig {
                default_timeout_secs: 300,
                enable_load_balancing: true,
                enable_failover: true,
                max_retries: 3,
                enable_cache: true,
                cache_ttl_secs: 300,
            },
            platforms: HashMap::new(),
            statuses: HashMap::new(),
            capabilities: HashMap::new(),
            history: vec![],
        }
    }
    
    /// 使用配置创建
    pub fn with_config(config: PlatformGatewayConfig) -> Self {
        Self {
            config,
            platforms: HashMap::new(),
            statuses: HashMap::new(),
            capabilities: HashMap::new(),
            history: vec![],
        }
    }
    
    /// 注册平台
    pub fn register_platform(
        &mut self,
        config: PlatformConfig,
        capability: PlatformCapability,
    ) {
        let id = config.id.clone();
        self.platforms.insert(id.clone(), config);
        self.capabilities.insert(id.clone(), capability);
        self.statuses.insert(id, PlatformStatus::Online);
    }
    
    /// 获取可用平台
    pub fn get_available_platforms(&self, task_type: &str) -> Vec<&PlatformConfig> {
        self.platforms.values()
            .filter(|p| {
                p.enabled
                    && self.statuses.get(&p.id) == Some(&PlatformStatus::Online)
                    && self.capabilities.get(&p.id)
                        .map_or(false, |c| c.task_types.contains(&task_type.to_string()))
            })
            .collect()
    }
    
    /// 选择最佳平台
    pub fn select_best_platform(&self, task_type: &str) -> Option<&PlatformConfig> {
        let mut available = self.get_available_platforms(task_type);
        available.sort_by(|a, b| b.priority.cmp(&a.priority));
        available.into_iter().next()
    }
    
    /// 发送请求
    pub fn send_request(
        &mut self,
        platform_id: &str,
        _request: PlatformRequest,
    ) -> PlatformResponse {
        let _platform = match self.platforms.get(platform_id) {
            Some(p) => p,
            None => {
                return PlatformResponse {
                    success: false,
                    task_id: None,
                    output_path: None,
                    processing_time_ms: 0,
                    error: Some("平台不存在".to_string()),
                    metadata: HashMap::new(),
                };
            }
        };
        
        // TODO: 实际调用平台 API
        let response = PlatformResponse {
            success: true,
            task_id: Some(format!("task_{}", platform_id)),
            output_path: Some(format!("{}_output.png", platform_id)),
            processing_time_ms: 2000,
            error: None,
            metadata: HashMap::new(),
        };
        
        self.history.push((platform_id.to_string(), response.clone()));
        response
    }
    
    /// 带故障转移的请求
    pub fn send_request_with_failover(
        &mut self,
        task_type: &str,
        request: &PlatformRequest,
    ) -> PlatformResponse {
        let platform_ids: Vec<String> = self.get_available_platforms(task_type)
            .iter().map(|p| p.id.clone()).collect();

        for pid in &platform_ids {
            let response = self.send_request(pid, request.clone());
            if response.success {
                return response;
            }
        }
        
        PlatformResponse {
            success: false,
            task_id: None,
            output_path: None,
            processing_time_ms: 0,
            error: Some("所有平台都失败".to_string()),
            metadata: HashMap::new(),
        }
    }
    
    /// 获取平台状态
    pub fn get_platform_status(&self, platform_id: &str) -> Option<PlatformStatus> {
        self.statuses.get(platform_id).cloned()
    }
    
    /// 更新平台状态
    pub fn update_platform_status(&mut self, platform_id: &str, status: PlatformStatus) {
        self.statuses.insert(platform_id.to_string(), status);
    }
    
    /// 获取统计信息
    pub fn statistics(&self) -> GatewayStats {
        let total_platforms = self.platforms.len();
        let online = self.statuses.values().filter(|s| **s == PlatformStatus::Online).count();
        let total_requests = self.history.len();
        let successful = self.history.iter().filter(|(_, r)| r.success).count();
        
        GatewayStats {
            total_platforms,
            online_platforms: online,
            total_requests,
            successful_requests: successful,
        }
    }
}

/// 网关统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct GatewayStats {
    /// 总平台数
    pub total_platforms: usize,
    /// 在线平台数
    pub online_platforms: usize,
    /// 总请求数
    pub total_requests: usize,
    /// 成功请求数
    pub successful_requests: usize,
}

// ============================================================================
// 向后兼容别名
// ============================================================================

/// 平台适配器 (向后兼容别名)
pub type PlatformAdapter = PlatformGateway;

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_platform_gateway() {
        let mut gateway = PlatformGateway::new();
        
        gateway.register_platform(
            PlatformConfig {
                id: "comfyui".to_string(),
                name: "ComfyUI".to_string(),
                platform_type: PlatformType::ComfyUI,
                api_endpoint: "http://localhost:8188".to_string(),
                api_key: None,
                enabled: true,
                priority: 1,
                rate_limit: 60,
                params: HashMap::new(),
            },
            PlatformCapability {
                task_types: vec!["img2img".to_string(), "txt2img".to_string()],
                input_formats: vec!["png".to_string(), "jpg".to_string()],
                output_formats: vec!["png".to_string()],
                max_resolution: (2048, 2048),
                max_frames: 1,
                supports_batch: true,
            },
        );
        
        let platforms = gateway.get_available_platforms("img2img");
        assert!(!platforms.is_empty());
        
        let stats = gateway.statistics();
        assert_eq!(stats.total_platforms, 1);
        assert_eq!(stats.online_platforms, 1);
    }
}