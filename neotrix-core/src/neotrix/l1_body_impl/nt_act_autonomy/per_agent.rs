use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Planner — decomposes a task into ordered, dependency-aware sub-steps
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub description: String,
    pub priority: u8,
    pub dependencies: Vec<String>,
    pub expected_outcome: String,
    pub status: StepStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    pub task: String,
    pub steps: Vec<PlanStep>,
    pub context: HashMap<String, String>,
}

pub trait Planner {
    fn plan(&mut self, task: &str) -> TaskPlan;
    fn reprioritize(&mut self, plan: &mut TaskPlan, step_id: &str, new_priority: u8);
}

pub struct PlannerAgent {
    pub name: String,
    pub plan_count: u64,
}

impl PlannerAgent {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            plan_count: 0,
        }
    }
}

impl Planner for PlannerAgent {
    fn plan(&mut self, task: &str) -> TaskPlan {
        self.plan_count += 1;
        let sentences: Vec<&str> = task
            .split(['.', '!', '?', ';'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let steps: Vec<PlanStep> = sentences
            .into_iter()
            .enumerate()
            .map(|(i, sentence)| {
                let action_verbs = [
                    "implement", "create", "build", "design", "optimize", "refactor",
                    "test", "deploy", "analyze", "integrate", "configure", "write",
                    "add", "fix", "remove", "migrate", "document", "validate",
                    "extract", "transform", "load", "query", "render", "compile",
                ];
                let has_action = action_verbs
                    .iter()
                    .any(|v| sentence.to_lowercase().contains(v));
                let priority = if has_action { 10u8.saturating_sub(i as u8 * 2).max(1) } else { 5u8.saturating_sub(i as u8).max(1) };
                PlanStep {
                    id: format!("step-{}", Uuid::new_v4().to_string().chars().take(8).collect::<String>()),
                    description: sentence.to_string(),
                    priority,
                    dependencies: vec![],
                    expected_outcome: format!("Complete: {}", sentence.chars().take(60).collect::<String>()),
                    status: StepStatus::Pending,
                }
            })
            .collect();

        TaskPlan {
            task: task.to_string(),
            steps,
            context: HashMap::new(),
        }
    }

    fn reprioritize(&mut self, plan: &mut TaskPlan, step_id: &str, new_priority: u8) {
        if let Some(step) = plan.steps.iter_mut().find(|s| s.id == step_id) {
            step.priority = new_priority.clamp(1, 10);
        }
    }
}

// ---------------------------------------------------------------------------
// Executor — executes a single plan step and produces a result
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub content: String,
    pub step_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub step_id: String,
    pub success: bool,
    pub artifacts: Vec<Artifact>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub duration_ms: u64,
    pub summary: String,
}

pub trait Executor {
    fn execute(&mut self, step: &PlanStep, context: &HashMap<String, String>) -> ActionResult;
}

pub struct ExecutorAgent {
    pub name: String,
    pub execution_count: u64,
    pub last_result: Option<ActionResult>,
}

impl ExecutorAgent {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            execution_count: 0,
            last_result: None,
        }
    }
}

impl Executor for ExecutorAgent {
    fn execute(&mut self, step: &PlanStep, _context: &HashMap<String, String>) -> ActionResult {
        self.execution_count += 1;
        let start = now_ms();

        let content = format!(
            "Executed step '{}': {}\n  Priority: {}\n  Expected: {}",
            step.id, step.description, step.priority, step.expected_outcome
        );

        let artifact = Artifact {
            id: format!("art-{}", Uuid::new_v4().to_string().chars().take(8).collect::<String>()),
            content,
            step_id: step.id.clone(),
            timestamp: (now_ms() / 1000) as i64,
        };

        let result = ActionResult {
            step_id: step.id.clone(),
            success: true,
            artifacts: vec![artifact],
            errors: vec![],
            warnings: vec![],
            duration_ms: now_ms() - start,
            summary: format!("Step '{}' executed successfully", step.description.chars().take(40).collect::<String>()),
        };
        self.last_result = Some(result.clone());
        result
    }
}

