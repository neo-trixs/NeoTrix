//! pipeline — SEAL BrainStage 调度管线 (God-file 拆分门面, 行为零变更).
//! 原 3224 行按 Stage 族拆为 `pipeline/nt_*` 7 内容模块 + `nt_tests`;
//! 本文件只留模块声明与重导出.

pub mod nt_assemble;
pub mod nt_core_stages;
pub mod nt_credit_stages;
pub mod nt_optimize_stages;
pub mod nt_quality_stages;
pub mod nt_types;
pub mod nt_wrapper_stages;

#[cfg(test)]
pub mod nt_tests;

pub use nt_assemble::*;
pub use nt_core_stages::*;
pub use nt_credit_stages::*;
pub use nt_optimize_stages::*;
pub use nt_quality_stages::*;
pub use nt_types::*;
pub use nt_wrapper_stages::*;
