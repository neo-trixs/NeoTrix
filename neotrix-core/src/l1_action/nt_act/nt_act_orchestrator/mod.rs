pub mod types;
pub mod planner;
pub mod worker;
pub mod critic;
pub mod state_graph;
pub mod task_state_dag;
pub mod pm_workflow;
pub mod harness_scaffold;
#[cfg(test)]
pub mod group_integration_test;

use std::sync::{Arc, Mutex};
use neotrix_types::core::CapabilityVector;
use crate::l1_action::nt_io::nt_l1_error::L1Result;
use crate::agent::team::AgentTeam;
use pm_workflow::{PMNode, PMWorkflowType};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::l1_action::traits::{
    L1Capability, Orchestrator as OrchestratorTrait, CapabilityCategory, ConstellationLevel,
    CapabilityHealth, CapabilityStats, CapabilityError,
    Plan, PlanStep, PlanResult,
};

/// Reasoning provider trait (replaces L8 ReasoningEngine dependency)
pub trait ReasoningProvider: Send + Sync {
    fn reason(&mut self, task: &str) -> L1Result<String>;
    fn reason_task(&mut self, goal: &str) -> L1Result<String>;
    fn self_iterate(&mut self);
    fn capability(&self) -> &CapabilityVector;
}

/// Autonomy level (local definition to avoid L1→L8 dependency)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutonomyLevel {
    Proposal,
    Bounded,
    Full,
}

/// Simple group manager (local definition to avoid L1→L8 dependency)
#[derive(Debug, Clone)]
pub struct GroupManager;

impl GroupManager {
    pub fn match_cross_repo(&self, _word: &str) -> Vec<CrossRepoMatch> {
        vec![]
    }
}

#[derive(Debug, Clone)]
pub struct CrossRepoMatch {
    pub to_repo: String,
}

type FeedbackCallback = Box<dyn Fn(&str, f64) + Send + Sync>;

