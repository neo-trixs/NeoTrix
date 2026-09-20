//! L1 Facade — L5 认知层对 L1/L2/L3 共享类型的统一 re-export 门面
//!
//! 单一事实源: L1/L2 类型定义仍在各自层，此处仅 re-export 保持跨层引用集中可审计。
//!
//! ⚠️ 向下依赖: L5 → L1/L2/L3, 已通过此单一 facade 集中化。

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

// ─── L6 observer/gold-standard types ────────────────────────────────────────
pub use crate::l6_meta::nt_core_observer::OneObserver;
pub use crate::l6_meta::nt_core_observer_error::ObserverErrorRecovery;
pub use crate::l6_meta::healing::nt_mind_consciousness_gold_standard::ConsciousnessGoldStandard;
pub use crate::l6_meta::healing::nt_mind_consciousness_gold_standard::E8HexagramState;
pub use crate::l6_meta::healing::nt_mind_consciousness_gold_standard::{
    DEFAULT_COHERENCE_THRESHOLD, DEFAULT_PHI_THRESHOLD,
};
pub use crate::l6_meta::healing::nt_mind_consciousness_monitor::ConsciousnessMonitor;

// ─── L6 healing/self-test types ─────────────────────────────────────────────
pub use crate::l6_meta::healing::nt_core_self_test::ExternalVerifier;
pub use crate::l6_meta::nt_core_self_review::SelfReviewGate;

// ─── L6 nt_meta sub-modules ─────────────────────────────────────────────────
pub use crate::l6_meta::nt_meta::scanner;
pub use crate::l6_meta::nt_meta::weakness;
pub use crate::l6_meta::nt_meta::knowledge_gap_detector;
pub use crate::l6_meta::nt_meta::metacognition_loop;
pub use crate::l6_meta::nt_meta::monitor;
pub use crate::l6_meta::nt_meta::nt_core_arch_lint;
pub use crate::l6_meta::nt_meta::nt_core_meta_auditor;
pub use crate::l6_meta::nt_meta::planner;
pub use crate::l6_meta::nt_meta::self_model;
pub use crate::l6_meta::nt_meta::auto_inspector;
pub use crate::l6_meta::nt_meta::arch_optimizer;

// ─── L6 nt_meta type re-exports ─────────────────────────────────────────────
pub use crate::l6_meta::nt_meta::MetaCognitiveLoop;
pub use crate::l6_meta::nt_meta::MetaCycleResult;
pub use crate::l6_meta::nt_meta::CodeScanner;
pub use crate::l6_meta::nt_meta::MetaAuditor;
pub use crate::l6_meta::nt_meta::ArchLint;
pub use crate::l6_meta::nt_meta::KnowledgeGapDetector;
pub use crate::l6_meta::nt_meta::MetaMonitor;
pub use crate::l6_meta::nt_meta::WeaknessAnalyzer;
pub use crate::l6_meta::nt_meta::WeaknessReport;
pub use crate::l6_meta::nt_meta::SelfArchitectureOptimizer;
pub use crate::l6_meta::nt_meta::self_model::SelfModel as MetaSelfModel;
pub use crate::l6_meta::nt_meta::nt_core_meta_auditor::AuditorFinding;
pub use crate::l6_meta::nt_meta::weakness::Weakness;
pub use crate::l6_meta::nt_meta::self_model::ModuleInfo;
pub use crate::l6_meta::nt_meta::planner::EvolutionPlanner;
pub use crate::l6_meta::nt_meta::planner::{
    weakness_to_goals, ActionStatus, EvolutionAction, ImpactEstimate, MetaGoal, MetaGoalBridge,
    PlannedEvolution, RiskLevel as MetaRiskLevel,
};
pub use crate::l6_meta::nt_meta::knowledge_gap_detector::{
    GapCategory, GapCluster, GapReport, KnowledgeGap,
};
pub use crate::l6_meta::nt_meta::monitor::{AlertSeverity, HealthCheck, HealthTrend, MetaAlert};
pub use crate::l6_meta::nt_meta::self_model::{
    CompilationHealth, ComponentMap, ComponentNode, DebtSeverity, DepEdge, DepGraph, DepKind,
    EventKind, EvolutionEvent, FileInfo, TechDebtInventory, TechDebtItem, TechDebtKind,
    TestCoverage,
};
pub use crate::l6_meta::nt_meta::weakness::WeaknessSummary;

// ─── L6 coordination types ──────────────────────────────────────────────────
pub use crate::l6_meta::coordination::self_improvement::{SelfImprovementLoop, SystemMetrics};
pub use crate::l6_meta::coordination::nt_meta_sentrux::SentruxSensor;
pub use crate::l6_meta::coordination::nt_meta_build_watchdog::{BuildWatchdog, WatchdogConfig};
pub use crate::l6_meta::coordination::verifier_agent::_VerifierAgent;
pub use crate::l6_meta::coordination::layered_qa::_LayeredQA;
pub use crate::l6_meta::coordination::quality_control::_QualityControlPipeline;
pub use crate::l6_meta::coordination::quality_gate::QualityGate;
pub use crate::l6_meta::coordination::template_tag_registry::_TemplateTagRegistry;

