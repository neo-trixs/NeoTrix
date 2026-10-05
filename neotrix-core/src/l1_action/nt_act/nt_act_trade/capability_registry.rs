//! Trade Capability Registry — 统一能力注册协议
//!
//! `TradeCapability` 继承 `UnifiedCapability`，实现能力生态统一。
//! 将 capabilities/ 下的 4 个能力和 process_engine 的 StepHandler
//! 统一注册到 CapabilityRegistry，支持按名称、类型查找能力。

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::l0_substrate::nt_core_capability_types::{
    CapabilityError, CapabilityHealth, CapabilityInput as CoreCapabilityInput, CapabilityMeta,
    CapabilityOutput as CoreCapabilityOutput, CapabilityState, CapabilityStatus,
    CapabilityMetrics, Domain, Layer, UnifiedCapability,
};

// ============================================================
// 1. TradeCapability trait — 继承 UnifiedCapability
// ============================================================

/// Trade 能力 trait — 继承 UnifiedCapability，添加贸易特有方法
#[async_trait]
pub trait TradeCapability: UnifiedCapability {
    /// 贸易能力类型
    fn trade_capability_type(&self) -> TradeCapabilityType;

    /// 执行贸易能力 (async，使用 trade 域专用 I/O 类型)
    async fn execute_trade(
        &self,
        input: TradeCapabilityInput,
    ) -> Result<TradeCapabilityOutput, TradeCapabilityError>;

    /// 检查能力是否可用
    async fn is_available(&self) -> bool;
}

// ============================================================
// 2. 能力类型枚举
// ============================================================

/// 能力类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TradeCapabilityType {
    /// 价格计算
    PriceCalculation,
    /// 产品匹配
    ProductMatching,
    /// 风险评估
    RiskAssessment,
    /// 供应商匹配
    SupplierMatching,
    /// 流程步骤
    ProcessStep,
    /// 数据提取
    DataExtraction,
    /// 数据分析
    DataAnalysis,
    /// 文档生成
    DocumentGeneration,
    /// 通信发送
    CommunicationSend,
    /// 其他
    Other,
}

impl std::fmt::Display for TradeCapabilityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PriceCalculation => write!(f, "PriceCalculation"),
            Self::ProductMatching => write!(f, "ProductMatching"),
            Self::RiskAssessment => write!(f, "RiskAssessment"),
            Self::SupplierMatching => write!(f, "SupplierMatching"),
            Self::ProcessStep => write!(f, "ProcessStep"),
            Self::DataExtraction => write!(f, "DataExtraction"),
            Self::DataAnalysis => write!(f, "DataAnalysis"),
            Self::DocumentGeneration => write!(f, "DocumentGeneration"),
            Self::CommunicationSend => write!(f, "CommunicationSend"),
            Self::Other => write!(f, "Other"),
        }
    }
}

// ============================================================
// 3. Trade 域专用输入/输出类型
// ============================================================

/// 贸易能力输入 (trade 域专用，与 CoreCapabilityInput 分离)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeCapabilityInput {
    /// 输入数据 (JSON)
    pub data: serde_json::Value,
    /// 上下文信息
    pub context: HashMap<String, String>,
}

/// 贸易能力输出 (trade 域专用，与 CoreCapabilityOutput 分离)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeCapabilityOutput {
    /// 输出数据 (JSON)
    pub data: serde_json::Value,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 贸易能力错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeCapabilityError {
    /// 能力不可用
    Unavailable(String),
    /// 输入无效
    InvalidInput(String),
    /// 执行失败
    ExecutionFailed(String),
    /// 能力未找到
    NotFound(String),
    /// 超时
    Timeout(String),
}

impl std::fmt::Display for TradeCapabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable(msg) => write!(f, "Capability unavailable: {}", msg),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Self::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            Self::NotFound(msg) => write!(f, "Capability not found: {}", msg),
            Self::Timeout(msg) => write!(f, "Timeout: {}", msg),
        }
    }
}

impl std::error::Error for TradeCapabilityError {}

impl From<TradeCapabilityError> for CapabilityError {
    fn from(e: TradeCapabilityError) -> Self {
        match e {
            TradeCapabilityError::Unavailable(_msg) => CapabilityError::NotReady,
            TradeCapabilityError::InvalidInput(msg) => CapabilityError::UnsupportedInput(msg),
            TradeCapabilityError::ExecutionFailed(msg) => CapabilityError::ExecutionFailed(msg),
            TradeCapabilityError::NotFound(msg) => CapabilityError::UnsupportedInput(msg),
            TradeCapabilityError::Timeout(_msg) => CapabilityError::Timeout,
        }
    }
}

// ============================================================
// 4. 能力信息
// ============================================================

