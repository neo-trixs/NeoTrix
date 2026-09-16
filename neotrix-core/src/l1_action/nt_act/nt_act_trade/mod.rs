pub mod capabilities;
pub mod capability_registry;
pub mod contract_parser;
pub mod error;
pub mod extractors;
pub mod data_model;
pub mod engine_traits;
pub mod event_bus;
pub mod finance_compliance;
pub mod knowledge_base;
pub mod full_cycle;
pub mod message;
pub mod mock_adapters;
pub mod nt_trade_crm;
pub mod nt_trade_dashboard;
pub mod nt_trade_documents;
pub mod nt_trade_email;
pub mod nt_trade_pipeline;
pub mod nt_trade_supplier_mgmt;
pub mod nt_trade_tasks;
pub mod orchestrator;
pub mod orchestrator_v2;
pub mod path_metadata;
pub mod platform_registry;
pub mod process_engine;
pub mod production_logistics;
pub mod quote_negotiation;
pub mod sqlite_knowledge_base;
pub mod template_detector;
pub mod trade_core;
pub mod trade_knowledge;
pub mod data_pipeline;
pub mod router;
pub mod unified_types;  // 新增：统一类型系统
pub mod workers;
// 从 L1 Action 层复用限流/熔断基础设施
pub use crate::l1_action::nt_act::actions::core::nt_act_rate_limiter::{RateLimiter, RateLimiterConfig};
pub use crate::l1_action::nt_act::actions::core::nt_act_circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};

#[cfg(test)]
pub mod tests;

// ── error: 统一错误类型 ──────────────────────────────────
pub use error::TradeError;

// ── unified_types: 统一类型系统 (Single Source of Truth) ──
pub use unified_types::{
    stable_id, Grade, Channel, ProductKind, Currency, FlowStep, OrderNode,
    ProductCategory, DriveType, ConnectionType, TradeTerms,
    OrderStatus, QuoteStatus, InquiryStatus,
    MaterialSpec, PressureRating, SizeSpec, PriceInfo,
    PerformanceMetrics, InquiryMetadata, TradeContactInfo,
    Product, Supplier, Customer, InquiryItem, Inquiry,
    QuoteItem, Quote, Order,
    // 通用通信能力 (从 nt_act::communication 重新导出)
    CommunicationChannel, ContactInfo, ContactMethod, InteractionDirection,
    InteractionRecord, InteractionType,
    SocialMediaPlatform,
};

// ── data_model: 统一数据模型 ─────────────────────────────
// Types already exported from unified_types above — no duplicate re-exports.

// ── event_bus: 外贸事件总线 ──────────────────────────────
pub use event_bus::{
    AsyncTradeEventHandler, ComplianceReviewResult, EventPayload, EventBusHandle,
    EventType, RiskSeverity, SubscriberId, TradeEvent, TradeEventBus, TradeEventHandler,
};

// ── knowledge_base: 外贸知识库接口 ──────────────────────
pub use knowledge_base::{
    CustomerRecord, KnowledgeBase, KnowledgeBaseError, KnowledgeResult, KnowledgeUpdateResult,
    PriceQuery, PriceResult, ProductFilters, ProductMatchEntry, ProductMatchResult,
    ProductQueryResult, ProductRecord, SupplierFilters, SupplierMatchEntry,
    SupplierMatchResult, SupplierQueryResult, SupplierRecord,
};

// ── trade_core: 算法骨架 (single source of truth) ────────
pub use trade_core::{
    CompetitorData, Concession, CostBreakdown, CostCalculator, CostContext, Milestone,
    MilestoneDelay, MilestoneStatus, NegotiationEngine, NegotiationStrategy, RiskAssessment,
    RiskAssessor, RiskFinding, RiskLevel, ScheduleDeviation, State, StateMachine,
};

// ── full_cycle: 域类型 (canonical domain types) ──────────
pub use full_cycle::{
    capability_spec, execute_trade_full_cycle, register_trade_full_cycle_capability,
    ActionRecommendation, BomItem, BuyerProfile, CompanyPolicy, Contract, ContractItem,
    InquiryDetail, IntentLevel, KnowledgeDelta, KnowledgeOperation, Lesson, MarketEnvironment,
    PackagingSpec, ProductSpec, ProductType, QuoteSheet, RiskAlert, RiskControl, RoutingStep,
    Schedule, TradeCapabilitySpec, TradeContext, TradePhase, TradePhaseGroup,
    TradeResult, TradeStateMachine, LogisticsDocSet, FinanceDocSet,
};

