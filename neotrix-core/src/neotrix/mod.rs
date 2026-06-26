//! # NeoTrix 核心模块
//!
//! 统一版本: 0.18.0

// NT-CORE domain — re-exported from core/ for backward compat
pub mod nt_core_error;
pub use crate::core::nt_core_iit_phi;
pub mod nt_core_kernel;
pub use crate::core::nt_core_fep_iit;
pub mod nt_core_parallel;
pub mod nt_core_signal;
// pub mod nt_core_knowledge_gap; // 🗑️ v6: delete — orphan, 0 consumers
pub mod nt_core_negentropy;

// NT-MIND domain
pub mod nt_mind;
pub mod nt_mind_autofixer; // was DEAD — re-enabled, still referenced by evolution_loop, self_diagnose, evolution_daemon
pub mod nt_mind_autonomy;
pub mod nt_mind_awakening;
pub mod nt_mind_background_config; // was DEAD — re-enabled, still referenced by background_loop
pub mod nt_mind_background_loop;
pub mod nt_mind_benchmark; // was DEAD — re-enabled, referenced by cli_utils benchmark command
pub mod nt_mind_cleanup; // was DEAD — re-enabled, still referenced by background_loop
pub mod nt_mind_consciousness_gold_standard;
pub mod nt_mind_consciousness_monitor; // was DEAD — re-enabled, still referenced by background_loop
pub mod nt_mind_distiller;
pub mod nt_mind_evolution_daemon; // was DEAD — re-enabled, still referenced by background_loop
pub mod nt_mind_evolution_loop;
pub mod nt_mind_goal;
pub mod nt_mind_ingestion;
pub mod nt_mind_self_diagnose; // was DEAD — re-enabled, still referenced by background_loop

// NT-MEMORY domain
pub mod nt_memory_kb;
pub mod nt_memory_knowledge_populator;
pub mod nt_memory_session;
pub mod nt_memory_storage;
pub mod nt_memory_vector_store;
pub mod nt_memory_wal;
pub mod nt_memory_ws; // was DEAD — re-enabled, still referenced by background_loop

// NT-IO domain
// pub mod nt_io_logging; // 🗑️ v6: delete — orphan, 0 consumers
pub mod nt_io_http_factory;
pub mod nt_io_mention;
pub mod nt_io_standalone;
#[cfg(feature = "telemetry")]
pub mod nt_io_telemetry;
// pub mod nt_io_avatar; // 🗑️ v6: delete — orphan, 0 consumers
pub mod nt_io_lsp;
// pub mod nt_io_hotreload; // 🗑️ v6: delete — orphan, 0 consumers
// pub mod nt_io_remote; // DEAD — orphan, 0 consumers
// pub mod nt_io_web; // 🗑️ v6: delete — orphan, 0 consumers
// pub mod nt_io_proxy; // DEAD — orphan, 0 consumers
// pub mod nt_io_proxy_server; // 🗑️ v6: delete — orphan, 0 consumers
pub mod nt_io_conn;
pub mod nt_io_design_token;
pub mod nt_io_gram;
pub mod nt_io_llm;
pub mod nt_io_llm_provider;
pub mod nt_io_llm_provider_registry;
pub mod nt_io_llm_router;
pub mod nt_io_network;
pub mod nt_io_output;
pub mod nt_io_plugin;
pub mod nt_io_provider;
pub mod nt_io_router;
pub mod nt_io_shutdown;
#[cfg(feature = "stealth-net")]
pub mod nt_io_stealth_net; // moved from nt_shield_stealth_net
pub mod nt_io_tokenopt; // moved from nt_act_gram

// NT-SHIELD domain
pub mod nt_shield; // canonical: nt_shield::inner_critic (CSS/UI design auditor)
pub mod nt_shield_audit;
pub mod nt_shield_prompt;
pub mod nt_shield_sandbox;
#[cfg(feature = "sandbox")]
pub mod nt_shield_sandbox_entry;
#[cfg(feature = "telemetry")]
pub mod nt_shield_sentry;
#[cfg(feature = "stealth-net")]
pub use crate::neotrix::nt_io_stealth_net as nt_shield_stealth_net;
pub mod nt_shield_protect;

// NT-ACT domain
pub mod nt_act_code;
pub use crate::neotrix::nt_io_gram as nt_act_gram;
pub use crate::neotrix::nt_mind_goal as nt_act_goal;
// pub mod nt_act_spear; // 🗑️ v6: delete — orphan, 0 consumers
pub use crate::neotrix::nt_mind_autonomy as nt_act_autonomy;
pub mod nt_act_crypto;
pub mod nt_act_earn;
pub mod nt_act_trading;
pub mod nt_act_voice;
pub mod nt_world_social;
pub use crate::neotrix::nt_world_social as nt_act_social;
pub mod nt_act_orchestrator;
pub mod nt_act_project_manager;
pub mod nt_act_sync; // was DEAD — re-enabled, still referenced by background_loop // was DEAD — re-enabled, still referenced by background_loop
                                                                                   // pub mod nt_act_remote_control; // 🗑️ v6: delete — orphan, 0 consumers

// NT-AGENT domain — consolidated: nt_act_mcp replaces nt_agent_mcp_tools + nt_agent_mcp_discovery
pub mod nt_act_mcp;
pub mod nt_agent_core;
pub mod nt_agent_protocol;
pub mod nt_tools;
// pub mod nt_agent_orchestrator; // DEAD — orphan, 0 consumers
pub mod nt_agent_arch;
pub mod nt_agent_hive;
pub mod nt_agent_mod;
pub mod nt_agent_plugin;
// NT-INFRA domain
pub(crate) mod nt_infra;

// NT-WORLD domain
pub mod nt_expert_routing;
pub mod nt_world_model_v2;
pub mod nt_world_search;
pub mod nt_world_translate;
pub mod nt_world_vision;
pub use crate::core::nt_core_e8_model as nt_world_e8; // moved to core/
pub use crate::core::nt_core_prediction as nt_world_jepa; // moved to core/
pub use crate::core::nt_core_prediction::nt_world_pred_hcube; // moved to core/
pub mod nt_world_browse;
pub mod nt_world_infer;
pub mod nt_world_scrape;
pub use crate::core::nt_core_prediction::rssm as nt_world_pred; // moved to core/
pub mod nt_world_code_search;
pub mod nt_world_crawl;
pub mod nt_world_document;
pub mod nt_world_journal_index;
pub mod nt_world_sense;
// pub mod nt_world_pet; // DEAD — orphan, 0 consumers
pub mod nt_world_exploration;

// Re-exports
pub use nt_io_mention::{resolve_mentions, MentionResult};
pub use nt_mind::export_import::ReasoningBankExporter;
pub use nt_mind_consciousness_gold_standard::{
    ConsciousnessGoldStandard, ConsciousnessLevel, DetectionTrend, E8HexagramState,
    GoldStandardReport,
};
pub use nt_shield_audit::{
    AuditMode, AuditReport, CheckResult, CheckStatus, SecurityAuditor, ShieldSeverity, VulnDomain,
    VulnerabilityCheck,
};
pub use nt_world_scrape::{
    AntiDetect, BrowserScraper, RequestScraper, ScrapeResult, ScraperConfig,
};