/// 能力信息 (只读快照)
#[derive(Debug, Clone)]
pub struct CapabilityInfo {
    /// 能力 ID
    pub id: String,
    /// 能力名称
    pub name: String,
    /// 能力类型
    pub capability_type: TradeCapabilityType,
    /// 能力描述
    pub description: String,
}

// ============================================================
// 5. TradeCapabilityRegistry — 桥接全局 CapabilityRegistry
// ============================================================

use crate::l0_substrate::nt_core_capability_types::CapabilityRegistry as GlobalCapabilityRegistry;

/// 能力注册表 — 管理所有可注册的 Trade 能力
///
/// 同时维护:
/// - `global`: 全局 `CapabilityRegistry` (UnifiedCapability 视图)
/// - `local`: Trade 域本地索引 (TradeCapability 视图，支持类型查找)
pub struct TradeCapabilityRegistry {
    /// 全局能力注册中心
    global: GlobalCapabilityRegistry,
    /// 本地能力存储 (id → capability)
    local: HashMap<String, Arc<dyn TradeCapability>>,
    /// 类型索引 (type → [id])
    type_index: HashMap<TradeCapabilityType, Vec<String>>,
}

impl Default for TradeCapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TradeCapabilityRegistry {
    /// 创建空注册表
    pub fn new() -> Self {
        Self {
            global: GlobalCapabilityRegistry::new(),
            local: HashMap::new(),
            type_index: HashMap::new(),
        }
    }

    /// 注册能力 — 同时注册到全局和本地
    pub fn register(&mut self, capability: Arc<dyn TradeCapability>) {
        let id = capability.meta().id.clone();
        let trade_type = capability.trade_capability_type();

        // 更新类型索引
        self.type_index.entry(trade_type).or_default().push(id.clone());

        // 注册到全局 (作为 UnifiedCapability)
        self.global.register(capability.clone() as Arc<dyn UnifiedCapability>);

        // 保留本地引用 (支持 TradeCapability 方法)
        self.local.insert(id, capability);
    }

    /// 注册能力 (返回 Self 以支持链式调用)
    pub fn with(mut self, capability: Arc<dyn TradeCapability>) -> Self {
        self.register(capability);
        self
    }

    /// 获取能力 (TradeCapability 视图)
    pub fn get(&self, capability_id: &str) -> Option<Arc<dyn TradeCapability>> {
        // ⭐⭐⭐⭐⭐ **金丝雀打点**（2026-10-04，吸收自 `plur` 的 `CapabilityCanary`）。
        //
        // ⭐⭐⭐⭐ **为什么打在这里**：⭐⭐ `get()` 是 trade 能力
        // ⭐⭐⭐⭐ **唯一的生产派发点**（⭐⭐ 拿到它 ⇒ 就要执行它）。
        // ⭐⭐⭐⭐ ⭐⭐ **纪律**：`signal()` ⭐⭐ 只许出现在**派发路径**内，
        // ⭐⭐⭐⭐ ⛔ 绝不许出现在 `register` 处、⛔ 绝不许出现在测试里 ——
        // ⭐⭐⭐⭐ 否则「注册即打点」会伪造健康，⭐⭐⭐⭐
        // ⭐⭐⭐⭐ 那就回到了「建成未用却看着健康」。
        //
        // ⭐⭐ 打点一个**未被 expect 登记**的 id 是**静默无害**的：
        // ⭐⭐ ⭐⭐ 金丝雀只报告**登记过却没打点**的（⭐⭐ 真正的问题方向）。
        neotrix_neobot::nt_capability_canary::signal(capability_id);
        self.local.get(capability_id).cloned()
    }

    /// 获取能力 (UnifiedCapability 视图，用于跨域调用)
    pub fn get_unified(&self, capability_id: &str) -> Option<Arc<dyn UnifiedCapability>> {
        self.global.get(capability_id)
    }

