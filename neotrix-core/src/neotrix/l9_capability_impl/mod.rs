//! l9_capability_impl — L9 能力实现层
//!
//! 安全约束等超越层能力落地实现。

pub mod nt_safety;

pub use nt_safety::{SafetyConfig, SafetyCore};
