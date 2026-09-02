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

// 通用能力模块 (从漫剧专用重构为通用)
pub mod resource_budget;      // 资源预算管理 (原 cost_manager)
pub mod temporal_continuity;  // 时序连续性检查 (原 shot_continuity)
pub mod parallel_task;        // 并行任务管理 (原 task_scheduler)
pub mod production_orchestrator; // 生产编排器 (原 batch_production)

// 视频规格系统 (类型化规格)
pub mod video_spec;

// 检查点持久化
pub mod checkpoint_persistence;

// 运行手册系统
pub mod operator_runbook;

// 视频拼接器
pub mod video_stitcher;

// 音频编排器
pub mod audio_orchestrator;

// 发布网关
pub mod publish_gateway;

// 向后兼容别名
pub use resource_budget::CostManager;
pub use temporal_continuity::ShotContinuityChecker;
pub use parallel_task::TaskScheduler;
pub use production_orchestrator::BatchProductionManager;

// 成本控制
pub mod cost_manager;

// 镜头衔接
pub mod shot_continuity;

// 任务调度优化
pub mod task_scheduler;

// Re-export from neotrix/l1_body_impl for backward compatibility
// pub use crate::l1_action::nt_l1_shared_types;
