//! # NeoTrix 核心模块
//!
//! 6-Layer Architecture:
//!   L6 Meta-Cognition — L5 Cognition — L4 Emotion —
//!   L3 Embodiment — L2 Perception — L1 Action
//!
//! 统一版本: 0.21.0

// ─── Six-Layer Architecture Modules ──────────────────────────────────────
pub use crate::l1_action::{nt_act, nt_io, nt_memory};
pub use crate::l2_perception::{nt_world, nt_sense};
pub use crate::l3_embodiment::{nt_shield, nt_feel, nt_physical};
pub use crate::l4_emotion;
pub use crate::l5_cognition::{nt_core, nt_mind};
pub use crate::l6_meta::{nt_meta, nt_repair, nt_nexus};

// ─── 经验 → 能力树迭代目标桥 ──────────────────────────────────────────
pub mod nt_capability_bridge;

// ─── iOS FFI bridge ─────────────────────────────────────────────────────
#[cfg(feature = "ios-bridge")]
pub mod ffi;

// ─── Infrastructure ─────────────────────────────────────────────────────
pub mod nt_core_error;
pub mod nt_core_event_bus;

// ─── Standalone modules at neotrix level ────────────────────────────────
pub mod nt_shanhai_geo;
pub mod nt_harness;
pub mod nt_file_ability;
pub mod proxy_daemon_wrapper;

// ─── L7 Capability Tree ────────────────────────────────────────────────
pub use nt_core_capability_tree::{
    CapabilityNode, CapabilityRegistry, ConstellationLevel, Domain as CapabilityDomain,
    EvolutionAction, EvolutionEngine, EvolutionOp, EvolutionPlan, NodeLayer, RuneSocket,
};

// ─── Re-exports from new architecture ───────────────────────────────────

// L1 Action — nt_io re-exports
pub use crate::l1_action::nt_io::nt_io_standalone::{
    format_kernel_output, text_to_vector, verify_answer, KernelStats, ReasoningKernel,
    ReasoningMethod, ReasoningOutput, SelfConsistencyResult, StageInfo, Vector, EVOLUTION,
    KERNEL_DIM,
};
pub use crate::l1_action::nt_io::nt_io_provider::types::{
    LlmError, LlmProvider, LlmRequest, LlmResponse, Message, Role,
};
pub use crate::l1_action::nt_io::{
    nt_io_agents_md, nt_io_avatar_channel, nt_io_digital_human, nt_io_hotreload,
    nt_io_http_factory, nt_io_logging, nt_io_mention, nt_io_neocodex, nt_io_notify, nt_io_plugin,
    nt_io_provider, nt_io_proxy_server, nt_io_session_recovery, nt_io_user_avatar, nt_io_web,
};
#[cfg(feature = "telemetry")]
pub use crate::l1_action::nt_io::nt_io_telemetry;

// L1 Action — nt_act re-exports
pub use crate::l1_action::nt_act::{
    nt_act_autonomy, nt_act_code, nt_act_crypto, nt_act_goal, nt_act_orchestrator, nt_act_sandbox,
    nt_act_voice,
};
pub use crate::l1_action::nt_act::nt_act_autonomy::nt_mind_automation::{
    AutomationAction, AutomationEngine, AutomationRule, AutomationTrigger,
};

// L1 Action — nt_memory re-exports
pub use crate::l1_action::nt_memory::{
    nt_memory_historian, nt_memory_kb, nt_memory_leann_store, nt_memory_spatial,
};

// L2 Perception — nt_world re-exports
pub use crate::l2_perception::nt_world::{
    nt_world_code_search, nt_world_crawl, nt_world_e8,
    nt_world_infer, nt_world_scrape,
    nt_world_search, nt_world_sense,
    nt_world_model, nt_world_jepa, nt_world_model_v2,
};

