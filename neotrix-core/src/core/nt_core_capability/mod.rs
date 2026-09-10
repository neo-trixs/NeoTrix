//! NeoTrix 统一能力接口
//!
//! 定义跨域调用的标准接口，支持:
//! - 能力注册与发现
//! - 统一输入/输出
//! - 路由调度
//! - 健康监控
//! - 跨域组合
//! - 结果缓存

use std::collections::HashMap;
use std::sync::Arc;

pub mod cache;
pub mod composer;
pub mod dependency;
pub mod discovery;
pub mod factory;
pub mod hotreload;
pub mod integrator;
pub mod loadbalancer;
pub mod monitor;
pub mod monitoring;
pub mod orchestrator;
pub mod performance;
pub mod security;
pub mod versioning;

// 测试和文档模块
#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod tests;

/// 层级定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    L1Action,
    L2Perception,
    L3Embodiment,
    L4Emotion,
    L5Cognition,
    L6Meta,
}

/// 域定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Domain {
    NtCore,
    NtMind,
    NtMemory,
    NtWorld,
    NtAct,
    NtIo,
    NtShield,
    NtPhysical,
    NtFeel,
}

/// 能力元数据
#[derive(Debug, Clone)]
pub struct CapabilityMeta {
    /// 能力ID
    pub id: String,
    /// 能力名称
    pub name: String,
    /// 所属层级
    pub layer: Layer,
    /// 所属域
    pub domain: Domain,
    /// 版本
    pub version: String,
    /// 描述
    pub description: String,
    /// 标签
    pub tags: Vec<String>,
    /// 状态
    pub status: CapabilityStatus,
    /// 指标
    pub metrics: CapabilityMetrics,
}

/// 能力状态指示器
#[derive(Debug, Clone)]
pub enum CapabilityStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl Default for CapabilityStatus {
    fn default() -> Self {
        Self::Healthy
    }
}

/// 能力指标
#[derive(Debug, Clone)]
pub struct CapabilityMetrics {
    pub total_calls: u64,
    pub total_errors: u64,
    pub avg_latency_ms: f64,
    pub last_called: Option<std::time::Instant>,
}

impl Default for CapabilityMetrics {
    fn default() -> Self {
        Self {
            total_calls: 0,
            total_errors: 0,
            avg_latency_ms: 0.0,
            last_called: None,
        }
    }
}

/// 能力状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityState {
    /// 就绪
    Ready,
    /// 健康
    Healthy,
    /// 运行中
    Running,
    /// 错误
    Error(String),
    /// 禁用
    Disabled,
}

/// 能力健康度
#[derive(Debug, Clone)]
pub struct CapabilityHealth {
    /// 状态
    pub state: CapabilityState,
    /// 成功率
    pub success_rate: f64,
    /// 平均延迟 (ms)
    pub avg_latency_ms: f64,
    /// 最后调用时间
    pub last_called: Option<std::time::Instant>,
    /// 调用次数
    pub call_count: u64,
}

/// 统一输入
#[derive(Debug, Clone)]
pub enum CapabilityInput {
    /// 文本输入
    Text(String),
    /// 网络数据
    Network(NetworkInput),
    /// 安全查询
    Security(SecurityInput),
    /// NLP请求
    Nlp(NlpInput),
    /// 资产查询
    Asset(AssetInput),
    /// 通用KV
    Kv(HashMap<String, String>),
}

/// 网络输入
#[derive(Debug, Clone)]
pub struct NetworkInput {
    pub target: String,
    pub ports: Vec<u16>,
    pub timeout_ms: u64,
}

/// 安全输入
#[derive(Debug, Clone)]
pub struct SecurityInput {
    pub query_type: String,
    pub parameters: HashMap<String, String>,
}

/// NLP输入
#[derive(Debug, Clone)]
pub struct NlpInput {
    pub task: NlpTask,
    pub text: String,
    pub language: Option<String>,
}

