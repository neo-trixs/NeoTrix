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

// ============================================================================
// 生产级基础设施模块
// ============================================================================

// 错误分类器 (错误类型分类 + 重试策略)
pub mod error_classifier;

// 可观测性栈 (traces, metrics, logs, evals)
pub mod observability_stack;

// 成本追踪器 (成本追踪 + 预算告警)
pub mod cost_tracker;

// ============================================================================
// 生产级调度和路由模块
// ============================================================================

// 视频作业管线 (异步作业队列 + 持久状态 + 检查点)
pub mod video_job_pipeline;

// GPU 调度器 (VRAM 感知路由 + 优先级队列 + 自动扩缩容)
pub mod gpu_scheduler;

// 模型路由器 (质量分级路由 + 成本优化 + 故障转移)
pub mod model_router;

// ============================================================================
// 生产级基础设施模块
// ============================================================================

// 提供商迁移路由器 (厂商抽象 + 迁移路径)
pub mod provider_migration_router;

// 多区域调度器 (跨区域路由 + 故障转移)
pub mod multi_region_scheduler;

// 视频对象存储 (持久存储 + 生命周期管理)
pub mod video_object_storage;

// 管线检查点 (中间结果存储 + 恢复)
pub mod pipeline_checkpointing;

// Re-export from neotrix/l1_body_impl for backward compatibility
// pub use crate::l1_action::nt_l1_shared_types;