    /// 获取指定类型的所有能力
    pub fn get_by_type(&self, cap_type: TradeCapabilityType) -> Vec<Arc<dyn TradeCapability>> {
        self.type_index
            .get(&cap_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.local.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 获取指定类型的一个能力 (返回第一个找到的)
    pub fn get_one_by_type(&self, cap_type: TradeCapabilityType) -> Option<Arc<dyn TradeCapability>> {
        self.type_index
            .get(&cap_type)
            .and_then(|ids| ids.first())
            .and_then(|id| self.local.get(id).cloned())
    }

    /// 列出所有能力信息
    pub fn list(&self) -> Vec<CapabilityInfo> {
        self.local
            .values()
            .map(|cap| {
                let meta = cap.meta();
                CapabilityInfo {
                    id: meta.id,
                    name: meta.name,
                    capability_type: cap.trade_capability_type(),
                    description: meta.description,
                }
            })
            .collect()
    }

    /// 检查能力是否存在
    pub fn has(&self, capability_id: &str) -> bool {
        self.local.contains_key(capability_id)
    }

    /// 获取能力数量
    pub fn len(&self) -> usize {
        self.local.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.local.is_empty()
    }

    /// 获取所有已注册的能力 ID
    pub fn ids(&self) -> Vec<&str> {
        self.local.keys().map(|s| s.as_str()).collect()
    }

    /// 获取所有已注册的能力类型
    pub fn types(&self) -> Vec<TradeCapabilityType> {
        self.type_index.keys().copied().collect()
    }

    /// 获取全局注册中心引用 (用于跨域路由)
    pub fn global_registry(&self) -> &GlobalCapabilityRegistry {
        &self.global
    }
}

// ============================================================
// 6. 为现有 capabilities 实现 UnifiedCapability + TradeCapability
// ============================================================

use super::capabilities::{
    PriceCalcConfig, PriceCalcRequest, PriceCalculator,
    ProductMatchConfig, ProductMatchRequest, ProductMatcher,
    RiskAssessConfig, RiskAssessor, RiskAssessRequest,
    SupplierMatchConfig, SupplierMatchRequest, SupplierMatcher,
};

/// PriceCalculator 的双 trait 适配器
pub struct PriceCalculatorCapability {
    inner: PriceCalculator,
    id: String,
}

impl PriceCalculatorCapability {
    pub fn new(config: PriceCalcConfig) -> Self {
        Self {
            inner: PriceCalculator::new(config),
            id: "trade.price_calculator".to_string(),
        }
    }
}

impl UnifiedCapability for PriceCalculatorCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: self.id.clone(),
            name: "PriceCalculator".to_string(),
            layer: Layer::L1Action,
            domain: Domain::Trade,
            version: "1.0.0".to_string(),
            description: "价格计算能力 — 基于成本、利润率、汇率、贸易条款计算最终报价".to_string(),
            tags: vec!["trade".to_string(), "price".to_string()],
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics::default(),
            cost_weight: 0.1,
            priority: 1.0,
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Ready,
            success_rate: 1.0,
            avg_latency_ms: 0.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, _input: CoreCapabilityInput) -> Result<CoreCapabilityOutput, CapabilityError> {
        Err(CapabilityError::UnsupportedInput(
            "Use execute_trade() for async trade operations".into(),
        ))
    }

    fn supports(&self, _input: &CoreCapabilityInput) -> bool {
        false
    }
}

#[async_trait]
impl TradeCapability for PriceCalculatorCapability {
    fn trade_capability_type(&self) -> TradeCapabilityType {
        TradeCapabilityType::PriceCalculation
    }

    async fn execute_trade(
        &self,
        input: TradeCapabilityInput,
    ) -> Result<TradeCapabilityOutput, TradeCapabilityError> {
        let request: PriceCalcRequest = serde_json::from_value(input.data)
            .map_err(|e| TradeCapabilityError::InvalidInput(e.to_string()))?;

        let result = self.inner.calculate(&request);

        let mut metadata = HashMap::new();
        metadata.insert("elapsed_us".to_string(), result.elapsed_us.to_string());
        metadata.insert("trade_term".to_string(), format!("{:?}", result.trade_term));

        Ok(TradeCapabilityOutput {
            data: serde_json::to_value(&result)
                .map_err(|e| TradeCapabilityError::ExecutionFailed(e.to_string()))?,
            metadata,
        })
    }

    async fn is_available(&self) -> bool {
        true
    }
}

/// ProductMatcher 的双 trait 适配器
pub struct ProductMatcherCapability {
    inner: ProductMatcher,
    id: String,
}

impl ProductMatcherCapability {
    pub fn new(config: ProductMatchConfig) -> Self {
        Self {
            inner: ProductMatcher::new(config),
            id: "trade.product_matcher".to_string(),
        }
    }
}

impl UnifiedCapability for ProductMatcherCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: self.id.clone(),
            name: "ProductMatcher".to_string(),
            layer: Layer::L1Action,
            domain: Domain::Trade,
            version: "1.0.0".to_string(),
            description: "产品匹配能力 — 根据买家询价参数从产品库中检索匹配产品".to_string(),
            tags: vec!["trade".to_string(), "product".to_string()],
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics::default(),
            cost_weight: 0.1,
            priority: 1.0,
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Ready,
            success_rate: 1.0,
            avg_latency_ms: 0.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, _input: CoreCapabilityInput) -> Result<CoreCapabilityOutput, CapabilityError> {
        Err(CapabilityError::UnsupportedInput(
            "Use execute_trade() for async trade operations".into(),
        ))
    }

    fn supports(&self, _input: &CoreCapabilityInput) -> bool {
        false
    }
}