// ── quote_negotiation: 报价谈判 ──────────────────────────
pub use quote_negotiation::{
    execute_quote_negotiation, register_quote_negotiation_capability, ConfirmedItem,
    NegotiationRecord, Objection, ObjectionCategory, ObjectionSeverity, QuoteGenerator,
    RequirementConfirmation,
};

// ── production_logistics: 生产物流 (single source for inspection/logistics types) ──
pub use production_logistics::{
    register_production_logistics_capability, AlertLevel, BillOfLading, BlStatus, BlType,
    BomRequirement, BookingConfirmation, BookingRequirements, BookingStatus, CargoInfo,
    CiqCertificate, CiqStatus, CustomsDeclaration, CustomsStatus, DailyProgress, Defect,
    DefectSeverity, InspectionReport, InspectionResult, LogisticsEngine, MaterialStatus,
    Measurement, PackingItem, PackingList, ProductionAlert, ProductionEngine, ProductionMilestone,
    ProductionOrder, ProductionSchedule, ProgressReport, RoutingRequirement, RoutingStatus,
    SupplierOrder, SupplierOrderItem, SupplierOrderStatus,
};

// ── finance_compliance: 财务合规 ──────────────────────────
pub use finance_compliance::{
    register_finance_compliance_capability, CollectionDocument, CollectionStatus, ContractFinding,
    ContractReview, Discrepancy, DocumentStatus, FinanceEngine, FindingSeverity, LcRecommendation,
    PaymentStatus, PaymentType, RefundDocument, RefundStatus, RiskFlag, SoftClause,
    VerificationStatus, PaymentProof, LcReview, CollectionRecord, SettlementRecord, TaxRefundClaim,
};

// ── mock_adapters ─────────────────────────────────────────
pub use mock_adapters::{
    MockBankSystem, MockContainer, MockCustomsStatus, MockCustomsSystem, MockDeclaration,
    MockErpSystem, MockLc, MockLcStatus, MockOrder, MockOrderStatus, MockPayment, MockShipment,
    MockShipmentStatus, MockShippingSystem, TradeIntegrationHarness,
};

// ── orchestrator ──────────────────────────────────────────
pub use orchestrator::{
    OrchBuyerProfile, OrchContract, OrchTradeContext, OrderReview, TradeGroup, TradeOrchestrator,
    TradePhase26,
};

// ── orchestrator_v2: 多智能体编排 (Orchestrator-Worker + Router) ──
pub use orchestrator_v2::{
    OrchestratorConfig as OrchestratorV2Config,
    OrchestratorResult as OrchestratorV2Result,
    OrchestratorStats as OrchestratorV2Stats,
    TaskPriority as OrchestratorV2TaskPriority,
    TaskStatus as OrchestratorV2TaskStatus,
    TradeMessage as OrchestratorV2Message,
    TradeRouter as OrchestratorV2Router,
    TradeTask as OrchestratorV2Task,
    WorkerPool as OrchestratorV2WorkerPool,
    DomainWorkerResult as OrchestratorV2WorkerResult,
    DomainWorkerType as OrchestratorV2WorkerType,
};

// ── process_engine: 流程引擎 ────────────────────────────
pub use process_engine::{
    EventHandler, NoOpHandler, ProcessDefinition, ProcessEngine, ProcessEvent,
    ProcessInstance, ProcessStatus, ProcessStep, StepCondition, StepHandler, StepResult,
    StepStatus,
};

// ── capability_registry: 统一能力注册表 ──────────────────
pub use capability_registry::{
    CapabilityInfo, PriceCalculatorCapability, ProductMatcherCapability,
    RiskAssessorCapability, StepHandlerCapability, SupplierMatcherCapability,
    TradeCapability, TradeCapabilityError, TradeCapabilityInput, TradeCapabilityOutput,
    TradeCapabilityRegistry, TradeCapabilityType,
    create_default_registry, create_empty_registry,
};

// ── path_metadata: 路径元数据提取 ────────────────────────
pub use path_metadata::{extract_order_metadata, OrderMetadata};

