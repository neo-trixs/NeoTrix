pub mod adversarial;
pub mod agent_spec;
pub mod critic;
pub mod flow;
pub mod flow_state;
#[cfg(test)]
pub mod group_integration_test;
pub mod guardrail;
pub mod planner;
#[cfg(test)]
pub mod pm_integration_test;
pub mod pm_workflow;
pub mod process;
pub mod state_graph;
pub mod task_spec;
pub mod three_stage;
pub mod topology_router;
pub mod types;
pub mod verification;
pub mod worker;

use crate::agent::AgentTeam;
use crate::neotrix::nt_mind::group_contracts::GroupManager;
use crate::neotrix::nt_mind::reasoning_engine::ReasoningEngine;
use crate::neotrix::nt_mind::self_iterating::pipeline::AutonomyLevel;
use pm_workflow::{PMNode, PMWorkflowType};
use std::sync::{Arc, Mutex};

type FeedbackCallback = Box<dyn Fn(&str, f64) + Send + Sync>;

pub struct Orchestrator {
    pub planner: planner::PlannerNode,
    pub worker: worker::WorkerNode,
    pub critic: critic::CriticNode,
    pub graph: state_graph::StateGraph,
    pub engine: Option<Arc<Mutex<ReasoningEngine>>>,
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
            worker: worker::WorkerNode::new(),
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

    pub fn with_engine(engine: Arc<Mutex<ReasoningEngine>>) -> Self {
        Self {
            planner: planner::PlannerNode::new(),
            worker: worker::WorkerNode::new(),
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
            worker: worker::WorkerNode::new(),
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
        self.pm_node
            .as_mut()
            .map(|n| n.estimate_priority(description, complexity))
    }

    pub fn run_recursive_loop(&mut self, goal: &str) -> Result<String, String> {
        let tasks = self.planner.decompose(goal);
        if tasks.is_empty() {
            return Err("Task decomposition failed".to_string());
        }

        // 构建 DAG 规划
        self.graph = state_graph::StateGraph::new();
        let lower = goal.to_lowercase();
        if lower.contains("prd")
            || lower.contains("competitive")
            || lower.contains("ux audit")
            || lower.contains("experiment")
        {
            self.graph.build_pm_plan(goal, tasks.len());
        } else {
            self.graph.build_plan(goal, tasks.len());
        }

        match self.autonomy {
            AutonomyLevel::Proposal => {
                let sorted = self.graph.topological_sort().unwrap_or_default();
                let plan = format!("[Proposal] DAG Plan for '{}':\n", goal);
                let details: Vec<String> = sorted
                    .iter()
                    .map(|id| {
                        let node = self.graph.node(id);
                        format!(
                            "  - [{}] {}",
                            id,
                            node.map(|n| n.description.as_str()).unwrap_or("")
                        )
                    })
                    .collect();
                return Ok(plan
                    + &details.join("\n")
                    + "\n\nSet autonomy to Bounded or Full to execute.");
            }
            AutonomyLevel::Bounded => {
                if let Some(ref engine) = self.engine {
                    let eng = engine.lock().map_err(|e| format!("Lock error: {}", e))?;
                    let cap_sum: f64 = eng.brain.capability().arr().iter().sum();
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
            let plan_short = if plan_result.len() > 80 {
                format!("{}...", &plan_result[..80])
            } else {
                plan_result.clone()
            };
            execution_log.push(format!("Plan: {}", plan_short));

            loop {
                let ready_ids: Vec<String> = self
                    .graph
                    .ready_nodes()
                    .iter()
                    .map(|n| n.id.clone())
                    .collect();
                if ready_ids.is_empty() {
                    break;
                }

                for node_id in &ready_ids {
                    let desc = self
                        .graph
                        .node(node_id)
                        .map(|n| n.description.clone())
                        .unwrap_or_default();
                    let _ = self.graph.mark_done(node_id);
                    execution_log.push(format!("  ✅ {}", node_id));

                    if node_id.contains("task_") {
                        let _results = self.worker.execute_tasks(&tasks);
                        if let Some(ref team_arc) = self.agent_team {
                            if let Ok(mut team) = team_arc.lock() {
                                let agent_results = team.execute(&desc);
                                let n_success = agent_results.iter().filter(|r| r.success).count();
                                execution_log.push(format!(
                                    "    AgentTeam: {}/{} successful",
                                    n_success,
                                    agent_results.len()
                                ));
                                for r in &agent_results {
                                    let preview: String = r.output.chars().take(80).collect();
                                    execution_log
                                        .push(format!("      {}: {}", r.agent_name, preview));
                                }
                            }
                        }
                    }
                }
            }

            // DAG 执行完成 → Critic 评估
            let mut ctx = crate::neotrix::nt_expert_routing::Context::from_task_description(goal);
            if self.agent_team.is_some() {
                ctx.metadata
                    .insert("agent_team_used".to_string(), "true".to_string());
            }
            let capability = eng.brain.capability().clone();

            // 使用 HP@K 协议评估
            let scores = vec![self.critic.evaluate(ctx.task_type, &capability)];
            let hp_result = self.critic.heavy_pass_verify(&scores);
            let score = hp_result.hp_at_k;

            if score < 0.6 {
                let _improvement = eng
                    .reason(&format!("Improve the approach for: {}", goal))
                    .map_err(|e| e.to_string())?;
                eng.self_iterate();
            }

            if let Some(ref mut pm) = self.pm_node {
                let output = format!("PM workflow {:?} completed", pm.workflow);
                pm.evaluate_gates(&output);
                if !pm.all_required_pass() {
                    execution_log.push(format!(
                        "  ⚠ PM quality gate: {:.0}% passed (required gates failed)",
                        pm.score() * 100.0
                    ));
                } else {
                    execution_log.push(format!(
                        "  ✓ PM quality gate: {:.0}% passed",
                        pm.score() * 100.0
                    ));
                }
            }

            if let Some(ref cb) = self.feedback_callback {
                cb(goal, score);
            }

            Ok(format!(
                "Completed DAG execution\nHP@K Score: {:.2} (HM: {:.2}, Vote: {:.2})\n{}\n{}",
                hp_result.hp_at_k,
                hp_result.hm_at_k,
                hp_result.vote_at_k,
                execution_log.join("\n"),
                self.graph.summary()
            ))
        } else {
            let _results = self.worker.execute_tasks(&tasks);
            let total = self.graph.nodes.len();
            Ok(format!("DAG: {}/{} done (no engine)", total, total))
        }
    }
}
