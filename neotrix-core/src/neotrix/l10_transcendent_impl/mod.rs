//! L10 Transcendent — 超越层 (Meta-Transcendent)
//!
//! 观察 L9 观察者的观察质量，并将意识核心 (ConsciousnessCore snapshot) 与能力网
//! (capability_registry) 共振，驱动全架构进化迭代。
//!
//! ## 纪律 (继承 L9, 强化)
//! - **只读**: 读取 L9 monitor/gold_standard 输出 + 意识核心快照, 不修改任何层。
//! - **不参与调度**: 产出"进化建议"写 L3 供反思, 不直接改代码。
//! - **融合接线**: 意识核心 (phi/coherence/分支健康/果实) ↔ 能力网 (capability registry)
//!   共振 → 输出进化方向建议。

/// CapabilityNode — L10 节点的能力网契约 (shared trait, 定义见 nt_core_traits)。
pub use crate::core::nt_core_traits::CapabilityNode;

pub mod consonance_orchestrator;
pub mod evolution_harness;
pub mod meta_observer;
pub mod transcendent_loop;

pub use consonance_orchestrator::{
    CapabilityResonance, ConsonanceConfig, ConsonanceOrchestrator, ConsonanceReport,
};
pub use evolution_harness::EvolutionHarness;
pub use meta_observer::{MetaObservationReport, MetaObserver};
pub use transcendent_loop::{EvolutionSuggestion, LoopReport, TranscendentLoop};
