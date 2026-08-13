//! # NeoTrix 核心模块
//!
//! 10-Layer Architecture:
//!   L10 Transcendent — L9 Transcendent — L8 Autonomic — L7 Capability — L6 Self —
//!   L5 Consciousness — L4 Cognition — L3 Memory — L2 Perception —
//!   L1 Body — L0 Substrate
//!
//! 统一版本: 0.18.0

// ─── Implementation layer directories ─────────────────────────────────────
pub mod l10_transcendent_impl;
pub mod l1_body_impl;
pub mod l2_world_impl;
pub mod l3_memory_impl;
pub mod l4_cognition_impl;
pub mod l5_consciousness_impl;
pub mod l6_self_impl;
pub mod l7_capability_impl;
pub mod l8_autonomic_impl;
pub mod l8_capability_impl;
pub mod l9_capability_impl;
pub mod l9_transcendent_impl;

// ─── 经验 → 能力树迭代目标桥 (experience-tree 蒸馏 → capability registry) ─
pub mod nt_capability_bridge;

// ─── iOS FFI bridge (uniffi → Swift) ──────────────────────────────────────
#[cfg(feature = "ios-bridge")]
pub mod ffi;

// ─── Migrated module re-exports (backward compat with crate::neotrix::nt_*) ─

// L9 — Transcendent (超验层) — 观察自身
pub use l9_transcendent_impl::nt_mind_consciousness_gold_standard;
pub use l9_transcendent_impl::nt_mind_consciousness_monitor;

// L10 — Transcendent (超越层) — 观察观察者, 意识↔能力网共振
pub use l10_transcendent_impl::consonance_orchestrator::{
    CapabilityNodeInfo, CapabilityResonance, ConsonanceConfig, ConsonanceOrchestrator,
    ConsonanceReport,
};
pub use l10_transcendent_impl::meta_observer::{MetaObservationReport, MetaObserver};
pub use l10_transcendent_impl::transcendent_loop::{
    EvolutionSuggestion, LoopConfig, LoopReport, TranscendentLoop,
};

// L8 — Autonomic (自主神经层) — 自我进化
pub use l8_autonomic_impl::nt_mind;
pub use l8_autonomic_impl::nt_mind_autofixer;
pub use l8_autonomic_impl::nt_mind_background_config;
pub use l8_autonomic_impl::nt_mind_background_loop;
pub use l8_autonomic_impl::nt_mind_benchmark;
pub use l8_autonomic_impl::nt_mind_cleanup;
pub use l8_autonomic_impl::nt_mind_distiller;
pub use l8_autonomic_impl::nt_mind_evolution_daemon;
pub use l8_autonomic_impl::nt_mind_evolution_loop;
pub use l8_autonomic_impl::nt_mind_hook;
pub use l8_autonomic_impl::nt_mind_knowledge_pipeline;
pub use l8_autonomic_impl::nt_mind_memory;
pub use l8_autonomic_impl::nt_mind_self_diagnose;
pub use l8_autonomic_impl::nt_mind_skill_engine;
pub use l8_autonomic_impl::nt_repair_causal_trace;

// L7 — Capability (能力层) — 统一路由
pub use l7_capability_impl::BoundedEdit;
pub use l7_capability_impl::GroupCoordinator;
pub use l7_capability_impl::SkillBank;
pub use l7_capability_impl::SkillDoc;
pub use l7_capability_impl::SkillExtractor;
pub use l7_capability_impl::SkillMetrics;
pub use l7_capability_impl::SkillOptimizer;
pub use l7_capability_impl::SkillRegistry;
pub use l7_capability_impl::ValidationGate;

// L6 — Self (自我层) — 身份
pub use l6_self_impl::nt_core_intra_reflection;

// L5 — Consciousness (意识层) — 体验
pub use l5_consciousness_impl::nt_core_fep_iit;
pub use l5_consciousness_impl::nt_core_iit_phi;
pub use l5_consciousness_impl::nt_core_signal;

