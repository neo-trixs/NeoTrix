pub mod ccr;
pub mod context_budget;
pub mod revertible;
pub use context_budget::{
    AllocatedSlice, AssembledContext, CompactionIntent, CompactionPriority, ContextBudget,
    SourceType,
};
pub use revertible::{ClosureEffect, RevertibleContext, RevertibleEffect};

// ContextAssembler — 五阶段上下文组装引擎 (从 l5_cognition 引入)
pub use crate::l5_cognition::nt_core::context_assembly::{
    AssemblyPlan, AssemblyResult, BudgetAllocation, ContextAssembler, ContextFragment,
    ContextSource, InformationNeed,
};