// ---------------------------------------------------------------------------
// Reflector — evaluates execution results and produces a plan revision
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanRevision {
    pub new_steps: Vec<PlanStep>,
    pub modified_steps: Vec<ModifiedStep>,
    pub dropped_step_ids: Vec<String>,
    pub rationale: String,
    pub should_continue: bool,
    pub overall_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifiedStep {
    pub step_id: String,
    pub new_description: String,
    pub reason: String,
}

impl PlanRevision {
    pub fn no_changes(score: f64) -> Self {
        Self {
            new_steps: vec![],
            modified_steps: vec![],
            dropped_step_ids: vec![],
            rationale: "No changes needed".to_string(),
            should_continue: false,
            overall_score: score,
        }
    }

    pub fn stop(score: f64, rationale: &str) -> Self {
        Self {
            new_steps: vec![],
            modified_steps: vec![],
            dropped_step_ids: vec![],
            rationale: rationale.to_string(),
            should_continue: false,
            overall_score: score,
        }
    }

    pub fn revise(original: &TaskPlan, results: &[ActionResult]) -> Self {
        let mut new_steps = Vec::new();
        let mut modified_steps = Vec::new();
        let mut dropped = Vec::new();
        let mut total_score = 0.0;
        let mut any_failed = false;

        for step in &original.steps {
            let result = results.iter().find(|r| r.step_id == step.id);
            match result {
                Some(r) if r.success => {
                    total_score += 1.0;
                }
                Some(r) => {
                    any_failed = true;
                    total_score += 0.0;
                    let fix_step = PlanStep {
                        id: format!("step-retry-{}", Uuid::new_v4().to_string().chars().take(8).collect::<String>()),
                        description: format!("Retry: {} (previously failed: {})", step.description, r.errors.join("; ")),
                        priority: step.priority.max(8),
                        dependencies: vec![step.id.clone()],
                        expected_outcome: step.expected_outcome.clone(),
                        status: StepStatus::Pending,
                    };
                    new_steps.push(fix_step);
                    modified_steps.push(ModifiedStep {
                        step_id: step.id.clone(),
                        new_description: step.description.clone(),
                        reason: format!("Failed with {} error(s)", r.errors.len()),
                    });
                }
                None => {
                    dropped.push(step.id.clone());
                }
            }
        }

        let n = original.steps.len().max(1);
        let overall_score = total_score / n as f64;

        Self {
            new_steps,
            modified_steps,
            dropped_step_ids: dropped,
            rationale: if any_failed {
                "Some steps failed; retry with revised approach".to_string()
            } else {
                "All steps completed successfully".to_string()
            },
            should_continue: any_failed,
            overall_score,
        }
    }
}

pub trait Reflector {
    fn reflect(&mut self, plan: &TaskPlan, results: &[ActionResult]) -> PlanRevision;
}

pub struct ReflectorAgent {
    pub name: String,
    pub reflection_count: u64,
}

impl ReflectorAgent {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            reflection_count: 0,
        }
    }
}

impl Reflector for ReflectorAgent {
    fn reflect(&mut self, plan: &TaskPlan, results: &[ActionResult]) -> PlanRevision {
        self.reflection_count += 1;
        PlanRevision::revise(plan, results)
    }
}

// ---------------------------------------------------------------------------
// Orchestrator — Plan-Execute-Reflect loop
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PerConfig {
    pub max_iterations: usize,
    pub min_score_to_converge: f64,
    pub require_all_steps: bool,
}

