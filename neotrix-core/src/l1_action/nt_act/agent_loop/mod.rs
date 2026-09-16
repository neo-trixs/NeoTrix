//! Unified agent loop architecture for NT-ACT.
//!
//! Based on Google ARTEMIS (planning with milestones and verify/assert checkpoints)
//! and LongHorizon-Harness (Manager/Executor/Auditor roles).
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐     ┌──────────────┐     ┌─────────────┐
//! │  Manager     │────▶│  Executor    │────▶│  Auditor    │
//! │  (assigns,   │     │  (performs   │     │  (verifies  │
//! │   monitors)  │     │   actions)   │     │   independently)│
//! └─────────────┘     └──────────────┘     └─────────────┘
//!                           │
//!                    ┌──────▼──────┐
//!                    │  AgentLoop  │
//!                    │  (trait)    │
//!                    └──────┬──────┘
//!              ┌──────────┼──────────┐
//!              ▼          ▼          ▼
//!     PlannerExecutor  Operator   CheckerExecutor
//!       (ARTEMIS)    (full tools)   (read-only)
//! ```

pub mod config;
pub mod trait_;
pub mod planner_executor;
pub mod operator_executor;
pub mod checker_executor;
pub mod manager;
pub mod executor;
pub mod auditor;

// Re-exports for convenience
pub use config::{AdaptationStrategy, FlashMode, LoopConfig};
pub use trait_::{
    Adaptation, AgentLoop, AgentLoopError, AgentPlan, ExecutionResult, Objective,
    Milestone, PlanStep, StepOutput, VerificationCheck, VerificationResult,
};
pub use manager::Manager;
pub use executor::AgentExecutor;
pub use auditor::Auditor;
pub use planner_executor::PlannerExecutor;
pub use operator_executor::OperatorExecutor;
pub use checker_executor::CheckerExecutor;