// L4 — Cognition (认知层) — 推理
pub use l4_cognition_impl::nt_core_kernel;
pub use l4_cognition_impl::nt_core_parallel;

// L3 — Memory (记忆层) — 记忆
pub use l3_memory_impl::nt_memory_historian;
pub use l3_memory_impl::nt_memory_kb;
pub use l3_memory_impl::nt_memory_leann_store;
pub use l3_memory_impl::nt_memory_spatial;

// L2 — Perception (感知层) — 感知
pub use l2_world_impl::nt_world_browse;
pub use l2_world_impl::nt_world_browse_auto;
pub use l2_world_impl::nt_world_code_search;
pub use l2_world_impl::nt_world_crawl;
pub use l2_world_impl::nt_world_e8;
pub use l2_world_impl::nt_world_infer;
pub use l2_world_impl::nt_world_jepa;
pub use l2_world_impl::nt_world_map;
pub use l2_world_impl::nt_world_model;
pub use l2_world_impl::nt_world_model_v2;
pub use l2_world_impl::nt_world_novel;
pub use l2_world_impl::nt_world_osint;
pub use l2_world_impl::nt_world_scrape;
pub use l2_world_impl::nt_world_search;
pub use l2_world_impl::nt_world_sense;
pub use l2_world_impl::nt_world_video_pipeline;

// L1 — Body (身体层) — 行动、界面、安全
pub use l1_body_impl::nt_io_http_factory;
pub use l1_body_impl::nt_io_logging;
pub use l1_body_impl::nt_io_mention;
pub use l1_body_impl::nt_io_standalone::{
    format_kernel_output, text_to_vector, verify_answer, KernelStats, ReasoningKernel,
    ReasoningMethod, ReasoningOutput, SelfConsistencyResult, StageInfo, Vector, EVOLUTION,
    KERNEL_DIM,
};

pub use l1_body_impl::nt_io_provider::types::{
    LlmError, LlmProvider, LlmRequest, LlmResponse, Message, Role,
};

pub use crate::core::nt_core_reasoning::{
    default_context_builder, default_method_registry, MethodRegistry, MethodSpec, ReasoningStep,
    ReasoningTrace, TraceSource,
};
pub use l1_body_impl::nt_io_agents_md;
pub use l1_body_impl::nt_io_avatar_channel;
pub use l1_body_impl::nt_io_digital_human;
pub use l1_body_impl::nt_io_hotreload;
pub use l1_body_impl::nt_io_neocodex;
pub use l1_body_impl::nt_io_notify;
pub use l1_body_impl::nt_io_plugin;
pub use l1_body_impl::nt_io_provider;
pub use l1_body_impl::nt_io_proxy_server;
pub use l1_body_impl::nt_io_session_recovery;
#[cfg(feature = "telemetry")]
pub use l1_body_impl::nt_io_telemetry;
pub use l1_body_impl::nt_io_user_avatar;
pub use l1_body_impl::nt_io_web;

pub use l1_body_impl::nt_shield;
pub use l1_body_impl::nt_shield_audit;
pub use l1_body_impl::nt_shield_comm;
pub use l1_body_impl::nt_shield_sandbox;
#[cfg(feature = "sandbox")]
pub use l1_body_impl::nt_shield_sandbox_entry;
pub use l1_body_impl::nt_shield_sentry;
#[cfg(feature = "stealth-net")]
pub use l1_body_impl::nt_shield_stealth_net;
pub use l1_body_impl::nt_shield_traffic;

pub use l1_body_impl::nt_shield_agentic_scan;

pub use l1_body_impl::nt_act_autonomy;
pub use l1_body_impl::nt_act_autonomy::nt_mind_automation::AutomationAction;
pub use l1_body_impl::nt_act_autonomy::nt_mind_automation::AutomationEngine;
pub use l1_body_impl::nt_act_autonomy::nt_mind_automation::AutomationRule;
pub use l1_body_impl::nt_act_autonomy::nt_mind_automation::AutomationTrigger;
pub use l1_body_impl::nt_act_code;
pub use l1_body_impl::nt_act_crypto;
pub use l1_body_impl::nt_act_goal;
pub use l1_body_impl::nt_act_orchestrator;
pub use l1_body_impl::nt_act_sandbox;
pub use l1_body_impl::nt_act_voice;
pub use l1_body_impl::nt_agent_agent_team;

