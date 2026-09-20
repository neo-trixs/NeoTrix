//! Multi-Agent Orchestration (crewAI pattern)
//!
//! Role-based agent crews with configurable execution strategies,
//! delegation protocols, and result aggregation.
//!
//! Architecture:
//! - `role` — Agent role definitions (Researcher, Coder, Reviewer, Planner, Executor)
//! - `crew` — Crew orchestration with Sequential/Parallel/Hierarchical strategies
//! - `delegation` — Context-enveloped delegation with provenance tracking
//! - `aggregation` — Result aggregation with conflict detection (R-P16)

pub mod role;
pub mod crew;
pub mod delegation;
pub mod aggregation;
pub mod coordinator;
pub mod graph_orch;

// Re-export primary types
pub use role::{AgentRole, RoleConfig};
pub use crew::{Crew, CrewAgent, CrewResult, CrewStrategy, CrewTask, AgentHandle};
pub use delegation::{
    DelegationRequest, DelegationResponse, DelegationStatus, DelegationLog,
    delegate, handback,
};
pub use aggregation::{
    aggregate, detect_conflicts, AggregatedResult, AggregationStrategy,
    AgentWeight, Conflict, ConflictKind,
};
pub use graph_orch::{
    Dag, DagNode, DagEdge, NodeType, EdgeType,
    DagScheduler, ScheduleEntry,
    DagMonitor, NodeStatus, DagStatus,
    DagOptimizer, CriticalPath, ParallelizationSuggestion,
};
