pub mod capability_evidence;
pub mod ccr;
pub mod context_budget;
pub mod context_gatherer;
pub mod context_os;
pub mod context_predictor;
pub mod prefix_volatility;
pub mod working_memory;
pub use capability_evidence::{
    CapabilityEvidence, CapabilityRegistry, CapabilityReport, EvidenceSource, EvidenceStatus,
};
pub use context_budget::{
    AllocatedSlice, AssembledContext, BudgetSourceType, CompactionIntent, CompactionPriority,
    ContextBudget,
};
