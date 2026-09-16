//! AgentLoop trait definition.
//!
//! Unified agent loop architecture with plan → execute → verify → adapt cycle.
//! Based on Google ARTEMIS (planning with milestones and verify/assert checkpoints)
//! and LongHorizon-Harness (Manager/Executor/Auditor roles).

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::fmt;

use crate::l1_action::traits::{CapabilityError, Plan, PlanResult};

/// High-level objective for the agent to achieve.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub description: String,
    pub constraints: Vec<String>,
    pub priority: u8,
}

/// Plan produced by the planning phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPlan {
    pub steps: Vec<PlanStep>,
    pub milestones: Vec<Milestone>,
    pub estimated_iterations: usize,
}

/// A single step in an agent plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub action: String,
    pub expected_outcome: String,
    pub verify_assertions: Vec<String>,
    pub depends_on: Vec<String>,
}

/// A milestone marker within a plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub name: String,
    pub step_id: String,
    pub assert_conditions: Vec<String>,
}

/// Result of executing a plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub outputs: Vec<StepOutput>,
    pub iterations_used: usize,
    pub errors: Vec<String>,
}

/// Output from a single plan step execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepOutput {
    pub step_id: String,
    pub success: bool,
    pub output: String,
    pub assertions_passed: usize,
    pub assertions_failed: usize,
}

/// Result of verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub passed: bool,
    pub depth: u8,
    pub checks: Vec<VerificationCheck>,
    pub score: f64,
}

/// Individual verification check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

/// Adaptation output after receiving feedback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adaptation {
    pub plan_modified: bool,
    pub new_steps: Vec<PlanStep>,
    pub strategy: AdaptationStrategy,
    pub confidence: f64,
}

/// Strategy for adapting when feedback indicates failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdaptationStrategy {
    Retry,
    Reformulate,
    Escalate,
    ReduceScope,
}

/// The core AgentLoop trait.
///
/// Defines the four-phase cycle: plan → execute → verify → adapt.
/// Implementations provide different strategies for each phase.
pub trait AgentLoop: Send + Sync {
    /// Plan an approach to achieve the objective.
    fn plan(&self, objective: &Objective) -> Result<AgentPlan, CapabilityError>;

    /// Execute the plan, returning execution results.
    fn execute(&self, plan: &AgentPlan) -> Result<ExecutionResult, CapabilityError>;

    /// Verify the execution result against expected criteria.
    fn verify(&self, result: &ExecutionResult, depth: u8) -> Result<VerificationResult, CapabilityError>;

    /// Adapt the plan based on verification feedback.
    fn adapt(&self, feedback: &VerificationResult) -> Result<Adaptation, CapabilityError>;

    /// Run the full loop: plan → execute → verify → adapt (iteratively).
    fn run(&self, objective: &Objective) -> Result<ExecutionResult, CapabilityError> {
        let plan = self.plan(objective)?;
        let result = self.execute(&plan)?;
        let verification = self.verify(&result, self.verification_depth())?;

        if verification.passed {
            return Ok(result);
        }

        let adaptation = self.adapt(&verification)?;
        if adaptation.plan_modified {
            let new_plan = AgentPlan {
                steps: adaptation.new_steps,
                ..plan
            };
            let result = self.execute(&new_plan)?;
            let verification = self.verify(&result, self.verification_depth())?;
            if !verification.passed {
                return Ok(result);
            }
        }

        Ok(result)
    }

    /// Return the verification depth this implementation uses.
    fn verification_depth(&self) -> u8;
}

/// Type alias for agent loop results.
pub type AgentLoopResult<T> = Result<T, AgentLoopError>;

/// Error type for agent loop operations.
#[derive(Debug)]
pub enum AgentLoopError {
    /// Planning failed with the given message.
    PlanningFailed(String),
    /// Execution failed with the given message.
    ExecutionFailed(String),
    /// Verification failed with the given message.
    VerificationFailed(String),
    /// Adaptation failed with the given message.
    AdaptationFailed(String),
    /// Maximum iterations exceeded.
    MaxIterationsExceeded,
    /// Capability error.
    Capability(CapabilityError),
}

impl fmt::Display for AgentLoopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlanningFailed(msg) => write!(f, "Planning failed: {}", msg),
            Self::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            Self::VerificationFailed(msg) => write!(f, "Verification failed: {}", msg),
            Self::AdaptationFailed(msg) => write!(f, "Adaptation failed: {}", msg),
            Self::MaxIterationsExceeded => write!(f, "Max iterations exceeded"),
            Self::Capability(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for AgentLoopError {}

impl From<CapabilityError> for AgentLoopError {
    fn from(e: CapabilityError) -> Self {
        AgentLoopError::Capability(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_loop_error_display() {
        let e = AgentLoopError::PlanningFailed("test".into());
        assert!(e.to_string().contains("Planning failed"));
    }

    #[test]
    fn test_objective_default() {
        let obj = Objective {
            description: "test".into(),
            constraints: vec![],
            priority: 1,
        };
        assert_eq!(obj.priority, 1);
    }
}