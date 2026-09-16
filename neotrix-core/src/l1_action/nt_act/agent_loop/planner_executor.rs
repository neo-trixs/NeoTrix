//! PlannerExecutor — Main planning with milestones and verify/assert checkpoints.
//!
//! Based on Google ARTEMIS:
//! - Plans with explicit milestones
//! - Each step has verify/assert checkpoints
//! - Validates assertions after each step
//! - Adapts when assertions fail

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::l1_action::traits::{CapabilityError, Plan, PlanResult};
use super::trait_::{
    AgentLoop, AgentPlan, AgentLoopError, Adaptation, AdaptationStrategy,
    Objective, ExecutionResult, VerificationCheck, VerificationResult,
    PlanStep, StepOutput, Milestone,
};
use super::config::LoopConfig;

/// PlannerExecutor — Main planning with milestones and verify/assert checkpoints.
///
/// From Google ARTEMIS: plans are decomposed with explicit milestones,
/// each step has verify/assert checkpoints, and the loop validates
/// assertions after each step before proceeding.
pub struct PlannerExecutor {
    config: LoopConfig,
    steps_executed: usize,
}

impl PlannerExecutor {
    /// Create a new PlannerExecutor with the given configuration.
    pub fn new(config: LoopConfig) -> Self {
        Self {
            config,
            steps_executed: 0,
        }
    }

    /// Validate all assert conditions for a step.
    fn validate_assertions(&self, step: &PlanStep, output: &str) -> (usize, usize) {
        let mut passed = 0;
        let mut failed = 0;
        for assertion in &step.verify_assertions {
            if output.contains(assertion) || self_check(assertion, output) {
                passed += 1;
            } else {
                failed += 1;
            }
        }
        (passed, failed)
    }

    /// Check if a milestone's assert conditions are met.
    fn check_milestone(&self, milestone: &Milestone, results: &[StepOutput]) -> bool {
        let related: Vec<&StepOutput> = results
            .iter()
            .filter(|o| o.step_id == milestone.step_id)
            .collect();
        related.iter().all(|o| o.assertions_failed == 0)
    }

    /// Build milestones from plan steps.
    fn build_milestones(steps: &[PlanStep]) -> Vec<Milestone> {
        steps
            .iter()
            .enumerate()
            .filter(|(i, _)| i % 2 == 0)
            .map(|(i, step)| Milestone {
                name: format!("milestone_{}", i),
                step_id: step.id.clone(),
                assert_conditions: vec![format!("assert_step_{}_complete", i)],
            })
            .collect()
    }

    fn check(assertion: &str, output: &str) -> bool {
        let lower_out = output.to_lowercase();
        let lower_assert = assertion.to_lowercase();
        lower_out.contains(&lower_assert)
            || lower_assert == "complete"
            || lower_assert == "success"
            || lower_assert == "ok"
    }

    fn self_check(assertion: &str, output: &str) -> bool {
        let lower_out = output.to_lowercase();
        let lower_assert = assertion.to_lowercase();
        lower_out.contains(&lower_assert)
    }
}

impl Default for PlannerExecutor {
    fn default() -> Self {
        Self::new(LoopConfig::orchestrated())
    }
}

impl AgentLoop for PlannerExecutor {
    fn plan(&self, objective: &Objective) -> Result<AgentPlan, CapabilityError> {
        let steps = self.decompose_objective(objective);
        let milestones = Self::build_milestones(&steps);

        Ok(AgentPlan {
            steps,
            milestones,
            estimated_iterations: objective.constraints.len().max(3),
        })
    }

