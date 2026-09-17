//! Consciousness Crystal Core — 晶体意识模型
//!
//! 六面体状态机，融合 E8/GWT/CTM/IIT Φ 理论

pub mod crystal_state;
pub mod crystal_cycle;
pub mod crystal_metrics;

pub use crystal_state::{CrystalFace, CrystalState, FaceState};
pub use crystal_cycle::{CrystalCycle, CrystalReport};
pub use crystal_metrics::CrystalMetrics;
