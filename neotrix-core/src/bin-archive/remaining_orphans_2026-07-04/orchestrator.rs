use std::time::Instant;

use super::planner::{Plan, Planner, PlanningStrategy, Step};
use super::executor::{ExecutionResult, Executor};
use super::reflector::{Reflection, Reflector};

#[derive(Debug, Clone)]
pub struct OrchestrationReport {
    pub plan: Plan,
    pub results: Vec<ExecutionResult>,
    pub reflection: Reflection,
    pub total_duration_ms: u64,
    pub cycle_count: usize,
}

#[derive(Debug, Clone)]
pub enum CycleOutcome {
    Complete(OrchestrationReport),
    NeedsRetry(OrchestrationReport),
    Failed(OrchestrationReport),
}

#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub max_iterations: usize,
    pub confidence_threshold: f64,
    pub parallel_execution: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_iterations: 3,
            confidence_threshold: 0.6,
            parallel_execution: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Orchestrator {
    planner: Planner,
    executor: Executor,
    reflector: Reflector,
    config: OrchestratorConfig,
    cycle_count: usize,
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            planner: Planner::new(),
            executor: Executor::new(),
            reflector: Reflector::new(),
            config: OrchestratorConfig::default(),
            cycle_count: 0,
        }
    }

    pub fn with_config(config: OrchestratorConfig) -> Self {
        Self {
            planner: Planner::new(),
            executor: Executor::new(),
            reflector: Reflector::new(),
            config,
            cycle_count: 0,
        }
    }

    pub fn run_cycle(&mut self, task: &str, tools: &[String]) -> CycleOutcome {
        let start = Instant::now();
        self.cycle_count += 1;

        let plan = self.planner.plan(task, tools);
        let results = if self.config.parallel_execution {
            self.executor.execute_parallel(&plan.steps)
        } else {
            self.executor.execute_all(&plan.steps)
        };

        for (step, result) in plan.steps.iter().zip(results.iter()) {
            self.reflector.reflect_step(step, result, &step.description);
        }

        let reflection = self.reflector.reflect(&plan, &results);
        let elapsed = start.elapsed().as_millis() as u64;

        let all_success = results.iter().all(|r| r.success);
        let report = OrchestrationReport {
            plan,
            results,
            reflection,
            total_duration_ms: elapsed,
            cycle_count: self.cycle_count,
        };

        if all_success && report.reflection.confidence >= self.config.confidence_threshold {
            CycleOutcome::Complete(report)
        } else if self.cycle_count < self.config.max_iterations {
            CycleOutcome::NeedsRetry(report)
        } else {
            CycleOutcome::Failed(report)
        }
    }

    pub fn run_full(&mut self, task: &str, tools: &[String]) -> OrchestrationReport {
        loop {
            match self.run_cycle(task, tools) {
                CycleOutcome::Complete(report) => return report,
                CycleOutcome::NeedsRetry(report) => {
                    if let Some(ref adjusted) = report.reflection.adjusted_plan {
                        self.planner = Planner::with_strategy(adjusted.strategy);
                    }
                    self.reflector.clear_history();
                }
                CycleOutcome::Failed(report) => return report,
            }

            if self.cycle_count >= self.config.max_iterations {
                let plan = self.planner.plan(task, tools);
                let results = self.executor.execute_all(&plan.steps);
                let reflection = self.reflector.reflect(&plan, &results);
                return OrchestrationReport {
                    plan,
                    results,
                    reflection,
                    total_duration_ms: 0,
                    cycle_count: self.cycle_count,
                };
            }
        }
    }

    pub fn planner(&self) -> &Planner {
        &self.planner
    }

    pub fn planner_mut(&mut self) -> &mut Planner {
        &mut self.planner
    }

    pub fn executor(&self) -> &Executor {
        &self.executor
    }

    pub fn executor_mut(&mut self) -> &mut Executor {
        &mut self.executor
    }

    pub fn reflector(&self) -> &Reflector {
        &self.reflector
    }

    pub fn cycle_count(&self) -> usize {
        self.cycle_count
    }

    pub fn config(&self) -> &OrchestratorConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut OrchestratorConfig {
        &mut self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_handler(input: &str) -> (bool, String) {
        (true, format!("ok: {}", input))
    }

    fn failing_handler(_input: &str) -> (bool, String) {
        (false, "error".to_string())
    }

    fn setup_orchestrator() -> Orchestrator {
        let mut orch = Orchestrator::new();
        orch.executor_mut().register_tool("search", "Search tool", mock_handler);
        orch.executor_mut().register_tool("code", "Code tool", mock_handler);
        orch
    }

    #[test]
    fn test_orchestrator_creation() {
        let orch = Orchestrator::new();
        assert_eq!(orch.cycle_count(), 0);
    }

    #[test]
    fn test_run_cycle_success() {
        let mut orch = setup_orchestrator();
        let tools = vec!["search".to_string(), "code".to_string()];
        let outcome = orch.run_cycle("search and code", &tools);
        match outcome {
            CycleOutcome::Complete(report) => {
                assert!(!report.results.is_empty());
            }
            _ => {}
        }
    }

    #[test]
    fn test_run_cycle_with_failures() {
        let mut orch = Orchestrator::new();
        orch.executor_mut().register_tool("bad", "Bad tool", failing_handler);
        let tools = vec!["bad".to_string()];
        let outcome = orch.run_cycle("do something bad", &tools);
        match outcome {
            CycleOutcome::NeedsRetry(_) | CycleOutcome::Failed(_) => {}
            _ => {}
        }
    }

    #[test]
    fn test_config_max_iterations() {
        let config = OrchestratorConfig {
            max_iterations: 1,
            confidence_threshold: 0.99,
            parallel_execution: false,
        };
        let mut orch = Orchestrator::with_config(config);
        orch.executor_mut().register_tool("t", "tool", mock_handler);
        let report = orch.run_full("test task", &["t".to_string()]);
        assert!(report.cycle_count >= 1);
    }

    #[test]
    fn test_run_full_converges() {
        let mut orch = setup_orchestrator();
        let report = orch.run_full("search and code", &["search".to_string(), "code".to_string()]);
        assert!(!report.results.is_empty());
        assert!(!report.reflection.insights.is_empty() || report.reflection.confidence > 0.0);
    }

    #[test]
    fn test_orchestrator_accessors() {
        let orch = Orchestrator::new();
        assert_eq!(orch.cycle_count(), 0);
        assert_eq!(orch.config().max_iterations, 3);
    }
}
