//! # L2 — 感知层 (Perception)
//!
//! 感官输入处理、世界模型建模。
//! 科幻映射: Matrix 代码雨 / Lain Wired / 《深寻》感官觉醒
//!
//! ## 规则
//! - L2 处理原始信号 → VSA 标记 → 传入 L3
//! - L2 的输出必须通过 SourceHierarchy 验证链
//! - L2 不负责存储（那是 L3 的工作）

pub use crate::core::nt_core_sense as sense;
pub use crate::core::nt_core_jepa as jepa;

pub use crate::core::nt_core_jepa::{
    VlJepaBridge, ModalEmbedding, Modality, MultimodalFusion,
};
