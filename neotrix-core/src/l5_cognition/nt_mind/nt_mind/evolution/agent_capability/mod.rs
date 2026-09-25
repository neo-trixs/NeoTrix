//! Agent 能力层 (Agent Capability Layer) — 记忆大脑 agent 化 (R-P42 强化现有节点)。
//!
//! 结论性设计 (架构评估): 元认知**不整体 agent 化**, 而是 "确定性内核 + agent 外壳"。
//! 本模块提供:
//!   1. `MemoryAgentCapability` — 记忆大脑统一能力 trait (写入/检索/巩固/证据/图查询),
//!      把 `KnowledgeBase` 的具体方法暴露为 agent 可调用的统一表面。
//!   2. `MetaAgentShell` — 元认知 agent 外壳, 用 AttentionManager 按任务类型路由
//!      到确定性内核 (MetaCognitiveLoop) 的对应阶段。
//!
//! 来源: ai-knowledge-graph agent 化吸收 + Onyx 决策管线 (P0-2) + 统一写入弧 (P0-1)。
//! 约束: 核心确定性管线 (nt_core_meta) 保持同步无运行时依赖, 外壳只在
//! `nt_mind` 层做路由 — 不引入平行适配器模块 (R-P42)。

pub mod nt_capability_compose;
pub mod nt_capability_eval;
pub mod nt_capability_registry;
pub mod nt_capability_types;

pub use nt_capability_compose::*;
pub use nt_capability_eval::*;
pub use nt_capability_registry::*;
pub use nt_capability_types::*;

#[cfg(test)]
mod tests;