#[async_trait]
impl TradeCapability for ProductMatcherCapability {
    fn trade_capability_type(&self) -> TradeCapabilityType {
        TradeCapabilityType::ProductMatching
    }

    async fn execute_trade(
        &self,
        input: TradeCapabilityInput,
    ) -> Result<TradeCapabilityOutput, TradeCapabilityError> {
        let request: ProductMatchRequest = serde_json::from_value(input.data)
            .map_err(|e| TradeCapabilityError::InvalidInput(e.to_string()))?;

        let result = self.inner.match_products(&request);

        let mut metadata = HashMap::new();
        metadata.insert("elapsed_us".to_string(), result.elapsed_us.to_string());
        metadata.insert("has_exact".to_string(), result.has_exact.to_string());
        metadata.insert("count".to_string(), result.entries.len().to_string());

        Ok(TradeCapabilityOutput {
            data: serde_json::to_value(&result)
                .map_err(|e| TradeCapabilityError::ExecutionFailed(e.to_string()))?,
            metadata,
        })
    }

    async fn is_available(&self) -> bool {
        true
    }
}

/// RiskAssessor 的双 trait 适配器
pub struct RiskAssessorCapability {
    inner: RiskAssessor,
    id: String,
}

impl RiskAssessorCapability {
    pub fn new(config: RiskAssessConfig) -> Self {
        Self {
            inner: RiskAssessor::new(config),
            id: "trade.risk_assessor".to_string(),
        }
    }
}

impl UnifiedCapability for RiskAssessorCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: self.id.clone(),
            name: "RiskAssessor".to_string(),
            layer: Layer::L1Action,
            domain: Domain::Trade,
            version: "1.0.0".to_string(),
            description: "风险评估能力 — 对外贸交易全链路进行信用/物流/合规/汇率风险评估"
                .to_string(),
            tags: vec!["trade".to_string(), "risk".to_string()],
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics::default(),
            cost_weight: 0.2,
            priority: 1.0,
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Ready,
            success_rate: 1.0,
            avg_latency_ms: 0.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, _input: CoreCapabilityInput) -> Result<CoreCapabilityOutput, CapabilityError> {
        Err(CapabilityError::UnsupportedInput(
            "Use execute_trade() for async trade operations".into(),
        ))
    }

    fn supports(&self, _input: &CoreCapabilityInput) -> bool {
        false
    }
}

#[async_trait]
impl TradeCapability for RiskAssessorCapability {
    fn trade_capability_type(&self) -> TradeCapabilityType {
        TradeCapabilityType::RiskAssessment
    }

    async fn execute_trade(
        &self,
        input: TradeCapabilityInput,
    ) -> Result<TradeCapabilityOutput, TradeCapabilityError> {
        let request: RiskAssessRequest = serde_json::from_value(input.data)
            .map_err(|e| TradeCapabilityError::InvalidInput(e.to_string()))?;

        let result = self.inner.assess(&request);

        let mut metadata = HashMap::new();
        metadata.insert("elapsed_us".to_string(), result.elapsed_us.to_string());
        metadata.insert(
            "overall_level".to_string(),
            format!("{:?}", result.overall_level),
        );
        metadata.insert("blocking".to_string(), result.blocking.to_string());
        metadata.insert("findings_count".to_string(), result.findings.len().to_string());

        Ok(TradeCapabilityOutput {
            data: serde_json::to_value(&result)
                .map_err(|e| TradeCapabilityError::ExecutionFailed(e.to_string()))?,
            metadata,
        })
    }

    async fn is_available(&self) -> bool {
        true
    }
}

/// SupplierMatcher 的双 trait 适配器
pub struct SupplierMatcherCapability {
    inner: SupplierMatcher,
    id: String,
}

impl SupplierMatcherCapability {
    pub fn new(config: SupplierMatchConfig) -> Self {
        Self {
            inner: SupplierMatcher::new(config),
            id: "trade.supplier_matcher".to_string(),
        }
    }
}

impl UnifiedCapability for SupplierMatcherCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: self.id.clone(),
            name: "SupplierMatcher".to_string(),
            layer: Layer::L1Action,
            domain: Domain::Trade,
            version: "1.0.0".to_string(),
            description: "供应商匹配能力 — 根据产品需求、地域偏好、信用评级筛选最佳供应商"
                .to_string(),
            tags: vec!["trade".to_string(), "supplier".to_string()],
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics::default(),
            cost_weight: 0.15,
            priority: 1.0,
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Ready,
            success_rate: 1.0,
            avg_latency_ms: 0.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, _input: CoreCapabilityInput) -> Result<CoreCapabilityOutput, CapabilityError> {
        Err(CapabilityError::UnsupportedInput(
            "Use execute_trade() for async trade operations".into(),
        ))
    }

    fn supports(&self, _input: &CoreCapabilityInput) -> bool {
        false
    }
}

