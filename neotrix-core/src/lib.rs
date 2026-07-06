#![recursion_limit = "256"]
//! # NeoTrix — 选择性矩阵运算架构 (Selective State-Space Agent)
//!
//! 核心公式: `Ψ(t+1) = Select(Ô, x) · Select(M, x) · Ψ(t)`
//!
//! ## 架构
//!
//! ```text
//! core/       — 纯数据模型（零外部依赖）
//! agent/      — Agent 运行时
//! cli/        — 终端 UI
//! server/     — HTTP/WebSocket 服务
//! neotrix/    — 全局模块
//! ```
//!
//! 统一版本: 0.18.0 — 推理内核 18 stages

#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(warnings)]
#![deny(dead_code)]
// Float clamp: project convention uses .max().min() pattern
#![allow(clippy::manual_clamp)]
// Functions in crypto/tool APIs legitimately need many parameters
#![allow(clippy::too_many_arguments)]
// Closure type complexity is inherent in event/middleware systems
#![allow(clippy::type_complexity)]
// &mut Vec needed for API compatibility in several subsystems
#![allow(clippy::ptr_arg)]
// Range loops in ML/AI code are idiomatic and clearer than iterator adaptations
#![allow(clippy::needless_range_loop)]
// Manual strip in pattern matching is more readable than strip_prefix chains
#![allow(clippy::manual_strip)]
// from_str/default methods are intentional; implementing traits would add ceremony
#![allow(clippy::should_implement_trait)]
// is_empty not needed for all collection-like types
#![allow(clippy::len_without_is_empty)]

pub mod core;
pub mod cli;
pub mod server;
pub mod agent;
pub mod neotrix;
pub mod nt_shield;

#[macro_export]
macro_rules! make_stage {
    ($name:ident) => {
        #[derive(Default)]
        pub struct $name;
        impl $name {
            pub fn new() -> Self { Self }
        }
    };
}

pub use neotrix::nt_mind;
pub use neotrix::nt_mind::{
    ReasoningBrain, SelfIteratingBrain, SelfEvolver,
};
pub use neotrix::nt_world_model;

pub use core::{
    CapabilityVector, KnowledgeSource, SelfEdit, MicroEdit,
    ReasoningBank, ReasoningMemory,

    AbsorbValidator, SelfIteration,
    KnowledgeProvider, MemoryProvider, AgentExecutor, ToolProvider, ToolDef, ToolOutput, SessionProvider,
    SelfModel, ModuleInfo, FileInfo, DepGraph, TechDebtInventory, TechDebtItem, TechDebtKind, DebtSeverity,
    EvolutionEvent, EventKind, ComponentMap, ComponentNode,
    CodeScanner, MetaMonitor, MetaAlert, AlertSeverity, HealthCheck, HealthTrend,
    WeaknessAnalyzer, Weakness, WeaknessReport, WeaknessSummary,
    EvolutionPlanner, PlannedEvolution, ImpactEstimate, RiskLevel,
    MetaCognitiveLoop, MetaCycleResult,
    SiliconSelfModel, SiliconSelfState, ContextWindow, CognitiveUnit, CognitiveUnitKind,
    AttentionHead, AttentionDomain, AttentionProfile, AttentionManager,
    SystemIdentity, CognitiveCapability, ValueConstraint,
    ReasoningStrategy, ReasoningStrategyRegistry, StrategyKind,
    ThinkingTrace, ThinkingStep, ReflectionGrade,
};
pub use neotrix::nt_act_orchestrator::Orchestrator;
