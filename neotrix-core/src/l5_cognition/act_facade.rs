//! ACT Facade — L5 对 L1 NT-ACT 共享类型的 re-export 门面
//!
//! L5 认知层通过此模块访问 ACT 共享函数/类型，避免散布 `use crate::l1_action::nt_act::*`。

pub use crate::l1_action::nt_act::nt_act_code::recipe_refactor;
pub use crate::l1_action::nt_act::nt_act_crypto::CryptoAgent;
pub use crate::l1_action::nt_act::nt_act_trade::{
    TradeStateMachine, TradeCapabilitySpec, TradeResult,
    InquiryDetail, IntentLevel, ProductSpec, ProductType,
    BomItem, RoutingStep, PackagingSpec, CompanyPolicy, RiskControl,
    MarketEnvironment, QuoteSheet, ContractItem, Schedule,
    Milestone, MilestoneStatus, InspectionReport, InspectionResult, Defect,
    DefectSeverity, LogisticsDocSet, CiqCertificate, BookingConfirmation,
    CustomsDeclaration, BillOfLading, PackingList, PackingItem,
    FinanceDocSet, PaymentProof, LcReview, CollectionRecord, SettlementRecord,
    TaxRefundClaim, RiskAlert, RiskLevel, ActionRecommendation, Lesson,
    KnowledgeDelta, KnowledgeOperation,
    execute_trade_full_cycle, register_trade_full_cycle_capability, capability_spec,
    NegotiationStrategy, ObjectionCategory, RequirementConfirmation, ConfirmedItem,
    QuoteGenerator, CostBreakdown, NegotiationEngine,
    Concession, CompetitorData, NegotiationRecord, Objection, ObjectionSeverity,
    execute_quote_negotiation, register_quote_negotiation_capability,
    ProductionOrder, BomRequirement, MaterialStatus, RoutingRequirement, RoutingStatus,
    ProductionSchedule, SupplierOrder, SupplierOrderItem, SupplierOrderStatus,
    ProductionMilestone, ProgressReport, DailyProgress, ScheduleDeviation, MilestoneDelay,
    ProductionAlert, AlertLevel, CustomsStatus, BlType, BlStatus,
    ProductionEngine, LogisticsEngine, CargoInfo, BookingRequirements,
    register_production_logistics_capability,
    ContractReview, ContractFinding, FindingSeverity, PaymentType, PaymentStatus, RiskFlag,
    SoftClause, Discrepancy, LcRecommendation,
    CollectionDocument, DocumentStatus, CollectionStatus,
    VerificationStatus, RefundDocument, RefundStatus,
    FinanceEngine, register_finance_compliance_capability,
    MockErpSystem, MockBankSystem, MockCustomsSystem, MockShippingSystem,
    TradeIntegrationHarness, MockOrder, MockOrderStatus, MockLc, MockLcStatus,
    MockPayment, MockDeclaration, MockCustomsStatus, MockShipment, MockShipmentStatus,
    MockContainer,
    TradeOrchestrator,
};
pub use crate::l1_action::nt_act::nt_act_trade::full_cycle::QuoteSheet as QNQuoteSheet;
pub use crate::l1_action::nt_act::nt_act_trade::production_logistics::{
    InspectionReport as PLInspectionReport, CiqCertificate as PLCiqCertificate,
    BookingConfirmation as PLBookingConfirmation, PackingList as PLPackingList,
    PackingItem as PLPackingItem, CustomsDeclaration as PLCustomsDeclaration,
    BillOfLading as PLBillOfLading,
};
pub use crate::l1_action::nt_act::nt_act_trade::trade_core::RiskLevel as PLRiskLevel;
pub use crate::l1_action::nt_act::nt_act_trade::finance_compliance::{
    PaymentProof as FCPaymentProof, LcReview as FCLcReview,
    CollectionRecord as FCCollectionRecord, SettlementRecord as FCSettlementRecord,
    TaxRefundClaim as FCTaxRefundClaim,
};
pub use crate::l1_action::nt_act::nt_act_trade::orchestrator::{
    TradePhase26 as TradePhase, TradeGroup, OrchTradeContext as TradeContext, TradeEvent,
    OrchBuyerProfile as BuyerProfile, Quotation, QuotationItem, OrchContract as Contract,
    ProductionStatus, LogisticsInfo, PaymentInfo, SettlementInfo,
};
