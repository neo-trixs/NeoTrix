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

pub mod core;
pub mod agent;
pub mod cli;
pub mod server;
pub mod neotrix;

pub use neotrix::nt_mind;
pub use neotrix::nt_mind::{
    ReasoningBrain, SelfIteratingBrain, SelfEvolver,
};
pub use neotrix::nt_world_model;

pub use core::{
    CapabilityVector, KnowledgeSource, SelfEdit, MicroEdit,
    ReasoningBank, ReasoningMemory,
    SelectiveState, SelectableOperator,
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
