//! Multi-Agent Coordinator
//!
//! Central coordination layer for multi-agent task assignment with
//! capability matching, priority-based routing, load balancing, and
//! assignment monitoring.
//!
//! Architecture:
//! - `coordinator` — `MultiAgentCoordinator`: agent registration + task dispatch
//! - `task_routing` — `TaskDescription`, `TaskRouter`: priority-queue based routing
//! - `load_balancer` — `LoadBalancer`, `AgentStats`: per-agent load tracking
//! - `monitor` — `CoordinatorMonitor`, `CoordinationMetrics`: assignment metrics

pub mod coordinator;
pub mod load_balancer;
pub mod monitor;
pub mod task_routing;

// Re-export primary types
pub use coordinator::{AgentEntry, Assignment, MultiAgentCoordinator};
pub use load_balancer::{AgentStats, LoadBalancer};
pub use monitor::{CoordinationMetrics, CoordinatorMonitor};
pub use task_routing::{TaskDescription, TaskPriority, TaskRouter};
