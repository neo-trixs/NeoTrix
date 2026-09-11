//! SEAL Pipeline 子模块
//!
//! 节奏重算、时长分配、节段管理、留白检查、转场建议
//! Four-stage training cycle: explore → distill → test → absorb

pub mod rhythm_recalculator;
pub mod blank_space_checker;
pub mod transition_gradient;
pub mod training_cycle;