pub use l1_body_impl::nt_agent_mcp_discovery;
pub use l1_body_impl::nt_agent_protocol;

pub use l1_body_impl::nt_agent_mcp_adapter;
pub use l1_body_impl::nt_agent_mcp_auth;
pub use l1_body_impl::nt_agent_mcp_registry;
pub use l1_body_impl::nt_agent_mcp_tools;
pub use l1_body_impl::nt_agent_mcp_transport;
pub use l1_body_impl::nt_agent_orchestrator;

// L3 — Memory (cont.)

// L0 — Substrate (基底层) — 硬件
// (no neotrix/ modules yet)

// Shanhai Geography — 山海世界双坐标系统
pub mod nt_shanhai_geo;

// Infrastructure — 核心基础设施
pub mod nt_core_error;
pub mod nt_core_event_bus;

// Re-exports (unchanged — absolute paths work regardless of migration)
pub use crate::core::nt_core_answer_engine::{
    AnswerEngine, AnswerEngineConfig, AnswerMode, AnswerResult, AnswerSegment, ContextSource,
    PreparedQuery, SearchResult, SourceType, WidgetKind, WidgetProvider,
};
pub use l1_body_impl::nt_io_mention::{resolve_mentions, MentionResult};
pub use l1_body_impl::nt_io_notify::{
    notify, notify_approval_needed, notify_task_complete, notify_with_level, Level,
};
pub use l1_body_impl::nt_shield_audit::{
    AuditDimension, AuditMode, AuditReport, CheckResult, CheckStatus, SecurityAuditor, Severity,
    VulnDomain, VulnerabilityCheck,
};
pub use l2_world_impl::nt_world_scrape::{
    AntiDetect, BrowserScraper, RequestScraper, ScrapeResult, ScraperConfig,
};
pub use l8_autonomic_impl::nt_mind::export_import::ReasoningBankExporter;
pub use l9_transcendent_impl::nt_mind_consciousness_gold_standard::{
    ConsciousnessGoldStandard, ConsciousnessLevel, DetectionTrend, E8HexagramState,
    GoldStandardReport,
};

mod nt_file_ability;
pub mod proxy_daemon_wrapper;

// ─── Unified File Ability (统一文件能力) — 文件读写编辑统一入口 ────────────────
// office_oxide (Office 6 格式) + neotrix-types FileParser (通用文本/PDF/图像/音频/视频)
// + image crate (图像元数据)。复用 ConstellationLevel 成熟度 + SelfTest T1-T3 接线。
// Dark Forest 合规: 模块需有消费者方可存活，每次编辑后需 register_consumer
pub use nt_file_ability::{
    check_health, content_similarity, create_from_markdown, embed_text, extract_text,
    replace_placeholder, route_attention, save_edited, specialist_index, to_markdown,
    ContentSnapshot, FileAbility, FileAbilityError, FileAbilitySelfTest, FileKind, FileOperation,
    ImageMetadata, OcrEngine, OcrResult, RuleBasedOcr, SheetCellData, SheetCellValueType,
    SheetData, SheetRowData,
};

// ─── L7 Capability Tree 採入 (经验 → 能力节点迭代目标) ─────────────────────
// 能力树 crate 作为 workspace 成员, 此处接入主管线 (Dark Forest 合规: 有消费者)
pub use nt_core_capability_tree::{
    CapabilityNode, CapabilityRegistry, ConstellationLevel, Domain as CapabilityDomain,
    EvolutionAction, EvolutionEngine, EvolutionOp, EvolutionPlan, NodeLayer, RuneSocket,
};
