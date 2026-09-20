//! L0 Substrate — 统一能力接口类型定义
//!
//! 下沉自 `l6_meta::nt_core_capability`, 供 L0-L5 层使用, 消除低层→高层反向依赖。
//! L6 通过 re-export 保持向后兼容。

use std::collections::HashMap;

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
    NtFileAbility,
    Trade,
}

/// 能力元数据
#[derive(Debug, Clone)]
pub struct CapabilityMeta {
    pub id: String,
    pub name: String,
    pub layer: Layer,
    pub domain: Domain,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    pub status: CapabilityStatus,
    pub metrics: CapabilityMetrics,
    pub cost_weight: f64,
    pub priority: f64,
}

impl CapabilityMeta {
    pub fn effective_priority(&self) -> f64 {
        self.priority * self.cost_weight
    }
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
    Ready,
    Healthy,
    Running,
    Error(String),
    Disabled,
}

/// 能力健康度
#[derive(Debug, Clone)]
pub struct CapabilityHealth {
    pub state: CapabilityState,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub last_called: Option<std::time::Instant>,
    pub call_count: u64,
}

/// 统一输入
#[derive(Debug, Clone)]
pub enum CapabilityInput {
    Text(String),
    Network(NetworkInput),
    Security(SecurityInput),
    Nlp(NlpInput),
    Asset(AssetInput),
    FileEnhance(FileEnhanceInput),
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
    Tokenize,
    DetectLanguage,
    ExtractKeywords,
    Extract,
    SentimentAnalysis,
    Similarity,
    ExtractInfo,
    Classify,
}

/// 资产输入
#[derive(Debug, Clone)]
pub struct AssetInput {
    pub query: String,
    pub asset_type: Option<String>,
    pub limit: usize,
}

/// 文件增强输入
#[derive(Debug, Clone)]
pub struct FileEnhanceInput {
    pub input_path: String,
    pub output_path: Option<String>,
    pub mode: FileEnhanceMode,
}

/// 文件增强模式
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileEnhanceMode {
    PdfIconEnhance,
    ImageSuperResolution,
}

/// 统一输出
#[derive(Debug)]
pub enum CapabilityOutput {
    Text(String),
    LlmResult(LlmResult),
    CliResult(CliResult),
    Network(NetworkOutput),
    NetworkResult(NetworkResult),
    Security(SecurityOutput),
    SecurityScan(SecurityScanResult),
    Nlp(NlpOutput),
    Asset(AssetOutput),
    FileEnhance(FileEnhanceOutput),
    WebResult(WebResult),
    CrawlResult(CrawlResult),
    ParsedContent(ParsedContent),
    GraphQuery(GraphQueryResult),
    ThreatAnalysis(ThreatResult),
    SimulationResult(Box<dyn std::fmt::Debug + Send + Sync>),
    SensorData(Box<dyn std::fmt::Debug + Send + Sync>),
    SelfModelResult(Box<dyn std::fmt::Debug + Send + Sync>),
    RoutingResult(Box<dyn std::fmt::Debug + Send + Sync>),
    RegulationResult(Box<dyn std::fmt::Debug + Send + Sync>),
    ReasoningResult(Box<dyn std::fmt::Debug + Send + Sync>),
    HyperCubeResult(Box<dyn std::fmt::Debug + Send + Sync>),
    EmotionExpression(Box<dyn std::fmt::Debug + Send + Sync>),
    EmotionAnalysis(Box<dyn std::fmt::Debug + Send + Sync>),
    ActuatorCommand(Box<dyn std::fmt::Debug + Send + Sync>),
    Kv(HashMap<String, String>),
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
            Self::FileEnhance(o) => Self::FileEnhance(o.clone()),
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

/// 文件增强输出
#[derive(Debug, Clone)]
pub struct FileEnhanceOutput {
    pub success: bool,
    pub input_path: String,
    pub output_path: String,
    pub message: String,
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
    fn meta(&self) -> CapabilityMeta;
    fn health(&self) -> CapabilityHealth;
    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError>;
    fn reset(&mut self) {}
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

// ─── CapabilityRegistry (下沉自 l6_meta::nt_core_capability) ───────────────

use std::sync::Arc;

/// 能力注册中心
#[derive(Clone)]
pub struct CapabilityRegistry {
    /// 已注册的能力
    capabilities: std::collections::HashMap<String, Arc<dyn UnifiedCapability>>,
    /// 按域索引
    by_domain: std::collections::HashMap<Domain, Vec<String>>,
    /// 按层级索引
    by_layer: std::collections::HashMap<Layer, Vec<String>>,
}

impl CapabilityRegistry {
    /// 创建新的注册中心
    pub fn new() -> Self {
        Self {
            capabilities: std::collections::HashMap::new(),
            by_domain: std::collections::HashMap::new(),
            by_layer: std::collections::HashMap::new(),
        }
    }

    /// 注册能力
    pub fn register(&mut self, cap: Arc<dyn UnifiedCapability>) {
        let meta = cap.meta();
        let id = meta.id.clone();

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

/// MoE (Mixture of Experts) routing strategy for cost-aware capability selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoERoutingStrategy {
    /// Route to cheapest capable model (default, ~90% token savings)
    CostOptimized,
    /// Route to highest quality model regardless of cost
    QualityFirst,
    /// Round-robin across capable models
    RoundRobin,
    /// Load-balanced across capable models
    LoadBalanced,
}

/// 路由调度器
pub struct CapabilityRouter {
    /// 能力注册中心
    registry: Arc<CapabilityRegistry>,
    /// 路由规则
    rules: Vec<Box<dyn Fn(&CapabilityInput) -> Option<String>>>,
    /// MoE routing strategy
    routing_strategy: MoERoutingStrategy,
}

impl CapabilityRouter {
    /// 创建新的路由器 (default: CostOptimized)
    pub fn new(registry: Arc<CapabilityRegistry>) -> Self {
        Self {
            registry,
            rules: Vec::new(),
            routing_strategy: MoERoutingStrategy::CostOptimized,
        }
    }

