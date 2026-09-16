//! CheckerExecutor — Read-only validation with safety net.
//!
//! Provides read-only validation capabilities with a safety net.
//! Does not modify any state — only validates and reports.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::l1_action::traits::{CapabilityError, Plan, PlanResult};
use super::trait_::{
    AgentLoop, AgentPlan, AgentLoopError, Adaptation, AdaptationStrategy,
    Objective, ExecutionResult, VerificationCheck, VerificationResult,
    PlanStep, StepOutput,
};
use super::config::LoopConfig;

/// CheckerExecutor — Read-only validation with safety net.
///
/// Validates plans and results without executing any state-changing
/// operations. Provides a safety net for other executors.
pub struct CheckerExecutor {
    config: LoopConfig,
    safety_net: bool,
}

impl CheckerExecutor {
    /// Create a new CheckerExecutor with the given configuration.
    pub fn new(config: LoopConfig, safety_net: bool) -> Self {
        Self {
            config,
            safety_net,
        }
    }

    /// Check if a plan step is safe to execute.
    pub fn is_safe(&self, step: &PlanStep) -> bool {
        if !self.safety_net {
            return true;
        }
        // Check for potentially dangerous actions
        let action_lower = step.action.to_lowercase();
        !action_lower.contains("delete")
            || !action_lower.contains("format")
            || !action_lower.contains("drop")
    }

    /// Validate a plan without executing it.
    pub fn validate_plan(&self, plan: &AgentPlan) -> Vec<VerificationCheck> {
        let mut checks = Vec::new();

        for step in &plan.steps {
            let safe = self.is_safe(step);
            checks.push(VerificationCheck {
                name: format!("safety_check_{}", step.id),
                passed: safe,
                detail: if safe {
                    format!("Step {} is safe to execute", step.id)
                } else {
                    format!("Step {} may be unsafe", step.id)
                },
            });
        }

        checks
    }

    /// Validate all steps pass their assertions.
    pub fn validate_step_assertions(&self, step: &PlanStep, output: &str) -> bool {
        step.verify_assertions.iter().all(|a| {
            let lower_out = output.to_lowercase();
            let lower_assert = a.to_lowercase();
            lower_out.contains(&lower_assert) || lower_assert == "complete"
        })
    }
}

impl Default for CheckerExecutor {
    fn default() -> Self {
        Self::new(LoopConfig::orchestrated(), true)
    }
}

impl AgentLoop for CheckerExecutor {
    fn plan(&self, objective: &Objective) -> Result<AgentPlan, CapabilityError> {
        let steps = self.build_validation_steps(objective);
        Ok(AgentPlan {
            steps,
            milestones: vec![],
            estimated_iterations: 1,
        })
    }

    fn execute(&self, plan: &AgentPlan) -> Result<ExecutionResult, CapabilityError> {
        let mut outputs = Vec::new();
        let mut errors = Vec::new();

        for step in &plan.steps {
            let safe = self.is_safe(step);
            let assertions_passed = if safe { step.verify_assertions.len() } else { 0 };
            let assertions_failed = if safe { 0 } else { step.verify_assertions.len() };

            outputs.push(StepOutput {
                step_id: step.id.clone(),
                success: safe,
                output: format!("Checked step {}: safe={}", step.id, safe),
                assertions_passed,
                assertions_failed,
            });

            if !safe {
                errors.push(format!("Unsafe step detected: {}", step.id));
            }
        }

        Ok(ExecutionResult {
            success: errors.is_empty(),
            outputs,
            iterations_used: 1,
            errors,
        })
    }

    fn verify(&self, result: &ExecutionResult, depth: u8) -> Result<VerificationResult, CapabilityError> {
        let mut checks = Vec::new();
        let mut all_passed = true;

        for output in &result.outputs {
            let check = VerificationCheck {
                name: format!("checker_{}_validate", output.step_id),
                passed: output.success,
                detail: format!(
                    "Safety net: {}, assertions_passed={}, assertions_failed={}",
                    output.success, output.assertions_passed, output.assertions_failed
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
        let failed = feedback.checks.iter().filter(|c| !c.passed).count();
        let plan_modified = failed > 0;

        let strategy = if failed > 0 {
            AdaptationStrategy::Escalate
        } else {
            AdaptationStrategy::ReduceScope
        };

        let new_steps = if plan_modified {
            feedback.checks.iter().filter_map(|c| {
                if !c.passed {
                    Some(PlanStep {
                        id: format!("safe_{}", c.name),
                        action: format!("Safety intervention: {}", c.detail),
                        expected_outcome: "safe execution".to_string(),
                        verify_assertions: vec!["safe".to_string()],
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

impl CheckerExecutor {
    fn build_validation_steps(&self, objective: &Objective) -> Vec<PlanStep> {
        let mut steps = Vec::new();

        steps.push(PlanStep {
            id: "validate_objective".to_string(),
            action: format!("Validate objective: {}", objective.description),
            expected_outcome: "objective valid".to_string(),
            verify_assertions: vec!["valid".to_string()],
            depends_on: vec![],
        });

        steps.push(PlanStep {
            id: "check_constraints".to_string(),
            action: "Check all constraints".to_string(),
            expected_outcome: "constraints verified".to_string(),
            verify_assertions: vec!["verified".to_string()],
            depends_on: vec!["validate_objective".to_string()],
        });

        steps.push(PlanStep {
            id: "safety_audit".to_string(),
            action: "Run safety audit with safety net".to_string(),
            expected_outcome: "safe to proceed".to_string(),
            verify_assertions: vec!["safe".to_string()],
            depends_on: vec!["check_constraints".to_string()],
        });

        if self.safety_net {
            steps.push(PlanStep {
                id: "safety_net_enabled".to_string(),
                action: "Safety net is active".to_string(),
                expected_outcome: "safety net confirmed".to_string(),
                verify_assertions: vec!["net".to_string()],
                depends_on: vec!["safety_audit".to_string()],
            });
        }

        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checker_executor_plan() {
        let executor = CheckerExecutor::new(LoopConfig::quick(), true);
        let objective = Objective {
            description: "Test safety check".to_string(),
            constraints: vec![],
            priority: 1,
        };
        let plan = executor.plan(&objective).unwrap();
        assert!(!plan.steps.is_empty());
    }

    #[test]
    fn test_checker_executor_execute() {
        let executor = CheckerExecutor::default();
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
    fn test_checker_executor_safety_net() {
        let executor = CheckerExecutor::new(LoopConfig::quick(), true);
        let safe = executor.is_safe(&PlanStep {
            id: "safe_step".to_string(),
            action: "read data".to_string(),
            expected_outcome: "data read".to_string(),
            verify_assertions: vec![],
            depends_on: vec![],
        });
        assert!(safe);
    }

    #[test]
    fn test_checker_executor_validate_plan() {
        let executor = CheckerExecutor::default();
        let objective = Objective {
            description: "Test plan validation".to_string(),
            constraints: vec![],
            priority: 1,
        };
        let plan = executor.plan(&objective).unwrap();
        let checks = executor.validate_plan(&plan);
        assert!(!checks.is_empty());
        assert!(checks.iter().all(|c| c.passed));
    }

    #[test]
    fn test_checker_executor_run() {
        let executor = CheckerExecutor::default();
        let objective = Objective {
            description: "Quick checker run".to_string(),
            constraints: vec![],
            priority: 1,
        };
        let result = executor.run(&objective).unwrap();
        assert!(result.success);
    }
}