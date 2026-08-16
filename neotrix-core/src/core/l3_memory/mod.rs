//! # L3 — 记忆层 (Memory)
//!
//! 多层记忆系统：Working → Episodic → Semantic → Procedural
//! 科幻映射: Lain 分布式网络记忆 / GitS 记忆植入 / Matrix 技能加载
//!
//! ## 规则
//! - 所有写入 L3 的数据必须带 VSA 标记
//! - L3 不验证数据真实性（那是 L2 的工作）
//! - 四层记忆梯度不可逆

pub use crate::core::nt_core_bank as bank;
pub use crate::core::nt_core_hcube as hcube;

pub use crate::core::nt_core_bank::{
    MemoryLifecycle, MemoryTier, ReasoningBank, ReasoningBankStats, ReasoningMemory,
    TemporalContext,
};

// VSA 标记体系（从 consciousness/移入中，所有记忆必须标记）
pub use crate::core::l5_consciousness::conscious::source_hierarchy::{
    KnowledgeLayer, ProvenanceChain, SourceHierarchy,
};
pub use crate::core::l5_consciousness::conscious::vsa_tag::{
    VsaOrigin, VsaSelfCategory, VsaTagged, VsaWorldCategory,
};
