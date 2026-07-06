//! # NeoTrix 核心模块
//!
//! 9-Layer Architecture:
//!   L9 Transcendent — L8 Autonomic — L7 Capability — L6 Self —
//!   L5 Consciousness — L4 Cognition — L3 Memory — L2 Perception —
//!   L1 Body — L0 Substrate
//!
//! 统一版本: 0.18.0

// ─── Implementation layer directories ─────────────────────────────────────
pub mod l9_transcendent_impl;
pub mod l8_autonomic_impl;
pub mod l7_capability_impl;
pub mod l6_self_impl;
pub mod l5_consciousness_impl;
pub mod l4_cognition_impl;
pub mod l3_memory_impl;
pub mod l2_world_impl;
pub mod l1_body_impl;

// ─── Migrated module re-exports (backward compat with crate::neotrix::nt_*) ─

// L9 — Transcendent (超验层) — 观察自身
pub use l9_transcendent_impl::nt_core_knowledge_gap;
pub use l9_transcendent_impl::nt_mind_consciousness_gold_standard;
pub use l9_transcendent_impl::nt_mind_consciousness_monitor;

// L8 — Autonomic (自主神经层) — 自我进化
pub use l8_autonomic_impl::nt_mind;
pub use l8_autonomic_impl::nt_mind_background_loop;
pub use l8_autonomic_impl::nt_mind_awakening;
pub use l8_autonomic_impl::nt_mind_autofixer;
pub use l8_autonomic_impl::nt_mind_evolution_loop;
pub use l8_autonomic_impl::nt_mind_evolution_daemon;
pub use l8_autonomic_impl::nt_mind_ingestion;
pub use l8_autonomic_impl::nt_mind_self_diagnose;
pub use l8_autonomic_impl::nt_mind_benchmark;
pub use l8_autonomic_impl::nt_mind_background_config;
pub use l8_autonomic_impl::nt_mind_cleanup;
pub use l8_autonomic_impl::nt_mind_distiller;
pub use l8_autonomic_impl::nt_mind_scheduler;
pub use l8_autonomic_impl::nt_mind_topic_aggregator;
pub use l8_autonomic_impl::nt_mind_evolve;
pub use l8_autonomic_impl::nt_mind_knowledge_pipeline;
pub use l8_autonomic_impl::nt_mind_hook;
pub use l8_autonomic_impl::nt_mind_skill_engine;
pub use l8_autonomic_impl::nt_mind_memory;

// L7 — Capability (能力层) — 统一路由
pub use l7_capability_impl::SkillDoc;
pub use l7_capability_impl::SkillMetrics;
pub use l7_capability_impl::SkillBank;
pub use l7_capability_impl::SkillExtractor;
pub use l7_capability_impl::SkillOptimizer;
pub use l7_capability_impl::ValidationGate;
pub use l7_capability_impl::BoundedEdit;
pub use l7_capability_impl::GroupCoordinator;
pub use l7_capability_impl::SkillRegistry;

// L6 — Self (自我层) — 身份
pub use l6_self_impl::nt_core_intra_reflection;

// L5 — Consciousness (意识层) — 体验
pub use l5_consciousness_impl::nt_core_iit_phi;
pub use l5_consciousness_impl::nt_core_fep_iit;
pub use l5_consciousness_impl::nt_core_signal;

// L4 — Cognition (认知层) — 推理
pub use l4_cognition_impl::nt_core_kernel;
pub use l4_cognition_impl::nt_core_code_query;
pub use l4_cognition_impl::nt_core_parallel;

// L3 — Memory (记忆层) — 记忆
pub use l3_memory_impl::nt_memory_kb;
pub use l3_memory_impl::nt_memory_knowledge_populator;
pub use l3_memory_impl::nt_memory_spatial;
pub use l3_memory_impl::nt_memory_historian;

// L2 — Perception (感知层) — 感知
pub use l2_world_impl::nt_world_map;
pub use l2_world_impl::nt_world_model;
pub use l2_world_impl::nt_world_model_v2;
pub use l2_world_impl::nt_world_jepa;
pub use l2_world_impl::nt_world_e8;
pub use l2_world_impl::nt_world_pred_hcube;
pub use l2_world_impl::nt_world_infer;
pub use l2_world_impl::nt_world_browse;
pub use l2_world_impl::nt_world_browse_auto;
pub use l2_world_impl::nt_world_scrape;
pub use l2_world_impl::nt_world_pred;
pub use l2_world_impl::nt_world_sense;
pub use l2_world_impl::nt_world_crawl;
pub use l2_world_impl::nt_world_search;
pub use l2_world_impl::nt_world_pet;
pub use l2_world_impl::nt_world_vision;
pub use l2_world_impl::nt_world_parse;
pub use l2_world_impl::nt_world_journal_index;
pub use l2_world_impl::nt_world_code_search;

