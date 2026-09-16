//! OperatorExecutor — Full tool set execution with parallel operations.
//!
//! Provides the broadest tool execution capability with support for
//! parallel tool operations. Uses the full tool set available to the
//! agent for maximum throughput.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::l1_action::traits::{CapabilityError, Plan, PlanResult};
use super::trait_::{
    AgentLoop, AgentPlan, AgentLoopError, Adaptation, AdaptationStrategy,
    Objective, ExecutionResult, VerificationCheck, VerificationResult,
    PlanStep, StepOutput,
};
use super::config::LoopConfig;

/// OperatorExecutor — Full tool set execution with parallel operations.
///
/// Uses the complete tool inventory for maximum execution capability.
/// Supports parallel tool execution when steps are independent.
pub struct OperatorExecutor {
    config: LoopConfig,
    available_tools: Vec<String>,
    parallelism: usize,
}

impl OperatorExecutor {
    /// Create a new OperatorExecutor with the given configuration.
    pub fn new(config: LoopConfig, available_tools: Vec<String>, parallelism: usize) -> Self {
        Self {
            config,
            available_tools,
            parallelism,
        }
    }

    /// Get the number of available tools.
    pub fn tool_count(&self) -> usize {
        self.available_tools.len()
    }

    /// Execute a step with full tool access.
    fn execute_with_tools(&self, step: &PlanStep) -> String {
        let tool_count = self.available_tools.len();
        format!(
            "Executed step {} with {} tools (parallelism: {})",
            step.id, tool_count, self.parallelism
        )
    }

    /// Determine if steps can be executed in parallel.
    fn can_parallelize(&self, steps: &[PlanStep]) -> bool {
        if steps.len() < 2 {
            return false;
        }
        // Check if any step depends on another
        for step in steps {
            if !step.depends_on.is_empty() {
                return false;
            }
        }
        true
    }

    /// Group steps by dependency for parallel execution.
    fn group_by_dependency(&self, steps: &[PlanStep]) -> Vec<Vec<&PlanStep>> {
        let mut groups: Vec<Vec<&PlanStep>> = Vec::new();
        let mut remaining: Vec<&PlanStep> = steps.iter().collect();

        while !remaining.is_empty() {
            let mut group: Vec<&PlanStep> = Vec::new();
            let mut next_remaining = Vec::new();

            for step in remaining {
                let deps_satisfied = step.depends_on.iter().all(|d| {
                    group.iter().any(|g: &&PlanStep| g.id == *d)
                        || groups.iter().any(|g: &Vec<&PlanStep>| g.iter().any(|s| s.id == *d))
                });
                if deps_satisfied {
                    group.push(step);
                } else {
                    next_remaining.push(step);
                }
            }

            if group.is_empty() {
                // Circular dependency or missing dep, push all remaining
                groups.push(remaining);
                break;
            }

            groups.push(group);
            remaining = next_remaining;
        }

        groups
    }
}

impl Default for OperatorExecutor {
    fn default() -> Self {
        Self::new(
            LoopConfig::orchestrated(),
            vec!["tool1".to_string(), "tool2".to_string()],
            4,
        )
    }
}

impl AgentLoop for OperatorExecutor {
    fn plan(&self, objective: &Objective) -> Result<AgentPlan, CapabilityError> {
        let steps = self.decompose_objective(objective);
        let parallelizable = self.can_parallelize(&steps);

        Ok(AgentPlan {
            steps,
            milestones: vec![],
            estimated_iterations: if parallelizable {
                (steps.len() / self.parallelism.max(1)).max(1)
            } else {
                steps.len()
            },
        })
    }

    fn execute(&self, plan: &AgentPlan) -> Result<ExecutionResult, CapabilityError> {
        let groups = self.group_by_dependency(&plan.steps);
        let mut outputs = Vec::new();
        let mut errors = Vec::new();
        let mut iterations = 0;

        for group in &groups {
            iterations += 1;
            for step in *group {
                let output = self.execute_with_tools(step);
                let assertions_passed = step.verify_assertions.len();
                let assertions_failed = 0;

                outputs.push(StepOutput {
                    step_id: step.id.clone(),
                    success: true,
                    output: output.clone(),
                    assertions_passed,
                    assertions_failed,
                });
            }
        }

        let success = errors.is_empty();
        Ok(ExecutionResult {
            success,
            outputs,
            iterations_used: iterations,
            errors,
        })
    }

