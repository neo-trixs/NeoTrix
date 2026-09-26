//! L6 Meta-Cognition Layer
//!
//! 4 目录架构:
//!   coordination/ — 协调类 (meta + governance)
//!   memory/       — 记忆类 (memory + nexus)
//!   healing/      — 修复类 (repair)
//!   evolution/    — 进化类 (mind + act)

pub mod coordination;
pub mod memory;
pub mod healing;
pub mod evolution;
/// L1 Facade
pub mod l1_facade;

/// NT-CORE-GUARDIAN — 统一自守护模块 (融合 8 个分散机制)
pub mod nt_core_guardian;

pub mod nt_meta;
pub use coordination as nt_governance;
pub use healing as nt_repair;
pub mod nt_nexus;

pub mod runtime_monitor;
pub use runtime_monitor::RuntimeMonitor;

pub mod evolving_evaluator;
pub use evolving_evaluator::EvolvingEvaluator;

/// Self-model (价值函数模型)
pub mod nt_core_self_model;
/// 统一自我模型 facade — 合并三个 SelfModel 变体
pub mod self_model_unified;

// 从 core/ 迁移的 L6 模块
pub mod nt_core_self;
pub mod nt_core_self_constitution;
pub mod nt_core_aware;
pub mod nt_core_observer;
pub mod nt_core_observer_error;
pub mod nt_core_kb_primitives;
pub mod nt_core_kb_types;
pub mod nt_core_memory_asset;
pub mod nt_core_absorb;
pub mod nt_core_iter;
pub mod nt_core_scheduler;
pub mod nt_core_self_review;
pub mod nt_core_capability;
/// Agent Identity System — Agent 一等公民身份管理 (absorbed from cumora + munder-difflin)
pub mod nt_agent_identity;
/// Agent Gallery — 浏览/安装预设 agent 角色 (absorbed from munder-difflin)
pub mod nt_agent_gallery;
/// AutoOrchestrator — 自动编排器（替代所有 CLI 管理命令，用户只说意图，系统自动路由）
pub mod nt_auto_orchestrator;

// ============================================================================
// Absorbed — 从 neotrix-sim 吸收
// ============================================================================
/// Safety Monitor with Anomaly Detection — absorbed from neotrix-sim
pub mod nt_safety_monitor;
/// Emergence Detector — absorbed from neotrix-sim
pub mod nt_emergence_detector;
// EVO-07 在线进化闭环（Serve收据/Observe评分/版本化artifact/热切换纯逻辑）
pub mod nt_evolve_loop;