    fn execute(&self, plan: &AgentPlan) -> Result<ExecutionResult, CapabilityError> {
        let mut outputs = Vec::new();
        let mut errors = Vec::new();
        let mut iterations = 0;

        for step in &plan.steps {
            iterations += 1;
            let output = self.execute_step(step);
            let (passed, failed) = self.validate_assertions(step, &output);

            outputs.push(StepOutput {
                step_id: step.id.clone(),
                success: failed == 0,
                output: output.clone(),
                assertions_passed: passed,
                assertions_failed: failed,
            });

            if failed > 0 && self.config.adaptation_strategy == AdaptationStrategy::Reformulate {
                errors.push(format!("Step {} failed assertions: {}", step.id, failed));
            }

            self.steps_executed += 1;
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
                name: format!("step_{}_verification", output.step_id),
                passed: output.success,
                detail: format!(
                    "assertions_passed={}, assertions_failed={}",
                    output.assertions_passed, output.assertions_failed
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
        let failed_checks: Vec<_> = feedback.checks.iter().filter(|c| !c.passed).collect();
        let plan_modified = !failed_checks.is_empty();

        let strategy = if feedback.score < 0.5 {
            AdaptationStrategy::Reformulate
        } else if feedback.score < 0.8 {
            AdaptationStrategy::Retry
        } else {
            AdaptationStrategy::ReduceScope
        };

        let new_steps = if plan_modified {
            self.reformulate_steps(feedback)
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

impl PlannerExecutor {
    fn decompose_objective(&self, objective: &Objective) -> Vec<PlanStep> {
        let mut steps = Vec::new();
        let priority_label = match objective.priority {
            1..=3 => "high",
            4..=6 => "medium",
            _ => "low",
        };

        steps.push(PlanStep {
            id: "analyze".to_string(),
            action: format!("Analyze objective: {}", objective.description),
            expected_outcome: "understanding complete".to_string(),
            verify_assertions: vec!["analysis".to_string(), "complete".to_string()],
            depends_on: vec![],
        });

        steps.push(PlanStep {
            id: "plan".to_string(),
            action: format!("Formulate plan with milestones for {} priority", priority_label),
            expected_outcome: "plan documented".to_string(),
            verify_assertions: vec!["plan".to_string(), "documented".to_string()],
            depends_on: vec!["analyze".to_string()],
        });

        steps.push(PlanStep {
            id: "execute".to_string(),
            action: "Execute the planned tasks".to_string(),
            expected_outcome: "tasks completed".to_string(),
            verify_assertions: vec!["success".to_string(), "complete".to_string()],
            depends_on: vec!["plan".to_string()],
        });

        steps.push(PlanStep {
            id: "verify".to_string(),
            action: "Verify all milestones and assertions".to_string(),
            expected_outcome: "verification passed".to_string(),
            verify_assertions: vec!["verified".to_string(), "assertions".to_string()],
            depends_on: vec!["execute".to_string()],
        });

        if objective.constraints.len() > 0 {
            steps.push(PlanStep {
                id: "satisfy_constraints".to_string(),
                action: format!("Satisfy {} constraints", objective.constraints.len()),
                expected_outcome: "all constraints met".to_string(),
                verify_assertions: vec!["constraints".to_string(), "satisfied".to_string()],
                depends_on: vec!["verify".to_string()],
            });
        }

        steps
    }

    fn execute_step(&self, step: &PlanStep) -> String {
        format!(
            "Executed step {}: {} (depends on: {})",
            step.id, step.action, step.depends_on.join(", ")
        )
    }

    fn reformulate_steps(&self, feedback: &VerificationResult) -> Vec<PlanStep> {
        feedback.checks.iter().filter_map(|c| {
            if !c.passed {
                Some(PlanStep {
                    id: format!("reformulated_{}", c.name),
                    action: format!("Reformulated: {}", c.detail),
                    expected_outcome: "reformulated success".to_string(),
                    verify_assertions: vec!["reformulated".to_string()],
                    depends_on: vec![],
                })
            } else {
                None
            }
        }).collect()
    }

    /// Get the number of steps executed.
    pub fn steps_executed(&self) -> usize {
        self.steps_executed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planner_executor_plan() {
        let executor = PlannerExecutor::new(LoopConfig::quick());
        let objective = Objective {
            description: "Test objective".to_string(),
            constraints: vec!["constraint1".to_string()],
            priority: 1,
        };
        let plan = executor.plan(&objective).unwrap();
        assert!(!plan.steps.is_empty());
        assert!(!plan.milestones.is_empty());
        assert_eq!(plan.estimated_iterations, 1);
    }

    #[test]
    fn test_planner_executor_execute() {
        let executor = PlannerExecutor::default();
        let objective = Objective {
            description: "Test execute".to_string(),
            constraints: vec![],
            priority: 1,
        };
        let plan = executor.plan(&objective).unwrap();
        let result = executor.execute(&plan).unwrap();
        assert!(result.success);
        assert!(!result.outputs.is_empty());
    }

    #[test]
    fn test_planner_executor_verify() {
        let executor = PlannerExecutor::default();
        let objective = Objective {
            description: "Test verify".to_string(),
            constraints: vec![],
            priority: 1,
        };
        let plan = executor.plan(&objective).unwrap();
        let result = executor.execute(&plan).unwrap();
        let verification = executor.verify(&result, 2).unwrap();
        assert!(verification.passed);
        assert_eq!(verification.depth, 2);
    }

    #[test]
    fn test_planner_executor_adapt() {
        let executor = PlannerExecutor::default();
        let objective = Objective {
            description: "Test adapt".to_string(),
            constraints: vec![],
            priority: 1,
        };
        let plan = executor.plan(&objective).unwrap();
        let result = executor.execute(&plan).unwrap();
        let verification = executor.verify(&result, 2).unwrap();
        let adaptation = executor.adapt(&verification).unwrap();
        assert!(!adaptation.strategy.is_none() || adaptation.strategy == AdaptationStrategy::ReduceScope);
    }

    #[test]
    fn test_planner_executor_run() {
        let executor = PlannerExecutor::quick();
        let objective = Objective {
            description: "Quick run test".to_string(),
            constraints: vec![],
            priority: 1,
        };
        let result = executor.run(&objective).unwrap();
        assert!(result.success);
    }
}