//! ACT Facade — L5 对 L1 NT-ACT 共享类型的 re-export 门面
//!
//! L5 认知层通过此模块访问 ACT 共享函数/类型，避免散布 `use crate::l1_action::nt_act::*`。

pub use crate::l1_action::nt_act::nt_act_code::recipe_refactor;
pub use crate::l1_action::nt_act::nt_act_crypto::CryptoAgent;
pub use crate::l1_action::nt_act::nt_act_trade::finance_compliance::{
    CollectionRecord as FCCollectionRecord, LcReview as FCLcReview, PaymentProof as FCPaymentProof,
    SettlementRecord as FCSettlementRecord, TaxRefundClaim as FCTaxRefundClaim,
};
pub use crate::l1_action::nt_act::nt_act_trade::full_cycle::QuoteSheet as QNQuoteSheet;
pub use crate::l1_action::nt_act::nt_act_trade::orchestrator::{
    LogisticsInfo, OrchBuyerProfile as BuyerProfile, OrchContract as Contract,
    OrchTradeContext as TradeContext, PaymentInfo, ProductionStatus, Quotation, QuotationItem,
    SettlementInfo, TradeEvent, TradeGroup, TradePhase26 as TradePhase,
};
pub use crate::l1_action::nt_act::nt_act_trade::production_logistics::{
    BillOfLading as PLBillOfLading, BookingConfirmation as PLBookingConfirmation,
    CiqCertificate as PLCiqCertificate, CustomsDeclaration as PLCustomsDeclaration,
    InspectionReport as PLInspectionReport, PackingItem as PLPackingItem,
    PackingList as PLPackingList,
};
pub use crate::l1_action::nt_act::nt_act_trade::trade_core::RiskLevel as PLRiskLevel;
pub use crate::l1_action::nt_act::nt_act_trade::{
    capability_spec, execute_quote_negotiation, execute_trade_full_cycle,
    register_finance_compliance_capability, register_production_logistics_capability,
    register_quote_negotiation_capability, register_trade_full_cycle_capability,
    ActionRecommendation, AlertLevel, BillOfLading, BlStatus, BlType, BomItem, BomRequirement,
    BookingConfirmation, BookingRequirements, CargoInfo, CiqCertificate, CollectionDocument,
    CollectionRecord, CollectionStatus, CompanyPolicy, CompetitorData, Concession, ConfirmedItem,
    ContractFinding, ContractItem, ContractReview, CostBreakdown, CustomsDeclaration,
    CustomsStatus, DailyProgress, Defect, DefectSeverity, Discrepancy, DocumentStatus,
    FinanceDocSet, FinanceEngine, FindingSeverity, InquiryDetail, InspectionReport,
    InspectionResult, IntentLevel, KnowledgeDelta, KnowledgeOperation, LcRecommendation, LcReview,
    Lesson, LogisticsDocSet, LogisticsEngine, MarketEnvironment, MaterialStatus, Milestone,
    MilestoneDelay, MilestoneStatus, MockBankSystem, MockContainer, MockCustomsStatus,
    MockCustomsSystem, MockDeclaration, MockErpSystem, MockLc, MockLcStatus, MockOrder,
    MockOrderStatus, MockPayment, MockShipment, MockShipmentStatus, MockShippingSystem,
    NegotiationEngine, NegotiationRecord, NegotiationStrategy, Objection, ObjectionCategory,
    ObjectionSeverity, PackagingSpec, PackingItem, PackingList, PaymentProof, PaymentStatus,
    PaymentType, ProductSpec, ProductType, ProductionAlert, ProductionEngine, ProductionMilestone,
    ProductionOrder, ProductionSchedule, ProgressReport, QuoteGenerator, QuoteSheet,
    RefundDocument, RefundStatus, RequirementConfirmation, RiskAlert, RiskControl, RiskFlag,
    RiskLevel, RoutingRequirement, RoutingStatus, RoutingStep, Schedule, ScheduleDeviation,
    SettlementRecord, SoftClause, SupplierOrder, SupplierOrderItem, SupplierOrderStatus,
    TaxRefundClaim, TradeCapabilitySpec, TradeIntegrationHarness, TradeOrchestrator, TradeResult,
    TradeStateMachine, VerificationStatus,
};
pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