/// NLP任务类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NlpTask {
    /// 分词
    Tokenize,
    /// 语言检测
    DetectLanguage,
    /// 关键词提取
    ExtractKeywords,
    /// 信息提取
    Extract,
    /// 情感分析
    SentimentAnalysis,
    /// 相似度计算
    Similarity,
    /// 信息提取
    ExtractInfo,
    /// 文本分类
    Classify,
}

/// 资产输入
#[derive(Debug, Clone)]
pub struct AssetInput {
    pub query: String,
    pub asset_type: Option<String>,
    pub limit: usize,
}

/// 统一输出
#[derive(Debug)]
pub enum CapabilityOutput {
    /// 文本输出
    Text(String),
    /// LLM结果
    LlmResult(LlmResult),
    /// CLI结果
    CliResult(CliResult),
    /// 网络结果
    Network(NetworkOutput),
    /// 网络结果 (别名)
    NetworkResult(NetworkResult),
    /// 安全结果
    Security(SecurityOutput),
    /// 安全扫描结果
    SecurityScan(SecurityScanResult),
    /// NLP结果
    Nlp(NlpOutput),
    /// 资产结果
    Asset(AssetOutput),
    /// Web结果
    WebResult(WebResult),
    /// 爬取结果
    CrawlResult(CrawlResult),
    /// 解析内容
    ParsedContent(ParsedContent),
    /// 图查询结果
    GraphQuery(GraphQueryResult),
    /// 威胁分析
    ThreatAnalysis(ThreatResult),
    /// 仿真结果
    SimulationResult(Box<dyn std::fmt::Debug + Send + Sync>),
    /// 传感器数据
    SensorData(Box<dyn std::fmt::Debug + Send + Sync>),
    /// 自模型结果
    SelfModelResult(Box<dyn std::fmt::Debug + Send + Sync>),
    /// 路由结果
    RoutingResult(Box<dyn std::fmt::Debug + Send + Sync>),
    /// 调控结果
    RegulationResult(Box<dyn std::fmt::Debug + Send + Sync>),
    /// 推理结果
    ReasoningResult(Box<dyn std::fmt::Debug + Send + Sync>),
    /// HyperCube结果
    HyperCubeResult(Box<dyn std::fmt::Debug + Send + Sync>),
    /// 情感表达
    EmotionExpression(Box<dyn std::fmt::Debug + Send + Sync>),
    /// 情感分析
    EmotionAnalysis(Box<dyn std::fmt::Debug + Send + Sync>),
    /// 执行器指令
    ActuatorCommand(Box<dyn std::fmt::Debug + Send + Sync>),
    /// 通用KV
    Kv(HashMap<String, String>),
    /// 错误
    Error(String),
}

impl Clone for CapabilityOutput {
    fn clone(&self) -> Self {
        match self {
            Self::Text(s) => Self::Text(s.clone()),
            Self::LlmResult(r) => Self::LlmResult(r.clone()),
            Self::CliResult(r) => Self::CliResult(r.clone()),
            Self::Network(o) => Self::Network(o.clone()),
            Self::NetworkResult(o) => Self::NetworkResult(o.clone()),
            Self::Security(o) => Self::Security(o.clone()),
            Self::SecurityScan(o) => Self::SecurityScan(o.clone()),
            Self::Nlp(o) => Self::Nlp(o.clone()),
            Self::Asset(o) => Self::Asset(o.clone()),
            Self::WebResult(r) => Self::WebResult(r.clone()),
            Self::CrawlResult(r) => Self::CrawlResult(r.clone()),
            Self::ParsedContent(r) => Self::ParsedContent(r.clone()),
            Self::GraphQuery(r) => Self::GraphQuery(r.clone()),
            Self::ThreatAnalysis(r) => Self::ThreatAnalysis(r.clone()),
            Self::SimulationResult(d) => Self::Text(format!("{:?}", d)),
            Self::SensorData(d) => Self::Text(format!("{:?}", d)),
            Self::SelfModelResult(d) => Self::Text(format!("{:?}", d)),
            Self::RoutingResult(d) => Self::Text(format!("{:?}", d)),
            Self::RegulationResult(d) => Self::Text(format!("{:?}", d)),
            Self::ReasoningResult(d) => Self::Text(format!("{:?}", d)),
            Self::HyperCubeResult(d) => Self::Text(format!("{:?}", d)),
            Self::EmotionExpression(d) => Self::Text(format!("{:?}", d)),
            Self::EmotionAnalysis(d) => Self::Text(format!("{:?}", d)),
            Self::ActuatorCommand(d) => Self::Text(format!("{:?}", d)),
            Self::Kv(m) => Self::Kv(m.clone()),
            Self::Error(s) => Self::Error(s.clone()),
        }
    }
}

