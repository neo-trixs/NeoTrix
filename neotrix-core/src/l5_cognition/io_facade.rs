//! IO Facade — L5 对 L1 NT-IO 共享类型的 re-export 门面
//!
//! L5 认知层通过此模块访问 IO 共享函数/类型，避免散布 `use crate::l1_action::nt_io::*`。

pub use crate::l1_action::nt_io::nt_io_provider::context_budget::estimate_tokens;
pub use crate::l1_action::nt_io::nt_io_standalone::{
    ReasoningKernel, ReasoningMethod, ReasoningOutput,
    StageInfo, KernelStats, SelfConsistencyResult, verify_answer,
    text_to_vector, format_kernel_output,
    EVOLUTION, KERNEL_DIM,
};
