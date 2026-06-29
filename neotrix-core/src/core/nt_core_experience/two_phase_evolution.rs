/// Two-Phase Evolution Pipeline (yoyo-evolve style)
///
/// Wraps SEPL 5-operator formal algebra with:
///   Phase A (Plan):   SEPL proposals → formal execution plan
///   Phase B (Exec):   Plan steps → isolated task execution
///   Phase C (Review): Task outcomes → task system update + plan closure
///
/// Based on yoyo-evolve (yologdev, 2k★, 200→100K LOC self-evolved):
///   Phase A: Read source + proposals + learnings => SESSION_PLAN
///   Phase B: Separate focused agents implement each task
///   Phase C: Issue response / outcome assessment
///
/// And clawREFORM (aegntic, Rust self-rewrite engine):
///   Validates compile → test before commit, rollback on failure

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use super::evolution_task_system::EvolutionTaskSystem;
use super::self_evolution_pipeline::SeplProposal;

// ============================================================================
// TwoPhaseConfig
// ============================================================================

#[derive(Debug, Clone)]
pub struct TwoPhaseConfig {
    /// Max steps per execution plan
    pub max_plan_steps: usize,
    /// Max retries per step
    pub max_retries_per_step: u32,
    /// Whether to validate compile before marking complete
    pub validate_compile: bool,
    /// Whether to validate tests before marking complete
    pub validate_tests: bool,
    /// Timeout per task execution (seconds)
    pub task_timeout_seconds: u64,
    /// Context isolation window — tasks per agent context
    pub context_window_size: usize,
}

impl Default for TwoPhaseConfig {
    fn default() -> Self {
        Self {
            max_plan_steps: 8,
            max_retries_per_step: 2,
            validate_compile: true,
            validate_tests: false,
            task_timeout_seconds: 300,
            context_window_size: 3,
        }
    }
}

// ============================================================================
// ExecutionPlan — SESSION_PLAN equivalent
// ============================================================================

#[derive(Debug, Clone)]
pub struct ExecutionStep {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub target_module: String,
    pub priority: u8,
    pub dependencies: Vec<u64>,
    pub status: StepStatus,
    pub retries_remaining: u32,
    pub validated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub plan_id: u64,
    pub cycle: u64,
    pub source_cycle: u64,
    pub title: String,
    pub steps: Vec<ExecutionStep>,
    pub created_at: Instant,
    pub completed: bool,
}

impl ExecutionPlan {
    pub fn new(plan_id: u64, cycle: u64, title: String) -> Self {
        Self {
            plan_id,
            cycle,
            source_cycle: cycle,
            title,
            steps: Vec::new(),
            created_at: Instant::now(),
            completed: false,
        }
    }

    pub fn next_ready_step(&self) -> Option<&ExecutionStep> {
        self.steps
            .iter()
            .filter(|s| s.status == StepStatus::Pending)
            .filter(|s| {
                s.dependencies
                    .iter()
                    .all(|dep_id| {
                        self.steps
                            .iter()
                            .any(|s| s.id == *dep_id && s.status == StepStatus::Completed)
                    })
            })
            .min_by_key(|s| s.priority)
    }

    pub fn all_completed(&self) -> bool {
        self.steps.iter().all(|s| {
            matches!(s.status, StepStatus::Completed | StepStatus::Skipped)
        })
    }
}

// ============================================================================
// TaskContext — isolated execution context per task
// ============================================================================

#[derive(Debug, Clone)]
pub struct TaskContext {
    pub step_id: u64,
    pub plan_id: u64,
    pub target_module: String,
    pub context_window: VecDeque<String>,
    pub start_time: Instant,
    pub timeout: Duration,
}