#[async_trait]
impl TradeCapability for SupplierMatcherCapability {
    fn trade_capability_type(&self) -> TradeCapabilityType {
        TradeCapabilityType::SupplierMatching
    }

    async fn execute_trade(
        &self,
        input: TradeCapabilityInput,
    ) -> Result<TradeCapabilityOutput, TradeCapabilityError> {
        let request: SupplierMatchRequest = serde_json::from_value(input.data)
            .map_err(|e| TradeCapabilityError::InvalidInput(e.to_string()))?;

        let result = self.inner.match_suppliers(&request);

        let mut metadata = HashMap::new();
        metadata.insert("elapsed_us".to_string(), result.elapsed_us.to_string());
        metadata.insert(
            "total_candidates".to_string(),
            result.total_candidates.to_string(),
        );

        Ok(TradeCapabilityOutput {
            data: serde_json::to_value(&result)
                .map_err(|e| TradeCapabilityError::ExecutionFailed(e.to_string()))?,
            metadata,
        })
    }

    async fn is_available(&self) -> bool {
        true
    }
}

// ============================================================
// 7. 为 process_engine::StepHandler 实现双 trait
// ============================================================

use super::process_engine::{ProcessInstance, ProcessStep, StepHandler};

/// StepHandler 的双 trait 适配器
///
/// 将 process_engine 的 StepHandler 适配为 UnifiedCapability + TradeCapability，
/// 使其可以注册到 TradeCapabilityRegistry 中统一管理。
pub struct StepHandlerCapability {
    handler: Arc<dyn StepHandler>,
    id: String,
    name: String,
    description: String,
}

impl StepHandlerCapability {
    /// 创建 StepHandler 适配器
    ///
    /// # 参数
    /// - `handler_name`: 处理器名称 (用于 StepHandler 注册)
    /// - `handler`: 步骤处理器实例
    /// - `description`: 能力描述
    pub fn new(
        handler_name: &str,
        handler: Arc<dyn StepHandler>,
        description: &str,
    ) -> Self {
        Self {
            handler,
            id: format!("trade.step.{}", handler_name),
            name: format!("StepHandler({})", handler_name),
            description: description.to_string(),
        }
    }
}

impl UnifiedCapability for StepHandlerCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: self.id.clone(),
            name: self.name.clone(),
            layer: Layer::L1Action,
            domain: Domain::Trade,
            version: "1.0.0".to_string(),
            description: self.description.clone(),
            tags: vec!["trade".to_string(), "process".to_string()],
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics::default(),
            cost_weight: 0.1,
            priority: 1.0,
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Ready,
            success_rate: 1.0,
            avg_latency_ms: 0.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, _input: CoreCapabilityInput) -> Result<CoreCapabilityOutput, CapabilityError> {
        Err(CapabilityError::UnsupportedInput(
            "Use execute_trade() for async trade operations".into(),
        ))
    }

    fn supports(&self, _input: &CoreCapabilityInput) -> bool {
        false
    }
}

#[async_trait]
impl TradeCapability for StepHandlerCapability {
    fn trade_capability_type(&self) -> TradeCapabilityType {
        TradeCapabilityType::ProcessStep
    }

    async fn execute_trade(
        &self,
        input: TradeCapabilityInput,
    ) -> Result<TradeCapabilityOutput, TradeCapabilityError> {
        let instance: ProcessInstance = serde_json::from_value(
            input.data
                .get("instance")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        )
        .map_err(|e| TradeCapabilityError::InvalidInput(format!("Invalid instance: {}", e)))?;

        let step: ProcessStep = serde_json::from_value(
            input.data
                .get("step")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        )
        .map_err(|e| TradeCapabilityError::InvalidInput(format!("Invalid step: {}", e)))?;

        let mut context: HashMap<String, String> = input.context.clone();

        let output = self
            .handler
            .execute(&instance, &step, &mut context)
            .map_err(TradeCapabilityError::ExecutionFailed)?;

        Ok(TradeCapabilityOutput {
            data: serde_json::to_value(&output)
                .map_err(|e| TradeCapabilityError::ExecutionFailed(e.to_string()))?,
            metadata: context,
        })
    }

    async fn is_available(&self) -> bool {
        true
    }
}

// ============================================================
// 8. 便捷构建函数
// ============================================================

/// 创建包含所有内置能力的注册表
pub fn create_default_registry() -> TradeCapabilityRegistry {
    TradeCapabilityRegistry::new()
        .with(Arc::new(PriceCalculatorCapability::new(PriceCalcConfig::default())))
        .with(Arc::new(ProductMatcherCapability::new(ProductMatchConfig::default())))
        .with(Arc::new(RiskAssessorCapability::new(RiskAssessConfig::default())))
        .with(Arc::new(SupplierMatcherCapability::new(SupplierMatchConfig::default())))
}