// L3 Embodiment — nt_shield re-exports
pub use crate::l3_embodiment::nt_shield::{
    nt_shield_audit, nt_shield_comm, nt_shield_sandbox,
    nt_shield_sentry, nt_shield_traffic, nt_shield_agentic_scan,
};
#[cfg(feature = "sandbox")]
pub use crate::l3_embodiment::nt_shield::nt_shield_sandbox_entry;
#[cfg(feature = "stealth-net")]
pub use crate::l3_embodiment::nt_shield::nt_shield_stealth_net;

// L5 Cognition — nt_mind re-exports
pub use crate::l5_cognition::nt_mind::{
    nt_mind_background_loop,
    nt_mind_benchmark,
    nt_mind_hook, nt_mind_knowledge_pipeline,
    nt_mind_skill_engine,
};
pub use crate::l5_cognition::nt_mind::evolution;
pub use crate::l5_cognition::nt_mind::foundation;

// L6 Meta — re-exports
pub use crate::l6_meta::nt_meta::nt_core_intra_reflection;
pub use crate::l6_meta::nt_repair::{
    nt_mind_consciousness_gold_standard, nt_mind_consciousness_monitor,
};

// ─── Specific type re-exports ───────────────────────────────────────────

pub use crate::core::nt_core_reasoning::{
    default_context_builder, default_method_registry, MethodRegistry, MethodSpec, ReasoningStep,
    ReasoningTrace, TraceSource,
};

pub use crate::core::nt_core_answer_engine::{
    AnswerEngine, AnswerEngineConfig, AnswerMode, AnswerResult, AnswerSegment, ContextSource,
    PreparedQuery, SearchResult, SourceType, WidgetKind, WidgetProvider,
};

pub use crate::l1_action::nt_io::nt_io_mention::{resolve_mentions, MentionResult};
pub use crate::l1_action::nt_io::nt_io_notify::{
    notify, notify_approval_needed, notify_task_complete, notify_with_level, Level,
};
pub use crate::l3_embodiment::nt_shield::nt_shield_audit::{
    AuditDimension, AuditMode, AuditReport, CheckResult, CheckStatus, SecurityAuditor, Severity,
    VulnDomain, VulnerabilityCheck,
};
pub use crate::l2_perception::nt_world::nt_world_scrape::{
    AntiDetect, BrowserScraper, RequestScraper, ScrapeResult, ScraperConfig,
};
pub use crate::l6_meta::nt_repair::nt_mind_consciousness_gold_standard::{
    ConsciousnessGoldStandard, ConsciousnessLevel, DetectionTrend, E8HexagramState,
    GoldStandardReport,
};

// ─── Unified File Ability ───────────────────────────────────────────────
pub use nt_file_ability::{
    check_health, consolidate_tables, consolidate_tables_first_sheet,
    consolidate_tables_with_mode, content_similarity,
    create_from_markdown, decode_bytes,
    detect_encoding, edit_pdf, edit_xlsx_table, embed_text, extract_text, extract_pdf_tables,
    extract_dir, merge_pdfs,
    load_snapshot,
    merge_tables_with, merge_tables_with_mode,
    normalize_column_name, read_csv, read_structured, read_xlsx_sheets_all, read_xlsx_table,
    FileModel,
    replace_placeholder, route_attention, save_edited, specialist_index, store_snapshot,
    suggest_schema, to_markdown, write_csv, write_json, write_xlsx_table, ConsolidationReport,
    ContentSnapshot, FileAbility, FileAbilityError, FileAbilitySelfTest, FileKind, FileOperation,
    ImageMetadata, MergeSchema, OcrEngine, OcrResult, PRICE_STANDARD_COLUMNS, PRICE_TABLE_SCHEMA,
    RuleBasedOcr, SchemaSuggestion, SchemaStore, MergeSchemaJson, SheetCellData,
    SheetCellValueType, SheetData, SheetMode,
    SheetRowData,
    StructuredData, TableData, TableEdit, TextEncoding, UnitRule, PdfEdit,
    DirExtractEntry, DirExtractReport,
    CollectionMergeRequest, MergeOutcome, MergeStrategy, collection_merge,
};

pub use nt_file_ability::merge_docx;

#[cfg(test)]
pub(crate) use nt_file_ability::{make_min_docx, make_min_pptx};