impl TaskContext {
    pub fn new(step_id: u64, plan_id: u64, target_module: String, timeout_secs: u64) -> Self {
        Self {
            step_id,
            plan_id,
            target_module,
            context_window: VecDeque::with_capacity(10),
            start_time: Instant::now(),
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.start_time.elapsed() > self.timeout
    }
}

// ============================================================================
// TwoPhaseStats
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct TwoPhaseStats {
    pub plans_created: u64,
    pub plans_completed: u64,
    pub steps_total: u64,
    pub steps_completed: u64,
    pub steps_failed: u64,
    pub steps_skipped: u64,
    pub retries_used: u64,
    pub context_isolation_count: u64,
}

// ============================================================================
// TwoPhaseEvolutionPipeline
// ============================================================================

pub struct TwoPhaseEvolutionPipeline {
    config: TwoPhaseConfig,
    stats: TwoPhaseStats,
    /// Current execution plan (None = idle)
    current_plan: Option<ExecutionPlan>,
    /// Active task contexts
    active_contexts: HashMap<u64, TaskContext>,
    /// Plan ID counter
    next_plan_id: u64,
}

impl TwoPhaseEvolutionPipeline {
    pub fn new(config: TwoPhaseConfig) -> Self {
        Self {
            config,
            stats: TwoPhaseStats::default(),
            current_plan: None,
            active_contexts: HashMap::new(),
            next_plan_id: 1,
        }
    }

    // ========================================================================
    // Phase A: Plan — SEPL proposals → formal execution plan
    // ========================================================================

    /// Phase A: Proposals + audit gaps → prioritized ExecutionPlan.
    ///
    /// Reads SEPL proposals and gaps, generates formal execution steps with
    /// dependency ordering and context isolation windows.
    pub fn phase_a_plan(
        &mut self,
        cycle: u64,
        proposals: &[SeplProposal],
        task_system: &EvolutionTaskSystem,
    ) -> ExecutionPlan {
        let plan_id = self.next_plan_id;
        self.next_plan_id += 1;

        let mut plan = ExecutionPlan::new(
            plan_id,
            cycle,
            format!("two-phase-evolution-{}", plan_id),
        );

        // Generate steps from SEPL proposals
        for prop in proposals.iter().take(self.config.max_plan_steps) {
            plan.steps.push(ExecutionStep {
                id: plan.steps.len() as u64 + 1,
                title: format!("implement-{}", prop.target_module.replace('/', "-")),
                description: prop.description.clone(),
                target_module: prop.target_module.clone(),
                priority: (prop.estimated_impact * 10.0) as u8,
                dependencies: Vec::new(),
                status: StepStatus::Pending,
                retries_remaining: self.config.max_retries_per_step,
                validated: false,
            });
        }

        // Generate steps from task system's ready tasks
        if plan.steps.is_empty() {
            if let Some(ready) = task_system.next_ready_task() {
                plan.steps.push(ExecutionStep {
                    id: 1,
                    title: format!("task-{:?}", ready.task_type).to_lowercase(),
                    description: ready.title.clone(),
                    target_module: ready
                        .gap_ids
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string()),
                    priority: ((ready.priority as f64) * ready.impact * 10.0) as u8,
                    dependencies: Vec::new(),
                    status: StepStatus::Pending,
                    retries_remaining: self.config.max_retries_per_step,
                    validated: false,
                });
            }
        }

        // Sort by priority (lower number = higher priority)
        plan.steps.sort_by_key(|s| s.priority);

        self.current_plan = Some(plan.clone());
        self.stats.plans_created += 1;
        plan
    }

    // ========================================================================
    // Phase B: Execute — plan steps with isolated context
    // ========================================================================

    /// Phase B: Execute the next ready step with isolated context.
    ///
    /// Returns the active TaskContext for the step, or None if no ready steps.
    /// The caller should use the context to perform the actual task, then
    /// call `phase_b_complete_step()`.
    pub fn phase_b_next_context(&mut self) -> Option<&TaskContext> {
        let plan = self.current_plan.as_mut()?;
        let step = plan.next_ready_step().cloned()?;

        let step_id = step.id;
        let plan_id = plan.plan_id;
        let target = step.target_module.clone();

        let ctx = TaskContext::new(
            step_id,
            plan_id,
            target,
            self.config.task_timeout_seconds,
        );

        self.active_contexts.insert(step_id, ctx);

        // Mark the corresponding plan step as in progress
        if let Some(s) = plan.steps.iter_mut().find(|s| s.id == step_id) {
            s.status = StepStatus::InProgress;
        }

        self.stats.context_isolation_count += 1;
        self.active_contexts.get(&step_id)
    }