/// 网络输出
#[derive(Debug, Clone)]
pub struct NetworkOutput {
    pub target: String,
    pub open_ports: Vec<u16>,
    pub services: Vec<String>,
    pub latency_ms: u64,
}

/// 安全输出
#[derive(Debug, Clone)]
pub struct SecurityOutput {
    pub decision: SecurityDecision,
    pub reason: String,
    pub details: HashMap<String, String>,
}

/// 安全决策
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityDecision {
    Allow,
    Deny,
    Challenge,
    Log,
}

/// NLP输出
#[derive(Debug, Clone)]
pub struct NlpOutput {
    pub task: NlpTask,
    pub result: NlpResult,
}

/// NLP结果
#[derive(Debug, Clone)]
pub enum NlpResult {
    Tokens(Vec<String>),
    Language(String),
    Keywords(Vec<KeywordResult>),
    Sentiment(SentimentResult),
    Similarity(f64),
    Entities(Vec<EntityResult>),
    Classification(ClassificationResult),
}

/// 关键词结果
#[derive(Debug, Clone)]
pub struct KeywordResult {
    pub word: String,
    pub score: f64,
}

/// 情感结果
#[derive(Debug, Clone)]
pub struct SentimentResult {
    pub polarity: String,
    pub score: f64,
}

/// 实体结果
#[derive(Debug, Clone)]
pub struct EntityResult {
    pub text: String,
    pub entity_type: String,
    pub start: usize,
    pub end: usize,
}

/// 分类结果
#[derive(Debug, Clone)]
pub struct ClassificationResult {
    pub label: String,
    pub confidence: f64,
}

/// 资产输出
#[derive(Debug, Clone)]
pub struct AssetOutput {
    pub assets: Vec<AssetInfo>,
    pub total: usize,
}

/// 资产信息
#[derive(Debug, Clone)]
pub struct AssetInfo {
    pub ip: String,
    pub ports: Vec<u16>,
    pub services: Vec<String>,
    pub tags: Vec<String>,
}

/// LLM结果
#[derive(Debug, Clone)]
pub struct LlmResult {
    pub model: String,
    pub input: String,
    pub output: String,
    pub tokens_used: u64,
    pub latency_ms: u64,
    pub cost: f64,
}

/// CLI结果
#[derive(Debug, Clone)]
pub struct CliResult {
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
}

/// Web结果
#[derive(Debug, Clone)]
pub struct WebResult {
    pub method: String,
    pub url: String,
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub latency_ms: u64,
}

/// ACP结果
#[derive(Debug, Clone)]
pub struct AcpResult {
    pub protocol: String,
    pub action: String,
    pub request_id: String,
    pub status: String,
    pub response: String,
    pub metadata: HashMap<String, String>,
}

/// 网络结果 (安全域)
#[derive(Debug, Clone)]
pub struct NetworkResult {
    pub action: String,
    pub source: String,
    pub destination: String,
    pub allowed: bool,
    pub reason: String,
    pub risk_score: f64,
}

/// 安全扫描结果
#[derive(Debug, Clone)]
pub struct SecurityScanResult {
    pub target: String,
    pub scan_type: String,
    pub findings: Vec<SecurityFinding>,
    pub risk_score: f64,
    pub scan_time_ms: u64,
}

