use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

use super::bus::AgentCommunicationBus;
use super::harness::HarnessRegistry;
use super::message::{AgentId, AgentRole, AgentStatus};
use super::sub_agent::{
    IsolationStrategy, LeadAgentPlan, RecoveryStrategy, SubAgentCapability, SubAgentConfig,
    SubAgentResult, SubAgentRuntime,
};
use super::task_list::SharedTaskList;

static NEXT_PLAN_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadAgentConfig {
    pub max_subagents: usize,
    pub max_parallel: usize,
    pub default_timeout_secs: u64,
    pub plan_effort: PlanEffort,
}

impl Default for LeadAgentConfig {
    fn default() -> Self {
        Self {
            max_subagents: 16,
            max_parallel: 8,
            default_timeout_secs: 600,
            plan_effort: PlanEffort::Balanced,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanEffort {
    Draft,
    Balanced,
    Deep,
}

impl PlanEffort {
    pub fn name(&self) -> &'static str {
        match self {
            PlanEffort::Draft => "draft",
            PlanEffort::Balanced => "balanced",
            PlanEffort::Deep => "deep",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadAgentState {
    pub current_goal: Option<String>,
    pub current_plan: Option<LeadAgentPlan>,
    pub active_subagents: Vec<SubAgentSummary>,
    pub completed_tasks: Vec<usize>,
    pub results: Vec<SubAgentResult>,
    #[serde(skip)]
    pub start_time: Option<Instant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentSummary {
    pub id: usize,
    pub agent_id: AgentId,
    pub task: String,
    pub capability: SubAgentCapability,
    pub status: SubAgentTaskStatus,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubAgentTaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Blocked,
}

pub struct LeadAgent {
    pub config: LeadAgentConfig,
    pub state: LeadAgentState,
    pub bus: AgentCommunicationBus,
    pub harness_registry: HarnessRegistry,
    subagents: HashMap<usize, SubAgentRuntime>,
    plan_count: u64,
    task_list: Option<SharedTaskList>,
}

impl LeadAgent {
    pub fn new(config: LeadAgentConfig) -> Self {
        Self {
            config,
            state: LeadAgentState {
                current_goal: None,
                current_plan: None,
                active_subagents: Vec::new(),
                completed_tasks: Vec::new(),
                results: Vec::new(),
                start_time: None,
            },
            bus: AgentCommunicationBus::new(1024),
            harness_registry: HarnessRegistry::new(),
            subagents: HashMap::new(),
            plan_count: 0,
            task_list: None,
        }
    }

    pub fn plan(&mut self, goal: &str) -> LeadAgentPlan {
        self.plan_count += 1;
        let _plan_id = NEXT_PLAN_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let keywords: Vec<String> = goal.split_whitespace().map(|s| s.to_lowercase()).collect();

        let harness = self
            .harness_registry
            .select(&keywords, self.config.plan_effort.name());
        let mut plan = harness.orchestrate(goal, &keywords);
        plan.strategy = harness.kind().name().to_string();

        self.state.current_goal = Some(goal.to_string());
        self.state.current_plan = Some(plan.clone());
        plan
    }

    pub fn spawn_ready_subagents(&mut self) -> Vec<usize> {
        let plan = match &self.state.current_plan {
            Some(p) => p.clone(),
            None => return vec![],
        };
        let ready = plan.decomposition.ready_tasks(&self.state.completed_tasks);
        let running: usize = self.subagents.len();
        let capacity = self.config.max_parallel.saturating_sub(running);
        let to_spawn: Vec<usize> = ready.into_iter().take(capacity).collect();

        for &task_id in &to_spawn {
            if let Some(spec) = plan
                .decomposition
                .sub_tasks
                .iter()
                .find(|s| s.id == task_id)
            {
                let agent_id = AgentId::with_random_instance(
                    &format!("sub-{}", spec.capability.label()),
                    "1.0",
                );
                let role = match spec.capability {
                    SubAgentCapability::Coder => AgentRole::Specialist,
                    SubAgentCapability::Reviewer => AgentRole::Critic,
                    SubAgentCapability::Researcher => AgentRole::Researcher,
                    SubAgentCapability::Tester => AgentRole::Verifier,
                    SubAgentCapability::SecurityAuditor => AgentRole::Critic,
                    SubAgentCapability::Planner => AgentRole::Coordinator,
                    SubAgentCapability::Integrator => AgentRole::Synthesizer,
                    SubAgentCapability::Documenter => AgentRole::Specialist,
                    SubAgentCapability::Visualizer => AgentRole::Specialist,
                    SubAgentCapability::InfraOps => AgentRole::Specialist,
                };
                let config = SubAgentConfig {
                    agent_id: agent_id.clone(),
                    role,
                    capability: spec.capability,
                    max_context_tokens: 8192,
                    max_iterations: 15,
                    timeout_secs: self.config.default_timeout_secs,
                    allowed_handlers: vec![],
                    isolation: IsolationStrategy::Local,
                    recovery: RecoveryStrategy::Retry { retry_limit: 2 },
                    sandbox_root: None,
                };
                let runtime = SubAgentRuntime::new(config, &spec.description, &mut self.bus);
                let _ = self.bus.register_agent(agent_id.clone(), AgentStatus::Idle);
                self.subagents.insert(task_id, runtime);
                self.state.active_subagents.push(SubAgentSummary {
                    id: task_id,
                    agent_id,
                    task: spec.description.clone(),
                    capability: spec.capability,
                    status: SubAgentTaskStatus::Running,
                    duration_ms: None,
                });
            }
        }
        to_spawn
    }

    pub fn tick_subagents(&mut self) -> Vec<SubAgentResult> {
        let mut completed = Vec::new();
        let mut to_remove = Vec::new();
        for (&task_id, runtime) in self.subagents.iter_mut() {
            let result = runtime.step(&mut self.bus);
            match result {
                super::sub_agent::SubAgentStepResult::Continue => {}
                _ => {
                    if let Some(res) = runtime.result.clone() {
                        completed.push(res);
                    } else {
                        completed.push(runtime.fail("Subagent stopped unexpectedly".into()));
                    }
                    to_remove.push(task_id);
                }
            }
        }
        for task_id in to_remove {
            let runtime = self.subagents.remove(&task_id);
            if let Some(r) = runtime {
                if let Some(summary) = self
                    .state
                    .active_subagents
                    .iter_mut()
                    .find(|s| s.id == task_id)
                {
                    summary.status = SubAgentTaskStatus::Completed;
                    summary.duration_ms = Some(r.start_time.elapsed().as_millis() as u64);
                }
            }
            self.state.completed_tasks.push(task_id);
        }
        for result in &completed {
            self.state.results.push(result.clone());
        }
        completed
    }

    pub fn all_tasks_complete(&self) -> bool {
        self.state.current_plan.as_ref().map_or(true, |plan| {
            plan.decomposition
                .all_completed(&self.state.completed_tasks)
        })
    }

    pub fn execute_goal(&mut self, goal: &str) -> Vec<SubAgentResult> {
        let plan = self.plan(goal);
        let max_rounds = 100;
        let mut failed_tasks: HashSet<usize> = HashSet::new();
        let mut retry_counts: HashMap<usize, u32> = HashMap::new();

        for round in 0..max_rounds {
            if self.all_tasks_complete() {
                break;
            }
            self.spawn_ready_subagents();
            let completed = self.tick_subagents();

            // Auto-recovery: detect failures and re-spawn
            for result in &completed {
                if !result.success {
                    let task_id = self
                        .state
                        .active_subagents
                        .iter()
                        .find(|s| s.agent_id == result.agent_id)
                        .map(|s| s.id);
                    if let Some(tid) = task_id {
                        let retries = retry_counts.entry(tid).or_insert(0);
                        *retries += 1;
                        let spec = plan.decomposition.sub_tasks.iter().find(|s| s.id == tid);
                        let max_retry = match spec.map(|s| s.recovery) {
                            Some(RecoveryStrategy::Retry { retry_limit }) => retry_limit,
                            Some(RecoveryStrategy::Escalate { retry_limit, .. }) => retry_limit,
                            Some(RecoveryStrategy::CircuitBreaker { .. }) => 3,
                            _ => 1,
                        };
                        if *retries <= max_retry {
                            let escalated_cap = match spec {
                                Some(s)
                                    if matches!(s.recovery, RecoveryStrategy::Escalate { .. }) =>
                                {
                                    SubAgentCapability::SecurityAuditor
                                }
                                _ => spec.map_or(SubAgentCapability::Coder, |s| s.capability),
                            };
                            let retry_config = SubAgentConfig {
                                agent_id: AgentId::with_random_instance(
                                    &format!("retry-{}", tid),
                                    "1.0",
                                ),
                                capability: escalated_cap,
                                ..SubAgentConfig::default()
                            };
                            let runtime = SubAgentRuntime::new(
                                retry_config,
                                &format!(
                                    "[Retry {}/{}] {}",
                                    retries,
                                    max_retry,
                                    spec.map_or("unknown", |s| &s.description)
                                ),
                                &mut self.bus,
                            );
                            self.subagents.insert(tid, runtime);
                            continue;
                        }
                        failed_tasks.insert(tid);
                    }
                }
            }

            if round > 5 && self.subagents.is_empty() && !self.all_tasks_complete() {
                break;
            }
        }

        // Fill failed tasks with error results
        if !self.all_tasks_complete() {
            if let Some(plan) = &self.state.current_plan {
                for task in &plan.decomposition.sub_tasks {
                    if !self.state.completed_tasks.contains(&task.id) {
                        self.state.results.push(SubAgentResult {
                            agent_id: AgentId::with_random_instance("failed", "1.0"),
                            task_description: task.description.clone(),
                            output: String::new(),
                            artifacts: vec![],
                            confidence: 0.0,
                            iterations_used: 0,
                            duration_ms: 0,
                            success: false,
                            error: Some(format!("Task {} failed after retries", task.id)),
                        });
                    }
                }
            }
        }

        self.state.results.clone()
    }

    pub fn execute_goal_with_task_list(&mut self, goal: &str) -> Vec<SubAgentResult> {
        let plan = self.plan(goal);
        let mut task_list = SharedTaskList::new();

        let dep_map: HashMap<usize, Vec<usize>> = {
            let mut map: HashMap<usize, Vec<usize>> = HashMap::new();
            for (from, to) in &plan.decomposition.dependency_graph {
                map.entry(*to).or_default().push(*from);
            }
            map
        };

        for spec in &plan.decomposition.sub_tasks {
            let deps = dep_map.get(&spec.id).cloned().unwrap_or_default();
            task_list.add_task(&spec.description, spec.capability, deps);
        }
        self.task_list = Some(task_list);

        let task_list = match self.task_list.as_mut() {
            Some(tl) => tl,
            None => {
                log::error!("[lead_agent] task_list not initialized after assignment");
                return vec![];
            }
        };

        let role_for = |cap: SubAgentCapability| -> AgentRole {
            match cap {
                SubAgentCapability::Coder => AgentRole::Specialist,
                SubAgentCapability::Reviewer => AgentRole::Critic,
                SubAgentCapability::Researcher => AgentRole::Researcher,
                SubAgentCapability::Tester => AgentRole::Verifier,
                SubAgentCapability::SecurityAuditor => AgentRole::Critic,
                SubAgentCapability::Planner => AgentRole::Coordinator,
                SubAgentCapability::Integrator => AgentRole::Synthesizer,
                SubAgentCapability::Documenter => AgentRole::Specialist,
                SubAgentCapability::Visualizer => AgentRole::Specialist,
                SubAgentCapability::InfraOps => AgentRole::Specialist,
            }
        };

        let mut workers: Vec<(AgentId, SubAgentCapability)> = Vec::new();
        let mut seen = HashSet::new();
        for spec in &plan.decomposition.sub_tasks {
            if seen.insert(spec.capability) {
                let agent_id = AgentId::with_random_instance(
                    &format!("worker-{}", spec.capability.label()),
                    "1.0",
                );
                let _ = self.bus.register_agent(agent_id.clone(), AgentStatus::Idle);
                workers.push((agent_id, spec.capability));
            }
        }

        let start = Instant::now();
        let timeout = std::time::Duration::from_secs(self.config.default_timeout_secs);
        let max_cycles = 200;
        let mut runtimes: HashMap<usize, SubAgentRuntime> = HashMap::new();
        let mut results: Vec<SubAgentResult> = Vec::new();

        for _cycle in 0..max_cycles {
            if start.elapsed() > timeout {
                break;
            }
            if task_list.all_complete() {
                break;
            }

            let active = runtimes.len();
            let capacity = self.config.max_parallel.saturating_sub(active);
            if capacity > 0 {
                for (agent_id, capability) in &workers {
                    if runtimes.len() >= self.config.max_parallel {
                        break;
                    }
                    if let Some(task) = task_list.claim_next(agent_id.clone(), *capability) {
                        let config = SubAgentConfig {
                            agent_id: agent_id.clone(),
                            role: role_for(*capability),
                            capability: *capability,
                            max_context_tokens: 8192,
                            max_iterations: 15,
                            timeout_secs: self.config.default_timeout_secs,
                            allowed_handlers: vec![],
                            isolation: IsolationStrategy::Local,
                            recovery: RecoveryStrategy::Retry { retry_limit: 2 },
                            sandbox_root: None,
                        };
                        let runtime =
                            SubAgentRuntime::new(config, &task.description, &mut self.bus);
                        runtimes.insert(task.id, runtime);
                    }
                }
            }

            let mut to_finish: Vec<(usize, SubAgentResult)> = Vec::new();
            for (&task_id, runtime) in runtimes.iter_mut() {
                match runtime.step(&mut self.bus) {
                    super::sub_agent::SubAgentStepResult::Continue => {}
                    _ => {
                        let result = match runtime.result.clone() {
                            Some(r) => r,
                            None => runtime.fail("Subagent stopped unexpectedly".into()),
                        };
                        to_finish.push((task_id, result));
                    }
                }
            }

            for (task_id, result) in to_finish {
                if result.success {
                    task_list.complete_task(task_id, result.output.clone());
                } else {
                    task_list.fail_task(task_id, "See error field");
                }
                results.push(result);
                runtimes.remove(&task_id);
            }

            if runtimes.is_empty() && task_list.unclaimed_count() == 0 && !task_list.all_complete()
            {
                break;
            }
        }

        for (task_id, mut runtime) in runtimes.drain() {
            let result = runtime.fail("Task timed out before completion".to_string());
            task_list.fail_task(task_id, "Task timed out before completion");
            results.push(result);
        }

        let completed_descriptions: HashSet<String> =
            results.iter().map(|r| r.task_description.clone()).collect();
        for spec in &plan.decomposition.sub_tasks {
            if !completed_descriptions.contains(&spec.description) {
                results.push(SubAgentResult {
                    agent_id: AgentId::with_random_instance("failed", "1.0"),
                    task_description: spec.description.clone(),
                    output: String::new(),
                    artifacts: vec![],
                    confidence: 0.0,
                    iterations_used: 0,
                    duration_ms: 0,
                    success: false,
                    error: Some(format!(
                        "Task '{}' never completed via task list",
                        spec.description
                    )),
                });
            }
        }

        results
    }

    /// Spawn sub-agents from a CSV batch definition.
    /// Format (header): `goal,kind,capability,timeout_secs`
    /// kind: Explorer | Worker | Planner
    /// capability: Coder | Reviewer | Researcher | Tester | etc.
    pub fn spawn_batch_from_csv(&mut self, csv_str: &str) -> Vec<String> {
        use super::agent_kind::AgentKind;
        let mut spawned = Vec::new();
        for (line_idx, line) in csv_str.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || line_idx == 0 && trimmed.starts_with("goal,") {
                continue; // skip header
            }
            let cols: Vec<&str> = trimmed.split(',').collect();
            if cols.len() < 3 {
                continue;
            }
            let goal = cols[0].trim();
            let kind_str = cols[1].trim().to_lowercase();
            let cap_str = cols[2].trim();
            let timeout = cols
                .get(3)
                .and_then(|s| s.trim().parse::<u64>().ok())
                .unwrap_or(120);

            let kind = match kind_str.as_str() {
                "explorer" => AgentKind::Explorer,
                "worker" => AgentKind::Worker,
                "planner" => AgentKind::Planner,
                _ => AgentKind::Worker,
            };
            let capability = match cap_str.to_lowercase().as_str() {
                "coder" => SubAgentCapability::Coder,
                "reviewer" => SubAgentCapability::Reviewer,
                "researcher" => SubAgentCapability::Researcher,
                "tester" => SubAgentCapability::Tester,
                "integrator" => SubAgentCapability::Integrator,
                "documenter" => SubAgentCapability::Documenter,
                "planner" => SubAgentCapability::Planner,
                "security_auditor" | "security" => SubAgentCapability::SecurityAuditor,
                "infra_ops" | "infra" => SubAgentCapability::InfraOps,
                "visualizer" => SubAgentCapability::Visualizer,
                _ => SubAgentCapability::Coder,
            };
            let agent_id =
                AgentId::with_random_instance(&format!("batch-{}-{}", cap_str, line_idx), "1.0");
            let config = SubAgentConfig {
                agent_id: agent_id.clone(),
                role: match capability {
                    SubAgentCapability::Coder => AgentRole::Specialist,
                    SubAgentCapability::Reviewer => AgentRole::Critic,
                    SubAgentCapability::Researcher => AgentRole::Researcher,
                    SubAgentCapability::Tester => AgentRole::Verifier,
                    SubAgentCapability::SecurityAuditor => AgentRole::Critic,
                    SubAgentCapability::Planner => AgentRole::Coordinator,
                    SubAgentCapability::Integrator => AgentRole::Synthesizer,
                    SubAgentCapability::Documenter => AgentRole::Specialist,
                    SubAgentCapability::Visualizer => AgentRole::Specialist,
                    SubAgentCapability::InfraOps => AgentRole::Specialist,
                },
                capability,
                max_context_tokens: kind.context_budget(),
                max_iterations: 15,
                timeout_secs: timeout,
                allowed_handlers: vec![],
                isolation: IsolationStrategy::Local,
                recovery: RecoveryStrategy::Retry { retry_limit: 2 },
                sandbox_root: None,
            };
            let runtime = SubAgentRuntime::new(config, goal, &mut self.bus);
            let _ = self.bus.register_agent(agent_id.clone(), AgentStatus::Idle);
            let task_id = NEXT_PLAN_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as usize;
            self.subagents.insert(task_id, runtime);
            self.state.active_subagents.push(SubAgentSummary {
                id: task_id,
                agent_id: agent_id.clone(),
                task: goal.to_string(),
                capability,
                status: SubAgentTaskStatus::Running,
                duration_ms: None,
            });
            spawned.push(agent_id.instance_id.to_string());
        }
        spawned
    }

    pub fn active_count(&self) -> usize {
        self.subagents.len()
    }

    pub fn summary(&self) -> String {
        let total = self.state.active_subagents.len();
        let done = self.state.completed_tasks.len();
        let running = self.subagents.len();
        let failed = self.state.results.iter().filter(|r| !r.success).count();
        format!(
            "LeadAgent: {} tasks, {} done, {} running, {} failed, {} results",
            total,
            done,
            running,
            failed,
            self.state.results.len()
        )
    }
}