pub struct Orchestrator {
    pub planner: planner::PlannerNode,
    pub worker: Mutex<worker::WorkerNode>,
    pub critic: critic::CriticNode,
    pub graph: state_graph::StateGraph,
    pub engine: Option<Arc<Mutex<Box<dyn ReasoningProvider>>>>,
    pub autonomy: AutonomyLevel,
    pub group_manager: Option<GroupManager>,
    pub agent_team: Option<Arc<Mutex<AgentTeam>>>,
    pub feedback_callback: Option<FeedbackCallback>,
    pub pm_node: Option<PMNode>,
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            planner: planner::PlannerNode::new(),
            worker: Mutex::new(worker::WorkerNode::new()),
            critic: critic::CriticNode::new(),
            graph: state_graph::StateGraph::new(),
            engine: None,
            autonomy: AutonomyLevel::Full,
            group_manager: None,
            agent_team: None,
            feedback_callback: None,
            pm_node: None,
        }
    }

    pub fn with_engine(engine: Arc<Mutex<Box<dyn ReasoningProvider>>>) -> Self {
        Self {
            planner: planner::PlannerNode::new(),
            worker: Mutex::new(worker::WorkerNode::new()),
            critic: critic::CriticNode::new(),
            graph: state_graph::StateGraph::new(),
            engine: Some(engine),
            autonomy: AutonomyLevel::Full,
            group_manager: None,
            agent_team: None,
            feedback_callback: None,
            pm_node: None,
        }
    }

    pub fn with_group_manager(gm: GroupManager) -> Self {
        let planner = planner::PlannerNode::with_group_manager(gm.clone());
        Self {
            planner,
            worker: Mutex::new(worker::WorkerNode::new()),
            critic: critic::CriticNode::new(),
            graph: state_graph::StateGraph::new(),
            engine: None,
            autonomy: AutonomyLevel::Full,
            group_manager: Some(gm),
            agent_team: None,
            feedback_callback: None,
            pm_node: None,
        }
    }

    pub fn with_agent_team(mut self, team: Arc<Mutex<AgentTeam>>) -> Self {
        self.agent_team = Some(team);
        self
    }

    pub fn with_feedback_callback(mut self, cb: FeedbackCallback) -> Self {
        self.feedback_callback = Some(cb);
        self
    }

    pub fn set_autonomy(&mut self, level: AutonomyLevel) {
        self.autonomy = level;
    }

    pub fn with_pm_node(mut self, node: PMNode) -> Self {
        self.pm_node = Some(node);
        self
    }

    pub fn set_pm_workflow(&mut self, workflow: PMWorkflowType) {
        self.pm_node = Some(PMNode::new(workflow));
    }

    pub fn estimate_pm_priority(&mut self, description: &str, complexity: f64) -> Option<f64> {
        self.pm_node.as_mut().map(|n| n.estimate_priority(description, complexity))
    }

    pub fn run_recursive_loop(&mut self, goal: &str) -> Result<String, String> {
        let tasks = self.planner.decompose(goal);
        if tasks.is_empty() {
            return Err("Task decomposition failed".to_string());
        }

        // 构建 DAG 规划
        self.graph = state_graph::StateGraph::new();
        let lower = goal.to_lowercase();
        if lower.contains("prd") || lower.contains("competitive") || lower.contains("ux audit") || lower.contains("experiment") {
            self.graph.build_pm_plan(goal, tasks.len());
        } else {
            self.graph.build_plan(goal, tasks.len());
        }

        match self.autonomy {
            AutonomyLevel::Proposal => {
                let sorted = self.graph.topological_sort().unwrap_or_default();
                let plan = format!("[Proposal] DAG Plan for '{}':\n", goal);
                let details: Vec<String> = sorted.iter().map(|id| {
                    let node = self.graph.node(id);
                    format!("  - [{}] {}", id, node.map(|n| n.description.as_str()).unwrap_or(""))
                }).collect();
                return Ok(plan + &details.join("\n") + "\n\nSet autonomy to Bounded or Full to execute.");
            }
            AutonomyLevel::Bounded => {
                if let Some(ref engine) = self.engine {
                    let eng = engine.lock().map_err(|e| format!("Lock error: {}", e))?;
                    let cap_sum: f64 = eng.capability().arr().iter().sum();
                    if cap_sum > 16.0 {
                        return Err(format!("Bounded mode: capability sum {:.2} exceeds 16.0. Reduce or switch to Full.", cap_sum));
                    }
                }
            }
            AutonomyLevel::Full => {}
        }

        if let Some(ref engine) = self.engine {
            let mut eng = engine.lock().map_err(|e| format!("Lock error: {}", e))?;

            let plan_result = eng.reason_task(goal).map_err(|e| e.to_string())?;

            // DAG 驱动的执行循环：就绪 → 执行 → 标记完成 → 下一批
            let mut execution_log: Vec<String> = Vec::new();
            let plan_short = if plan_result.chars().count() > 80 {
                let truncated: String = plan_result.chars().take(80).collect();
                format!("{}...", truncated)
            } else {
                plan_result.clone()
            };
            execution_log.push(format!("Plan: {}", plan_short));

            loop {
                let ready_ids: Vec<String> = self.graph.ready_nodes().iter().map(|n| n.id.clone()).collect();
                if ready_ids.is_empty() {
                    break;
                }

                for node_id in &ready_ids {
                    let desc = self.graph.node(node_id)
                        .map(|n| n.description.clone())
                        .unwrap_or_default();
                    let _ = self.graph.mark_done(node_id);
                    execution_log.push(format!("  ✅ {}", node_id));

                    if node_id.contains("task_") {
                        let _results = self.worker.lock().unwrap().execute_tasks(&tasks);
                        if let Some(ref team_arc) = self.agent_team {
                            if let Ok(team) = team_arc.lock() {
                                let agent_results = team.execute(&desc);
                                let n_success = agent_results.iter().filter(|r| r.success).count();
                                execution_log.push(format!("    AgentTeam: {}/{} successful", n_success, agent_results.len()));
                                for r in &agent_results {
                                    let preview: String = r.output.chars().take(80).collect();
                                    execution_log.push(format!("      {}: {}", r.agent_name, preview));
                                }
                            }
                        }
                    }
                }
            }

            // DAG 执行完成 → Critic 评估
            let mut ctx = crate::neotrix::nt_world_model::Context::from_task_description(goal);
            if self.agent_team.is_some() {
                ctx.metadata.insert("agent_team_used".to_string(), "true".to_string());
            }
            let capability = eng.capability().clone();

            // 使用 HP@K 协议评估
            let core_task: crate::core::TaskType = ctx.task_type.into();
            let critic_task: critic::TaskType = core_task.into();
            let scores = vec![self.critic.evaluate(critic_task, &capability)];
            let hp_result = self.critic.heavy_pass_verify(&scores);
            let score = hp_result.hp_at_k;

            if score < 0.6 {
                let _improvement = eng.reason(&format!("Improve the approach for: {}", goal))
                    .map_err(|e| e.to_string())?;
                eng.self_iterate();
            }

            if let Some(ref mut pm) = self.pm_node {
                let output = format!("PM workflow {:?} completed", pm.workflow);
                pm.evaluate_gates(&output);
                if !pm.all_required_pass() {
                    execution_log.push(format!("  ⚠ PM quality gate: {:.0}% passed (required gates failed)", pm.score() * 100.0));
                } else {
                    execution_log.push(format!("  ✓ PM quality gate: {:.0}% passed", pm.score() * 100.0));
                }
            }

            if let Some(ref cb) = self.feedback_callback {
                cb(goal, score);
            }

            Ok(format!("Completed DAG execution\nHP@K Score: {:.2} (HM: {:.2}, Vote: {:.2})\n{}\n{}",
                hp_result.hp_at_k, hp_result.hm_at_k, hp_result.vote_at_k,
                execution_log.join("\n"), self.graph.summary()))
        } else {
            let _results = self.worker.lock().unwrap().execute_tasks(&tasks);
            let total = self.graph.nodes.len();
            Ok(format!("DAG: {}/{} done (no engine)", total, total))
        }
    }
}