/// 安全发现
#[derive(Debug, Clone)]
pub struct SecurityFinding {
    pub severity: String,
    pub category: String,
    pub description: String,
    pub recommendation: String,
}

/// 威胁结果
#[derive(Debug, Clone)]
pub struct ThreatResult {
    pub input: String,
    pub threats: Vec<String>,
    pub risk_level: String,
    pub confidence: f64,
    pub recommended_actions: Vec<String>,
}

/// 爬取结果
#[derive(Debug, Clone)]
pub struct CrawlResult {
    pub url: String,
    pub status: String,
    pub content_type: String,
    pub content_length: u64,
    pub extracted_text: String,
    pub links: Vec<String>,
    pub metadata: CrawlMetadata,
}

/// 爬取元数据
#[derive(Debug, Clone)]
pub struct CrawlMetadata {
    pub title: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub author: String,
}

/// 解析内容
#[derive(Debug, Clone)]
pub struct ParsedContent {
    pub format: String,
    pub title: String,
    pub sections: Vec<ContentSection>,
    pub entities: Vec<ParsedEntity>,
    pub relationships: Vec<ParsedRelationship>,
}

/// 内容章节
#[derive(Debug, Clone)]
pub struct ContentSection {
    pub heading: String,
    pub content: String,
    pub level: u32,
}

/// 解析实体
#[derive(Debug, Clone)]
pub struct ParsedEntity {
    pub name: String,
    pub entity_type: String,
    pub confidence: f64,
}

/// 解析关系
#[derive(Debug, Clone)]
pub struct ParsedRelationship {
    pub source: String,
    pub target: String,
    pub relationship_type: String,
    pub weight: f64,
}

/// 图查询结果
#[derive(Debug, Clone)]
pub struct GraphQueryResult {
    pub query: String,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub confidence: f64,
}

/// 图节点
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub properties: HashMap<String, String>,
}

/// 图边
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relationship: String,
    pub weight: f64,
}

/// 统一能力 trait
pub trait UnifiedCapability: Send + Sync {
    /// 获取元数据
    fn meta(&self) -> CapabilityMeta;

    /// 获取健康状态
    fn health(&self) -> CapabilityHealth;

    /// 执行能力
    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError>;

    /// 重置状态
    fn reset(&mut self) {}

    /// 检查是否支持该输入类型
    fn supports(&self, input: &CapabilityInput) -> bool;
}

/// 能力错误
#[derive(Debug, thiserror::Error)]
pub enum CapabilityError {
    #[error("不支持的输入类型: {0}")]
    UnsupportedInput(String),
    #[error("执行失败: {0}")]
    ExecutionFailed(String),
    #[error("超时")]
    Timeout,
    #[error("资源不足")]
    InsufficientResources,
    #[error("未就绪")]
    NotReady,
}

/// 能力注册中心
#[derive(Clone)]
pub struct CapabilityRegistry {
    /// 已注册的能力
    capabilities: HashMap<String, Arc<dyn UnifiedCapability>>,
    /// 按域索引
    by_domain: HashMap<Domain, Vec<String>>,
    /// 按层级索引
    by_layer: HashMap<Layer, Vec<String>>,
}

