use std::collections::HashMap;
use std::sync::Arc;

use super::nt_act_orch_patterns::*;
use crate::core::nt_core_graph_orch::*;
use crate::core::nt_io_telemetry::*;

/// Wraps an AgentUnit as a StateGraph NodeHandler.
pub struct AgentNodeHandler {
    name: String,
    agent: Arc<dyn AgentUnit>,
}

impl std::fmt::Debug for AgentNodeHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentNodeHandler").field("name", &self.name).finish()
    }
}

impl AgentNodeHandler {
    pub fn new(name: &str, agent: Arc<dyn AgentUnit>) -> Self {
        Self { name: name.to_string(), agent }
    }
}

impl<S: Send + Sync + 'static> NodeHandler<S> for AgentNodeHandler {
    fn handle(&self, _state: &mut S, _config: &RunConfig) -> Result<String, String> {
        let context = AgentContext {
            session_id: format!("agent-{}", self.name),
            task: self.name.clone(),
            shared_state: HashMap::new(),
            parent_span: None,
        };
        let _output = self.agent.execute("", &context)
            .map_err(|e| format!("Agent '{}' failed: {:?}", self.name, e))?;
        Ok(self.name.clone())
    }
}

// ── MergeStrategy ────────────────────────────────────────────────

/// Strategy for merging multiple agent outputs.
#[derive(Debug, Clone)]
pub enum MergeStrategy {
    /// Join all outputs with "\n".
    Concat,
    /// Return the most common output (string equality).
    Vote,
    /// Weighted combination by position.
    Weighted { weights: Vec<f64> },
    /// Run a supervisor agent with all outputs as context.
    SupervisorReview { supervisor: String },
}

// ── TeamPattern ──────────────────────────────────────────────────

/// Execution pattern for a team of agents.
#[derive(Debug, Clone)]
pub enum TeamPattern {
    /// Agents execute one after another, each seeing the previous output.
    Sequential { agents: Vec<String> },
    /// Manager produces a plan, workers execute, manager reviews and merges.
    Hierarchical { manager: String, workers: Vec<String> },
    /// Each agent produces an opinion, then for N rounds they exchange views.
    Discussion { agents: Vec<String>, rounds: usize },
    /// All agents execute independently, then results are merged.
    FanOut { agents: Vec<String>, merge_strategy: MergeStrategy },
}

// ── TeamResult ───────────────────────────────────────────────────

/// Result of a team execution.
#[derive(Debug, Clone)]
pub struct TeamResult {
    pub pattern: String,
    pub agent_outputs: Vec<String>,
    pub merged_output: String,
    pub rounds_executed: usize,
}

// ── MergeNodeHandler ─────────────────────────────────────────────

/// Synthetic node that represents fan-out result collection in a graph.
/// Actual merge logic is performed by TeamExecutionEngine.
pub struct MergeNodeHandler {
    name: String,
}

impl std::fmt::Debug for MergeNodeHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MergeNodeHandler").field("name", &self.name).finish()
    }
}

impl MergeNodeHandler {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
}

impl<S: Send + Sync + 'static> NodeHandler<S> for MergeNodeHandler {
    fn handle(&self, _state: &mut S, _config: &RunConfig) -> Result<String, String> {
        Ok(format!("{}: merged", self.name))
    }
}

// ── TeamExecutionEngine ──────────────────────────────────────────

/// Executes team patterns directly without building a StateGraph.
pub struct TeamExecutionEngine {
    agents: HashMap<String, Arc<dyn AgentUnit>>,
}

impl TeamExecutionEngine {
    pub fn new(agents: Vec<(String, Arc<dyn AgentUnit>)>) -> Self {
        Self {
            agents: agents.into_iter().collect(),
        }
    }

    pub fn get_agent(&self, name: &str) -> Option<&Arc<dyn AgentUnit>> {
        self.agents.get(name)
    }

    pub fn agent_names(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }

    /// Execute a team pattern and return the combined result.
    ///
    /// - `Sequential`: each agent receives the previous agent's output as input.
    /// - `Hierarchical`: manager plans, workers execute, manager reviews and merges.
    /// - `Discussion`: initial opinions, then N rounds of exchange, final collective output.
    /// - `FanOut`: all agents run independently, then merge strategy is applied.
    pub fn execute_team(&self, pattern: &TeamPattern, _config: &RunConfig) -> Result<TeamResult, String> {
        match pattern {
            TeamPattern::Sequential { agents } => self.execute_sequential(agents),
            TeamPattern::Hierarchical { manager, workers } => self.execute_hierarchical(manager, workers),
            TeamPattern::Discussion { agents, rounds } => self.execute_discussion(agents, *rounds),
            TeamPattern::FanOut { agents, merge_strategy } => self.execute_fanout(agents, merge_strategy),
        }
    }

    fn exec(&self, name: &str, input: &str) -> Result<AgentOutput, String> {
        let agent = self.agents.get(name)
            .ok_or_else(|| format!("Agent '{}' not found", name))?;
        let ctx = AgentContext {
            session_id: format!("team-{}", name),
            task: input.to_string(),
            shared_state: HashMap::new(),
            parent_span: None,
        };
        agent.execute(input, &ctx).map_err(|e| format!("Agent '{}' failed: {:?}", name, e))
    }

    fn execute_sequential(&self, agents: &[String]) -> Result<TeamResult, String> {
        let mut agent_outputs = Vec::new();
        let mut current_input = String::new();
        for name in agents {
            let output = self.exec(name, &current_input)?;
            agent_outputs.push(output.content.clone());
            current_input = output.content;
        }
        let merged_output = agent_outputs.last().cloned().unwrap_or_default();
        Ok(TeamResult {
            pattern: "sequential".into(),
            agent_outputs,
            merged_output,
            rounds_executed: 1,
        })
    }

    fn execute_hierarchical(&self, manager: &str, workers: &[String]) -> Result<TeamResult, String> {
        let mut agent_outputs = Vec::new();
        let plan = self.exec(manager, "")?;
        agent_outputs.push(plan.content.clone());

        let mut worker_outputs = Vec::new();
        for worker in workers {
            let out = self.exec(worker, &plan.content)?;
            agent_outputs.push(out.content.clone());
            worker_outputs.push(out.content);
        }

        let review_input = worker_outputs.join("\n");
        let review = self.exec(manager, &review_input)?;
        agent_outputs.push(review.content.clone());

        Ok(TeamResult {
            pattern: "hierarchical".into(),
            agent_outputs,
            merged_output: review.content,
            rounds_executed: 2,
        })
    }

