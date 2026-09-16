pub mod auto_inspector;

// 从 L1 nt_act_autonomy 迁移过来的模块
pub mod arch_optimizer;

pub mod meta_cognition;

pub mod whale;

pub use arch_optimizer::SelfArchitectureOptimizer;
pub use whale::{AdaptiveSwitching, OptimizationPhase, WHALEConfig, WhaleCycleDetector, WhaleCycleResult};

// Re-exports from l6_meta for nt_meta namespace
pub use crate::l6_meta::runtime_monitor::RuntimeMonitor;
pub use crate::l6_meta::evolving_evaluator::EvolvingEvaluator;
