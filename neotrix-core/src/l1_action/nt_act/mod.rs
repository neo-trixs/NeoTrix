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

// Types module
pub mod nt_act_types;

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
pub mod nt_act_ai_assistant;
pub mod nt_act_3d_dev;
pub mod nt_act_3d_render;

pub mod nt_act_disk_guard;

pub mod nt_act_media;

pub mod nt_act_sandbox;

pub mod nt_act_seo;

pub mod nt_act_security;

// 批量生产工作流
pub mod production_pipeline;

// 成本控制
pub mod cost_manager;

// 镜头衔接
pub mod shot_continuity;

// 任务调度优化
pub mod task_scheduler;

// Re-export from neotrix/l1_body_impl for backward compatibility
// pub use crate::l1_action::nt_l1_shared_types;