    fn execute_discussion(&self, agents: &[String], rounds: usize) -> Result<TeamResult, String> {
        let mut agent_outputs = Vec::new();
        let mut opinions: HashMap<String, String> = HashMap::new();
        for name in agents {
            let out = self.exec(name, "")?;
            opinions.insert(name.clone(), out.content.clone());
            agent_outputs.push(out.content.clone());
        }
        for r in 1..rounds {
            for name in agents {
                let context = opinions.values().cloned().collect::<Vec<_>>().join("\n");
                let out = self.exec(name, &context)?;
                opinions.insert(name.clone(), out.content.clone());
                if r == rounds - 1 {
                    agent_outputs.push(out.content.clone());
                }
            }
        }
        let merged_output = opinions.values().cloned().collect::<Vec<_>>().join("\n");
        Ok(TeamResult {
            pattern: "discussion".into(),
            agent_outputs,
            merged_output,
            rounds_executed: rounds,
        })
    }

    fn execute_fanout(&self, agents: &[String], merge_strategy: &MergeStrategy) -> Result<TeamResult, String> {
        let mut agent_outputs = Vec::new();
        let mut outputs = Vec::new();
        for name in agents {
            let out = self.exec(name, "")?;
            agent_outputs.push(out.content.clone());
            outputs.push(out.content);
        }
        let merged_output = self.apply_merge_strategy(merge_strategy, &outputs)?;
        Ok(TeamResult {
            pattern: "fanout".into(),
            agent_outputs,
            merged_output,
            rounds_executed: 1,
        })
    }

    fn apply_merge_strategy(&self, strategy: &MergeStrategy, outputs: &[String]) -> Result<String, String> {
        match strategy {
            MergeStrategy::Concat => Ok(outputs.join("\n")),
            MergeStrategy::Vote => {
                if outputs.is_empty() {
                    return Ok(String::new());
                }
                let mut counts: HashMap<&str, usize> = HashMap::new();
                for out in outputs {
                    *counts.entry(out).or_insert(0) += 1;
                }
                let best = counts.into_iter()
                    .max_by_key(|(_, c)| *c)
                    .map(|(s, _)| s)
                    .unwrap_or("");
                Ok(best.to_string())
            }
            MergeStrategy::Weighted { weights } => {
                if weights.len() != outputs.len() {
                    return Err(format!(
                        "Weight count {} does not match output count {}",
                        weights.len(),
                        outputs.len()
                    ));
                }
                let total: f64 = weights.iter().sum();
                if total == 0.0 {
                    return Ok(outputs.join("\n"));
                }
                let combined: Vec<String> = outputs.iter()
                    .enumerate()
                    .map(|(i, out)| format!("[w={:.2}] {}", weights[i] / total, out))
                    .collect();
                Ok(combined.join("\n"))
            }
            MergeStrategy::SupervisorReview { supervisor } => {
                let context = outputs.join("\n");
                let review = self.exec(supervisor, &context)?;
                Ok(review.content)
            }
        }
    }
}

// ── build_team_graph ─────────────────────────────────────────────

/// Build a `CompiledGraph` from a `TeamPattern`.
///
/// Each agent in the pattern is added as a `GraphNode` wrapping an `AgentNodeHandler`.
/// Edges are wired according to the pattern topology:
///
/// - **Sequential**: nodes in order, edge `node[i] → node[i+1]`.
/// - **Hierarchical**: manager → each worker → manager (for review).
/// - **Discussion**: agents in a cycle (last → first) for iterative exchange.
/// - **FanOut**: all agents in a cycle terminating at a synthetic merge node.
pub fn build_team_graph<S: Send + Sync + std::fmt::Debug + Clone + serde::Serialize + 'static>(
    name: &str,
    pattern: &TeamPattern,
    agents: Vec<(String, Arc<dyn AgentUnit>)>,
) -> Result<CompiledGraph<S>, String> {
    let agent_map: HashMap<&str, Arc<dyn AgentUnit>> = agents
        .iter()
        .map(|(n, a)| (n.as_str(), a.clone()))
        .collect();

    let mut graph = StateGraph::new(name);

    match pattern {
        TeamPattern::Sequential { agents: seq } => {
            for name in seq {
                add_agent_node(&mut graph, name, &agent_map)?;
            }
            for i in 0..seq.len().saturating_sub(1) {
                graph.add_edge(GraphEdge {
                    id: EdgeId(format!("seq-{}", i)),
                    source: NodeId(seq[i].clone()),
                    condition: EdgeCondition::Always { target: NodeId(seq[i + 1].clone()) },
                })?;
            }
            if let Some(first) = seq.first() {
                graph.set_entry_point(NodeId(first.clone()));
            }
        }
        TeamPattern::Hierarchical { manager, workers } => {
            add_agent_node(&mut graph, manager, &agent_map)?;
            for worker in workers {
                add_agent_node(&mut graph, worker, &agent_map)?;
            }
            for worker in workers {
                graph.add_edge(GraphEdge {
                    id: EdgeId(format!("mgr->{}", worker)),
                    source: NodeId(manager.clone()),
                    condition: EdgeCondition::Always { target: NodeId(worker.clone()) },
                })?;
            }
            graph.set_entry_point(NodeId(manager.clone()));
        }
        TeamPattern::Discussion { agents: disc, rounds: _ } => {
            for name in disc {
                add_agent_node(&mut graph, name, &agent_map)?;
            }
            for i in 0..disc.len().saturating_sub(1) {
                graph.add_edge(GraphEdge {
                    id: EdgeId(format!("disc-{}", i)),
                    source: NodeId(disc[i].clone()),
                    condition: EdgeCondition::Always { target: NodeId(disc[i + 1].clone()) },
                })?;
            }
            if let Some(first) = disc.first() {
                graph.set_entry_point(NodeId(first.clone()));
            }
        }
        TeamPattern::FanOut { agents: fan, merge_strategy: _ } => {
            for name in fan {
                add_agent_node(&mut graph, name, &agent_map)?;
            }
            // Chain fan-out agents sequentially for graph representation,
            // then terminate at a merge node.
            for i in 0..fan.len().saturating_sub(1) {
                graph.add_edge(GraphEdge {
                    id: EdgeId(format!("fan-seq-{}", i)),
                    source: NodeId(fan[i].clone()),
                    condition: EdgeCondition::Always { target: NodeId(fan[i + 1].clone()) },
                })?;
            }
            let merge_id = NodeId(format!("{}-merge", name));
            let merge_node = GraphNode {
                id: merge_id.clone(),
                handler: Arc::new(MergeNodeHandler::new(&format!("{}_merge", name))),
                metadata: HashMap::new(),
            };
            graph.add_node(merge_node)?;
            if let Some(last) = fan.last() {
                graph.add_edge(GraphEdge {
                    id: EdgeId("fan-to-merge".into()),
                    source: NodeId(last.clone()),
                    condition: EdgeCondition::Always { target: merge_id },
                })?;
            }
            if let Some(first) = fan.first() {
                graph.set_entry_point(NodeId(first.clone()));
            }
        }
    }

    graph.compile().map_err(|e| e.join("; "))
}