/// 创建空注册表
pub fn create_empty_registry() -> TradeCapabilityRegistry {
    TradeCapabilityRegistry::new()
}

// ============================================================
// 9. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use nt_core_capability_tree::node::CapabilityNode;
    use nt_core_capability_tree::registry::CapabilityTreeRegistry;

    use super::*;
    use serde_json::json;

    #[test]
    fn test_registry_create() {
        let registry = TradeCapabilityRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = TradeCapabilityRegistry::new();
        let cap = Arc::new(PriceCalculatorCapability::new(PriceCalcConfig::default()));

        registry.register(cap.clone());

        assert!(registry.has("trade.price_calculator"));
        assert_eq!(registry.len(), 1);

        let retrieved = registry.get("trade.price_calculator").unwrap();
        assert_eq!(retrieved.meta().id, "trade.price_calculator");
        assert_eq!(retrieved.meta().name, "PriceCalculator");
    }

    #[test]
    fn test_unified_view() {
        let mut registry = TradeCapabilityRegistry::new();
        registry.register(Arc::new(PriceCalculatorCapability::new(PriceCalcConfig::default())));

        // UnifiedCapability 视图
        let unified = registry.get_unified("trade.price_calculator").unwrap();
        assert_eq!(unified.meta().domain, Domain::Trade);
        assert_eq!(unified.meta().layer, Layer::L1Action);
    }

    #[test]
    fn test_get_by_type() {
        let mut registry = TradeCapabilityRegistry::new();
        registry.register(Arc::new(PriceCalculatorCapability::new(PriceCalcConfig::default())));
        registry.register(Arc::new(ProductMatcherCapability::new(ProductMatchConfig::default())));
        registry.register(Arc::new(RiskAssessorCapability::new(RiskAssessConfig::default())));
        registry.register(Arc::new(SupplierMatcherCapability::new(SupplierMatchConfig::default())));

        assert_eq!(registry.len(), 4);

        let price_caps = registry.get_by_type(TradeCapabilityType::PriceCalculation);
        assert_eq!(price_caps.len(), 1);
        assert_eq!(price_caps[0].meta().id, "trade.price_calculator");

        let product_caps = registry.get_by_type(TradeCapabilityType::ProductMatching);
        assert_eq!(product_caps.len(), 1);
    }

    #[test]
    fn test_list_capabilities() {
        let mut registry = TradeCapabilityRegistry::new();
        registry.register(Arc::new(PriceCalculatorCapability::new(PriceCalcConfig::default())));
        registry.register(Arc::new(ProductMatcherCapability::new(ProductMatchConfig::default())));

        let list = registry.list();
        assert_eq!(list.len(), 2);

        let ids: Vec<&str> = list.iter().map(|c| c.id.as_str()).collect();
        assert!(ids.contains(&"trade.price_calculator"));
        assert!(ids.contains(&"trade.product_matcher"));
    }

    #[test]
    fn test_with_chain() {
        let registry = TradeCapabilityRegistry::new()
            .with(Arc::new(PriceCalculatorCapability::new(PriceCalcConfig::default())))
            .with(Arc::new(ProductMatcherCapability::new(ProductMatchConfig::default())));

        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_create_default_registry() {
        let registry = create_default_registry();
        assert_eq!(registry.len(), 4);

        assert!(registry.has("trade.price_calculator"));
        assert!(registry.has("trade.product_matcher"));
        assert!(registry.has("trade.risk_assessor"));
        assert!(registry.has("trade.supplier_matcher"));
    }

    #[test]
    fn test_types_index() {
        let registry = create_default_registry();
        let types = registry.types();
        assert_eq!(types.len(), 4);

        assert!(types.contains(&TradeCapabilityType::PriceCalculation));
        assert!(types.contains(&TradeCapabilityType::ProductMatching));
        assert!(types.contains(&TradeCapabilityType::RiskAssessment));
        assert!(types.contains(&TradeCapabilityType::SupplierMatching));
    }

    #[test]
    fn test_get_one_by_type() {
        let registry = create_default_registry();
        let cap = registry.get_one_by_type(TradeCapabilityType::PriceCalculation);
        assert!(cap.is_some());
        assert_eq!(cap.unwrap().meta().id, "trade.price_calculator");
    }

    #[test]
    fn test_ids() {
        let registry = create_default_registry();
        let ids = registry.ids();
        assert_eq!(ids.len(), 4);
        assert!(ids.contains(&"trade.price_calculator"));
    }

    #[test]
    fn test_trait_inheritance() {
        let cap = PriceCalculatorCapability::new(PriceCalcConfig::default());

        // 通过 UnifiedCapability trait 访问
        let unified: &dyn UnifiedCapability = &cap;
        assert_eq!(unified.meta().domain, Domain::Trade);

        // 通过 TradeCapability trait 访问
        let trade: &dyn TradeCapability = &cap;
        assert_eq!(trade.trade_capability_type(), TradeCapabilityType::PriceCalculation);
    }

    #[test]
    fn test_global_registry_bridge() {
        let mut registry = TradeCapabilityRegistry::new();
        registry.register(Arc::new(PriceCalculatorCapability::new(PriceCalcConfig::default())));

        // 全局注册中心可以通过 Domain::Trade 查询
        let trade_caps = registry.global_registry().by_domain(Domain::Trade);
        assert_eq!(trade_caps.len(), 1);
    }
}


