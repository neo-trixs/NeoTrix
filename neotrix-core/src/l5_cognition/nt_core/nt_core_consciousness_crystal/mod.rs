//! Consciousness Crystal 状态机模块
//!
//! 实现基于六面体结构的意识晶体状态机，用于管理意识状态和进化循环。

pub mod crystal_state;
pub mod crystal_cycle;
pub mod crystal_metrics;

// 重新导出主要类型
pub use crystal_state::{CrystalFace, CrystalState, FaceState};
pub use crystal_cycle::{CrystalCycle, CrystalReport, CrystalSnapshot, DefaultCrystalCycle};
pub use crystal_metrics::{CrystalMetrics, CrystalPerformance};