impl CapabilityRegistry {
    /// 创建新的注册中心
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
            by_domain: HashMap::new(),
            by_layer: HashMap::new(),
        }
    }

    /// 注册能力
    pub fn register(&mut self, cap: Arc<dyn UnifiedCapability>) {
        let meta = cap.meta();
        let id = meta.id.clone();

        // 索引
        self.by_domain
            .entry(meta.domain)
            .or_default()
            .push(id.clone());
        self.by_layer
            .entry(meta.layer)
            .or_default()
            .push(id.clone());

        self.capabilities.insert(id, cap);
    }

    /// 获取能力
    pub fn get(&self, id: &str) -> Option<Arc<dyn UnifiedCapability>> {
        self.capabilities.get(id).cloned()
    }

    /// 按域获取能力列表
    pub fn by_domain(&self, domain: Domain) -> Vec<Arc<dyn UnifiedCapability>> {
        self.by_domain
            .get(&domain)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.capabilities.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 按层级获取能力列表
    pub fn by_layer(&self, layer: Layer) -> Vec<Arc<dyn UnifiedCapability>> {
        self.by_layer
            .get(&layer)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.capabilities.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 列出所有能力
    pub fn list_all(&self) -> Vec<CapabilityMeta> {
        self.capabilities.values().map(|cap| cap.meta()).collect()
    }

    /// 获取所有健康状态
    pub fn health_all(&self) -> Vec<(CapabilityMeta, CapabilityHealth)> {
        self.capabilities
            .values()
            .map(|cap| (cap.meta(), cap.health()))
            .collect()
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 路由调度器
pub struct CapabilityRouter {
    /// 能力注册中心
    registry: Arc<CapabilityRegistry>,
    /// 路由规则
    rules: Vec<Box<dyn Fn(&CapabilityInput) -> Option<String>>>,
}

impl CapabilityRouter {
    /// 创建新的路由器
    pub fn new(registry: Arc<CapabilityRegistry>) -> Self {
        Self {
            registry,
            rules: Vec::new(),
        }
    }

    /// 添加路由规则
    pub fn add_rule<F>(&mut self, rule: F)
    where
        F: Fn(&CapabilityInput) -> Option<String> + 'static,
    {
        self.rules.push(Box::new(rule));
    }

    /// 路由调用
    pub fn route(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        // 尝试路由规则
        for rule in &self.rules {
            if let Some(cap_id) = rule(&input) {
                if let Some(cap) = self.registry.get(&cap_id) {
                    return cap.execute(input);
                }
            }
        }

        // 默认路由: 根据输入类型
        match &input {
            CapabilityInput::Text(_) => {
                // 默认路由到NLP
                self.route_to_domain(input, Domain::NtWorld)
            }
            CapabilityInput::Network(_) => self.route_to_domain(input, Domain::NtShield),
            CapabilityInput::Security(_) => self.route_to_domain(input, Domain::NtShield),
            CapabilityInput::Nlp(_) => self.route_to_domain(input, Domain::NtWorld),
            CapabilityInput::Asset(_) => self.route_to_domain(input, Domain::NtWorld),
            CapabilityInput::Kv(_) => self.route_to_domain(input, Domain::NtMemory),
        }
    }

    /// 按域路由
    fn route_to_domain(
        &self,
        input: CapabilityInput,
        domain: Domain,
    ) -> Result<CapabilityOutput, CapabilityError> {
        let caps = self.registry.by_domain(domain);
        for cap in caps {
            if cap.supports(&input) {
                return cap.execute(input);
            }
        }
        Err(CapabilityError::UnsupportedInput("无匹配能力".into()))
    }

    /// 获取注册中心引用
    pub fn registry(&self) -> &Arc<CapabilityRegistry> {
        &self.registry
    }
}

#[cfg(test)]
mod inline_tests {
    use super::*;

    #[test]
    fn registry_creation() {
        let registry = CapabilityRegistry::new();
        assert!(registry.list_all().is_empty());
    }

    #[test]
    fn layer_domain_coverage() {
        let layers = vec![
            Layer::L1Action,
            Layer::L2Perception,
            Layer::L3Embodiment,
            Layer::L4Emotion,
            Layer::L5Cognition,
            Layer::L6Meta,
        ];
        let domains = vec![
            Domain::NtCore,
            Domain::NtMind,
            Domain::NtMemory,
            Domain::NtWorld,
            Domain::NtAct,
            Domain::NtIo,
            Domain::NtShield,
            Domain::NtPhysical,
            Domain::NtFeel,
        ];
        assert_eq!(layers.len(), 6);
        assert_eq!(domains.len(), 9);
    }
}
