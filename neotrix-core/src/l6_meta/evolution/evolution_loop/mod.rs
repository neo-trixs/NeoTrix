//! Evolution Loop Closure — 进化闭环
//!
//! 实现 observe → orient → decide → act → verify 五阶段进化闭环。
//! 吸收 KB 经验:
//! - R-P119: 吸收周期
//! - 能力树 C1→C2 晋升判定
//! - 进化前后验证

pub mod loop_runner;
pub mod absorber;
pub mod self_evolver;
pub mod verifier;

pub use loop_runner::*;
pub use absorber::*;
pub use self_evolver::*;
pub use verifier::*;
