//! L2 Perception Layer - Sense Modules
//! Re-exports from old l2_world_impl

pub mod nt_world_sense {
    pub use crate::l2_perception::nt_world::nt_world_sense::*;
}

// 计算机视觉模块
pub mod nt_sense_cv;

// 空间记忆 (从 L1 迁移，属于感知层)
pub mod nt_memory_spatial;

// 语义路由器 (从 L1 迁移，属于感知匹配)
pub mod nt_infra_semantic_router;