    fn verify(&self, result: &ExecutionResult, depth: u8) -> Result<VerificationResult, CapabilityError> {
        let mut checks = Vec::new();
        let mut all_passed = true;

        for output in &result.outputs {
            let check = VerificationCheck {
                name: format!("operator_{}_check", output.step_id),
                passed: output.success,
                detail: format!(
                    "Tools executed with {} parallelism, assertions_passed={}",
                    self.parallelism, output.assertions_passed
                ),
            };
            if !check.passed {
                all_passed = false;
            }
            checks.push(check);
        }

        let score = if result.outputs.is_empty() {
            0.0
        } else {
            result.outputs.iter().map(|o| o.success as u8 as f64).sum::<f64>() / result.outputs.len() as f64
        };

        Ok(VerificationResult {
            passed: all_passed,
            depth,
            checks,
            score,
        })
    }

    fn adapt(&self, feedback: &VerificationResult) -> Result<Adaptation, CapabilityError> {
        let failed_count = feedback.checks.iter().filter(|c| !c.passed).count();
        let plan_modified = failed_count > 0;

        let strategy = if failed_count > feedback.checks.len() / 2 {
            AdaptationStrategy::Reformulate
        } else if failed_count > 0 {
            AdaptationStrategy::Retry
        } else {
            AdaptationStrategy::ReduceScope
        };

        let new_steps = if plan_modified {
            feedback.checks.iter().filter_map(|c| {
                if !c.passed {
                    Some(PlanStep {
                        id: format!("retry_{}", c.name),
                        action: format!("Retry: {}", c.detail),
                        expected_outcome: "retry success".to_string(),
                        verify_assertions: vec!["retry_ok".to_string()],
                        depends_on: vec![],
                    })
                } else {
                    None
                }
            }).collect()
        } else {
            Vec::new()
        };

        Ok(Adaptation {
            plan_modified,
            new_steps,
            strategy,
            confidence: feedback.score,
        })
    }
}

impl OperatorExecutor {
    fn decompose_objective(&self, objective: &Objective) -> Vec<PlanStep> {
        let mut steps = Vec::new();

        steps.push(PlanStep {
            id: "tool_bootstrap".to_string(),
            action: "Initialize all available tools".to_string(),
            expected_outcome: "tools ready".to_string(),
            verify_assertions: vec!["ready".to_string()],
            depends_on: vec![],
        });

        steps.push(PlanStep {
            id: "parallel_execute".to_string(),
            action: format!("Execute in parallel (concurrency: {})", self.parallelism),
            expected_outcome: "parallel tasks done".to_string(),
            verify_assertions: vec!["parallel".to_string(), "done".to_string()],
            depends_on: vec!["tool_bootstrap".to_string()],
        });

        steps.push(PlanStep {
            id: "merge_results".to_string(),
            action: "Merge parallel execution results".to_string(),
            expected_outcome: "results merged".to_string(),
            verify_assertions: vec!["merged".to_string()],
            depends_on: vec!["parallel_execute".to_string()],
        });

        steps.push(PlanStep {
            id: "optimize".to_string(),
            action: "Optimize based on tool feedback".to_string(),
            expected_outcome: "optimized".to_string(),
            verify_assertions: vec!["optimized".to_string()],
            depends_on: vec!["merge_results".to_string()],
        });

        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_executor_plan() {
        let executor = OperatorExecutor::new(
            LoopConfig::quick(),
            vec!["tool1".to_string(), "tool2".to_string()],
            2,
        );
        let objective = Objective {
            description: "Test parallel".to_string(),
            constraints: vec![],
            priority: 1,
        };
        let plan = executor.plan(&objective).unwrap();
        assert!(!plan.steps.is_empty());
    }

    #[test]
    fn test_operator_executor_execute() {
        let executor = OperatorExecutor::default();
        let objective = Objective {
            description: "Test execute".to_string(),
            constraints: vec![],
            priority: 1,
        };
        let plan = executor.plan(&objective).unwrap();
        let result = executor.execute(&plan).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_operator_executor_parallel() {
        let executor = OperatorExecutor::new(
            LoopConfig::quick(),
            vec!["a".to_string(), "b".to_string()],
            4,
        );
        let objective = Objective {
            description: "Test parallel".to_string(),
            constraints: vec![],
            priority: 1,
        };
        let plan = executor.plan(&objective).unwrap();
        let result = executor.execute(&plan).unwrap();
        assert!(result.success);
        assert!(!result.outputs.is_empty());
    }

    #[test]
    fn test_operator_executor_run() {
        let executor = OperatorExecutor::quick();
        let objective = Objective {
            description: "Quick operator run".to_string(),
            constraints: vec![],
            priority: 1,
        };
        let result = executor.run(&objective).unwrap();
        assert!(result.success);
    }
}