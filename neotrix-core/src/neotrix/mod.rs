//! # NeoTrix 核心模块
//!
//! 统一版本: 0.18.0 — 推理内核进化到 18 stages
//!
//! 所有代码都是 neotrix，不区分 V1/V2/V3。
//! 旧代码保持编译即可，新代码直接加在对应位置。
//! 需要重构就重构，需要融合就融合。

pub mod error;
pub mod logging;
pub mod nt_shield_audit;
pub mod parallel;
pub mod provider;
pub mod signal;
pub mod nt_world_model;
pub mod nt_world_model_v2;
pub mod nt_world_jepa;
pub mod nt_world_e8;
pub mod nt_world_pred_hcube;
pub mod nt_world_infer;
pub mod iit_phi;
pub mod fep_iit_bridge;
pub mod autofixer;
pub mod evolution_loop;
pub mod evolution_daemon;
pub mod nt_act_code;
pub mod self_diagnose;
pub mod nt_act_goal;
pub mod nt_mind;
pub mod nt_io_notify;
pub mod mention;
pub mod kernel_core;
pub use nt_io_notify::{notify, notify_with_level, notify_task_complete, notify_approval_needed, Level};
pub use mention::{resolve_mentions, MentionResult};
pub mod standalone;
pub mod benchmark;
pub mod nt_world_browse;
pub mod nt_io_server;
pub mod agent_protocol;
pub mod nt_io_remote;
pub mod code_query;
pub mod lsp;
pub mod orchestrator;
#[cfg(feature = "stealth-net")]
pub mod stealth_net;
pub mod nt_world_browse_auto;
pub mod background_config;
pub mod background_loop;
pub mod hotreload;
pub mod intra_reflection;
#[cfg(feature = "sandbox")]
pub mod sandbox;
pub mod nt_world_scrape;
pub mod nt_shield_prompt;
pub mod subagent;
pub mod cleanup;
pub mod nt_shield;
pub mod nt_act_gram;
pub mod nt_act_spear;
pub mod nt_world_pred;

pub mod nt_memory_kb;
pub mod nt_io_web;
pub mod http_factory;
pub mod project_manager;
pub mod mcp_tools;
pub mod mcp_discovery;
pub mod plugin;
pub mod nt_act_autonomy;
pub mod consciousness_gold_standard;
pub mod consciousness_monitor;
pub mod distiller;
pub mod user_avatar;
pub mod avatar_channel;
pub mod knowledge_populator;
pub mod nt_io_proxy;
pub mod nt_io_proxy_server;
pub mod nt_shield_sandbox;
pub mod nt_shield_manager;
pub mod event_bus;
pub mod nt_world_sense;
pub mod nt_act_voice;
pub mod nt_world_crawl;
pub mod nt_act_crypto;
pub mod nt_act_earn;
pub mod nt_act_social;
pub mod nt_world_search;
pub mod nt_act_sync;
pub mod sentry;
#[cfg(feature = "telemetry")]
pub mod telemetry;
pub mod knowledge_gap_detector;

pub use consciousness_gold_standard::{ConsciousnessGoldStandard, GoldStandardReport, ConsciousnessLevel, DetectionTrend, E8HexagramState};
pub use nt_mind::export_import::ReasoningBankExporter;
pub use nt_io_server::{NeoTrixACPServer, ServerInfo};
pub use code_query::CodeQueryEngine;
pub use nt_world_scrape::{ScraperConfig, ScrapeResult, BrowserScraper, RequestScraper, AntiDetect};
pub use nt_shield_audit::{
    SecurityAuditor, AuditReport, AuditMode, VulnDomain, Severity,
    VulnerabilityCheck, CheckResult, CheckStatus,
};
// pub mod neotrix_interface; // removed — dead empty trait stub
pub mod neotrix_interface;
