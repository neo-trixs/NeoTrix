//! L1 Action Layer - Action Modules

// Actions subdirectory
pub mod actions;

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

pub mod nt_act_3d_dev;
pub mod nt_act_3d_render;

pub mod nt_act_seo;

// Backward-compat re-exports for moved modules
pub use actions::{action_cache as nt_act_action_cache, disk_guard as nt_act_disk_guard, media as nt_act_media, sandbox as nt_act_sandbox, security as nt_act_security};

// 通用能力模块 (从漫剧专用重构为通用)
pub mod resource_budget;      // 资源预算管理 (原 cost_manager)
pub mod temporal_continuity;  // 时序连续性检查 (原 shot_continuity)
pub mod parallel_task;        // 并行任务管理 (原 task_scheduler)

// 向后兼容别名
pub use resource_budget::CostManager;
pub use temporal_continuity::ShotContinuityChecker;
pub use parallel_task::TaskScheduler;
pub use actions::production_orchestrator::BatchProductionManager;

// 成本控制
pub mod cost_manager;

// 镜头衔接
pub mod shot_continuity;

// 任务调度优化
pub mod task_scheduler;

// ============================================================================
// 生产级基础设施模块 (moved to actions/)
// ============================================================================

pub use actions::{error_classifier, observability_stack, cost_tracker, gpu_scheduler, model_router, multi_region_scheduler};

// ============================================================================
// Video pipeline modules (moved to actions/)
// ============================================================================

pub use actions::{video_job_pipeline, video_object_storage, video_spec, video_stitcher, audio_orchestrator};

// 管线检查点 (中间结果存储 + 恢复)
pub mod pipeline_checkpointing;

// Re-export from neotrix/l1_body_impl for backward compatibility
// pub use crate::l1_action::nt_l1_shared_types;