// ══════════════════════════════════════════════════════════════════
// ⭐⭐⭐⭐⭐ **能力市场元数据**（2026-10-04，⭐⭐ 对标 hermes 的 `IndexEntry`）
// ══════════════════════════════════════════════════════════════════
//
// ⭐⭐⭐⭐⭐ **为什么集中在这里而不是 5 个 registrar 各写一份**：
// ⭐⭐⭐⭐⭐ ① ⭐⭐ **单一真源** —— 5 处散写必然漂（⭐⭐ 这正是本轮一路在治的病）
// ⭐⭐⭐⭐⭐ ② ⭐⭐⭐⭐⭐ **可反查**：⭐⭐ `check-trade-market.sh` 能断言
// ⭐⭐⭐⭐⭐ 「每个 trade 节点都过了这个函数」，⭐⭐ ⭐⭐ 而不是
// ⭐⭐⭐⭐⭐ 「代码里看起来有 metadata」（⭐⭐ grep ⭐⭐ 不是证据）。
// ⭐⭐⭐⭐⭐ ③ ⭐⭐ 版本号 ⭐⭐ **一处升版**，⭐⭐ 5 个能力同时生效。
//
// ⭐⭐⭐⭐⭐ ⭐⭐⭐⭐⭐ **⚠️ 纪律③：`license` 为空 ⭐⭐ 不是「未知」，
// ⭐⭐⭐⭐⭐ ⭐⭐ 而是「**不可上架**」** —— 本仓有商业许可阻断门
// ⭐⭐⭐⭐⭐ ⭐⭐⭐⭐⭐ （`check-license-js.sh` / `deny.toml`），⭐⭐⭐⭐⭐ 但那只管构建期；
// ⭐⭐⭐⭐⭐ ⭐⭐⭐⭐⭐ 运行期上架的插件许可无从审计 ⇒ ⭐⭐⭐⭐⭐ **必须显式声明**。

/// ⭐⭐⭐⭐⭐ 本模块需要的类型（⭐⭐⭐ 与 `full_cycle.rs` 等 registrar 同一来源）
// ⭐⭐⭐⭐⭐ `Domain` ⭐⭐ 本文件已从 `crate::l1_action` 导入（⭐⭐ 见 `:18`）
// ⭐⭐⭐⭐⭐ ⇒ ⭐⭐⭐⭐⭐ **只补真正缺的**，⭐⭐⭐⭐⭐ ⛔ 不重复引入（⭐⭐ 否则同名冲突）。
use nt_core_capability_tree::node::CapabilityNode;

// ⭐⭐ 贸易能力统一版本号（⭐⭐⭐ 一处升版，5 个能力同时生效）
pub const TRADE_ABILITY_VERSION: &str = env!("CARGO_PKG_VERSION");

/// ⭐⭐⭐⭐⭐ 能力在市场里的类别（⭐⭐ 对标 hermes 的 `category`）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeCategory {
    /// ⭐⭐ 贸易全链（⭐⭐ 编排型）
    FullCycle,
    /// ⭐⭐ 报价
    Quote,
    /// ⭐⭐ 生产物流
    Logistics,
    /// ⭐⭐ 金融合规
    Finance,
    /// ⭐⭐ 产品规格
    ProductSpec,
}

impl TradeCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FullCycle => "trade/full-cycle",
            Self::Quote => "trade/quote",
            Self::Logistics => "trade/logistics",
            Self::Finance => "trade/finance",
            Self::ProductSpec => "trade/product-spec",
        }
    }
}