// ─── L6 memory types ────────────────────────────────────────────────────────
pub use crate::l6_meta::memory::evolution_harness::EvolutionHarness;
pub use crate::l6_meta::memory::meta_observer::{MetaObserver, MetaObserverConfig};
pub use crate::l6_meta::memory::meta_observer::MetaObserverSelfTest;
pub use crate::l6_meta::memory::transcendent_loop::LoopConfig;

// ─── L6 scheduler types ─────────────────────────────────────────────────────
pub use crate::l6_meta::nt_core_scheduler::{SchedulerEngine, default_scheduler};

// ─── L6 constitution types ──────────────────────────────────────────────────
pub use crate::l6_meta::nt_core_self_constitution::ConstitutionLoader;
pub use crate::l6_meta::nt_core_self_constitution::Constitution;
pub use crate::l6_meta::nt_core_self_constitution::global_constitution;
pub use crate::l6_meta::nt_core_self_constitution::GovernanceConstitutionSelfTest;
pub use crate::l6_meta::nt_core_self_constitution::RuleCategory;

// ─── L6 nexus types ─────────────────────────────────────────────────────────
pub use crate::l6_meta::nt_nexus::cross_session_memory::{CrossSessionMemory, CrossSessionMemorySelfTest, MemoryCategory};

// ─── L6 nt_core_kb_types ────────────────────────────────────────────────────
pub use neotrix_types::knowledge_access::{NodeType, RelationType};

// ─── L6 auto-inspector ──────────────────────────────────────────────────────
pub use crate::l6_meta::nt_meta::auto_inspector::AutoInspector;

// ─── L6 nt_core_self modules (L5 → L0 abstraction) ──────────────────────────
// Module re-exports (preserves submodule paths for L5 code)
pub use crate::l6_meta::nt_core_self::attention_head;
pub use crate::l6_meta::nt_core_self::emotion_state;
pub use crate::l6_meta::nt_core_self::metacognitive_evaluator;
pub use crate::l6_meta::nt_core_self::silicon_self;
pub use crate::l6_meta::nt_core_self::self_referential;
pub use crate::l6_meta::nt_core_self::intrinsic_motivation;
pub use crate::l6_meta::nt_core_self::archive;
pub use crate::l6_meta::nt_core_self::skill_crystal;
pub use crate::l6_meta::nt_core_self::reasoning_strategy;
pub use crate::l6_meta::nt_core_self::thinking_trace;
pub use crate::l6_meta::nt_core_self::self_audit;
pub use crate::l6_meta::nt_core_self::seal;
pub use crate::l6_meta::nt_core_self::affective_interface;

// Type re-exports (direct access without submodule)
pub use crate::l6_meta::nt_core_self::attention_head::{
    AttentionDomain, AttentionManager, ThinkingMode, WeaponSet,
};
pub use crate::l6_meta::nt_core_self::emotion_state::{
    EmotionDimension, EmotionEngine, EmotionLabel, EmotionReport,
};
pub use crate::l6_meta::nt_core_self::metacognitive_evaluator::{
    CognitiveEvaluator, CognitiveHealthReport, FlagSeverity,
};
pub use crate::l6_meta::nt_core_self::silicon_self::{SiliconSelfModel, SiliconSelfState};
pub use crate::l6_meta::nt_core_self::self_model::SelfModel;
pub use crate::l6_meta::nt_core_self::self_referential::SelfReferentialMonitor;
pub use crate::l6_meta::nt_core_self::intrinsic_motivation::{IntrinsicMotivation, MotivationState};
pub use crate::l6_meta::nt_core_self::archive::SiliconArchive;
pub use crate::l6_meta::nt_core_self::skill_crystal::{
    CrystalRegistry, SkillCrystal, VerificationContract, VerificationStatus as CrystalVerificationStatus,
};
pub use crate::l6_meta::nt_core_self::reasoning_strategy::StrategyKind;
pub use crate::l6_meta::nt_core_self::thinking_trace::{ReflectionGrade, ThinkingStep, ThinkingTrace};
pub use crate::l6_meta::nt_core_self::self_audit::{
    AuditFinding, AuditReport, AuditSeverity, ConvergeCheckFn, MultiSignalEval,
    ToolGroundingMonitor, converge_check, scan_build_status, scan_disk_pressure,
    scan_memory_pressure, scan_system_health, scan_test_flakiness,
};
pub use crate::l6_meta::nt_core_self::seal::ConstitutionGate;
pub use crate::l6_meta::nt_core_self::seal::grpo::{GrpoConfig, GRPOLoop};
pub use crate::l6_meta::nt_core_self::affective_interface::AffectiveInterface;

// ─── L6 value-function self-model ───────────────────────────────────────────
pub use crate::l6_meta::nt_core_self_model::SelfModel as ValueSelfModel;

// ─── L6 spec-driven pipeline ────────────────────────────────────────────────
pub use crate::l6_meta::nt_core_absorb::spec_driven::{
    EvolutionSpec, SpecDiff, SpecDrivenPipeline, SpecPipelineConfig, SpecPipelineStats, SpecStatus,
    SpecVerification, SpecVerifier,
};

// ─── L6 self-ref code ───────────────────────────────────────────────────────
pub use crate::l6_meta::nt_core_iter::self_ref_code::{SelfCodeMonitor, MutationResult};
