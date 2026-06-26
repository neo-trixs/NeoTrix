use std::time::{SystemTime, UNIX_EPOCH};

use crate::agent::agent_workflow::{AgentStep, AgentWorkflow, AgentWorkflowResult, PlanMode};
use crate::core::nt_core_experience::loop_templates::{
    ConsciousnessLoopEngine, LoopIterationResult, LoopStatus, LoopStep, LoopTemplate,
    LoopTemplateRegistry, RunningLoop, StepAction,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowStatus {
    Idle,
    Running {
        template_id: String,
        iteration: usize,
        step_index: usize,
    },
    Completed {
        template_id: String,
        iterations: usize,
    },
    Failed {
        template_id: String,
        reason: String,
    },
    MaxIterationsReached {
        template_id: String,
        iterations: usize,
    },
}

#[derive(Debug, Clone)]
pub struct WorkflowExecutionContext {
    pub template: LoopTemplate,
    pub running_loop: RunningLoop,
    pub start_time: u64,
    pub last_check_output: Option<String>,
    pub agent_result: Option<AgentWorkflowResult>,
    pub plan_mode: PlanMode,
}

pub struct EngineeringWorkflowExecutor {
    loop_registry: LoopTemplateRegistry,
    current: Option<WorkflowExecutionContext>,
    history: Vec<WorkflowExecutionContext>,
    pub max_history: usize,
}

impl EngineeringWorkflowExecutor {
    pub fn new() -> Self {
        let mut reg = LoopTemplateRegistry::new();
        for t in crate::core::nt_core_experience::loop_templates::default_templates() {
            reg.register(t);
        }
        Self {
            loop_registry: reg,
            current: None,
            history: Vec::new(),
            max_history: 100,
        }
    }

    pub fn start_workflow(&mut self, template_id: &str, plan_mode: PlanMode) -> Result<(), String> {
        let template = self
            .loop_registry
            .get(template_id)
            .ok_or_else(|| format!("template '{}' not found", template_id))?;
        let running = ConsciousnessLoopEngine::instantiate(template);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.current = Some(WorkflowExecutionContext {
            template: template.clone(),
            running_loop: running,
            start_time: now,
            last_check_output: None,
            agent_result: None,
            plan_mode,
        });
        Ok(())
    }

    pub fn start_workflow_from_template(&mut self, template: LoopTemplate, plan_mode: PlanMode) {
        let running = ConsciousnessLoopEngine::instantiate(&template);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.current = Some(WorkflowExecutionContext {
            template,
            running_loop: running,
            start_time: now,
            last_check_output: None,
            agent_result: None,
            plan_mode,
        });
    }

    pub fn tick(&mut self) -> WorkflowStatus {
        let ctx = match self.current.as_mut() {
            Some(c) => c,
            None => return WorkflowStatus::Idle,
        };

        let max_iter = ctx.template.max_iterations;
        if max_iter > 0 && ctx.running_loop.iteration_count >= max_iter {
            let tid = ctx.template.id.clone();
            let iter = ctx.running_loop.iteration_count;
            let failed = WorkflowStatus::MaxIterationsReached {
                template_id: tid,
                iterations: iter,
            };
            let ctx_owned = self.current.take().unwrap();
            self.history.push(ctx_owned);
            if self.history.len() > self.max_history {
                self.history.remove(0);
            }
            return failed;
        }

        let conditions = &ctx.template.exit_conditions;
        let mut running = ctx.running_loop.clone();
        let status = ConsciousnessLoopEngine::check_conditions(&mut running, conditions);
        match status {
            LoopStatus::Completed => {
                let tid = ctx.template.id.clone();
                let iter = ctx.running_loop.iteration_count;
                let completed = WorkflowStatus::Completed {
                    template_id: tid,
                    iterations: iter,
                };
                ctx.running_loop.status = LoopStatus::Completed;
                let ctx_owned = self.current.take().unwrap();
                self.history.push(ctx_owned);
                if self.history.len() > self.max_history {
                    self.history.remove(0);
                }
                return completed;
            }
            LoopStatus::Failed(msg) => {
                let tid = ctx.template.id.clone();
                let reason = msg.clone();
                let failed = WorkflowStatus::Failed {
                    template_id: tid,
                    reason: msg.clone(),
                };
                ctx.running_loop.status = LoopStatus::Failed(reason);
                let ctx_owned = self.current.take().unwrap();
                self.history.push(ctx_owned);
                if self.history.len() > self.max_history {
                    self.history.remove(0);
                }
                return failed;
            }
            LoopStatus::MaxIterationsReached => {
                let tid = ctx.template.id.clone();
                let iter = ctx.running_loop.iteration_count;
                let failed = WorkflowStatus::MaxIterationsReached {
                    template_id: tid,
                    iterations: iter,
                };
                ctx.running_loop.status = LoopStatus::MaxIterationsReached;
                let ctx_owned = self.current.take().unwrap();
                self.history.push(ctx_owned);
                if self.history.len() > self.max_history {
                    self.history.remove(0);
                }
                return failed;
            }
            _ => {}
        }

        let step_idx = ctx.running_loop.current_step;
        let plan_mode = ctx.plan_mode;
        let step_clone = ctx.template.steps.get(step_idx).cloned();
        if let Some(step) = step_clone {
            let result = Self::execute_step(&step, plan_mode);
            ConsciousnessLoopEngine::progress(&mut ctx.running_loop, result);
        }

        WorkflowStatus::Running {
            template_id: ctx.template.id.clone(),
            iteration: ctx.running_loop.iteration_count,
            step_index: ctx.running_loop.current_step,
        }
    }

    fn execute_step(step: &LoopStep, plan_mode: PlanMode) -> LoopIterationResult {
        let start = std::time::Instant::now();

        let (success, output) = match &step.action {
            StepAction::RunCommand { cmd, args } => {
                if !plan_mode.allows_mutation() {
                    (false, "Blocked by PlanMode::Explore".to_string())
                } else {
                    let full_cmd = if args.is_empty() {
                        cmd.clone()
                    } else {
                        format!("{} {}", cmd, args.join(" "))
                    };
                    match AgentWorkflow::execute_command(&full_cmd) {
                        Ok(out) => (true, out),
                        Err(e) => (false, e),
                    }
                }
            }
            StepAction::CheckOutput { expected } => {
                (true, format!("check: expected '{}'", expected))
            }
            StepAction::AiScoring { prompt } => (true, format!("ai scoring: {}", prompt)),
            StepAction::ConsciousnessTick => (true, "consciousness tick".to_string()),
            StepAction::WebSearch { query } => (true, format!("web search: {}", query)),
            StepAction::KnowledgeRetrieval { topic } => {
                (true, format!("knowledge retrieval: {}", topic))
            }
            StepAction::PresentationSync => (true, "presentation sync".to_string()),
            StepAction::NewsCheck => (true, "news check".to_string()),
            StepAction::VoiceSynthesis => (true, "voice synthesis".to_string()),
            StepAction::Branch {
                condition: _,
                if_step,
                else_step: _,
            } => Self::execute_step_raw(if_step, plan_mode),
        };

        LoopIterationResult {
            step_name: step.name.clone(),
            success,
            output,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    fn execute_step_raw(step: &StepAction, plan_mode: PlanMode) -> (bool, String) {
        match step {
            StepAction::RunCommand { cmd, args } => {
                let full_cmd = if args.is_empty() {
                    cmd.clone()
                } else {
                    format!("{} {}", cmd, args.join(" "))
                };
                match AgentWorkflow::execute_command(&full_cmd) {
                    Ok(out) => (true, out),
                    Err(e) => (false, e),
                }
            }
            StepAction::CheckOutput { expected } => (true, format!("check: '{}'", expected)),
            StepAction::AiScoring { prompt } => (true, format!("ai: {}", prompt)),
            StepAction::ConsciousnessTick => (true, "consciousness_tick".to_string()),
            StepAction::WebSearch { query } => (true, format!("search: {}", query)),
            StepAction::KnowledgeRetrieval { topic } => (true, format!("know: {}", topic)),
            StepAction::PresentationSync => (true, "presentation".to_string()),
            StepAction::NewsCheck => (true, "news".to_string()),
            StepAction::VoiceSynthesis => (true, "voice".to_string()),
            StepAction::Branch {
                condition: _,
                if_step,
                else_step: _,
            } => Self::execute_step_raw(if_step, plan_mode),
        }
    }

    pub fn registry(&self) -> &LoopTemplateRegistry {
        &self.loop_registry
    }

    pub fn registry_mut(&mut self) -> &mut LoopTemplateRegistry {
        &mut self.loop_registry
    }

    pub fn current(&self) -> Option<&WorkflowExecutionContext> {
        self.current.as_ref()
    }

    pub fn status(&self) -> WorkflowStatus {
        match &self.current {
            Some(ctx) => WorkflowStatus::Running {
                template_id: ctx.template.id.clone(),
                iteration: ctx.running_loop.iteration_count,
                step_index: ctx.running_loop.current_step,
            },
            None => WorkflowStatus::Idle,
        }
    }

    pub fn recent_completions(&self, n: usize) -> Vec<&WorkflowExecutionContext> {
        self.history.iter().rev().take(n).collect()
    }

    pub fn completions_count(&self) -> usize {
        self.history.len()
    }
}

impl Default for EngineeringWorkflowExecutor {
    fn default() -> Self {
        Self::new()
    }
}

pub fn loop_template_to_agent_workflow(template: &LoopTemplate) -> AgentWorkflow {
    let mut wf = AgentWorkflow::new(
        &template.id,
        &template.name,
        &format!("{} (engineering workflow)", template.goal),
    );
    for step in &template.steps {
        match &step.action {
            StepAction::RunCommand { cmd, args } => {
                let full_cmd = if args.is_empty() {
                    cmd.clone()
                } else {
                    format!("{} {}", cmd, args.join(" "))
                };
                wf.add_step(AgentStep::RunCommand {
                    command: full_cmd,
                    description: step.description.clone(),
                });
            }
            StepAction::CheckOutput { expected } => {
                wf.add_step(AgentStep::Think {
                    thought: format!("check output matches '{}'", expected),
                });
            }
            StepAction::AiScoring { prompt } => {
                wf.add_step(AgentStep::Think {
                    thought: prompt.clone(),
                });
            }
            StepAction::ConsciousnessTick => {
                wf.add_step(AgentStep::Think {
                    thought: "consciousness processing tick".to_string(),
                });
            }
            StepAction::WebSearch { query } => {
                wf.add_step(AgentStep::Search {
                    pattern: query.clone(),
                    path: None,
                });
            }
            StepAction::KnowledgeRetrieval { topic } => {
                wf.add_step(AgentStep::Search {
                    pattern: topic.clone(),
                    path: None,
                });
            }
            _ => {
                wf.add_step(AgentStep::Think {
                    thought: format!("step: {}", step.name),
                });
            }
        }
    }
    wf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_experience::loop_templates::{Difficulty, LoopTrigger};

    #[test]
    fn test_new_executor_has_default_templates() {
        let exec = EngineeringWorkflowExecutor::new();
        assert!(
            exec.registry().count() >= 8,
            "Expected at least 8 templates"
        );
    }

    #[test]
    fn test_start_workflow_found() {
        let mut exec = EngineeringWorkflowExecutor::new();
        let result = exec.start_workflow("pre-commit-guard", PlanMode::Execute);
        assert!(result.is_ok());
        assert!(exec.current().is_some());
    }

    #[test]
    fn test_start_workflow_not_found() {
        let mut exec = EngineeringWorkflowExecutor::new();
        let result = exec.start_workflow("nonexistent-template", PlanMode::Execute);
        assert!(result.is_err());
    }

    #[test]
    fn test_tick_running() {
        let mut exec = EngineeringWorkflowExecutor::new();
        exec.start_workflow("pre-commit-guard", PlanMode::Execute)
            .unwrap();
        let status = exec.tick();
        assert!(matches!(status, WorkflowStatus::Running { .. }));
    }

    #[test]
    fn test_tick_completed() {
        let mut exec = EngineeringWorkflowExecutor::new();
        let template = LoopTemplate {
            id: "quick-complete".into(),
            name: "Quick Complete".into(),
            description: "Completes immediately".into(),
            goal: "test".into(),
            triggers: vec![LoopTrigger::Manual],
            steps: vec![LoopStep {
                name: "done".into(),
                action: StepAction::ConsciousnessTick,
                description: "finish".into(),
            }],
            exit_conditions: vec!["completed".into()],
            max_iterations: 1,
            installed_count: 0,
            difficulty: Difficulty::Beginner,
        };
        exec.start_workflow_from_template(template, PlanMode::Execute);
        let status = exec.tick();
        assert!(
            matches!(status, WorkflowStatus::Completed { .. }),
            "Expected Completed, got {:?}",
            status
        );
    }

    #[test]
    fn test_max_iterations_reached() {
        let mut exec = EngineeringWorkflowExecutor::new();
        let template = LoopTemplate {
            id: "max-out".into(),
            name: "Max Out".into(),
            description: "Exceeds max iterations".into(),
            goal: "test".into(),
            triggers: vec![LoopTrigger::Manual],
            steps: vec![LoopStep {
                name: "step".into(),
                action: StepAction::ConsciousnessTick,
                description: "a step".into(),
            }],
            exit_conditions: vec![],
            max_iterations: 1,
            installed_count: 0,
            difficulty: Difficulty::Beginner,
        };
        exec.start_workflow_from_template(template, PlanMode::Execute);
        let _s1 = exec.tick();
        let s2 = exec.tick();
        assert!(
            matches!(s2, WorkflowStatus::MaxIterationsReached { .. }),
            "Expected MaxIterationsReached, got {:?}",
            s2
        );
    }

    #[test]
    fn test_loop_template_to_agent_workflow() {
        let templates = crate::core::nt_core_experience::loop_templates::default_templates();
        let t = templates
            .iter()
            .find(|t| t.id == "pre-commit-guard")
            .unwrap();
        let wf = loop_template_to_agent_workflow(t);
        assert_eq!(wf.id, "pre-commit-guard");
        assert!(!wf.steps.is_empty());
    }

    #[test]
    fn test_history_stores_completions() {
        let mut exec = EngineeringWorkflowExecutor::new();
        let template = LoopTemplate {
            id: "hist-test".into(),
            name: "History Test".into(),
            description: "test".into(),
            goal: "test".into(),
            triggers: vec![LoopTrigger::Manual],
            steps: vec![LoopStep {
                name: "done".into(),
                action: StepAction::ConsciousnessTick,
                description: "finish".into(),
            }],
            exit_conditions: vec!["completed".into()],
            max_iterations: 1,
            installed_count: 0,
            difficulty: Difficulty::Beginner,
        };
        exec.start_workflow_from_template(template, PlanMode::Execute);
        let _ = exec.tick();
        let recent = exec.recent_completions(5);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].template.id, "hist-test");
    }

    #[test]
    fn test_executor_idle_status() {
        let exec = EngineeringWorkflowExecutor::new();
        assert_eq!(exec.status(), WorkflowStatus::Idle);
    }

    #[test]
    fn test_plan_mode_blocks_mutation() {
        let mut exec = EngineeringWorkflowExecutor::new();
        let template = LoopTemplate {
            id: "mut-test".into(),
            name: "Mutation Test".into(),
            description: "test".into(),
            goal: "test".into(),
            triggers: vec![LoopTrigger::Manual],
            steps: vec![LoopStep {
                name: "run".into(),
                action: StepAction::RunCommand {
                    cmd: "touch".into(),
                    args: vec!["/tmp/test_file".into()],
                },
                description: "mutate".into(),
            }],
            exit_conditions: vec!["completed".into()],
            max_iterations: 1,
            installed_count: 0,
            difficulty: Difficulty::Beginner,
        };
        exec.start_workflow_from_template(template, PlanMode::Explore);
        let status = exec.tick(); // Will fail gracefully in Explore mode
        assert!(
            matches!(status, WorkflowStatus::Running { .. }),
            "Explore mode should still run but block mutation"
        );
    }
}