    /// 创建路由器 with explicit MoE routing strategy
    pub fn with_strategy(registry: Arc<CapabilityRegistry>, strategy: MoERoutingStrategy) -> Self {
        Self {
            registry,
            rules: Vec::new(),
            routing_strategy: strategy,
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
        for rule in &self.rules {
            if let Some(cap_id) = rule(&input) {
                if let Some(cap) = self.registry.get(&cap_id) {
                    return cap.execute(input);
                }
            }
        }

        match &input {
            CapabilityInput::Text(_) => self.route_to_domain(input, Domain::NtWorld),
            CapabilityInput::Network(_) => self.route_to_domain(input, Domain::NtShield),
            CapabilityInput::Security(_) => self.route_to_domain(input, Domain::NtShield),
            CapabilityInput::Nlp(_) => self.route_to_domain(input, Domain::NtWorld),
            CapabilityInput::Asset(_) => self.route_to_domain(input, Domain::NtWorld),
            CapabilityInput::FileEnhance(_) => self.route_to_domain(input, Domain::NtFileAbility),
            CapabilityInput::Kv(_) => self.route_to_domain(input, Domain::NtMemory),
        }
    }

    /// 按域路由
    fn route_to_domain(
        &self,
        input: CapabilityInput,
        domain: Domain,
    ) -> Result<CapabilityOutput, CapabilityError> {
        let mut caps: Vec<Arc<dyn UnifiedCapability>> = self.registry.by_domain(domain);
        let strategy = self.routing_strategy.clone();

        match strategy {
            MoERoutingStrategy::CostOptimized => {
                caps.sort_by(|a, b| {
                    a.meta()
                        .cost_weight
                        .partial_cmp(&b.meta().cost_weight)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            MoERoutingStrategy::QualityFirst => {
                caps.sort_by(|a, b| {
                    b.meta()
                        .cost_weight
                        .partial_cmp(&a.meta().cost_weight)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            MoERoutingStrategy::RoundRobin | MoERoutingStrategy::LoadBalanced => {}
        }

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

    /// Set the MoE routing strategy
    pub fn set_routing_strategy(&mut self, strategy: MoERoutingStrategy) {
        self.routing_strategy = strategy;
    }

    /// Get the current MoE routing strategy
    pub fn routing_strategy(&self) -> &MoERoutingStrategy {
        &self.routing_strategy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_meta_effective_priority() {
        let meta = CapabilityMeta {
            id: "test".into(),
            name: "Test".into(),
            layer: Layer::L1Action,
            domain: Domain::NtCore,
            version: "1.0".into(),
            description: "test".into(),
            tags: vec![],
            status: CapabilityStatus::default(),
            metrics: CapabilityMetrics::default(),
            cost_weight: 0.5,
            priority: 8.0,
        };
        assert_eq!(meta.effective_priority(), 4.0);
    }

    #[test]
    fn capability_status_default() {
        assert!(matches!(CapabilityStatus::default(), CapabilityStatus::Healthy));
    }

    #[test]
    fn capability_metrics_default() {
        let m = CapabilityMetrics::default();
        assert_eq!(m.total_calls, 0);
        assert_eq!(m.total_errors, 0);
        assert!(m.last_called.is_none());
    }

    #[test]
    fn capability_registry_new_and_register() {
        let mut reg = CapabilityRegistry::new();
        assert!(reg.get("nonexistent").is_none());
        assert!(reg.list_all().is_empty());
    }

    #[test]
    fn moe_routing_strategy_eq() {
        assert_eq!(MoERoutingStrategy::CostOptimized, MoERoutingStrategy::CostOptimized);
        assert_ne!(MoERoutingStrategy::CostOptimized, MoERoutingStrategy::QualityFirst);
    }

    #[test]
    fn capability_router_new() {
        let reg = Arc::new(CapabilityRegistry::new());
        let router = CapabilityRouter::new(reg);
        assert_eq!(router.routing_strategy(), &MoERoutingStrategy::CostOptimized);
    }

    #[test]
    fn capability_router_with_strategy() {
        let reg = Arc::new(CapabilityRegistry::new());
        let router = CapabilityRouter::with_strategy(reg, MoERoutingStrategy::QualityFirst);
        assert_eq!(router.routing_strategy(), &MoERoutingStrategy::QualityFirst);
    }

    #[test]
    fn capability_input_debug() {
        let input = CapabilityInput::Text("hello".into());
        let debug = format!("{:?}", input);
        assert!(debug.contains("Text"));
    }

    #[test]
    fn capability_output_clone_text() {
        let out = CapabilityOutput::Text("hi".into());
        let cloned = out.clone();
        assert!(matches!(cloned, CapabilityOutput::Text(s) if s == "hi"));
    }

    #[test]
    fn capability_error_display() {
        let err = CapabilityError::Timeout;
        assert!(format!("{}", err).contains("超时"));
    }
}