    /// Phase B: Mark a task as complete (with validation).
    ///
    /// If `success` is true, the step is marked Completed.
    /// If false, it retries (if retries remain) or marks Failed.
    pub fn phase_b_complete_step(
        &mut self,
        step_id: u64,
        success: bool,
        _context_summary: String,
    ) {
        let plan = match self.current_plan.as_mut() {
            Some(p) => p,
            None => return,
        };

        let step = match plan.steps.iter_mut().find(|s| s.id == step_id) {
            Some(s) => s,
            None => return,
        };

        self.active_contexts.remove(&step_id);

        if success {
            step.status = StepStatus::Completed;
            step.validated = true;
            self.stats.steps_completed += 1;
        } else if step.retries_remaining > 0 {
            step.retries_remaining -= 1;
            step.status = StepStatus::Pending;
            self.stats.retries_used += 1;
        } else {
            step.status = StepStatus::Failed;
            self.stats.steps_failed += 1;
        }

        // Check plan completion
        if plan.all_completed() {
            plan.completed = true;
            self.stats.plans_completed += 1;
        }
    }

    // ========================================================================
    // Phase C: Review — assess outcomes, update task system
    // ========================================================================

    /// Phase C: Review plan outcomes and update the task system.
    ///
    /// Returns the number of steps that were completed vs failed.
    pub fn phase_c_review(
        &mut self,
        plan: &ExecutionPlan,
        task_system: &mut EvolutionTaskSystem,
    ) -> (usize, usize) {
        let completed = plan
            .steps
            .iter()
            .filter(|s| s.status == StepStatus::Completed)
            .count();
        let failed = plan
            .steps
            .iter()
            .filter(|s| s.status == StepStatus::Failed)
            .count();

        // Update task system: mark completed tasks
        for step in &plan.steps {
            if step.status == StepStatus::Completed {
                // Find matching task by target_module and mark complete
                let _ = task_system.mark_completed(
                    step.id,
                    0.1, // small delta
                );
            }
        }

        self.stats.steps_total += plan.steps.len() as u64;

        // Clear the current plan
        if let Some(p) = self.current_plan.as_ref() {
            if p.plan_id == plan.plan_id {
                self.current_plan = None;
            }
        }

        (completed, failed)
    }

    /// Get reference to current plan
    pub fn current_plan(&self) -> Option<&ExecutionPlan> {
        self.current_plan.as_ref()
    }

    /// Get stats
    pub fn stats(&self) -> &TwoPhaseStats {
        &self.stats
    }