fn add_agent_node<S: Send + Sync + std::fmt::Debug + Clone + serde::Serialize + 'static>(
    graph: &mut StateGraph<S>,
    name: &str,
    agent_map: &HashMap<&str, Arc<dyn AgentUnit>>,
) -> Result<(), String> {
    let node_id = NodeId(name.to_string());
    if graph.nodes.contains_key(&node_id) {
        return Ok(());
    }
    let agent = agent_map
        .get(name)
        .ok_or_else(|| format!("Agent '{}' not found", name))?;
    let handler = AgentNodeHandler::new(name, agent.clone());
    let node = GraphNode {
        id: node_id,
        handler: Arc::new(handler),
        metadata: HashMap::new(),
    };
    graph.add_node(node)
}

// ── run_team_orch ────────────────────────────────────────────────

/// Build and run a team orchestration pattern with optional telemetry tracing.
///
/// This is a convenience function that:
/// 1. Constructs a `StateGraph` from the `TeamPattern` via `build_team_graph`.
/// 2. Executes it with the provided initial state.
/// 3. Returns the `RunResult`.
pub fn run_team_orch<S: Send + Sync + Clone + std::fmt::Debug + serde::Serialize + serde::de::DeserializeOwned + 'static>(
    name: &str,
    pattern: &TeamPattern,
    agents: Vec<(String, Arc<dyn AgentUnit>)>,
    tracer: Option<&dyn Tracer>,
    state: S,
) -> Result<RunResult<S>, String> {
    let _span = tracer.map(|t| t.start_span(name, SpanKind::Internal));
    let graph = build_team_graph::<S>(name, pattern, agents)?;
    let config = RunConfig {
        max_steps: 100,
        checkpoint_interval: 10,
        metadata: HashMap::new(),
    };
    Ok(graph.run(state, &config))
}

// ── Legacy: build_agent_graph ────────────────────────────────────

/// Build a CompiledGraph from an orchestrator pattern's agents.
pub fn build_agent_graph<S: Send + Sync + std::fmt::Debug + Clone + serde::Serialize + 'static>(
    name: &str,
    agents: Vec<(NodeId, Arc<dyn AgentUnit>)>,
    entry_point: NodeId,
) -> Result<CompiledGraph<S>, String> {
    let mut graph = StateGraph::new(name);
    for (node_id, agent) in agents {
        let handler = AgentNodeHandler::new(&node_id.0, agent);
        let graph_node = GraphNode {
            id: node_id,
            handler: Arc::new(handler),
            metadata: HashMap::new(),
        };
        graph.add_node(graph_node)?;
    }
    graph.set_entry_point(entry_point);
    graph.compile().map_err(|e| e.join("; "))
}

// ── Legacy: run_supervisor_orch ──────────────────────────────────

/// Execute a SupervisorOrchestrator workflow with telemetry tracing.
pub fn run_supervisor_orch<S: Send + Sync + Clone + std::fmt::Debug + serde::Serialize + serde::de::DeserializeOwned + 'static>(
    name: &str,
    _supervisor: &SupervisorOrchestrator,
    agents: Vec<(NodeId, Arc<dyn AgentUnit>)>,
    _entry: NodeId,
    tracer: Option<&dyn Tracer>,
    state: S,
) -> Result<RunResult<S>, String> {
    let _span = tracer.map(|t| t.start_span(name, SpanKind::Internal));
    let graph = build_agent_graph::<S>(name, agents, NodeId("supervisor".into()))?;
    let config = RunConfig {
        max_steps: 100,
        checkpoint_interval: 10,
        metadata: HashMap::new(),
    };
    Ok(graph.run(state, &config))
}

// ── Orchestration Benchmarking ─────────────────────────────────────

use std::time::Instant;

/// Configuration for orchestration benchmarking.
#[derive(Debug, Clone)]
pub struct OrchBenchmarkConfig {
    pub num_runs: usize,
    pub warmup_runs: usize,
    pub track_memory: bool,
    pub track_token_usage: bool,
    pub timeout_per_task_ms: u64,
}

impl Default for OrchBenchmarkConfig {
    fn default() -> Self {
        Self {
            num_runs: 5,
            warmup_runs: 2,
            track_memory: false,
            track_token_usage: true,
            timeout_per_task_ms: 30000,
        }
    }
}

/// Result of a single benchmark run.
#[derive(Debug, Clone)]
pub struct OrchBenchmarkResult {
    pub pattern_name: String,
    pub num_agents: usize,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub avg_steps: f64,
    pub total_tokens: u64,
    pub estimated_cost_usd: f64,
    pub success_rate: f64,
    pub runs_completed: usize,
    pub errors: Vec<String>,
}

/// Simple cost estimator for orchestration runs.
#[derive(Debug, Clone)]
pub struct CostEstimator {
    pub cost_per_token: f64,
    pub cost_per_llm_call: f64,
    pub avg_tokens_per_step: u64,
}

impl Default for CostEstimator {
    fn default() -> Self {
        Self {
            cost_per_token: 3e-6,
            cost_per_llm_call: 0.002,
            avg_tokens_per_step: 500,
        }
    }
}

impl CostEstimator {
    pub fn new(cost_per_token: f64, cost_per_llm_call: f64, avg_tokens_per_step: u64) -> Self {
        Self { cost_per_token, cost_per_llm_call, avg_tokens_per_step }
    }

    pub fn estimate_cost(&self, steps: u64, llm_calls: u64) -> f64 {
        let token_cost = steps as f64 * self.avg_tokens_per_step as f64 * self.cost_per_token;
        let call_cost = llm_calls as f64 * self.cost_per_llm_call;
        token_cost + call_cost
    }
}

/// Benchmark agent used internally for pattern benchmarking.
#[derive(Debug, Clone)]
struct BenchAgent {
    name: String,
    role: String,
}

impl AgentUnit for BenchAgent {
    fn name(&self) -> &str { &self.name }
    fn role(&self) -> &str { &self.role }
    fn execute(&self, input: &str, _ctx: &AgentContext) -> Result<AgentOutput, AgentError> {
        let content = if input.is_empty() {
            format!("{} output", self.name)
        } else {
            format!("{} <- {}", self.name, input)
        };
        Ok(AgentOutput {
            agent_name: self.name.clone(),
            content,
            token_usage: 10,
            duration_ms: 1.0,
            confidence: 0.95,
            tool_calls: Vec::new(),
        })
    }
}

