//! AgentExecutor — Performs actions.
//!
//! From LongHorizon-Harness: the Executor component is responsible for
/// performing the actual actions. It wraps an AgentLoop implementation
/// and provides the execution interface.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::l1_action::traits::{CapabilityError, Plan, PlanResult};
use super::trait_::{AgentLoop, AgentPlan, AgentLoopError, Adaptation, AdaptationStrategy, Objective, ExecutionResult, VerificationResult, PlanStep, StepOutput};
use super::config::LoopConfig;
use super::planner_executor::PlannerExecutor;
use super::operator_executor::OperatorExecutor;
use super::checker_executor::CheckerExecutor;

/// Strategy type for the executor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutorStrategy {
    /// Use PlannerExecutor (ARTEMIS-style planning).
    Planner,
    /// Use OperatorExecutor (full tool execution).
    Operator,
    /// Use CheckerExecutor (read-only validation).
    Checker,
}

/// AgentExecutor — Performs actions using the configured strategy.
///
/// Wraps an AgentLoop implementation and provides a clean execution
/// interface. The executor delegates to the chosen strategy implementation.
pub struct AgentExecutor {
    strategy: ExecutorStrategy,
    inner_planner: PlannerExecutor,
    inner_operator: OperatorExecutor,
    inner_checker: CheckerExecutor,
}

impl AgentExecutor {
    /// Create a new AgentExecutor with the default strategy.
    pub fn new(strategy: ExecutorStrategy) -> Self {
        let config = LoopConfig::default();
        Self {
            strategy,
            inner_planner: PlannerExecutor::new(config),
            inner_operator: OperatorExecutor::new(config, vec![], 1),
            inner_checker: CheckerExecutor::new(config, true),
        }
    }

    /// Create a quick-mode executor using PlannerExecutor.
    pub fn quick() -> Self {
        Self::new(ExecutorStrategy::Planner)
    }

    /// Create an orchestrated executor using OperatorExecutor.
    pub fn orchestrated() -> Self {
        Self::new(ExecutorStrategy::Operator)
    }

    /// Create a validation-only executor using CheckerExecutor.
    pub fn validating() -> Self {
        Self::new(ExecutorStrategy::Checker)
    }

    /// Set the executor strategy.
    pub fn set_strategy(&mut self, strategy: ExecutorStrategy) {
        self.strategy = strategy;
    }

    /// Get the current strategy.
    pub fn strategy(&self) -> ExecutorStrategy {
        self.strategy
    }

    /// Execute an objective using the configured strategy.
    pub fn execute(&self, objective: &Objective) -> Result<ExecutionResult, CapabilityError> {
        match self.strategy {
            ExecutorStrategy::Planner => {
                let loop_impl: &dyn AgentLoop = &self.inner_planner;
                loop_impl.run(objective)
            }
            ExecutorStrategy::Operator => {
                let loop_impl: &dyn AgentLoop = &self.inner_operator;
                loop_impl.run(objective)
            }
            ExecutorStrategy::Checker => {
                let loop_impl: &dyn AgentLoop = &self.inner_checker;
                loop_impl.run(objective)
            }
        }
    }

    /// Plan an objective using the configured strategy.
    pub fn plan(&self, objective: &Objective) -> Result<AgentPlan, CapabilityError> {
        match self.strategy {
            ExecutorStrategy::Planner => self.inner_planner.plan(objective),
            ExecutorStrategy::Operator => self.inner_operator.plan(objective),
            ExecutorStrategy::Checker => self.inner_checker.plan(objective),
        }
    }

    /// Verify an execution result.
    pub fn verify(&self, result: &ExecutionResult, depth: u8) -> Result<VerificationResult, CapabilityError> {
        match self.strategy {
            ExecutorStrategy::Planner => self.inner_planner.verify(result, depth),
            ExecutorStrategy::Operator => self.inner_operator.verify(result, depth),
            ExecutorStrategy::Checker => self.inner_checker.verify(result, depth),
        }
    }

    /// Adapt based on verification feedback.
    pub fn adapt(&self, feedback: &VerificationResult) -> Result<Adaptation, CapabilityError> {
        match self.strategy {
            ExecutorStrategy::Planner => self.inner_planner.adapt(feedback),
            ExecutorStrategy::Operator => self.inner_operator.adapt(feedback),
            ExecutorStrategy::Checker => self.inner_checker.adapt(feedback),
        }
    }

    /// Get the strategy name.
    pub fn strategy_name(&self) -> &'static str {
        match self.strategy {
            ExecutorStrategy::Planner => "PlannerExecutor",
            ExecutorStrategy::Operator => "OperatorExecutor",
            ExecutorStrategy::Checker => "CheckerExecutor",
        }
    }
}

impl Default for AgentExecutor {
    fn default() -> Self {
        Self::new(ExecutorStrategy::Planner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_planner() {
        let executor = AgentExecutor::quick();
        assert_eq!(executor.strategy(), ExecutorStrategy::Planner);
        let objective = Objective {
            description: "Test planner".to_string(),
            constraints: vec![],
            priority: 1,
        };
        let result = executor.execute(&objective).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_executor_operator() {
        let executor = AgentExecutor::orchestrated();
        assert_eq!(executor.strategy(), ExecutorStrategy::Operator);
        let objective = Objective {
            description: "Test operator".to_string(),
            constraints: vec![],
            priority: 1,
        };
        let result = executor.execute(&objective).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_executor_checker() {
        let executor = AgentExecutor::validating();
        assert_eq!(executor.strategy(), ExecutorStrategy::Checker);
        let objective = Objective {
            description: "Test checker".to_string(),
            constraints: vec![],
            priority: 1,
        };
        let result = executor.execute(&objective).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_executor_strategy_names() {
        assert_eq!(AgentExecutor::quick().strategy_name(), "PlannerExecutor");
        assert_eq!(AgentExecutor::orchestrated().strategy_name(), "OperatorExecutor");
        assert_eq!(AgentExecutor::validating().strategy_name(), "CheckerExecutor");
    }

    #[test]
    fn test_executor_plan() {
        let executor = AgentExecutor::quick();
        let objective = Objective {
            description: "Test plan".to_string(),
            constraints: vec![],
            priority: 1,
        };
        let plan = executor.plan(&objective).unwrap();
        assert!(!plan.steps.is_empty());
    }
}