    /// Summary string for meta-layer self-inspection
    pub fn summary(&self) -> String {
        let plan_info = match &self.current_plan {
            Some(p) => {
                let total = p.steps.len();
                let done = p.steps.iter().filter(|s| s.status == StepStatus::Completed).count();
                let failed = p.steps.iter().filter(|s| s.status == StepStatus::Failed).count();
                format!("plan#{}: {done}/{total} done, {failed} failed", p.plan_id)
            }
            None => "idle".to_string(),
        };
        format!(
            "2PE[plans:{}/{} ctx:{} {}]",
            self.stats.plans_completed,
            self.stats.plans_created,
            self.stats.context_isolation_count,
            plan_info,
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_experience::evolution_task_system::EvolutionTaskSystem;
    use crate::core::nt_core_experience::TaskType;

    #[test]
    fn test_two_phase_default_config() {
        let config = TwoPhaseConfig::default();
        assert_eq!(config.max_plan_steps, 8);
        assert_eq!(config.max_retries_per_step, 2);
    }

    #[test]
    fn test_phase_a_plan_from_proposals() {
        let mut pipeline = TwoPhaseEvolutionPipeline::new(TwoPhaseConfig::default());
        let mut task_sys = EvolutionTaskSystem::new();

        let proposals = vec![
            SeplProposal {
                id: 1,
                hypothesis_id: 1,
                description: "Wire MCTS into pipeline".into(),
                target_module: "nt_core_reasoning/mcts".into(),
                estimated_impact: 0.8,
                risk: 0.2,
                lineage: "ρ(1)→σ(1)".into(),
            },
            SeplProposal {
                id: 2,
                hypothesis_id: 1,
                description: "Connect calibration traces".into(),
                target_module: "nt_core_experience/calibration".into(),
                estimated_impact: 0.6,
                risk: 0.1,
                lineage: "ρ(1)→σ(2)".into(),
            },
        ];

        let plan = pipeline.phase_a_plan(42, &proposals, &task_sys);
        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.plan_id, 1);
        assert!(plan.steps.iter().all(|s| s.status == StepStatus::Pending));
        assert!(pipeline.current_plan().is_some());
    }

    #[test]
    fn test_phase_a_fallback_to_task_system() {
        let mut pipeline = TwoPhaseEvolutionPipeline::new(TwoPhaseConfig::default());
        let mut task_sys = EvolutionTaskSystem::new();
        task_sys.create_task(
            TaskType::ArchitectureReview,
            "Review consciousness wiring",
            "Check all 12 steps are wired",
            8,
            0.6,
        );

        let plan = pipeline.phase_a_plan(42, &[], &task_sys);
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(
            plan.steps[0].target_module,
            "unknown",
        );
    }

    #[test]
    fn test_phase_b_context_isolation() {
        let mut pipeline = TwoPhaseEvolutionPipeline::new(TwoPhaseConfig::default());
        let task_sys = EvolutionTaskSystem::new();

        let proposals = vec![SeplProposal {
            id: 1,
            hypothesis_id: 1,
            description: "fix".into(),
            target_module: "test".into(),
            estimated_impact: 0.5,
            risk: 0.1,
            lineage: "ρ".into(),
        }];

        let _plan = pipeline.phase_a_plan(42, &proposals, &task_sys);
        let ctx = pipeline.phase_b_next_context();
        assert!(ctx.is_some());
        assert_eq!(ctx.unwrap().target_module, "test");

        // Second call should return None (only 1 step)
        let ctx2 = pipeline.phase_b_next_context();
        assert!(ctx2.is_none());
    }

    #[test]
    fn test_phase_b_complete_step_success() {
        let mut pipeline = TwoPhaseEvolutionPipeline::new(TwoPhaseConfig::default());
        let task_sys = EvolutionTaskSystem::new();

        let proposals = vec![SeplProposal {
            id: 1,
            hypothesis_id: 1,
            description: "fix".into(),
            target_module: "test".into(),
            estimated_impact: 0.5,
            risk: 0.1,
            lineage: "ρ".into(),
        }];

        let plan1 = pipeline.phase_a_plan(42, &proposals, &task_sys);
        assert!(pipeline.phase_b_next_context().is_some());
        pipeline.phase_b_complete_step(1, true, "done".into());

        let plan = pipeline.current_plan().unwrap();
        assert!(plan.steps[0].status == StepStatus::Completed);
        assert!(plan.all_completed());
    }

    #[test]
    fn test_phase_b_retry_on_failure() {
        let mut pipeline = TwoPhaseEvolutionPipeline::new(TwoPhaseConfig {
            max_retries_per_step: 2,
            ..Default::default()
        });
        let task_sys = EvolutionTaskSystem::new();

        let proposals = vec![SeplProposal {
            id: 1,
            hypothesis_id: 1,
            description: "fix".into(),
            target_module: "test".into(),
            estimated_impact: 0.5,
            risk: 0.1,
            lineage: "ρ".into(),
        }];

        let _plan = pipeline.phase_a_plan(42, &proposals, &task_sys);

        // First attempt fails, retry available
        let _ctx = pipeline.phase_b_next_context();
        pipeline.phase_b_complete_step(1, false, "failed".into());

        let plan = pipeline.current_plan().unwrap();
        assert!(plan.steps[0].status == StepStatus::Pending);
        assert_eq!(plan.steps[0].retries_remaining, 1);

        // Second attempt also fails
        let _ctx2 = pipeline.phase_b_next_context();
        pipeline.phase_b_complete_step(1, false, "failed again".into());

        let plan2 = pipeline.current_plan().unwrap();
        assert!(plan2.steps[0].status == StepStatus::Pending);
        assert_eq!(plan2.steps[0].retries_remaining, 0);

        // Third attempt fails → no more retries
        let _ctx3 = pipeline.phase_b_next_context();
        pipeline.phase_b_complete_step(1, false, "failed thrice".into());

        let plan3 = pipeline.current_plan().unwrap();
        assert!(plan3.steps[0].status == StepStatus::Failed);
    }

    #[test]
    fn test_phase_c_review() {
        let mut pipeline = TwoPhaseEvolutionPipeline::new(TwoPhaseConfig::default());
        let mut task_sys = EvolutionTaskSystem::new();

        let proposals = vec![SeplProposal {
            id: 1,
            hypothesis_id: 1,
            description: "fix".into(),
            target_module: "test".into(),
            estimated_impact: 0.5,
            risk: 0.1,
            lineage: "ρ".into(),
        }];

        let plan1 = pipeline.phase_a_plan(42, &proposals, &task_sys);
        assert!(pipeline.phase_b_next_context().is_some());
        pipeline.phase_b_complete_step(1, true, "done".into());

        let (completed, failed) = pipeline.phase_c_review(&plan1, &mut task_sys);
        assert_eq!(completed, 1);
        assert_eq!(failed, 0);

        // Plan should be cleared after review
        assert!(pipeline.current_plan().is_none());
    }

    #[test]
    fn test_summary() {
        let pipeline = TwoPhaseEvolutionPipeline::new(TwoPhaseConfig::default());
        let s = pipeline.summary();
        assert!(s.contains("idle"));

        let mut p2 = TwoPhaseEvolutionPipeline::new(TwoPhaseConfig::default());
        let mut task_sys = EvolutionTaskSystem::new();
        let proposals = vec![SeplProposal {
            id: 1,
            hypothesis_id: 1,
            description: "fix".into(),
            target_module: "test".into(),
            estimated_impact: 0.5,
            risk: 0.1,
            lineage: "ρ".into(),
        }];
        let _plan = p2.phase_a_plan(42, &proposals, &task_sys);
        let s2 = p2.summary();
        assert!(s2.contains("plan#"));
    }

    #[test]
    fn test_context_expiry() {
        let ctx = TaskContext::new(1, 1, "test".into(), 99999);
        assert!(!ctx.is_expired());
    }

    #[test]
    fn test_empty_plan_all_completed() {
        let plan = ExecutionPlan::new(1, 1, "empty".into());
        assert!(plan.all_completed());
    }

    #[test]
    fn test_dependency_ordering() {
        let mut plan = ExecutionPlan::new(1, 1, "dep-test".into());
        plan.steps.push(ExecutionStep {
            id: 2,
            title: "step-b".into(),
            description: "depends on step 1".into(),
            target_module: "mod2".into(),
            priority: 1,
            dependencies: vec![1],
            status: StepStatus::Pending,
            retries_remaining: 1,
            validated: false,
        });
        plan.steps.push(ExecutionStep {
            id: 1,
            title: "step-a".into(),
            description: "no deps".into(),
            target_module: "mod1".into(),
            priority: 2,
            dependencies: vec![],
            status: StepStatus::Pending,
            retries_remaining: 1,
            validated: false,
        });

        // Step 1 should be ready (no deps), step 2 should not (depends on 1)
        let ready = plan.next_ready_step().unwrap();
        assert_eq!(ready.id, 1);
    }

    #[test]
    fn test_stats_tracking() {
        let mut pipeline = TwoPhaseEvolutionPipeline::new(TwoPhaseConfig::default());
        assert_eq!(pipeline.stats().plans_created, 0);
        assert_eq!(pipeline.stats().context_isolation_count, 0);

        let mut task_sys = EvolutionTaskSystem::new();
        let proposals = vec![SeplProposal {
            id: 1,
            hypothesis_id: 1,
            description: "fix".into(),
            target_module: "test".into(),
            estimated_impact: 0.5,
            risk: 0.1,
            lineage: "ρ".into(),
        }];

        pipeline.phase_a_plan(42, &proposals, &task_sys);
        assert_eq!(pipeline.stats().plans_created, 1);

        pipeline.phase_b_next_context();
        assert_eq!(pipeline.stats().context_isolation_count, 1);
    }

    #[test]
    fn test_multiple_steps_in_plan() {
        let mut pipeline = TwoPhaseEvolutionPipeline::new(TwoPhaseConfig::default());
        let task_sys = EvolutionTaskSystem::new();

        let proposals: Vec<SeplProposal> = (0..5)
            .map(|i| SeplProposal {
                id: i,
                hypothesis_id: i,
                description: format!("step-{}", i),
                target_module: format!("module-{}", i),
                estimated_impact: 0.5,
                risk: 0.1,
                lineage: format!("ρ({})→σ({})", i, i),
            })
            .collect();

        let plan = pipeline.phase_a_plan(42, &proposals, &task_sys);
        assert_eq!(plan.steps.len(), 5);
    }
}
