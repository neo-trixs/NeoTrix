//! # L8 — 自主神经层 (Autonomic)
//!
//! 维持生命周期：进化、修复、回收、休眠。
//! 科幻映射: Matrix 自维护 / 硅基生命简史熵神学
//!
//! ## 规则
//! - L8 运行在「意识之下」— 不需要 L5 参与
//! - L8 可读取 L3 和 L7，但不可直接修改推理状态
//! - L8 的输出通过 L7 的 Capability 机制提交

pub use crate::core::nt_core_iter as iter;
pub use crate::core::nt_core_iter::SelfIteration;
pub use crate::core::nt_core_iter::self_ref_code::{
    CodeMutation, MutationRequest, MutationResult, MutationRisk, MutationType,
    RollbackPlan, SelfCodeMonitor, SelfRefStats,
};

// SEAL 管线（会从 neotrix/nt_mind 移入）
pub use crate::core::nt_core_absorb as absorb;
pub use crate::core::nt_core_absorb::AbsorbValidator;
pub use crate::core::nt_core_absorb::spec_driven::{
    EvolutionSpec, SpecDiff, SpecDrivenPipeline, SpecPipelineConfig, SpecPipelineStats,
    SpecStatus, SpecVerification, SpecVerifier,
};

// 睡眠巩固（从 consciousness 移入）
pub use crate::core::nt_core_consciousness::sleep_gate::{
    SleepGate, SleepReport,
};

// 梦境巩固（HyperCube 子系统）
pub use crate::core::nt_core_hcube::dream_consolidation::{
    DreamConfig, DreamEvent, DreamReport, DreamPhase,
};
