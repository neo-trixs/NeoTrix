pub mod finance_compliance;
pub mod full_cycle;
pub mod mock_adapters;
pub mod orchestrator;
pub mod production_logistics;
pub mod quote_negotiation;
pub mod trade_core;

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
