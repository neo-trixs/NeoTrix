//! L1 Action Layer - Action Modules

// Orchestration and planning
pub mod nt_act_orchestrator;

// Code-related actions
pub mod nt_act_code;

// Goal management
pub mod nt_act_goal;

// Autonomy and self-evolution
pub mod nt_act_autonomy;

// Crypto operations
pub mod nt_act_crypto;

// Voice commands
pub mod nt_act_voice;

// ============================================================================
// Standalone modules
// ============================================================================

pub mod nt_act_action_cache;
pub mod nt_act_eventbus;
pub mod nt_act_cache;
pub mod nt_act_circuit_breaker;
pub mod nt_act_rate_limiter;
pub mod nt_act_workflow;

pub mod nt_act_disk_guard;

pub mod nt_act_media;

pub mod nt_act_sandbox;

pub mod nt_act_seo;

// Re-export from neotrix/l1_body_impl for backward compatibility
pub use crate::l1_action::nt_l1_shared_types;
