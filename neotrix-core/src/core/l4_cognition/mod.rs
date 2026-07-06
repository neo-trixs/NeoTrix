//! # L4 — 认知层 (Cognition)
//!
//! 64 态推理引擎、策略搜索、过程奖励。
//! 科幻映射: Matrix Architect / 《本书记载》超大宇宙数学
//!
//! ## 规则
//! - L4 的 E8 不是决策者 — 只做提案者
//! - E8 状态变化 → CapabilityRequest 到 L7
//! - L4 不决定执行什么，只决定「需要什么」

pub use crate::core::nt_core_e8 as e8;
pub use crate::core::nt_core_hex as hex;
pub use crate::core::nt_core_e8_vsa as e8_vsa;
pub use crate::core::nt_core_policy as policy;
pub use crate::core::nt_core_sae as sae;
pub use crate::core::nt_core_sae_bridge as sae_bridge;
pub use crate::core::nt_core_td as td;
pub use crate::core::nt_core_crt as crt;
pub use crate::core::nt_core_graph as graph;
pub use crate::core::nt_core_walsh as walsh;
pub use crate::core::nt_core_kron as kron;

pub use crate::core::nt_core_e8::{
    E8_DIM, E8_RANK, E8_ROOTS, HEXAGRAM_COUNT,
};
pub use crate::core::nt_core_hex::{
    ReasoningHexagram, MetaState, FullReasoningState,
    ModeFit, ReasoningPath, ReasoningApproach, ProblemDomain,
    all_reasoning_states, optimal_starting_mode,
    rank_modes_for_task, strategy_matrix, evolve_strategy_entry,
};
pub use crate::core::nt_core_policy::{
    E8Outcome, E8TransitionLearner, E8Policy, NUM_E8_FACTORS,
};
pub use crate::core::nt_core_sae::{
    SparseAutoencoder, SaeConfig, SaeOutput, SaeFeature,
    MonosemanticFeature, SAE_LATENT_DIM, SAE_INPUT_DIM,
};
pub use crate::core::nt_core_sae_bridge::SAEBridge;
pub use crate::core::nt_core_e8_vsa::E8VsaEmbedding;
pub use crate::core::nt_core_crt::{CrtTimeScale, CrtPlan};
