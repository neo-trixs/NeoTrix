//! L1 Body Implementation Layer
//!
//! IO interfaces, Shield security, Act tools, MCP transport.

pub mod nt_l1_error;
pub mod nt_l1_shared_types;

// IO
pub mod nt_io_avatar_channel;
pub mod nt_io_hotreload;
pub mod nt_io_http_factory;
pub mod nt_io_logging;
pub mod nt_io_lsp;
pub mod nt_io_mention;
pub mod nt_io_notify;
pub mod nt_io_plugin;
pub mod nt_io_provider;
pub mod nt_io_proxy;
pub mod nt_io_proxy_server;
pub mod nt_io_push_channel;
pub mod nt_io_remote;
pub mod nt_io_acp;
pub mod nt_io_server;
pub mod nt_io_standalone;
#[cfg(feature = "telemetry")]
pub mod nt_io_telemetry;
pub mod nt_io_user_avatar;
pub mod nt_io_web;
pub mod nt_io_map_tile;
pub mod nt_io_session_recovery;
pub mod nt_io_agents_md;
pub mod nt_io_digital_human;
pub mod nt_io_neocodex;

// Shield
pub mod nt_shield;
pub mod nt_shield_audit;
pub mod nt_shield_manager;
pub mod nt_shield_prompt;
pub mod nt_shield_sandbox;
#[cfg(feature = "sandbox")]
pub mod nt_shield_sandbox_entry;
pub mod nt_shield_sentry;
#[cfg(feature = "stealth-net")]
pub mod nt_shield_stealth_net;
pub mod nt_shield_traffic;
pub mod nt_shield_redteam;
pub mod nt_shield_agentic_scan;
pub mod nt_shield_comm;

// Act
pub mod nt_act_autonomy;
pub mod nt_act_code;
pub mod nt_act_crypto;
pub mod nt_act_earn;
pub mod nt_act_goal;
pub mod nt_act_gram;
pub mod nt_act_orchestrator;
pub mod nt_act_project_manager;
pub mod nt_act_remote_control;
pub mod nt_act_sandbox;
pub mod nt_act_social;
pub mod nt_act_spear;
pub mod nt_act_sub_agent_middleware;
pub mod nt_act_sync;
pub mod nt_act_voice;
pub mod nt_act_goose_bridge;
pub mod nt_agent_agent_team;

// Misc
pub mod nt_tools;

// Agent
pub mod nt_agent_mcp_adapter;
pub mod nt_agent_mcp_auth;
pub mod nt_agent_mcp_discovery;
pub mod nt_agent_mcp_tools;
pub mod nt_agent_mcp_transport;
pub mod nt_agent_mod;
pub mod nt_agent_orchestrator;
pub mod nt_agent_protocol;
pub mod nt_agent_subagent;

// MCP Bridge & Registry & Gateway & Media Consumer
pub mod mcp_bridge;
pub mod mcp_media;
pub mod nt_agent_mcp_registry;
pub mod nt_agent_mcp_openapi;
