//! PRM (Process Reward Model) — 多智能体推理轨迹的过程奖励模型
//!
//! D53 拓扑修复: god-file 拆分 (3688 行 → 6 子模块)。
//! 入口纯 re-export, pub API 100% 保留:
//! `crate::core::nt_core_prm::*` 全部公开项不变。

mod types;
mod collector;
mod learner;
mod step_grpo;
mod ws_grpo;
mod verifier;

pub use types::*;
pub use collector::*;
pub use learner::*;
pub use step_grpo::*;
pub use ws_grpo::*;
pub use verifier::*;
