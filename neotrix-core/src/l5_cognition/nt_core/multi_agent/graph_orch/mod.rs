//! Graph-based Orchestration (R-P125 / R-P126)
//!
//! DAG-driven multi-agent task orchestration with scheduling, monitoring,
//! and optimization. Complements the crewAI-style orchestration with
//! explicit dependency graphs and parallelism.
//!
//! Architecture:
//! - `dag` — DAG data structure: nodes, edges, topological sort
//! - `scheduler` — Turn-based scheduling with parallelism
//! - `monitor` — Runtime execution status tracking
//! - `optimizer` — DAG optimization, critical path, parallelization

pub mod dag;
pub mod monitor;
pub mod optimizer;
pub mod scheduler;

// Re-export primary types
pub use dag::{Dag, DagEdge, DagNode, EdgeType, NodeType};
pub use monitor::{DagMonitor, DagStatus, NodeStatus, StatusTransition};
pub use optimizer::{CriticalPath, DagOptimizer, ParallelizationSuggestion};
pub use scheduler::{DagScheduler, ScheduleEntry};
