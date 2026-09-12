//! WorldModel 模块 — 向后兼容 re-export
//!
//! 单一事实源在 `sense::world_model`，此处仅做 re-export 保持旧路径可用。

pub use super::sense::world_model::{WorldModel, ContextEncoder, WifiStatus};
pub use super::sense::nt_world_model_types::{Context, TaskType, Domain, LatentState, Vector, Matrix, LATENT_DIM};
pub use super::sense::nt_world_model_predict::{LatentTransition, ExpertPredictor};