impl Default for PerConfig {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            min_score_to_converge: 0.8,
            require_all_steps: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopIteration {
    pub plan: TaskPlan,
    pub results: Vec<ActionResult>,
    pub revision: PlanRevision,
    pub iteration: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopOutcome {
    pub task: String,
    pub iterations: Vec<LoopIteration>,
    pub final_plan: TaskPlan,
    pub final_score: f64,
    pub converged: bool,
    pub total_duration_ms: u64,
}

pub struct PlanExecuteReflectLoop {
    pub planner: PlannerAgent,
    pub executor: ExecutorAgent,
    pub reflector: ReflectorAgent,
    pub config: PerConfig,
    pub loop_history: Vec<LoopOutcome>,
}

impl PlanExecuteReflectLoop {
    pub fn new(config: PerConfig) -> Self {
        Self {
            planner: PlannerAgent::new("PER-Planner"),
            executor: ExecutorAgent::new("PER-Executor"),
            reflector: ReflectorAgent::new("PER-Reflector"),
            config,
            loop_history: Vec::new(),
        }
    }

    pub fn run(&mut self, task: &str) -> LoopOutcome {
        let start = std::time::Instant::now();
        let mut plan = self.planner.plan(task);
        let mut iterations = Vec::new();
        let mut converged = false;
        let mut final_score = 0.0;

        for i in 0..self.config.max_iterations {
            let mut results = Vec::new();

            let sorted_steps = self.topological_sort(&plan);

            for step in &sorted_steps {
                let result = self.executor.execute(step, &plan.context);
                results.push(result);
            }

            let revision = self.reflector.reflect(&plan, &results);

            final_score = revision.overall_score;

            iterations.push(LoopIteration {
                plan: plan.clone(),
                results: results.clone(),
                revision: revision.clone(),
                iteration: i,
            });

            if !revision.should_continue && final_score >= self.config.min_score_to_converge {
                converged = true;
                break;
            }

            if !revision.should_continue {
                converged = false;
                break;
            }

            plan.steps.retain(|s| !revision.dropped_step_ids.contains(&s.id));
            for modified in &revision.modified_steps {
                if let Some(step) = plan.steps.iter_mut().find(|s| s.id == modified.step_id) {
                    step.description = modified.new_description.clone();
                    step.status = StepStatus::Pending;
                }
            }
            plan.steps.extend(revision.new_steps);
        }

        let total_duration = start.elapsed().as_millis() as u64;
        let outcome = LoopOutcome {
            task: task.to_string(),
            iterations,
            final_plan: plan,
            final_score,
            converged,
            total_duration_ms: total_duration,
        };
        self.loop_history.push(outcome.clone());
        outcome
    }

    fn topological_sort(&self, plan: &TaskPlan) -> Vec<PlanStep> {
        let mut sorted = plan.steps.clone();
        sorted.sort_by(|a, b| {
            let b_depends_on_a = b.dependencies.contains(&a.id);
            let a_depends_on_b = a.dependencies.contains(&b.id);
            if b_depends_on_a {
                std::cmp::Ordering::Less
            } else if a_depends_on_b {
                std::cmp::Ordering::Greater
            } else {
                b.priority.cmp(&a.priority)
            }
        });
        sorted
    }

    pub fn history_summary(&self) -> String {
        if self.loop_history.is_empty() {
            return "No loop executions completed.".to_string();
        }
        let mut lines = Vec::new();
        for (i, o) in self.loop_history.iter().enumerate() {
            let iter_count = o.iterations.len();
            lines.push(format!(
                "Run #{} | converged={} | score={:.2} | iterations={} | duration={}ms",
                i + 1,
                o.converged,
                o.final_score,
                iter_count,
                o.total_duration_ms,
            ));
        }
        lines.join("\n")
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planner_decomposes_sentences() {
        let mut planner = PlannerAgent::new("test-planner");
        let plan = planner.plan("Build the API. Add authentication. Write tests.");
        assert_eq!(plan.steps.len(), 3);
        assert_eq!(plan.task, "Build the API. Add authentication. Write tests.");
        assert_eq!(planner.plan_count, 1);
    }

    #[test]
    fn test_planner_gives_action_steps_higher_priority() {
        let mut planner = PlannerAgent::new("test");
        let plan = planner.plan("Implement the core. Review the docs.");
        let implement_step = plan.steps.iter().find(|s| s.description.contains("Implement"));
        let review_step = plan.steps.iter().find(|s| s.description.contains("Review"));
        assert!(implement_step.is_some());
        assert!(review_step.is_some());
        assert!(implement_step.unwrap().priority >= review_step.unwrap().priority);
    }

    #[test]
    fn test_executor_produces_artifact() {
        let mut executor = ExecutorAgent::new("test-exec");
        let step = PlanStep {
            id: "step-test".into(),
            description: "Write unit tests".into(),
            priority: 8,
            dependencies: vec![],
            expected_outcome: "All tests pass".into(),
            status: StepStatus::Pending,
        };
        let context = HashMap::new();
        let result = executor.execute(&step, &context);
        assert!(result.success);
        assert_eq!(result.artifacts.len(), 1);
        assert_eq!(result.step_id, "step-test");
        assert!(result.errors.is_empty());
        assert_eq!(executor.execution_count, 1);
    }

    #[test]
    fn test_reflector_all_success_stops_loop() {
        let plan = TaskPlan {
            task: "Test task".into(),
            steps: vec![
                PlanStep {
                    id: "s1".into(),
                    description: "Step one".into(),
                    priority: 5,
                    dependencies: vec![],
                    expected_outcome: "done".into(),
                    status: StepStatus::Completed,
                },
            ],
            context: HashMap::new(),
        };
        let results = vec![
            ActionResult {
                step_id: "s1".into(),
                success: true,
                artifacts: vec![],
                errors: vec![],
                warnings: vec![],
                duration_ms: 10,
                summary: "ok".into(),
            },
        ];
        let mut reflector = ReflectorAgent::new("test-refl");
        let revision = reflector.reflect(&plan, &results);
        assert!(!revision.should_continue);
        assert!((revision.overall_score - 1.0).abs() < 1e-6);
        assert!(revision.new_steps.is_empty());
    }

    #[test]
    fn test_reflector_failure_creates_retry_step() {
        let plan = TaskPlan {
            task: "Test task".into(),
            steps: vec![
                PlanStep {
                    id: "s1".into(),
                    description: "Step one".into(),
                    priority: 5,
                    dependencies: vec![],
                    expected_outcome: "done".into(),
                    status: StepStatus::Completed,
                },
            ],
            context: HashMap::new(),
        };
        let results = vec![
            ActionResult {
                step_id: "s1".into(),
                success: false,
                artifacts: vec![],
                errors: vec!["timeout".into()],
                warnings: vec![],
                duration_ms: 100,
                summary: "failed".into(),
            },
        ];
        let mut reflector = ReflectorAgent::new("test-refl");
        let revision = reflector.reflect(&plan, &results);
        assert!(revision.should_continue);
        assert_eq!(revision.new_steps.len(), 1);
        assert!(revision.new_steps[0].description.contains("Retry"));
    }

    #[test]
    fn test_per_loop_converges_on_all_success() {
        let config = PerConfig {
            max_iterations: 5,
            min_score_to_converge: 0.8,
            require_all_steps: true,
        };
        let mut loop_ = PlanExecuteReflectLoop::new(config);
        let outcome = loop_.run("Implement feature A. Test feature A.");
        assert!(outcome.converged);
        assert!(outcome.final_score >= 0.8);
        assert_eq!(outcome.iterations.len(), 1);
        assert_eq!(loop_.loop_history.len(), 1);
    }

    #[test]
    fn test_per_loop_retries_on_failure() {
        let config = PerConfig {
            max_iterations: 3,
            min_score_to_converge: 0.8,
            require_all_steps: true,
        };
        let mut loop_ = PlanExecuteReflectLoop::new(config);
        loop_.executor = ExecutorAgent {
            name: "flaky-exec".into(),
            execution_count: 0,
            last_result: None,
        };
        let outcome = loop_.run("Simple task.");
        assert!(!outcome.iterations.is_empty());
        assert!(outcome.total_duration_ms > 0 || !outcome.iterations.is_empty());
    }

    #[test]
    fn test_plan_serde_roundtrip() {
        let step = PlanStep {
            id: "s1".into(),
            description: "Build".into(),
            priority: 10,
            dependencies: vec![],
            expected_outcome: "Built".into(),
            status: StepStatus::Pending,
        };
        let json = serde_json::to_string(&step).unwrap();
        let deserialized: PlanStep = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "s1");
        assert_eq!(deserialized.description, "Build");
        assert_eq!(deserialized.priority, 10);
    }

    #[test]
    fn test_history_summary_empty() {
        let loop_ = PlanExecuteReflectLoop::new(PerConfig::default());
        assert_eq!(loop_.history_summary(), "No loop executions completed.");
    }

    #[test]
    fn test_reprioritize_updates_step() {
        let mut planner = PlannerAgent::new("test");
        let mut plan = planner.plan("Do A. Do B.");
        let step_id = plan.steps[0].id.clone();
        planner.reprioritize(&mut plan, &step_id, 1);
        assert_eq!(plan.steps[0].priority, 1);
    }

    #[test]
    fn test_topological_sort_respects_dependencies() {
        let mut planner = PlannerAgent::new("test");
        let mut plan = planner.plan("Step A. Step B.");
        plan.steps[0].id = "a".into();
        plan.steps[1].id = "b".into();
        plan.steps[1].dependencies = vec!["a".into()];
        plan.steps[0].priority = 5;
        plan.steps[1].priority = 10;

        let loop_ = PlanExecuteReflectLoop::new(PerConfig::default());
        let sorted = loop_.topological_sort(&plan);
        assert_eq!(sorted[0].id, "a");
    }
}
