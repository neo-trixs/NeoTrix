//! L1 Action Layer - Action Modules

// Actions subdirectory
pub mod actions;

// Trade cluster
pub mod nt_act_trade;

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
// Backward-compat re-exports
// ============================================================================

pub use actions::core::action_cache as nt_act_action_cache;
pub use actions::security::disk_guard as nt_act_disk_guard;
pub use actions::media::media as nt_act_media;
pub use actions::security::sandbox as nt_act_sandbox;
pub use actions::security::security as nt_act_security;

pub mod resource_budget;
pub mod temporal_continuity;
pub mod parallel_task;

pub use resource_budget::CostManager;
pub use temporal_continuity::ShotContinuityChecker;
pub use parallel_task::TaskScheduler;
pub use actions::orchestration::production_orchestrator::BatchProductionManager;

pub use actions::infra::{error_classifier, observability_stack, cost_tracker, gpu_scheduler, model_router, multi_region_scheduler};
pub use actions::video::{video_job_pipeline, video_object_storage, video_spec, video_stitcher, audio_orchestrator};

pub mod pipeline_checkpointing;

pub mod nt_act_cleanup;
pub mod goal_lock;

pub mod reference_view;
pub mod video_quality_scorer;
pub mod video_audit_trail;

// Re-exports for cross-module integration
pub use reference_view::ReferenceManager;
