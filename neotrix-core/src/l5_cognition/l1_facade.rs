//! L1 Facade — L5 认知层对 L1/L2/L3 共享类型的统一 re-export 门面
//!
//! 单一事实源: L1/L2 类型定义仍在各自层，此处仅 re-export 保持跨层引用集中可审计。
//!
//! ⚠️ 向下依赖: L5 → L1/L2/L3, 已通过此单一 facade 集中化。

// L6 再导出已收敛至两个子门面（T04 facade 收敛）：
// - `l1_facade_observer`: 观察/健康/自测类
// - `l1_facade_meta`: nt_meta/进化/审计/governance 类
// `#[path]` 必需：`l1_facade.rs` 为文件模块，其 `pub mod` 默认寻址
// `l1_facade/<name>.rs` 子目录； sibling 文件需显式 path，否则 rustc E0583。
#[path = "l1_facade_meta.rs"]
pub mod l1_facade_meta;
#[path = "l1_facade_observer.rs"]
pub mod l1_facade_observer;

// ═══════════════════════════════════════════════════════════════════════════════
// NT-ACT 共享类型 (原 act_facade)
// ═══════════════════════════════════════════════════════════════════════════════
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

// ═══════════════════════════════════════════════════════════════════════════════
// NT-IO 共享类型 (原 io_facade)
// ═══════════════════════════════════════════════════════════════════════════════
pub use crate::l1_action::nt_io::nt_io_provider::context_budget::estimate_tokens;
pub use crate::l1_action::nt_io::nt_io_provider::common::types::{
    LlmError, LlmProvider, LlmProviderType, LlmRequest, LlmResponse,
};
pub use crate::l1_action::nt_io::nt_io_provider::common::factory::{create_gateway, create_provider_from_type};
pub use crate::l1_action::nt_io::nt_io_standalone::{
    ReasoningKernel, ReasoningMethod, ReasoningOutput,
    StageInfo, KernelStats, SelfConsistencyResult, verify_answer,
    text_to_vector, format_kernel_output,
    EVOLUTION, KERNEL_DIM,
};

// ═══════════════════════════════════════════════════════════════════════════════
// NT-IO 技能模块 (原 io_skills_facade)
// ═══════════════════════════════════════════════════════════════════════════════
pub use crate::l1_action::nt_io::l3_vendor_skills::nt_io_excalidraw::*;
pub use crate::l1_action::nt_io::l3_vendor_skills::nt_io_hermes_community::*;

// ═══════════════════════════════════════════════════════════════════════════════
// NT-MEMORY KB 共享类型 (原 kb_facade)
// ═══════════════════════════════════════════════════════════════════════════════
pub use crate::l4_emotion::nt_memory::nt_memory_kb::bm25;
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_coeffect::*;
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_community::{
    CommunityAwareSearch, CommunityDetector,
};
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_confidence::RetrievalStrategy;
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_crawl::{
    enqueue_seed_urls, extract_html_content, is_safe_fetch_url,
};
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_store::{
    claim_next_crawl_url, count_nodes_by_domain, ensure_domain_cluster, get_all_edges,
    get_all_nodes, mark_crawl_complete, update_cluster_stats,
};
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_types::{
    ConversationRecord, ProceduralMemoryRecord, SearchResult,
};
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_unify::{
    skill_content_hash, skill_list_all, skill_upsert, SkillRecord,
};
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_write_guard::*;
pub use crate::l4_emotion::nt_memory::nt_memory_kb::KnowledgeBase;
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_pipeline::AbsorbEntry;

// ═══════════════════════════════════════════════════════════════════════════════
// L3 共享类型 (原 l3_facade)
// ═══════════════════════════════════════════════════════════════════════════════
pub use crate::l3_embodiment::nt_shield::nt_shield_audit::{
    write_guard_check_result, CheckStatus,
};

// ═══════════════════════════════════════════════════════════════════════════════
// NT-WORLD 感知层类型 (原 l2_facade)
// ═══════════════════════════════════════════════════════════════════════════════
// ── nt_world_model_v2 ──
pub use crate::l2_perception::nt_world::nt_world_model_v2::WorldModelV2;

// ── nt_world_search ──
pub use crate::l2_perception::nt_world::nt_world_search::{SearchResult as WorldSearchResult, UnifiedSearch};

// ── nt_world_novel ──
pub use crate::l2_perception::nt_world::nt_world_novel::{
    drain_novel_queue, classify_unanalyzed_books, QidianIngestReport,
};

// ═══════════════════════════════════════════════════════════════════════════════
// L6 Concrete Type Re-exports (moved from l0_substrate::nt_core_cross_layer)
// ═══════════════════════════════════════════════════════════════════════════════
// L5 is the proper location for L6 re-exports — L0 should not depend on L6.
// These types have deep field access patterns that prevent trait-object usage.

// ─── L6 observer/health/self-test (源：l1_facade_observer) ────────────────────
// 实定义已迁至 `l1_facade_observer.rs`；此处 glob 回导保持 `l1_facade::Xxx` 零断裂。
pub use self::l1_facade_observer::*;

// ─── L6 nt_meta/evolution/governance (源：l1_facade_meta) ─────────────────────
// 实定义已迁至 `l1_facade_meta.rs`；此处 glob 回导保持 `l1_facade::Xxx` 零断裂。
pub use self::l1_facade_meta::*;
