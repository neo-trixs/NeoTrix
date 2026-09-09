//! 信息吸收器 (InformationAbsorber)
//! 
//! 从外部资源获取信息，进行初步处理和分类
//! 
//! 支持资源: GPT-4o, Claude, DeepSeek, Qwen3, Perplexity, GitHub, arXiv, HuggingFace

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 信息吸收器
pub struct InformationAbsorber {
    /// 已注册的资源
    pub resources: HashMap<String, Resource>,
    /// 吸收历史
    pub absorption_history: Vec<AbsorptionRecord>,
    /// 吸收配置
    pub config: AbsorberConfig,
}

/// 吸收配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorberConfig {
    /// 最大历史记录
    pub max_history: usize,
    /// 最大并发请求
    pub max_concurrent: usize,
    /// 请求超时（秒）
    pub request_timeout: u64,
    /// 最大重试次数
    pub max_retries: u32,
}

impl Default for AbsorberConfig {
    fn default() -> Self {
        Self {
            max_history: 1000,
            max_concurrent: 5,
            request_timeout: 30,
            max_retries: 3,
        }
    }
}

/// 资源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// 资源ID
    pub id: String,
    /// 资源名称
    pub name: String,
    /// 资源类型
    pub resource_type: ResourceType,
    /// 资源URL或端点
    pub endpoint: String,
    /// 能力描述
    pub capabilities: Vec<String>,
    /// 可用性
    pub availability: f64,
    /// 质量评分
    pub quality_score: f64,
}

/// 资源类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    /// LLM 推理
    LLMInference,
    /// 知识检索
    KnowledgeRetrieval,
    /// 代码搜索
    CodeSearch,
    /// 论文搜索
    PaperSearch,
    /// 模型仓库
    ModelRepository,
    /// 自定义资源
    Custom(String),
}

/// 吸收记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorptionRecord {
    /// 记录ID
    pub id: String,
    /// 周期
    pub cycle: u32,
    /// 请求
    pub request: AbsorptionRequest,
    /// 响应
    pub response: Option<AbsorptionResponse>,
    /// 状态
    pub status: AbsorptionStatus,
    /// 时间戳
    pub timestamp: String,
    /// 耗时（毫秒）
    pub duration_ms: u64,
}

/// 吸收请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorptionRequest {
    /// 请求ID
    pub id: String,
    /// 目标资源
    pub resource_id: String,
    /// 查询内容
    pub query: String,
    /// 查询类型
    pub query_type: QueryType,
    /// 上下文
    pub context: HashMap<String, String>,
}

/// 查询类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    /// 知识查询
    Knowledge,
    /// 代码查询
    Code,
    /// 论文查询
    Paper,
    /// 推理查询
    Reasoning,
    /// 创意查询
    Creative,
}

/// 吸收响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorptionResponse {
    /// 响应ID
    pub id: String,
    /// 内容
    pub content: String,
    /// 置信度
    pub confidence: f64,
    /// 来源
    pub source: String,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 吸收状态
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AbsorptionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Timeout,
    Retry,
}

impl InformationAbsorber {
    /// 创建新的信息吸收器
    pub fn new(config: AbsorberConfig) -> Self {
        Self {
            resources: HashMap::new(),
            absorption_history: Vec::new(),
            config,
        }
    }

    /// 注册资源
    pub fn register_resource(&mut self, resource: Resource) {
        self.resources.insert(resource.id.clone(), resource);
    }

