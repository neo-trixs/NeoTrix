//! # NeoTrix 核心模块
//!
//! 统一版本: 0.18.0

// NT-CORE domain
pub mod nt_core_error;
pub mod nt_core_event_bus;
pub mod nt_core_iit_phi;
pub mod nt_core_kernel;
pub mod nt_core_code_query;
pub mod nt_core_fep_iit;
pub mod nt_core_intra_reflection;
pub mod nt_core_parallel;
pub mod nt_core_signal;
pub mod nt_core_knowledge_gap;
pub mod nt_core_negentropy;

// NT-MIND domain
pub mod nt_mind;
pub mod nt_mind_background_loop;
pub mod nt_mind_autofixer;
pub mod nt_mind_evolution_loop;
pub mod nt_mind_evolution_daemon;
pub mod nt_mind_self_diagnose;
pub mod nt_mind_benchmark;
pub mod nt_mind_background_config;
pub mod nt_mind_cleanup;
pub mod nt_mind_distiller;
pub mod nt_mind_scheduler;
pub mod nt_mind_topic_aggregator;
pub mod nt_mind_consciousness_gold_standard;
pub mod nt_mind_consciousness_monitor;
pub mod nt_mind_ingestion;

// NT-MEMORY domain
pub mod nt_memory_kb;
pub mod nt_memory_knowledge_populator;

// NT-IO domain
pub mod nt_io_logging;
pub mod nt_io_http_factory;
pub mod nt_io_mention;
pub mod nt_io_neotrix_interface;
pub mod nt_io_standalone;
#[cfg(feature = "telemetry")]
pub mod nt_io_telemetry;
pub mod nt_io_avatar;
pub mod nt_io_lsp;
pub mod nt_io_hotreload;
pub mod nt_io_server;
pub mod nt_io_remote;
pub mod nt_io_web;
pub mod nt_io_proxy;
pub mod nt_io_proxy_server;
pub mod nt_io_plugin;
pub mod nt_io_provider;

// NT-SHIELD domain
pub mod nt_shield;
pub mod nt_shield_audit;
pub mod nt_shield_sentry;
#[cfg(feature = "sandbox")]
pub mod nt_shield_sandbox_entry;
pub mod nt_shield_prompt;
pub mod nt_shield_sandbox;
pub mod nt_shield_manager;
#[cfg(feature = "stealth-net")]
pub mod nt_shield_stealth_net;

// NT-ACT domain
pub mod nt_act_code;
pub mod nt_act_goal;
pub mod nt_act_gram;
pub mod nt_act_spear;
pub mod nt_act_autonomy;
pub mod nt_act_voice;
pub mod nt_act_crypto;
pub mod nt_act_earn;
pub mod nt_act_social;
pub mod nt_act_sync;
pub mod nt_act_orchestrator;
pub mod nt_act_project_manager;
pub mod nt_act_remote_control;

// NT-AGENT domain — consolidated: nt_act_mcp replaces nt_agent_mcp_tools + nt_agent_mcp_discovery
pub mod nt_agent_protocol;
pub mod nt_act_mcp;
pub mod nt_tools;
pub mod nt_agent_orchestrator;
pub mod nt_agent_mod;
pub mod nt_mind_awakening;
// NT-WORLD domain
pub mod nt_world_model;
pub mod nt_world_model_v2;
pub mod nt_world_jepa;
pub mod nt_world_e8;
pub mod nt_world_pred_hcube;
pub mod nt_world_infer;
pub mod nt_world_browse;
pub mod nt_world_browse_auto;
pub mod nt_world_scrape;
pub mod nt_world_pred;
pub mod nt_world_sense;
pub mod nt_world_crawl;
pub mod nt_world_search;
pub mod nt_world_code_search;
pub mod nt_world_journal_index;
pub mod nt_world_vision;
pub mod nt_world_pet;

// Re-exports
pub use nt_io_mention::{resolve_mentions, MentionResult};
pub use nt_mind_consciousness_gold_standard::{ConsciousnessGoldStandard, GoldStandardReport, ConsciousnessLevel, DetectionTrend, E8HexagramState};
pub use nt_mind::export_import::ReasoningBankExporter;
pub use nt_io_server::{NeoTrixACPServer, ServerInfo};
pub use nt_core_code_query::CodeQueryEngine;
pub use nt_world_scrape::{ScraperConfig, ScrapeResult, BrowserScraper, RequestScraper, AntiDetect};
pub use nt_shield_audit::{
    SecurityAuditor, AuditReport, AuditMode, VulnDomain, Severity,
    VulnerabilityCheck, CheckResult, CheckStatus,
};

