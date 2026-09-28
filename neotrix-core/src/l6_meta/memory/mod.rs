//! L6 元层记忆/超越层组件 — **2026-09-28 完成层归属裁决**。
//!
//! 本目录 8 个文件**混装两种职责**，此前在 `.neotrix/layer-map.json` 的
//! `unresolved` 里挂了 2,048 行「⛔ 待裁决」，现裁决如下（详见该文件 `trees["l6_meta/memory"]`）：
//!
//! **一、4 个超越层组件留在 L6（正确）**
//! `consonance_orchestrator` / `evolution_harness` / `meta_observer` / `transcendent_loop`
//! - 它们自述源自 `nt_mind::transcendent::`（见各自文件头 `//!`），但 `l5/nt_mind`
//!   下**无同名文件** ⇒ 不是重复实现，只是历史上搬到了 L6。
//! - 关键证据：L5 的意识循环**主动消费**它们 **12 处**
//!   （`nt_mind_background_loop/handlers_consciousness/nt_transcendent.rs` 5 处 +
//!   `nt_audit.rs` 1 处 + `l6_meta/healing/nt_core_self_test_integration.rs` 2 处），
//!   依赖方向是 **L6 → L5**。
//! - 语义上「元观察 / 共鸣编排 / 进化闭环 / 生产接线载体」本就是元层职责。
//! ⇒ 层归属留 L6 正确，不搬。
//!
//! **二、3 个记忆存储模块层归属错位，待迁移/归档（本轮不搬）**
//! `nt_memory_experience_tree.rs`(458) / `nt_memory_knowledge_pipeline.rs`(228) /
//! `nt_memory_wikiskill.rs`(298) —— 它们是**记忆存储**（`Experience` / `NtFailureCapsule` /
//! `PipelineStage` / `KnowledgeLayer` / `Skill`），不是元能力，语义上属 L4 `nt_memory`。
//! 且三者**库外引用均为 0**，测试 3/0/0 个。
//! **为何本轮不搬**：搬动会牵涉各自的数据落盘路径与序列化格式，属独立任务，
//! 需另行验证兼容性；先在 `layer-map.json` 的 `dead_modules` 字段留证，
//! 避免下个 agent 重复调查。**注意 `pub use nt_memory_experience_tree::*` 等
//! 三个 glob 导出使这些类型已进入 `l6_meta::memory` 命名空间**，迁移时需同步。

pub use crate::l0_substrate::nt_core_traits::CapabilityNode;

pub mod consonance_orchestrator;
pub mod evolution_harness;
pub mod meta_observer;
pub mod transcendent_loop;
pub mod nt_memory_experience_tree;
pub mod nt_memory_knowledge_pipeline;
pub mod nt_memory_wikiskill;

pub use consonance_orchestrator::{
    CapabilityResonance, ConsonanceConfig, ConsonanceOrchestrator, ConsonanceReport,
};
pub use evolution_harness::EvolutionHarness;
pub use meta_observer::{MetaObservationReport, MetaObserver};
pub use transcendent_loop::{EvolutionSuggestion, LoopReport, TranscendentLoop};
pub use nt_memory_experience_tree::*;
pub use nt_memory_knowledge_pipeline::*;
pub use nt_memory_wikiskill::*;