/// ⭐⭐⭐⭐⭐ **给贸易能力节点填市场元数据 + 类型**（⭐⭐ **五个 registrar 的唯一入口**）。
///
/// ⭐⭐ `kind` ⭐⭐ 一律 `Skill`：⭐⭐ 贸易能力 ⭐⭐ **不是 MCP 工具**
/// （⭐⭐ 它们不进 `McpBridge` 的工具表）、⭐⭐ ⭐⭐ **不是 workflow**
/// （⭐⭐ 各自独立）、⭐⭐ ⭐⭐ 也不是 agent ⇒ ⭐⭐ `Skill` 是诚实的分类。
pub fn apply_market_meta(node: &mut CapabilityNode, category: TradeCategory, description: &str) {
    use neotrix_neobot::nt_capability_market::meta_keys;
    node.kind = nt_core_capability_tree::node::CapabilityKind::Skill;
    node.metadata.insert(
        meta_keys::VERSION.to_owned(),
        serde_json::Value::String(TRADE_ABILITY_VERSION.to_owned()),
    );
    // ⭐⭐⭐⭐⭐ **许可**（⭐⭐⭐ 纪律③ 的关键字段）。
    // ⭐⭐⭐⭐⭐ 本仓能力是**仓内自有实现**（⭐⭐ ⛔ 不是 vendored 第三方），
    // ⭐⭐⭐⭐⭐ ⇒ 许可 = 本仓许可。⭐⭐⭐⭐⭐ ⭐⭐ **显式写出** ⭐⭐ 而不是留空
    // ⭐⭐⭐⭐⭐ ⭐⭐ ⭐⭐ ⭐⭐ 因为留空 ⭐⭐ 会被市场判「不可上架」，
    // ⭐⭐⭐⭐⭐ ⭐⭐⭐⭐⭐ ⭐⭐ 而**那需要人先想清楚凭什么**才能填。
    node.metadata.insert(
        meta_keys::LICENSE.to_owned(),
        serde_json::Value::String("LicenseRef-NeoTrix-Internal".to_owned()),
    );
    node.metadata.insert(
        meta_keys::CATEGORY.to_owned(),
        serde_json::Value::String(category.as_str().to_owned()),
    );
    node.metadata.insert(
        meta_keys::DESCRIPTION.to_owned(),
        serde_json::Value::String(description.to_owned()),
    );
}

#[cfg(test)]
mod market_meta_tests {
    use super::*;
    // ⭐⭐⭐⭐⭐ `CapabilityTreeRegistry` ⭐⭐ **只有测试用** ⇒ ⭐⭐ 留在测试块内，
    // ⭐⭐⭐⭐⭐ ⭐⭐ ⛔ 不在模块级引入（⭐⭐ 否则是无用的公开依赖面）。
    use nt_core_capability_tree::registry::CapabilityTreeRegistry;
use nt_core_capability_tree::node::Domain as CapabilityTreeDomain;

    /// ⭐⭐⭐⭐⭐ **元数据齐全 ⇒ 市场可上架**（⭐⭐ 端到端，⭐⭐ 跨两个 crate）。
    #[test]
    fn 贸易能力填完市场元数据即可上架() {
        let mut n = CapabilityNode::new_primitive(
            "NT-MIND::trade::x".to_owned(),
            CapabilityTreeDomain::Mind,
            vec!["x".to_owned()],
        );
        apply_market_meta(&mut n, TradeCategory::Quote, "报价谈判能力");
        let mut reg = CapabilityTreeRegistry::default();
        reg.register(n).expect("register");
        let listable = neotrix_neobot::nt_capability_market::listable(&reg);
        assert_eq!(listable.len(), 1, "⭐⭐ 填完元数据 ⇒ ⭐⭐ **必须真的出现在市场里**");
        let e = &listable[0];
        assert_eq!(e.category, "trade/quote", "⭐⭐ category 对标 hermes");
        assert!(!e.license.is_empty(), "⭐⭐⭐ 许可必须显式（纪律③）");
        assert!(!e.version.is_empty(), "⭐⭐ 版本必须显式（⭐⭐ 否则无法判兼容性）");
        assert_eq!(e.description, "报价谈判能力");
    }

    /// ⭐⭐⭐⭐⭐ **⛔ 不填元数据 ⇒ ⭐⭐ 不可上架**（⭐⭐⭐ **反向可证伪**）。
    #[test]
    fn 未填市场元数据的能力不可上架且必须报出原因() {
        let n = CapabilityNode::new_primitive(
            "NT-MIND::trade::bare".to_owned(),
            CapabilityTreeDomain::Mind,
            vec!["bare".to_owned()],
        );
        let mut reg = CapabilityTreeRegistry::default();
        reg.register(n).expect("register");
        assert!(neotrix_neobot::nt_capability_market::listable(&reg).is_empty(),
            "⭐⭐⭐ 未填 ⇒ 不可上架");
        let blocked = neotrix_neobot::nt_capability_market::blocked(&reg);
        assert_eq!(blocked.len(), 1, "⭐⭐⭐ 且必须在 blocked 清单里（⭐⭐ ⛔ 不许静默消失）");
        assert!(blocked[0].1.contains("license"), "⭐⭐⭐ 原因要说清是 license");
    }
}