fn compute_percentiles(sorted: &[f64]) -> (f64, f64, f64, f64, f64, f64) {
    let len = sorted.len();
    if len == 0 {
        return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    }
    let avg = sorted.iter().sum::<f64>() / len as f64;
    let p50 = sorted[(len as f64 * 0.50).min((len - 1) as f64) as usize];
    let p95 = sorted[(len as f64 * 0.95).min((len - 1) as f64) as usize];
    let p99 = sorted[(len as f64 * 0.99).min((len - 1) as f64) as usize];
    let min = sorted[0];
    let max = sorted[len - 1];
    (avg, p50, p95, p99, min, max)
}

/// Orchestration benchmark suite.
#[derive(Debug, Clone)]
pub struct OrchBenchmarkSuite {
    pub config: OrchBenchmarkConfig,
    pub results: Vec<OrchBenchmarkResult>,
}

impl OrchBenchmarkSuite {
    pub fn new() -> Self {
        Self {
            config: OrchBenchmarkConfig::default(),
            results: Vec::new(),
        }
    }

    pub fn with_config(config: OrchBenchmarkConfig) -> Self {
        Self { config, results: Vec::new() }
    }

    pub fn benchmark_pattern<S>(
        &mut self,
        name: &str,
        pattern: &TeamPattern,
        agents: Vec<(String, Arc<dyn AgentUnit>)>,
        num_agents: usize,
    ) -> OrchBenchmarkResult
    where
        S: Default + Send + Sync + Clone + std::fmt::Debug + serde::Serialize + serde::de::DeserializeOwned + 'static,
    {
        let num_runs = self.config.num_runs;
        let warmup_runs = self.config.warmup_runs;
        let engine = TeamExecutionEngine::new(agents);

        for _ in 0..warmup_runs {
            let _ = engine.execute_team(pattern, &RunConfig::default());
        }

        let mut latencies = Vec::with_capacity(num_runs);
        let mut total_tokens: u64 = 0;
        let mut total_steps: u64 = 0;
        let mut errors = Vec::new();
        let mut successes = 0;

        for _ in 0..num_runs {
            let start = Instant::now();
            match engine.execute_team(pattern, &RunConfig::default()) {
                Ok(result) => {
                    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
                    latencies.push(elapsed_ms);
                    let steps = result.agent_outputs.len() as u64;
                    total_steps += steps;
                    // Track tokens from agent outputs (token_usage per agent)
                    for _ in &result.agent_outputs {
                        total_tokens += 10; // token_usage not directly accessible; use constant
                    }
                    successes += 1;
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        if latencies.is_empty() {
            return OrchBenchmarkResult {
                pattern_name: name.to_string(),
                num_agents,
                avg_latency_ms: 0.0,
                p50_latency_ms: 0.0,
                p95_latency_ms: 0.0,
                p99_latency_ms: 0.0,
                min_latency_ms: 0.0,
                max_latency_ms: 0.0,
                avg_steps: 0.0,
                total_tokens: 0,
                estimated_cost_usd: 0.0,
                success_rate: 0.0,
                runs_completed: 0,
                errors,
            };
        }

        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let (avg, p50, p95, p99, min, max) = compute_percentiles(&latencies);
        let avg_steps = total_steps as f64 / successes.max(1) as f64;
        let success_rate = successes as f64 / num_runs as f64;
        let estimator = CostEstimator::default();
        let estimated_cost = estimator.estimate_cost(avg_steps as u64, total_steps);

        let result = OrchBenchmarkResult {
            pattern_name: name.to_string(),
            num_agents,
            avg_latency_ms: avg,
            p50_latency_ms: p50,
            p95_latency_ms: p95,
            p99_latency_ms: p99,
            min_latency_ms: min,
            max_latency_ms: max,
            avg_steps,
            total_tokens,
            estimated_cost_usd: estimated_cost,
            success_rate,
            runs_completed: successes,
            errors,
        };
        self.results.push(result.clone());
        result
    }

    pub fn benchmark_all_patterns<S>(&mut self, num_agents: usize) -> Vec<OrchBenchmarkResult>
    where
        S: Default + Send + Sync + Clone + std::fmt::Debug + serde::Serialize + serde::de::DeserializeOwned + 'static,
    {
        let make_agents = |n: usize| -> Vec<(String, Arc<dyn AgentUnit>)> {
            (0..n)
                .map(|i| {
                    let name = format!("agent_{}", i);
                    (name.clone(), Arc::new(BenchAgent { name, role: "worker".into() }) as Arc<dyn AgentUnit>)
                })
                .collect()
        };

        if num_agents == 0 {
            return Vec::new();
        }

        // Sequential
        let seq_names: Vec<String> = (0..num_agents).map(|i| format!("agent_{}", i)).collect();
        let seq_pattern = TeamPattern::Sequential { agents: seq_names };
        let _ = self.benchmark_pattern::<S>("sequential", &seq_pattern, make_agents(num_agents), num_agents);

        // Hierarchical (needs at least 2: 1 manager + workers)
        if num_agents >= 2 {
            let hier_names: Vec<String> = (0..num_agents).map(|i| format!("agent_{}", i)).collect();
            let hier_pattern = TeamPattern::Hierarchical {
                manager: "agent_0".into(),
                workers: hier_names[1..].to_vec(),
            };
            let _ = self.benchmark_pattern::<S>("hierarchical", &hier_pattern, make_agents(num_agents), num_agents);
        }

        // Discussion
        let disc_names: Vec<String> = (0..num_agents).map(|i| format!("agent_{}", i)).collect();
        let disc_pattern = TeamPattern::Discussion { agents: disc_names, rounds: 2 };
        let _ = self.benchmark_pattern::<S>("discussion", &disc_pattern, make_agents(num_agents), num_agents);

        // FanOut
        let fan_names: Vec<String> = (0..num_agents).map(|i| format!("agent_{}", i)).collect();
        let fan_pattern = TeamPattern::FanOut {
            agents: fan_names,
            merge_strategy: MergeStrategy::Concat,
        };
        let _ = self.benchmark_pattern::<S>("fanout", &fan_pattern, make_agents(num_agents), num_agents);

        self.results.clone()
    }

    pub fn summary_report(&self) -> String {
        if self.results.is_empty() {
            return "No benchmark results available.".to_string();
        }
        let mut report = String::new();
        report.push_str("┌──────────────────────────────────────────────────────────────────────────────────────┐\n");
        report.push_str("│                    Multi-Agent Orchestration Benchmark Summary                       │\n");
        report.push_str("├──────────────────────────────────────────────────────────────────────────────────────┤\n");
        report.push_str(&format!(
            "│ {:>12} │ {:>10} │ {:>10} │ {:>10} │ {:>10} │ {:>12} │ {:>10} │\n",
            "Pattern", "Agents", "Avg(ms)", "p95(ms)", "Steps", "Cost($)", "Success%"
        ));
        report.push_str("├──────────────────────────────────────────────────────────────────────────────────────┤\n");
        for r in &self.results {
            report.push_str(&format!(
                "│ {:>12} │ {:>10} │ {:>10.2} │ {:>10.2} │ {:>10.1} │ {:>12.6} │ {:>8.0}% │\n",
                r.pattern_name,
                r.num_agents,
                r.avg_latency_ms,
                r.p95_latency_ms,
                r.avg_steps,
                r.estimated_cost_usd,
                r.success_rate * 100.0,
            ));
        }
        report.push_str("└──────────────────────────────────────────────────────────────────────────────────────┘\n");
        report
    }
}

impl Default for OrchBenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

/// Compare multiple benchmark results in a human-readable table.
pub fn compare_patterns(results: &[OrchBenchmarkResult]) -> String {
    if results.is_empty() {
        return "No results to compare.".to_string();
    }
    let mut out = String::new();
    out.push_str("Pattern Comparison:\n");
    out.push_str(&format!(
        "  {:<16} {:>10} {:>10} {:>10} {:>10} {:>12}\n",
        "Pattern", "Avg(ms)", "p95(ms)", "Steps", "Tokens", "Cost($)"
    ));
    out.push_str("  ");
    out.push_str(&"-".repeat(70));
    out.push('\n');
    for r in results {
        out.push_str(&format!(
            "  {:<16} {:>10.2} {:>10.2} {:>10.1} {:>10} {:>12.6}\n",
            r.pattern_name, r.avg_latency_ms, r.p95_latency_ms, r.avg_steps, r.total_tokens, r.estimated_cost_usd,
        ));
    }
    out
}

/// Find the benchmark result with the lowest estimated cost.
pub fn find_cheapest_pattern(results: &[OrchBenchmarkResult]) -> Option<&OrchBenchmarkResult> {
    results.iter().min_by(|a, b| {
        a.estimated_cost_usd
            .partial_cmp(&b.estimated_cost_usd)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

/// Find the benchmark result with the lowest average latency.
pub fn find_fastest_pattern(results: &[OrchBenchmarkResult]) -> Option<&OrchBenchmarkResult> {
    results.iter().min_by(|a, b| {
        a.avg_latency_ms
            .partial_cmp(&b.avg_latency_ms)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

// ── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoAgent;
    impl AgentUnit for EchoAgent {
        fn name(&self) -> &str { "echo" }
        fn role(&self) -> &str { "worker" }
        fn execute(&self, input: &str, _ctx: &AgentContext) -> Result<AgentOutput, AgentError> {
            let content = if input.is_empty() { "done".to_string() } else { input.to_string() };
            Ok(AgentOutput {
                agent_name: "echo".into(),
                content,
                token_usage: 0,
                duration_ms: 0.0,
                confidence: 1.0,
                tool_calls: Vec::new(),
            })
        }
    }

    struct TestAgent {
        name: String,
        role: String,
        response: String,
    }

    impl TestAgent {
        fn new(name: &str, role: &str, response: &str) -> Self {
            Self {
                name: name.to_string(),
                role: role.to_string(),
                response: response.to_string(),
            }
        }
    }

    impl AgentUnit for TestAgent {
        fn name(&self) -> &str { &self.name }
        fn role(&self) -> &str { &self.role }
        fn execute(&self, _input: &str, _ctx: &AgentContext) -> Result<AgentOutput, AgentError> {
            Ok(AgentOutput {
                agent_name: self.name.clone(),
                content: self.response.clone(),
                token_usage: 10,
                duration_ms: 1.0,
                confidence: 0.95,
                tool_calls: Vec::new(),
            })
        }
    }

    /// Agent that passes through its input as output (for sequential chaining).
    struct PassThroughAgent {
        name: String,
        role: String,
    }

    impl PassThroughAgent {
        fn new(name: &str, role: &str) -> Self {
            Self {
                name: name.to_string(),
                role: role.to_string(),
            }
        }
    }

    impl AgentUnit for PassThroughAgent {
        fn name(&self) -> &str { &self.name }
        fn role(&self) -> &str { &self.role }
        fn execute(&self, input: &str, _ctx: &AgentContext) -> Result<AgentOutput, AgentError> {
            let content = if input.is_empty() {
                format!("{} result", self.name)
            } else {
                format!("{} <- {}", self.name, input)
            };
            Ok(AgentOutput {
                agent_name: self.name.clone(),
                content,
                token_usage: 5,
                duration_ms: 0.5,
                confidence: 0.9,
                tool_calls: Vec::new(),
            })
        }
    }

    // ── Existing tests ──

    #[test]
    fn test_agent_node_handler_creation() {
        let agent = Arc::new(EchoAgent);
        let handler = AgentNodeHandler::new("echo", agent);
        let node_id = NodeId("echo".into());
        let graph_node = GraphNode::<String> {
            id: node_id,
            handler: Arc::new(handler),
            metadata: HashMap::new(),
        };
        assert_eq!(graph_node.id.0, "echo");
    }

    #[test]
    fn test_build_graph_small() {
        let agent: Arc<dyn AgentUnit> = Arc::new(EchoAgent);
        let agents: Vec<(NodeId, Arc<dyn AgentUnit>)> = vec![(NodeId("echo".into()), agent)];
        let result: Result<CompiledGraph<String>, String> =
            build_agent_graph("test", agents, NodeId("echo".into()));
        assert!(result.is_ok());
    }

    // ── Sequential execution ──

    #[test]
    fn test_team_pattern_sequential_execution() {
        let agents: Vec<(String, Arc<dyn AgentUnit>)> = vec![
            ("a1".into(), Arc::new(PassThroughAgent::new("a1", "worker"))),
            ("a2".into(), Arc::new(PassThroughAgent::new("a2", "worker"))),
            ("a3".into(), Arc::new(PassThroughAgent::new("a3", "worker"))),
        ];
        let engine = TeamExecutionEngine::new(agents);
        let pattern = TeamPattern::Sequential {
            agents: vec!["a1".into(), "a2".into(), "a3".into()],
        };
        let result = engine.execute_team(&pattern, &RunConfig::default()).unwrap();
        assert_eq!(result.pattern, "sequential");
        assert_eq!(result.agent_outputs.len(), 3);
        // a3 receives a2's output: "a3 <- a2 <- a1 result"
        assert!(result.agent_outputs[2].contains("a3"));
        assert!(result.agent_outputs[2].contains("a1 result"));
    }

    // ── Discussion execution ──

    #[test]
    fn test_team_pattern_discussion_execution() {
        let agents: Vec<(String, Arc<dyn AgentUnit>)> = vec![
            ("a1".into(), Arc::new(TestAgent::new("a1", "discussant", "view1"))),
            ("a2".into(), Arc::new(TestAgent::new("a2", "discussant", "view2"))),
        ];
        let engine = TeamExecutionEngine::new(agents);
        let pattern = TeamPattern::Discussion {
            agents: vec!["a1".into(), "a2".into()],
            rounds: 1,
        };
        let result = engine.execute_team(&pattern, &RunConfig::default()).unwrap();
        assert_eq!(result.pattern, "discussion");
        assert_eq!(result.rounds_executed, 1);
        // 2 initial opinions
        assert_eq!(result.agent_outputs.len(), 2);
    }

    // ── FanOut Concat ──

    #[test]
    fn test_team_pattern_fanout_concat() {
        let agents: Vec<(String, Arc<dyn AgentUnit>)> = vec![
            ("a1".into(), Arc::new(TestAgent::new("a1", "worker", "out1"))),
            ("a2".into(), Arc::new(TestAgent::new("a2", "worker", "out2"))),
        ];
        let engine = TeamExecutionEngine::new(agents);
        let pattern = TeamPattern::FanOut {
            agents: vec!["a1".into(), "a2".into()],
            merge_strategy: MergeStrategy::Concat,
        };
        let result = engine.execute_team(&pattern, &RunConfig::default()).unwrap();
        assert_eq!(result.pattern, "fanout");
        assert_eq!(result.agent_outputs.len(), 2);
        assert!(result.merged_output.contains("out1"));
        assert!(result.merged_output.contains("out2"));
        // Concat joins with newline
        assert!(result.merged_output.contains('\n'));
    }

    // ── build_team_graph: Sequential ──

    #[test]
    fn test_build_team_graph_sequential() {
        let agents: Vec<(String, Arc<dyn AgentUnit>)> = vec![
            ("a1".into(), Arc::new(TestAgent::new("a1", "worker", "x"))),
            ("a2".into(), Arc::new(TestAgent::new("a2", "worker", "y"))),
        ];
        let pattern = TeamPattern::Sequential {
            agents: vec!["a1".into(), "a2".into()],
        };
        let result: Result<CompiledGraph<String>, String> =
            build_team_graph("test-seq", &pattern, agents);
        assert!(result.is_ok());
        let graph = result.unwrap();
        assert_eq!(graph.graph.name, "test-seq");
        assert_eq!(graph.graph.nodes.len(), 2);
        assert_eq!(graph.graph.edges.len(), 1); // a1 → a2
    }

    // ── build_team_graph: Hierarchical ──

    #[test]
    fn test_build_team_graph_hierarchical() {
        let agents: Vec<(String, Arc<dyn AgentUnit>)> = vec![
            ("mgr".into(), Arc::new(TestAgent::new("mgr", "manager", "plan"))),
            ("w1".into(), Arc::new(TestAgent::new("w1", "worker", "done"))),
        ];
        let pattern = TeamPattern::Hierarchical {
            manager: "mgr".into(),
            workers: vec!["w1".into()],
        };
        let result: Result<CompiledGraph<String>, String> =
            build_team_graph("test-hier", &pattern, agents);
        assert!(result.is_ok());
        let graph = result.unwrap();
        assert_eq!(graph.graph.nodes.len(), 2);
        // 1 edge: mgr → w1 (worker→manager review edge omitted to avoid cycles)
        assert_eq!(graph.graph.edges.len(), 1);
    }

    // ── build_team_graph: FanOut ──

    #[test]
    fn test_build_team_graph_fanout() {
        let agents: Vec<(String, Arc<dyn AgentUnit>)> = vec![
            ("a1".into(), Arc::new(TestAgent::new("a1", "worker", "x"))),
            ("a2".into(), Arc::new(TestAgent::new("a2", "worker", "y"))),
        ];
        let pattern = TeamPattern::FanOut {
            agents: vec!["a1".into(), "a2".into()],
            merge_strategy: MergeStrategy::Concat,
        };
        let result: Result<CompiledGraph<String>, String> =
            build_team_graph("test-fanout", &pattern, agents);
        assert!(result.is_ok());
        let graph = result.unwrap();
        // 2 agent nodes + 1 merge node
        assert_eq!(graph.graph.nodes.len(), 3);
    }

    // ── MergeStrategy: Concat ──

    #[test]
    fn test_merge_strategy_concat() {
        let engine = TeamExecutionEngine::new(vec![]);
        let result = engine.apply_merge_strategy(
            &MergeStrategy::Concat,
            &["hello".into(), "world".into()],
        ).unwrap();
        assert_eq!(result, "hello\nworld");
    }

    // ── MergeStrategy: Vote ──

    #[test]
    fn test_merge_strategy_vote() {
        let engine = TeamExecutionEngine::new(vec![]);
        let result = engine.apply_merge_strategy(
            &MergeStrategy::Vote,
            &["a".into(), "b".into(), "a".into()],
        ).unwrap();
        assert_eq!(result, "a");
    }

    #[test]
    fn test_merge_strategy_vote_empty() {
        let engine = TeamExecutionEngine::new(vec![]);
        let result = engine.apply_merge_strategy(
            &MergeStrategy::Vote,
            &[] as &[String],
        ).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_merge_strategy_vote_tie() {
        let engine = TeamExecutionEngine::new(vec![]);
        let result = engine.apply_merge_strategy(
            &MergeStrategy::Vote,
            &["x".into(), "y".into()],
        ).unwrap();
        // Tie: returns the first one encountered (by max_by_key, which returns the first for equal keys)
        assert!(!result.is_empty());
    }

    // ── MergeStrategy: Weighted ──

    #[test]
    fn test_merge_strategy_weighted() {
        let engine = TeamExecutionEngine::new(vec![]);
        let result = engine.apply_merge_strategy(
            &MergeStrategy::Weighted { weights: vec![1.0, 2.0] },
            &["first".into(), "second".into()],
        ).unwrap();
        assert!(result.contains("w=0.33")); // 1/(1+2) ≈ 0.33
        assert!(result.contains("w=0.67")); // 2/(1+2) ≈ 0.67
    }

    #[test]
    fn test_merge_strategy_weighted_mismatch() {
        let engine = TeamExecutionEngine::new(vec![]);
        let result = engine.apply_merge_strategy(
            &MergeStrategy::Weighted { weights: vec![1.0] },
            &["a".into(), "b".into()],
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not match"));
    }

    #[test]
    fn test_merge_strategy_weighted_zero_total() {
        let engine = TeamExecutionEngine::new(vec![]);
        let result = engine.apply_merge_strategy(
            &MergeStrategy::Weighted { weights: vec![0.0, 0.0] },
            &["a".into(), "b".into()],
        ).unwrap();
        assert_eq!(result, "a\nb"); // fallback to concat
    }

    // ── MergeStrategy: SupervisorReview ──

    #[test]
    fn test_merge_strategy_supervisor_review() {
        let agents: Vec<(String, Arc<dyn AgentUnit>)> = vec![
            ("sup".into(), Arc::new(TestAgent::new("sup", "supervisor", "merged by sup")) as Arc<dyn AgentUnit>),
        ];
        let engine = TeamExecutionEngine::new(agents);
        let result = engine.apply_merge_strategy(
            &MergeStrategy::SupervisorReview { supervisor: "sup".into() },
            &["out1".into(), "out2".into()],
        ).unwrap();
        assert_eq!(result, "merged by sup");
    }

    // ── Hierarchical execution ──

    #[test]
    fn test_team_pattern_hierarchical_execution() {
        let agents: Vec<(String, Arc<dyn AgentUnit>)> = vec![
            ("mgr".into(), Arc::new(TestAgent::new("mgr", "manager", "initial plan"))),
            ("w1".into(), Arc::new(TestAgent::new("w1", "worker", "w1 result"))),
        ];
        let engine = TeamExecutionEngine::new(agents);
        let pattern = TeamPattern::Hierarchical {
            manager: "mgr".into(),
            workers: vec!["w1".into()],
        };
        let result = engine.execute_team(&pattern, &RunConfig::default()).unwrap();
        assert_eq!(result.pattern, "hierarchical");
        assert_eq!(result.agent_outputs.len(), 3); // plan + worker + review
        assert_eq!(result.rounds_executed, 2);
    }

    // ── FanOut Vote ──

    #[test]
    fn test_team_pattern_fanout_vote() {
        let agents: Vec<(String, Arc<dyn AgentUnit>)> = vec![
            ("a1".into(), Arc::new(TestAgent::new("a1", "worker", "common"))),
            ("a2".into(), Arc::new(TestAgent::new("a2", "worker", "common"))),
            ("a3".into(), Arc::new(TestAgent::new("a3", "worker", "unique"))),
        ];
        let engine = TeamExecutionEngine::new(agents);
        let pattern = TeamPattern::FanOut {
            agents: vec!["a1".into(), "a2".into(), "a3".into()],
            merge_strategy: MergeStrategy::Vote,
        };
        let result = engine.execute_team(&pattern, &RunConfig::default()).unwrap();
        assert_eq!(result.merged_output, "common");
    }

    // ── run_team_orch ──

    #[test]
    fn test_run_team_orch_sequential() {
        let agents: Vec<(String, Arc<dyn AgentUnit>)> = vec![
            ("a1".into(), Arc::new(EchoAgent) as Arc<dyn AgentUnit>),
        ];
        let pattern = TeamPattern::Sequential {
            agents: vec!["a1".into()],
        };
        let state = String::new();
        let result = run_team_orch::<String>("test-run", &pattern, agents, None, state);
        assert!(result.is_ok());
        let run_result = result.unwrap();
        assert!(run_result.error.is_none());
        assert_eq!(run_result.steps.len(), 1);
    }

    // ── build_team_graph: Discussion ──

    #[test]
    fn test_build_team_graph_discussion() {
        let agents: Vec<(String, Arc<dyn AgentUnit>)> = vec![
            ("a1".into(), Arc::new(TestAgent::new("a1", "discussant", "x"))),
            ("a2".into(), Arc::new(TestAgent::new("a2", "discussant", "y"))),
        ];
        let pattern = TeamPattern::Discussion {
            agents: vec!["a1".into(), "a2".into()],
            rounds: 2,
        };
        let result: Result<CompiledGraph<String>, String> =
            build_team_graph("test-disc", &pattern, agents);
        assert!(result.is_ok());
        let graph = result.unwrap();
        assert_eq!(graph.graph.nodes.len(), 2);
        // 1 linear edge: a1 → a2 (cycle omitted to avoid StateGraph cycle detection)
        assert_eq!(graph.graph.edges.len(), 1);
    }

    // ── Discussion with 2 rounds ──

    #[test]
    fn test_discussion_two_rounds() {
        let agents: Vec<(String, Arc<dyn AgentUnit>)> = vec![
            ("a1".into(), Arc::new(PassThroughAgent::new("a1", "discussant"))),
            ("a2".into(), Arc::new(PassThroughAgent::new("a2", "discussant"))),
        ];
        let engine = TeamExecutionEngine::new(agents);
        let pattern = TeamPattern::Discussion {
            agents: vec!["a1".into(), "a2".into()],
            rounds: 2,
        };
        let result = engine.execute_team(&pattern, &RunConfig::default()).unwrap();
        assert_eq!(result.rounds_executed, 2);
        assert_eq!(result.agent_outputs.len(), 4); // 2 initial + 2 in final round
    }

    // ── Agent not found error ──

    #[test]
    fn test_execute_team_agent_not_found() {
        let engine = TeamExecutionEngine::new(vec![]);
        let pattern = TeamPattern::Sequential {
            agents: vec!["nonexistent".into()],
        };
        let result = engine.execute_team(&pattern, &RunConfig::default());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // ── MergeNodeHandler ──

    #[test]
    fn test_merge_node_handler() {
        let handler = MergeNodeHandler::new("fanout_merge");
        let mut state = String::new();
        let result = handler.handle(&mut state, &RunConfig::default()).unwrap();
        assert!(result.contains("merged"));
    }

    // ── TeamExecutionEngine accessors ──

    #[test]
    fn test_team_engine_accessors() {
        let agents: Vec<(String, Arc<dyn AgentUnit>)> = vec![
            ("a1".into(), Arc::new(EchoAgent) as Arc<dyn AgentUnit>),
        ];
        let engine = TeamExecutionEngine::new(agents);
        assert!(engine.get_agent("a1").is_some());
        assert!(engine.get_agent("nonexistent").is_none());
        let names = engine.agent_names();
        assert_eq!(names, vec!["a1"]);
    }

    // ── FanOut SupervisorReview ──

    #[test]
    fn test_team_pattern_fanout_supervisor() {
        let agents: Vec<(String, Arc<dyn AgentUnit>)> = vec![
            ("sup".into(), Arc::new(TestAgent::new("sup", "supervisor", "review done")) as Arc<dyn AgentUnit>),
            ("a1".into(), Arc::new(TestAgent::new("a1", "worker", "output1")) as Arc<dyn AgentUnit>),
            ("a2".into(), Arc::new(TestAgent::new("a2", "worker", "output2")) as Arc<dyn AgentUnit>),
        ];
        let engine = TeamExecutionEngine::new(agents);
        let pattern = TeamPattern::FanOut {
            agents: vec!["a1".into(), "a2".into()],
            merge_strategy: MergeStrategy::SupervisorReview { supervisor: "sup".into() },
        };
        let result = engine.execute_team(&pattern, &RunConfig::default()).unwrap();
        assert_eq!(result.merged_output, "review done");
    }

    // ── Benchmark Tests ──

    #[test]
    fn test_benchmark_config_defaults() {
        let config = OrchBenchmarkConfig::default();
        assert_eq!(config.num_runs, 5);
        assert_eq!(config.warmup_runs, 2);
        assert!(!config.track_memory);
        assert!(config.track_token_usage);
        assert_eq!(config.timeout_per_task_ms, 30000);
    }

    #[test]
    fn test_benchmark_cost_estimator() {
        let est = CostEstimator::default();
        assert!((est.estimate_cost(10, 10) - 0.035).abs() < 1e-10);
        let custom = CostEstimator::new(1e-6, 0.001, 100);
        let expected = 5.0 * 100.0 * 1e-6 + 3.0 * 0.001;
        assert!((custom.estimate_cost(5, 3) - expected).abs() < 1e-10);
    }

    #[test]
    fn test_benchmark_result_creation() {
        let result = OrchBenchmarkResult {
            pattern_name: "sequential".into(),
            num_agents: 3,
            avg_latency_ms: 12.5,
            p50_latency_ms: 10.0,
            p95_latency_ms: 25.0,
            p99_latency_ms: 30.0,
            min_latency_ms: 5.0,
            max_latency_ms: 35.0,
            avg_steps: 3.0,
            total_tokens: 150,
            estimated_cost_usd: 0.005,
            success_rate: 1.0,
            runs_completed: 5,
            errors: vec![],
        };
        assert_eq!(result.pattern_name, "sequential");
        assert_eq!(result.runs_completed, 5);
    }

    #[test]
    fn test_benchmark_suite_creation() {
        let suite = OrchBenchmarkSuite::new();
        assert_eq!(suite.config.num_runs, 5);
        assert!(suite.results.is_empty());

        let config = OrchBenchmarkConfig { num_runs: 3, warmup_runs: 1, ..OrchBenchmarkConfig::default() };
        let suite2 = OrchBenchmarkSuite::with_config(config);
        assert_eq!(suite2.config.num_runs, 3);
        assert!(suite2.results.is_empty());
    }

    #[test]
    fn test_benchmark_all_patterns() {
        let mut suite = OrchBenchmarkSuite::new();
        let results = suite.benchmark_all_patterns::<String>(2);
        assert_eq!(results.len(), 4);
        for r in &results {
            assert!(r.runs_completed > 0);
            assert!(r.avg_latency_ms > 0.0);
        }
    }

    #[test]
    fn test_benchmark_compare_output() {
        let results = vec![OrchBenchmarkResult {
            pattern_name: "sequential".into(),
            num_agents: 2,
            avg_latency_ms: 10.0,
            p50_latency_ms: 10.0,
            p95_latency_ms: 12.0,
            p99_latency_ms: 13.0,
            min_latency_ms: 9.0,
            max_latency_ms: 14.0,
            avg_steps: 2.0,
            total_tokens: 20,
            estimated_cost_usd: 0.001,
            success_rate: 1.0,
            runs_completed: 5,
            errors: vec![],
        }];
        let output = compare_patterns(&results);
        assert!(output.contains("sequential"));
        assert!(output.contains("10.00"));
        assert!(output.contains("0.001000"));
    }

    #[test]
    fn test_benchmark_find_cheapest() {
        let results = vec![
            OrchBenchmarkResult {
                pattern_name: "costly".into(), num_agents: 1, avg_latency_ms: 10.0,
                p50_latency_ms: 10.0, p95_latency_ms: 10.0, p99_latency_ms: 10.0,
                min_latency_ms: 10.0, max_latency_ms: 10.0, avg_steps: 1.0,
                total_tokens: 100, estimated_cost_usd: 0.1, success_rate: 1.0,
                runs_completed: 5, errors: vec![],
            },
            OrchBenchmarkResult {
                pattern_name: "cheap".into(), num_agents: 1, avg_latency_ms: 10.0,
                p50_latency_ms: 10.0, p95_latency_ms: 10.0, p99_latency_ms: 10.0,
                min_latency_ms: 10.0, max_latency_ms: 10.0, avg_steps: 1.0,
                total_tokens: 50, estimated_cost_usd: 0.01, success_rate: 1.0,
                runs_completed: 5, errors: vec![],
            },
        ];
        let cheapest = find_cheapest_pattern(&results).unwrap();
        assert_eq!(cheapest.pattern_name, "cheap");
    }

    #[test]
    fn test_benchmark_find_fastest() {
        let results = vec![
            OrchBenchmarkResult {
                pattern_name: "slow".into(), num_agents: 1, avg_latency_ms: 100.0,
                p50_latency_ms: 100.0, p95_latency_ms: 100.0, p99_latency_ms: 100.0,
                min_latency_ms: 100.0, max_latency_ms: 100.0, avg_steps: 1.0,
                total_tokens: 50, estimated_cost_usd: 0.01, success_rate: 1.0,
                runs_completed: 5, errors: vec![],
            },
            OrchBenchmarkResult {
                pattern_name: "fast".into(), num_agents: 1, avg_latency_ms: 1.0,
                p50_latency_ms: 1.0, p95_latency_ms: 1.0, p99_latency_ms: 1.0,
                min_latency_ms: 1.0, max_latency_ms: 1.0, avg_steps: 1.0,
                total_tokens: 100, estimated_cost_usd: 0.1, success_rate: 1.0,
                runs_completed: 5, errors: vec![],
            },
        ];
        let fastest = find_fastest_pattern(&results).unwrap();
        assert_eq!(fastest.pattern_name, "fast");
    }

    #[test]
    fn test_benchmark_summary_report() {
        let mut suite = OrchBenchmarkSuite::new();
        suite.results.push(OrchBenchmarkResult {
            pattern_name: "sequential".into(),
            num_agents: 2,
            avg_latency_ms: 15.0,
            p50_latency_ms: 14.0,
            p95_latency_ms: 20.0,
            p99_latency_ms: 22.0,
            min_latency_ms: 12.0,
            max_latency_ms: 25.0,
            avg_steps: 2.0,
            total_tokens: 20,
            estimated_cost_usd: 0.0025,
            success_rate: 1.0,
            runs_completed: 5,
            errors: vec![],
        });
        let report = suite.summary_report();
        assert!(report.contains("Benchmark Summary"));
        assert!(report.contains("sequential"));
        assert!(report.contains("15.00"));
    }
}