// ── template_detector: 合同模板检测 ───────────────────────
pub use template_detector::{build_column_map, detect_template, ContractTemplate, TemplateColumnMap};

// ── contract_parser: 合同 Excel 解析器 ──────────────────────
pub use contract_parser::{ContractParser, ContractType, ParsedContract, ParsedContractItem};

// ── nt_trade_crm: 客户关系管理 ─────────────────────────────
pub use nt_trade_crm::{
    Contact, Company, CustomerProfile, CustomerGrade, CustomerSource, CustomerStatus,
    CustomerFilters, CustomerQueryResult, CrmSummary,
    InteractionRecord as CrmInteractionRecord, InteractionType as CrmInteractionType,
    TradeCrmEngine,
};

// ── nt_trade_pipeline: 销售管道 ─────────────────────────────
pub use nt_trade_pipeline::{
    Deal, DealStage, DealSummary, LossReason, PipelineStageSummary, PipelineView,
    StageChange, TradePipelineEngine,
};

// ── nt_trade_email: 邮件集成 ───────────────────────────────
pub use nt_trade_email::{
    EmailAttachment, EmailRecord, EmailStatus, EmailSummary, EmailTemplate,
    EmailTemplateType, EmailTracking, TradeEmailEngine,
};

// ── nt_trade_documents: 单证管理 ────────────────────────────
pub use nt_trade_documents::{
    DocField, DocumentSet, TradeDocStatus, TradeDocTemplate, TradeDocument,
    TradeDocumentEngine, TradeDocumentType,
};

// ── nt_trade_tasks: 任务/日历 ──────────────────────────────
pub use nt_trade_tasks::{
    CalendarEvent, TaskPriority, TradeTask, TradeTaskEngine, TradeTaskStatus, TradeTaskType,
};

// ── nt_trade_dashboard: 数据看板 ───────────────────────────
pub use nt_trade_dashboard::{
    CustomerAnalytics, FunnelStage, KpiSummary, LogisticsMetrics, ProductAnalytics,
    ProductRanking, RegionDistribution, RevenueDataPoint, RevenueTrend, SalesFunnel,
    TimeGranularity, TradeDashboard, TradeDashboardEngine,
};

// ── nt_trade_supplier_mgmt: 供应商评估 ─────────────────────
pub use nt_trade_supplier_mgmt::{
    SupplierComparison, SupplierEvaluation, SupplierGrade, SupplierMgmtEngine,
    SupplierProfile,
};

// ── data_pipeline: 外部平台数据管线 ──────────────────────
pub use data_pipeline::{
    DataNormalizer, Email, EmailConfig, ExtractConfig, ExtractionResult, Interaction,
    PlatformRegistry, SyncResult, TradeDataPipeline,
};

// ── platform_registry: 平台注册中心 ────────────────────────
// Types accessible via `platform_registry::PlatformRegistry` etc.
// Not re-exported here to avoid collision with data_pipeline::PlatformRegistry.

// ── extractors: 数据提取/解密 ───────────────────────────────
pub use extractors::{ChromeDecryptor, LoginEntry};

// ── workers: 异步工作池 ──────────────────────────────────────
pub use workers::{
    AnalyzeWorker, ExtractWorker, SendWorker, TrackWorker, WorkerPool, WorkerResult, WorkerTask,
    TaskWorkerType, WriteWorker,
};

// ── engine_traits: 统一 Engine 契约层 ──────────────────────
pub use engine_traits::{
    EngineInfo, EngineMetrics, EngineStatus, EngineType, TradeEngine, TradeEngineRegistry,
};

// ── trade_knowledge: 外贸知识库查询接口 ──────────────────────
pub use trade_knowledge::{CustomerQueryResult as TradeCustomerQueryResult, CustomerStats, TradeKnowledgeQuery};

// ── message: 多 Agent 通信协议 ──────────────────────────────
pub use message::{
    AlertSeverity, AnalyzeCustomers, DataExtracted, DataNormalized, DataStored, Envelope,
    ExtractCustomers, GenerateReport, MessageHeader, MessagePriority, ReportFormat, SyncCompleted,
    SyncStarted, TaskError, TaskProgress, TaskRequest, TaskResponse, TaskStatus, TradeMessage,
};