// L1 — Body (身体层) — 行动、界面、安全
pub use l1_body_impl::nt_io_logging;
pub use l1_body_impl::nt_io_http_factory;
pub use l1_body_impl::nt_io_mention;
pub use l1_body_impl::nt_io_push_channel;
pub use l1_body_impl::nt_io_standalone;
#[cfg(feature = "telemetry")]
pub use l1_body_impl::nt_io_telemetry;
pub use l1_body_impl::nt_io_user_avatar;
pub use l1_body_impl::nt_io_avatar_channel;
pub use l1_body_impl::nt_io_lsp;
pub use l1_body_impl::nt_io_hotreload;
pub use l1_body_impl::nt_io_notify;
pub use l1_body_impl::nt_io_server;
pub use l1_body_impl::nt_io_remote;
pub use l1_body_impl::nt_io_web;
pub use l1_body_impl::nt_io_proxy;
pub use l1_body_impl::nt_io_proxy_server;
pub use l1_body_impl::nt_io_plugin;
pub use l1_body_impl::nt_io_provider;
pub use l1_body_impl::nt_io_map_tile;
pub use l1_body_impl::nt_io_acp;
pub use l1_body_impl::nt_io_session_recovery;
pub use l1_body_impl::nt_io_agents_md;

pub use l1_body_impl::nt_shield;
pub use l1_body_impl::nt_shield_audit;
pub use l1_body_impl::nt_shield_sentry;
#[cfg(feature = "sandbox")]
pub use l1_body_impl::nt_shield_sandbox_entry;
pub use l1_body_impl::nt_shield_prompt;
pub use l1_body_impl::nt_shield_sandbox;
pub use l1_body_impl::nt_shield_manager;
#[cfg(feature = "stealth-net")]
pub use l1_body_impl::nt_shield_stealth_net;

pub use l1_body_impl::nt_act_code;
pub use l1_body_impl::nt_act_goal;
pub use l1_body_impl::nt_act_gram;
pub use l1_body_impl::nt_act_spear;
pub use l1_body_impl::nt_act_autonomy;
pub use l1_body_impl::nt_act_autonomy::nt_mind_automation::AutomationEngine;
pub use l1_body_impl::nt_act_autonomy::nt_mind_automation::AutomationRule;
pub use l1_body_impl::nt_act_autonomy::nt_mind_automation::AutomationTrigger;
pub use l1_body_impl::nt_act_autonomy::nt_mind_automation::AutomationAction;
pub use l1_body_impl::nt_act_voice;
pub use l1_body_impl::nt_act_crypto;
pub use l1_body_impl::nt_act_earn;
pub use l1_body_impl::nt_act_social;
pub use l1_body_impl::nt_act_sync;
pub use l1_body_impl::nt_act_worktree;
pub use l1_body_impl::nt_act_sub_agent_middleware;
pub use l1_body_impl::nt_act_orchestrator;
pub use l1_body_impl::nt_act_project_manager;
pub use l1_body_impl::nt_act_remote_control;

pub use l1_body_impl::nt_agent_protocol;
pub use l1_body_impl::nt_agent_subagent;
pub use l1_body_impl::nt_agent_mcp_discovery;

pub use l1_body_impl::nt_agent_mcp_adapter;
pub use l1_body_impl::nt_agent_mcp_auth;
pub use l1_body_impl::nt_agent_mcp_tools;
pub use l1_body_impl::nt_agent_mcp_transport;
pub use l1_body_impl::nt_tools;
pub use l1_body_impl::nt_agent;
pub use l1_body_impl::nt_agent_orchestrator;
pub use l1_body_impl::nt_agent_mod;

// L3 — Memory (cont.)
pub use l3_memory_impl::nt_memory_negentropy;

// L0 — Substrate (基底层) — 硬件
// (no neotrix/ modules yet)

// Shanhai Geography — 山海世界双坐标系统
pub mod nt_shanhai_geo;

// Infrastructure — 核心基础设施
pub mod nt_core_error;
pub mod nt_core_event_bus;

// Re-exports (unchanged — absolute paths work regardless of migration)
pub use l1_body_impl::nt_io_notify::{notify, notify_with_level, notify_task_complete, notify_approval_needed, Level};
pub use l1_body_impl::nt_io_mention::{resolve_mentions, MentionResult};
pub use l9_transcendent_impl::nt_mind_consciousness_gold_standard::{ConsciousnessGoldStandard, GoldStandardReport, ConsciousnessLevel, DetectionTrend, E8HexagramState};
pub use l8_autonomic_impl::nt_mind::export_import::ReasoningBankExporter;
pub use l1_body_impl::nt_io_server::{NeoTrixACPServer, ServerInfo};
pub use l1_body_impl::nt_io_acp::AcpAgent;
pub use crate::neotrix::nt_core_code_query::CodeQueryEngine;
pub use l2_world_impl::nt_world_scrape::{ScraperConfig, ScrapeResult, BrowserScraper, RequestScraper, AntiDetect};
pub use l1_body_impl::nt_shield_audit::{
    SecurityAuditor, AuditReport, AuditMode, AuditDimension, VulnDomain, Severity,
    VulnerabilityCheck, CheckResult, CheckStatus,
};

pub mod proxy_daemon_wrapper;