    /// 吸收信息
    pub fn absorb(&mut self, cycle: u32, request: AbsorptionRequest) -> AbsorptionResponse {
        let start_time = std::time::Instant::now();
        
        // 查找资源
        let resource = self.resources.get(&request.resource_id)
            .cloned()
            .unwrap_or_else(|| {
                // 默认资源
                Resource {
                    id: "default".to_string(),
                    name: "Default Resource".to_string(),
                    resource_type: ResourceType::LLMInference,
                    endpoint: "https://api.openai.com".to_string(),
                    capabilities: vec!["reasoning".to_string()],
                    availability: 0.95,
                    quality_score: 0.85,
                }
            });

        // 模拟吸收过程
        let response = AbsorptionResponse {
            id: format!("resp_{}", uuid::Uuid::new_v4()),
            content: format!("从{}吸收: {}", resource.name, request.query),
            confidence: resource.quality_score,
            source: resource.name.clone(),
            metadata: HashMap::from([
                ("resource_type".to_string(), format!("{:?}", resource.resource_type)),
                ("query_type".to_string(), format!("{:?}", request.query_type)),
            ]),
        };

        let duration = start_time.elapsed().as_millis() as u64;

        // 记录
        let record = AbsorptionRecord {
            id: format!("abs_{}", uuid::Uuid::new_v4()),
            cycle,
            request,
            response: Some(response.clone()),
            status: AbsorptionStatus::Completed,
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms: duration,
        };

        self.absorption_history.push(record);
        self.trim_history();

        response
    }

    /// 获取资源列表
    pub fn list_resources(&self) -> Vec<&Resource> {
        self.resources.values().collect()
    }

    /// 获取吸收统计
    pub fn stats(&self) -> AbsorberStats {
        let total_absorptions = self.absorption_history.len();
        let successful_absorptions = self.absorption_history.iter()
            .filter(|r| r.status == AbsorptionStatus::Completed)
            .count();
        
        let avg_duration = if total_absorptions > 0 {
            self.absorption_history.iter()
                .map(|r| r.duration_ms)
                .sum::<u64>() / total_absorptions as u64
        } else {
            0
        };

        let avg_confidence = if total_absorptions > 0 {
            self.absorption_history.iter()
                .filter_map(|r| r.response.as_ref())
                .map(|r| r.confidence)
                .sum::<f64>() / total_absorptions as f64
        } else {
            0.0
        };

        AbsorberStats {
            total_absorptions,
            successful_absorptions,
            success_rate: if total_absorptions > 0 {
                successful_absorptions as f64 / total_absorptions as f64
            } else {
                0.0
            },
            avg_duration_ms: avg_duration,
            avg_confidence,
            registered_resources: self.resources.len(),
        }
    }

    /// 裁剪历史
    fn trim_history(&mut self) {
        while self.absorption_history.len() > self.config.max_history {
            self.absorption_history.remove(0);
        }
    }
}

/// 吸收统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorberStats {
    pub total_absorptions: usize,
    pub successful_absorptions: usize,
    pub success_rate: f64,
    pub avg_duration_ms: u64,
    pub avg_confidence: f64,
    pub registered_resources: usize,
}

impl std::fmt::Display for AbsorberStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "        InformationAbsorber 统计")?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "总吸收次数:      {}", self.total_absorptions)?;
        writeln!(f, "成功次数:        {}", self.successful_absorptions)?;
        writeln!(f, "成功率:          {:.2}%", self.success_rate * 100.0)?;
        writeln!(f, "平均耗时:        {}ms", self.avg_duration_ms)?;
        writeln!(f, "平均置信度:      {:.4}", self.avg_confidence)?;
        writeln!(f, "注册资源:        {}", self.registered_resources)?;
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_absorber_creation() {
        let absorber = InformationAbsorber::new(AbsorberConfig::default());
        assert_eq!(absorber.resources.len(), 0);
        assert_eq!(absorber.absorption_history.len(), 0);
    }

    #[test]
    fn test_absorb_information() {
        let mut absorber = InformationAbsorber::new(AbsorberConfig::default());
        let request = AbsorptionRequest {
            id: "req_1".to_string(),
            resource_id: "default".to_string(),
            query: "测试查询".to_string(),
            query_type: QueryType::Knowledge,
            context: HashMap::new(),
        };
        
        let response = absorber.absorb(0, request);
        assert_eq!(absorber.absorption_history.len(), 1);
        assert!(response.confidence > 0.0);
    }
}