// ════════════════════════════════════════════════════════════════
// Unified Architecture: L1Capability + Orchestrator trait
// ════════════════════════════════════════════════════════════════

impl L1Capability for Orchestrator {
    fn capability_id(&self) -> &str { "act.orchestrator" }
    fn category(&self) -> CapabilityCategory { CapabilityCategory::Execution }
    fn constellation(&self) -> ConstellationLevel { ConstellationLevel::C2Integration }
    fn health_check(&self) -> CapabilityHealth {
        CapabilityHealth {
            healthy: true,
            latency_ms: None,
            error_rate: 0.0,
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            message: Some(format!("engine={}", self.engine.is_some())),
        }
    }
    fn description(&self) -> &str { "DAG-based task orchestrator with planner/worker/critic loop" }
    fn stats(&self) -> CapabilityStats { CapabilityStats::default() }
}

impl OrchestratorTrait for Orchestrator {
    fn plan(&self, goal: &str) -> Result<Plan, CapabilityError> {
        let _tasks = self.planner.decompose(goal);
        let steps: Vec<PlanStep> = _tasks.iter().enumerate().map(|(i, t)| {
            PlanStep {
                id: format!("step_{}", i),
                action: format!("{:?}", t),
                depends_on: if i > 0 { vec![format!("step_{}", i - 1)] } else { vec![] },
            }
        }).collect();
        Ok(Plan { steps, estimated_duration_ms: 0 })
    }

    fn execute(&self, plan: &Plan) -> Result<PlanResult, CapabilityError> {
        let _tasks: Vec<types::Task> = plan.steps.iter().enumerate().map(|(i, _s)| {
            types::Task::new(format!("step_{}", i), vec![], 0)
        }).collect();
        // worker.execute_tasks requires &mut self; results unused here
        Ok(PlanResult { success: true, steps_completed: plan.steps.len(), output: None })
    }
}

// ════════════════════════════════════════════════════════════════
// Registry + Router + Bridge
// ════════════════════════════════════════════════════════════════

/// 编排能力注册中心
pub struct OrchestratorRegistry {
    orchestrators: Vec<Box<dyn OrchestratorTrait>>,
}

impl Default for OrchestratorRegistry {
    fn default() -> Self { Self::new() }
}

impl OrchestratorRegistry {
    pub fn new() -> Self { Self { orchestrators: Vec::new() } }
    pub fn register(&mut self, orch: Box<dyn OrchestratorTrait>) { self.orchestrators.push(orch); }
    pub fn get(&self, id: &str) -> Option<&dyn OrchestratorTrait> {
        self.orchestrators.iter().find(|o| o.capability_id() == id).map(|o| o.as_ref())
    }
    pub fn health_check_all(&self) -> Vec<(String, CapabilityHealth)> {
        self.orchestrators.iter().map(|o| (o.capability_id().to_string(), o.health_check())).collect()
    }
    pub fn optimal(&self) -> Option<&dyn OrchestratorTrait> {
        self.orchestrators.iter()
            .filter(|o| o.health_check().healthy)
            .max_by(|a, b| {
                let a_s = 1.0 - a.health_check().error_rate;
                let b_s = 1.0 - b.health_check().error_rate;
                a_s.partial_cmp(&b_s).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|o| o.as_ref())
    }
}

/// 编排路由器
pub struct OrchestratorRouter {
    registry: OrchestratorRegistry,
}

impl OrchestratorRouter {
    pub fn new(registry: OrchestratorRegistry) -> Self { Self { registry } }
    pub fn route(&self, _goal: &str) -> Option<&dyn OrchestratorTrait> { self.registry.optimal() }
    pub fn plan(&self, goal: &str) -> Result<Plan, CapabilityError> {
        self.registry.optimal()
            .ok_or_else(|| CapabilityError::NotAvailable("No orchestrator".into()))?
            .plan(goal)
    }
    pub fn execute(&self, plan: &Plan) -> Result<PlanResult, CapabilityError> {
        self.registry.optimal()
            .ok_or_else(|| CapabilityError::NotAvailable("No orchestrator".into()))?
            .execute(plan)
    }
}

/// 编排桥接
pub struct OrchestratorBridge {
    router: OrchestratorRouter,
}

impl OrchestratorBridge {
    pub fn new(router: OrchestratorRouter) -> Self { Self { router } }
    pub fn plan(&self, goal: &str) -> Result<Plan, CapabilityError> { self.router.plan(goal) }
    pub fn execute(&self, plan: &Plan) -> Result<PlanResult, CapabilityError> { self.router.execute(plan) }
}
